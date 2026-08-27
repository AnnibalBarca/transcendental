mod auth;
mod cache;
pub mod config;
mod db;
mod http;
mod metrics;
mod metrics_discovery;
mod port_utils;
mod services;
mod trace;
mod types;

use api_core::auth::init_jwt_manager;
use api_core::redis::pool::get_redis_pool;
use api_core::redis::service::{
    IncomingRequest, OutgoingResponse, RedisService, ServiceConfig as RedisServiceConfig,
};
use api_core::types::ServiceRequest;
use config::service::ServiceConfig;
use db::Database;
use log::{error, info, warn};
use metrics::app_metrics::AppMetrics;
use metrics_discovery::discover_metrics_manager;
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::cache::redis::RedisCache;
use crate::http::router::{build_redis_router, create_auth_router, AppState};

#[tokio::main]
async fn main() {
    trace::init_trace();
    info!("[Auth] Initializing service...");

    let mut config = ServiceConfig::from_env();
    info!("[Auth] Domain email: {}", config.domain_email);
    let metrics = Arc::new(AppMetrics::new());

    info!("[Auth] Starting Metrics Manager discovery...");
    discover_metrics_manager(&mut config, 1000).await;

    let Some(api_port) = config.api_port else {
        return error!("[Auth] API port missing !");
    };
    let Some(database_url) = config.database_url.clone() else {
        return error!("[Auth] Database URL missing !");
    };
    let Some(redis_url) = config.redis_url.clone() else {
        return error!("[Auth] Redis URL missing !");
    };

    let database = match Database::new(&database_url).await {
        Ok(db) => {
            info!("[Auth] Database initialized successfully");
            Arc::new(db)
        }
        Err(e) => {
            return error!("[Auth] Failed to initialize database: {}", e);
        }
    };

    let redis_pool = get_redis_pool(&redis_url).await;

    if let Err(e) = init_jwt_manager(&redis_pool).await {
        return error!("[Auth] Failed to initialize JWT manager: {}", e);
    }

    info!("[Auth] Service started, waiting for requests...");

    let cache = Arc::new(RedisCache::new(redis_pool.clone()));

    // Bootstrap admin en arrière-plan : réessaie jusqu'à ce que l'User-API
    // ait créé les tables `roles` / `user_roles`.
    {
        let db = database.clone();
        tokio::spawn(async move {
            loop {
                match crate::services::user::bootstrap_admin_user(&db).await {
                    Ok(_) => {
                        info!("[Auth] Admin bootstrap done");
                        break;
                    }
                    Err(e) => {
                        warn!("[Auth] Admin bootstrap retry (tables pas prêtes ?): {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });
    }

    let app_state = AppState {
        metrics: Arc::clone(&metrics),
        database,
        cache,
        config: Arc::new(config),
        redis_pool: redis_pool.clone(),
    };

    let app = create_auth_router(app_state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new());

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], api_port));
    let Ok(listener) = tokio::net::TcpListener::bind(addr).await else {
        return error!("[Auth] Failed to bind to {}", addr);
    };

    info!("[Auth] Starting Auth API server on {}", addr);
    let http_task = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("[Auth] Server error: {}", e);
        }
    });

    let router_redis = build_redis_router();

    let metrics_port = std::env::var("ROUTER_METRICS_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok());
    if let Some(port) = router_redis.start_metrics_server(metrics_port).await {
        info!("[Auth] Router metrics available at http://localhost:{}/metrics", port);
    }
    let redis_service = RedisService::new(
        redis_pool.clone(),
        RedisServiceConfig {
            request_stream: "auth:requests".to_string(),
            response_stream: "gateway:responses".to_string(),
            group_name: "auth-service-group".to_string(),
            consumer_name: "auth-consumer-1".to_string(),
        },
    );

    let redis_task = tokio::spawn(async move {
        redis_service
            .listen_forever(move |req: IncomingRequest| {
                let ctx = app_state.clone();
                let router = router_redis.clone();
                async move {
                    let request: ServiceRequest = serde_json::from_str(&req.payload)?;
                    let request_id = request.id.clone();
                    let response = router.route(ctx, request).await;
                    Ok(OutgoingResponse {
                        request_id,
                        payload: response,
                    })
                }
            })
            .await;
    });

    let _ = tokio::join!(http_task, redis_task);
}

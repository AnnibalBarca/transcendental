mod cache;
mod db;
mod http;
mod services;
mod types;
mod user_state;
mod xp;

use std::sync::Arc;

use api_core::auth::init_jwt_discover;
use api_core::db::Database;
use api_core::redis::pool::get_redis_pool;
use api_core::redis::service::{IncomingRequest, OutgoingResponse, RedisService, ServiceConfig};
use api_core::types::ServiceRequest;
use log::{error, info};

use crate::db::migrations;
use crate::http::router::build_router;
use crate::services::storage::Storage;
use crate::user_state::RedisSessionManager;

#[derive(Clone)]
pub struct AppContext {
    pub db: Arc<Database>,
    pub redis_pool: deadpool_redis::Pool,
    pub session_manager: Arc<RedisSessionManager>,
    pub storage: Option<Arc<Storage>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Arc::new(
        Database::new(&database_url, &migrations::all())
            .await
            .expect("Failed to connect to database"),
    );

    let redis_pool = get_redis_pool(&redis_url).await;

    if let Err(e) = init_jwt_discover(&redis_pool).await {
        return error!("[User] Failed to initialize JWT manager: {}", e);
    }

    let storage = match Storage::from_env() {
        Some(storage) => match storage.ensure_bucket().await {
            Ok(()) => {
                info!("[User] Shop bucket '{}' ready", storage.bucket());
                Some(Arc::new(storage))
            }
            Err(e) => {
                error!("[User] MinIO bucket setup failed: {}", e);
                None
            }
        },
        None => {
            info!("[User] MinIO not configured, shop assets will use stored URLs");
            None
        }
    };

    let ctx = AppContext {
        db,
        redis_pool: redis_pool.clone(),
        session_manager: Arc::new(RedisSessionManager::new(redis_pool.clone())),
        storage,
    };

    match crate::db::routes::list(ctx.db.get_pool()).await {
        Ok(routes) => {
            let mut synced = 0usize;
            for route in routes {
                if let Some(rpm) = route.requests_per_minute {
                    if api_core::ratelimit::set_limit(
                        &ctx.redis_pool,
                        &route.method,
                        &route.path,
                        rpm as i64,
                    )
                    .await
                    .is_ok()
                    {
                        synced += 1;
                    }
                }
            }
            info!("[User] Rate limits synced to Redis: {} routes", synced);
        }
        Err(e) => {
            error!("[User] Failed to list routes for rate limit sync: {}", e);
        }
    }

    match crate::services::permission::sync_permissions_to_redis(&ctx).await {
        Ok(_) => info!("[User] Permissions synced to Redis"),
        Err(e) => error!("[User] Failed to sync permissions to Redis: {}", e),
    }

    info!("[User] Service started, waiting for requests...");

    let router = build_router();

    let metrics_port = std::env::var("ROUTER_METRICS_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok());
    if let Some(port) = router.start_metrics_server(metrics_port).await {
        info!("[User] Router metrics available at http://localhost:{}/metrics", port);
    }

    let redis_service = RedisService::new(
        redis_pool.clone(),
        ServiceConfig {
            request_stream: "user:requests".to_string(),
            response_stream: "gateway:responses".to_string(),
            group_name: "user-service-group".to_string(),
            consumer_name: "user-consumer-1".to_string(),
        },
    );

    let redis_task = tokio::spawn(async move {
        redis_service
            .listen_forever(move |req: IncomingRequest| {
                let ctx = ctx.clone();
                let router = router.clone();
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

    if let Err(e) = redis_task.await {
        error!("[User] Redis listener task ended with error: {}", e);
    }
}

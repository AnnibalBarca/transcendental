mod chess_discovery;
mod config;
mod http;
mod metrics;
mod metrics_discovery;
mod port_utils;
mod trace;

use api_core::redis::pool::get_redis_pool;
use config::ServiceConfig;
use http::create_gateway_router;
use http::response_listener::ResponseListener;
use http::router::Router as RequestRouter;
use metrics::app_metrics::AppMetrics;
use metrics_discovery::discover_metrics_manager;
use reqwest::Client;
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use log::{error, info};

use crate::http::handlers::AppState;

#[tokio::main]
async fn main() {
    trace::init_trace();

    info!("[Gateway] Initializing service...");

    let mut config = ServiceConfig::from_env();

    let metrics = Arc::new(AppMetrics::new());

    info!("[Gateway] Starting Metrics Manager discovery...");
    discover_metrics_manager(&mut config, 1000).await;

    let Some(api_port) = config.api_port else {
        return error!("[Gateway] API port missing !");
    };

    let Some(redis_url) = config.redis_url.clone() else {
        return error!("[Gateway] Redis URL missing !");
    };

    if config.auth_api_url.is_none() {
        return error!("[Gateway] Auth API URL missing !");
    }

    if config.user_api_url.is_none() {
        return error!("[Gateway] User API URL missing !");
    }

    if config.notification_api_url.is_none() {
        return error!("[Gateway] Notification API URL missing !");
    }

    let redis_pool = get_redis_pool(&redis_url).await;

    let request_router = RequestRouter::new();
    let request_router_clone = request_router.clone();
    let redis_pool_clone = redis_pool.clone();

    tokio::spawn(async move {
        let listener = ResponseListener::new(redis_pool_clone, request_router_clone);
        listener.run().await;
    });

    let http_client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build HTTP client");

    let config_static = Arc::new(config);

    let app_state = AppState {
        metrics: Arc::clone(&metrics),
        http_client,
        request_router,
        redis_pool,
        config: config_static,
    };

    let app = create_gateway_router(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new());

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], api_port));

    let Ok(listener) = tokio::net::TcpListener::bind(addr).await else {
        return error!("[Gateway] Failed to bind to {}", addr);
    };

    info!("[Gateway] Starting Gateway API server on {}", addr);

    let server = axum::serve(listener, app);

    if let Err(e) = server.await {
        error!("[Gateway] Server error: {}", e);
    }
}

mod event;
mod http;
mod notification_service;

use std::sync::Arc;

use api_core::auth::init_jwt_discover;
use api_core::redis::pool::get_redis_pool;
use api_core::sse::SseConnectionManager;
use log::{error, info};

use crate::event::{NotificationEvent, NotificationMetadata, NotificationTarget};
use notification_service::NotificationService;

#[derive(Clone)]
pub struct AppState {
    pub redis_pool: deadpool_redis::Pool,
    pub notification_service: Arc<NotificationService>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("[Notification-API] Initializing service...");

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let port = std::env::var("NOTIFICATION_API_PORT")
        .unwrap_or_else(|_| "3006".to_string())
        .parse::<u16>()
        .expect("NOTIFICATION_API_PORT must be a valid port number");

    let redis_pool = get_redis_pool(&redis_url).await;

    if let Err(e) = init_jwt_discover(&redis_pool).await {
        return error!(
            "[Notification-API] Failed to initialize JWT discovery: {}",
            e
        );
    }

    let manager =
        SseConnectionManager::<NotificationEvent, NotificationMetadata, NotificationTarget>::new(
            redis_url,
            "sse:events",
        );

    let notification_service = Arc::new(NotificationService::new(manager));

    let state = AppState {
        redis_pool,
        notification_service,
    };

    let app = http::router::build_router(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    info!("[Notification-API] Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("[Notification-API] Failed to bind");

    if let Err(e) = axum::serve(listener, app).await {
        error!("[Notification-API] Server error: {}", e);
    }
}

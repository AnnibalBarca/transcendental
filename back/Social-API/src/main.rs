mod cache;
mod db;
mod http;
mod service;
mod services;

mod types;
mod user_state;
mod utils;

use std::sync::Arc;

use api_core::auth::init_jwt_discover;
use api_core::redis::pool::get_redis_pool;
use api_core::sse::SsePublisher;
use notification::event::{NotificationBus, NotificationEvent, NotificationTarget};
use tracing::{error, info};

use crate::{db::db::Database, user_state::RedisSessionManager};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Database::new(&database_url)
        .await
        .expect("Failed to connect to database");
    let db_arc = std::sync::Arc::new(db);

    let redis_pool = get_redis_pool(&redis_url).await;

    if let Err(e) = init_jwt_discover(&redis_pool).await {
        error!("[Social] Failed to initialize JWT manager: {}", e);
        return;
    }

    let session_manager = Arc::new(RedisSessionManager::new(redis_pool.clone()));

    let publisher = SsePublisher::<NotificationEvent, NotificationTarget>::new(
        redis_pool.clone(),
        "sse:events",
    );
    let notification_bus = NotificationBus::new(publisher);

    info!("[Social] Service started, waiting for requests...");

    if let Err(e) =
        service::listen_for_requests(db_arc, &redis_pool, session_manager, notification_bus).await
    {
        error!("[Social] Listener stopped with error: {}", e);
    }
}

mod cache;
mod db;
mod http;
mod service;
mod services;
mod types;
mod user_state;
mod utils;

use std::sync::Arc;

use crate::{
    db::db::Database, services::matchmaking::run_matchmaking_loop,
    services::room::run_auto_fill_loop, user_state::RedisSessionManager,
};
use api_core::auth::init_jwt_discover;
use api_core::redis::pool::get_redis_pool;
use api_core::sse::SsePublisher;
use notification::event::{NotificationBus, NotificationEvent, NotificationTarget};
use log::{error, info};

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

    let db = Database::new(&database_url)
        .await
        .expect("Failed to connect to database");
    let db_arc = std::sync::Arc::new(db);

    let redis_pool = get_redis_pool(&redis_url).await;

    if let Err(e) = init_jwt_discover(&redis_pool).await {
        error!("[Room] Failed to initialize JWT manager: {}", e);
        return;
    }

    let session_manager = Arc::new(RedisSessionManager::new(redis_pool.clone()));

    let publisher = SsePublisher::<NotificationEvent, NotificationTarget>::new(
        redis_pool.clone(),
        "sse:events",
    );
    let notification_bus = NotificationBus::new(publisher);

    let matchmaking_pool = redis_pool.clone();
    let matchmaking_session_manager = Arc::clone(&session_manager);
    let matchmaking_bus = notification_bus.clone();
    let matchmaking_db = db_arc.get_pool().clone();
    tokio::spawn(async move {
        run_matchmaking_loop(
            matchmaking_pool,
            matchmaking_session_manager,
            matchmaking_bus,
            matchmaking_db,
        )
        .await;
    });

    let auto_fill_pool = redis_pool.clone();
    let auto_fill_bus = notification_bus.clone();
    tokio::spawn(async move {
        run_auto_fill_loop(auto_fill_pool, auto_fill_bus).await;
    });

    let game_result_pool = redis_pool.clone();
    let game_result_db = db_arc.get_pool().clone();
    let game_result_session_manager = Arc::clone(&session_manager);
    let game_result_bus = notification_bus.clone();
    tokio::spawn(async move {
        services::game_result::run_game_result_listener(
            game_result_pool,
            game_result_db,
            game_result_session_manager,
            game_result_bus,
        )
        .await;
    });

    let tournament_pool = redis_pool.clone();
    let tournament_db = db_arc.get_pool().clone();
    let tournament_session_manager = Arc::clone(&session_manager);
    let tournament_bus = notification_bus.clone();
    tokio::spawn(async move {
        services::tournament::run_tournament_loop(
            tournament_pool,
            tournament_db,
            tournament_session_manager,
            tournament_bus,
        )
        .await;
    });

    let live_games_pool = redis_pool.clone();
    let live_games_db = db_arc.get_pool().clone();
    let live_games_bus = notification_bus.clone();
    tokio::spawn(async move {
        services::live_games::run_live_games_loop(live_games_pool, live_games_db, live_games_bus)
            .await;
    });

    let public_rooms_pool = redis_pool.clone();
    let public_rooms_bus = notification_bus.clone();
    tokio::spawn(async move {
        services::room::run_public_rooms_loop(public_rooms_pool, public_rooms_bus).await;
    });

    info!("[Room] Service started, waiting for requests...");

    if let Err(e) =
        service::listen_for_requests(db_arc, &redis_pool, session_manager, notification_bus).await
    {
        error!("[Room] Listener stopped with error: {}", e);
    }
}

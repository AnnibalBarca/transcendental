mod db;
mod game;
mod http;
mod registry;
mod service;
mod websocket;

use std::sync::Arc;
use std::time::Duration;

use api_core::auth::init_jwt_discover;
use api_core::redis::pool::get_redis_pool;
use api_core::sse::SseEnvelope;
use futures::StreamExt;
use log::{error, info};
use notification::event::{NotificationEvent, NotificationTarget};

use crate::game::manager::GameManager;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let redis_pool = get_redis_pool(&redis_url).await;

    let db_pool = match std::env::var("DATABASE_URL") {
        Ok(url) => match sqlx::PgPool::connect(&url).await {
            Ok(pool) => {
                info!("[Chess] PostgreSQL pool connected");
                Some(pool)
            }
            Err(e) => {
                error!("[Chess] Failed to connect to PostgreSQL: {}", e);
                None
            }
        },
        Err(_) => None,
    };

    if let Err(e) = init_jwt_discover(&redis_pool).await {
        return error!("[Chess] Failed to initialize JWT manager: {}", e);
    }

    let game_manager = Arc::new(GameManager::new(db_pool));

    let instance_id = std::env::var("CHESS_INSTANCE_ID").unwrap_or_else(|_| "chess-1".to_string());
    let ws_url = std::env::var("CHESS_PUBLIC_WS_URL")
        .unwrap_or_else(|_| format!("ws://{}:8082/chess", instance_id));

    info!(
        "[Chess] Instance {} starting, public URL: {}",
        instance_id, ws_url
    );

    if let Err(e) = registry::register_instance(&redis_pool, &instance_id, &ws_url).await {
        error!("[Chess] Failed to register instance: {}", e);
    }

    let heartbeat_pool = redis_pool.clone();
    let heartbeat_instance_id = instance_id.clone();
    let heartbeat_ws_url = ws_url.clone();
    let heartbeat_manager = Arc::clone(&game_manager);
    tokio::spawn(async move {
        registry::start_heartbeat(
            heartbeat_pool,
            heartbeat_instance_id,
            heartbeat_ws_url,
            move || {
                let manager = Arc::clone(&heartbeat_manager);
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(async move { manager.game_count().await })
                })
            },
        )
        .await;
    });

    info!("[Chess] Service started, waiting for requests...");

    let ws_task = {
        let pool = redis_pool.clone();
        let manager = Arc::clone(&game_manager);
        let instance_id = instance_id.clone();
        tokio::spawn(async move {
            if let Err(e) = websocket::start_websocket_server(pool, manager, instance_id).await {
                error!("[Chess] WebSocket server error: {}", e);
            }
        })
    };

    let redis_task = {
        let pool = redis_pool.clone();
        let manager = Arc::clone(&game_manager);
        tokio::spawn(async move {
            service::listen_for_requests(pool, manager, instance_id).await;
        })
    };

    let picture_listener_manager = Arc::clone(&game_manager);
    tokio::spawn(async move {
        run_picture_listener(picture_listener_manager).await;
    });

    tokio::select! {
        result = ws_task => {
            if let Err(e) = result {
                error!("[Chess] WebSocket task panicked: {}", e);
            }
        }
        _ = redis_task => {
            error!("[Chess] Redis listener task ended");
        }
    }

    error!("[Chess] One of the services has stopped, shutting down...");
}

async fn run_picture_listener(manager: Arc<GameManager>) {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    loop {
        let client = match redis::Client::open(redis_url.clone()) {
            Ok(c) => c,
            Err(e) => {
                error!("[Chess] Picture listener Redis error: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let mut pubsub = match client.get_async_pubsub().await {
            Ok(p) => p,
            Err(e) => {
                error!("[Chess] Picture listener pubsub error: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        if let Err(e) = pubsub.subscribe("sse:events").await {
            error!("[Chess] Failed to subscribe to sse:events: {}", e);
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        info!("[Chess] Picture listener subscribed to sse:events");

        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let envelope: SseEnvelope<NotificationEvent, NotificationTarget> =
                match serde_json::from_str(&payload) {
                    Ok(e) => e,
                    Err(_) => continue,
                };

            if let NotificationEvent::ProfilePictureUpdated { user_id, picture_id } = envelope.event {
                info!("[Chess] Profile picture updated for {}, broadcasting to lobbies", user_id);
                manager.broadcast_picture_update(&user_id.to_string(), &picture_id).await;
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use log::info;

use crate::game::manager::GameManager;
use crate::websocket::handler::ws_handler;

pub async fn start_websocket_server(
    redis_pool: deadpool_redis::Pool,
    game_manager: Arc<GameManager>,
    instance_id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_port = std::env::var("CHESS_WS_PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse::<u16>()?;

    let app_state = WebSocketState {
        redis_pool,
        game_manager,
        instance_id,
    };

    let app = Router::new()
        .route("/chess", get(ws_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], ws_port));
    info!("[Chess-WS] WebSocket server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct WebSocketState {
    pub redis_pool: deadpool_redis::Pool,
    pub game_manager: Arc<GameManager>,
    pub instance_id: String,
}

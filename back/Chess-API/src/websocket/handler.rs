use axum::{
    extract::{Query, State, WebSocketUpgrade},
    http::{header::COOKIE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use headers::{Cookie, Header};
use serde::Deserialize;

use log::{info, warn};

use api_core::auth::jwt_manager;

use crate::game::session::handle_player_session;
use crate::websocket::server::WebSocketState;

#[derive(Debug, Deserialize)]
pub struct WsParams {
    pub game_id: Option<String>,
    pub access_token: Option<String>,
    pub picture_id: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    Query(params): Query<WsParams>,
    State(state): State<WebSocketState>,
) -> Response {
    let game_id = params.game_id.unwrap_or_else(|| "default".to_string());

    let mut cookie_values = headers.get_all(COOKIE).iter();

    let token = Cookie::decode(&mut cookie_values)
        .ok()
        .and_then(|cookie| cookie.get("access_token").map(|t| t.to_string()))
        .or_else(|| params.access_token.clone());

    let claims = match token {
        Some(token) => match jwt_manager().validate_access_token(&token) {
            Ok(claims) => claims,
            Err(e) => {
                warn!("[Chess-WS] Invalid access token: {}", e);
                return (
                    StatusCode::UNAUTHORIZED,
                    "Invalid or expired token",
                )
                    .into_response();
            }
        },
        None => {
            warn!("[Chess-WS] Missing access token");
            return (
                StatusCode::UNAUTHORIZED,
                "Missing access token",
            )
                .into_response();
        }
    };

    let user_id = claims.sub.clone();
    if !is_registered_player(&state.redis_pool, &game_id, &user_id).await {
        warn!(
            "[Chess-WS] User {} is not a player of game {}, connection forbidden",
            user_id, game_id
        );
        return (
            StatusCode::FORBIDDEN,
            "You are not a player of this game",
        )
            .into_response();
    }

    let instance = match state.game_manager.get_game(&game_id).await {
        Some(instance) => instance,
        None => {
            warn!(
                "[Chess-WS] Game {} not found on instance {}",
                game_id, state.instance_id
            );
            return axum::response::IntoResponse::into_response((
                axum::http::StatusCode::NOT_FOUND,
                format!("Game {} not found", game_id),
            ));
        }
    };

    info!("[Chess-WS] New connection for game {}", game_id);

    let redis_pool = state.redis_pool.clone();
    let picture_id = params.picture_id.unwrap_or_default();
    ws.on_upgrade(move |socket| async move {
        let _ = handle_player_session(socket, instance, claims, redis_pool, picture_id).await;
    })
}

async fn is_registered_player(
    redis_pool: &deadpool_redis::Pool,
    game_id: &str,
    user_id: &str,
) -> bool {
    use deadpool_redis::redis::cmd;

    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            warn!("[Chess-WS] Redis pool error while checking players: {}", e);
            return false;
        }
    };

    let key = format!("chess:game_players:{}", game_id);
    let data: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .ok()
        .flatten();

    let Some(data) = data else {
        return false;
    };

    let players: Vec<String> = serde_json::from_str(&data).unwrap_or_default();
    players.iter().any(|p| p == user_id)
}

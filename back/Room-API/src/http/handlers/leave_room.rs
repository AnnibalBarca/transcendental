use std::sync::Arc;

use crate::cache::room as room_cache;
use crate::http::response::json_error;
use crate::services::chess_client;
use crate::services::room as room_service;
use crate::types::ServiceRequest;
use crate::user_state::RedisSessionManager;
use crate::utils::extract_user_id_from_access_token;
use log::{error, info};
use notification::event::{NotificationBus, NotificationEvent};
use serde_json::json;

pub async fn handle_leave_room(
    redis_pool: &deadpool_redis::Pool,
    request: &ServiceRequest,
    session_manager: Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(token) => token,
        None => return json_error(401, "Missing access token"),
    };

    let user_id = match extract_user_id_from_access_token(token) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let player_session = match session_manager.get_session(&user_id).await {
        Ok(Some(session)) => session,
        Ok(None) => return json_error(400, "Not in any room"),
        Err(e) => {
            error!("Session retrieval failed: {}", e);
            return json_error(500, "Session retrieval failed");
        }
    };

    let is_abandon = player_session.status.as_str() == "playing";

    if player_session.status.as_str() != "waiting" && !is_abandon {
        return json_error(409, "Cannot leave room in current state");
    }

    let room_id = &player_session.room_id;

// En cours de partie : abandon = la partie se termine, l'adversaire gagne.
    if is_abandon {
        info!("[LeaveRoom] User {} abandons room {}", user_id, room_id);
        if let Ok(Some(room)) = room_cache::get(redis_pool, room_id).await {
            if let Some(game_id) = room.chess_game_id.as_deref() {
                if let Err(e) = chess_client::abandon_game(redis_pool, game_id, &user_id.to_string()).await {
                    error!("[LeaveRoom] Failed to abandon game {}: {}", game_id, e);
                }
            }
        }
    }

    let remaining_room = match room_service::leave_room(redis_pool, room_id, user_id).await {
        Ok(room) => room,
        Err(e) => {
            error!("Failed to leave room: {}", e);
            return json_error(500, "Failed to leave room");
        }
    };

    if let Some(room) = &remaining_room {
        room_service::publish_room_update(redis_pool, notification_bus, room).await;
    }

    let cleared_session = crate::user_state::PlayerSession {
        room_id: "0".into(),
        status: "none".into(),
        chess_ws_url: String::new(),
        chess_game_id: String::new(),
    };

    if let Err(e) = session_manager.save_session(&user_id, &cleared_session).await {
        error!("Session save failed: {}", e);
        return json_error(500, "Session save failed");
    }

    notification_bus
        .send_to_user(
            user_id,
            &NotificationEvent::SetState {
                user_id,
                state: "none".into(),
                room_id: None,
                chess_ws_url: None,
                chess_game_id: None,
            },
        )
        .await;

    publish_room_update(redis_pool).await;

    json!({
        "status": 200,
        "message": "Left room successfully"
    })
}

async fn publish_room_update(redis_pool: &deadpool_redis::Pool) {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(_) => return,
    };
    let _: redis::RedisResult<()> = redis::cmd("PUBLISH")
        .arg("room:public:updates")
        .arg("updated")
        .query_async(&mut *conn)
        .await;
}

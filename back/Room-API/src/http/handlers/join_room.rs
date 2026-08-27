use std::sync::Arc;

use crate::cache::room::{self as cache_room, RoomRecord};
use crate::http::response::json_error;
use crate::services::room as room_service;
use crate::types::ServiceRequest;
use crate::user_state::RedisSessionManager;
use crate::utils::extract_user_id_from_access_token;
use log::{error, info};
use notification::event::{NotificationBus, NotificationEvent};
use serde_json::json;

pub async fn handle_join_room(
    db_pool: &sqlx::PgPool,
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

    let body: serde_json::Value = match serde_json::from_str(&request.body) {
        Ok(v) => v,
        Err(_) => return json_error(400, "Invalid request body"),
    };

    let room_id = body.get("room_id").and_then(|v| v.as_str()).map(|s| s.to_string());
    let join_code = body.get("join_code").and_then(|v| v.as_str()).map(|s| s.to_string());

    let resolved_room_id = if let Some(code) = &join_code {
        match cache_room::get_room_id_by_code(redis_pool, code).await {
            Ok(Some(rid)) => rid,
            Ok(None) => return json_error(404, "Invalid join code"),
            Err(e) => {
                error!("Failed to look up join code: {}", e);
                return json_error(500, "Failed to look up join code");
            }
        }
    } else if let Some(rid) = room_id {
        rid
    } else {
        return json_error(400, "Either room_id or join_code is required");
    };

    let player_session = match session_manager.get_session(&user_id).await {
        Ok(Some(session)) => session,
        Ok(None) => crate::user_state::PlayerSession {
            room_id: "0".into(),
            status: "none".into(),
            chess_ws_url: String::new(),
            chess_game_id: String::new(),
        },
        Err(e) => {
            error!("Session retrieval failed: {}", e);
            return json_error(500, "Session retrieval failed");
        }
    };

    if player_session.status.as_str() != "none" {
        return json_error(409, "Already in a room or matchmaking");
    }

    let joiner_username = match crate::db::user::get_by_id(db_pool, &user_id).await {
        Ok(Some(rec)) => rec.username,
        Ok(None) | Err(_) => "Player".to_string(),
    };

    let room = match room_service::join_room(redis_pool, &resolved_room_id, user_id, &joiner_username).await {
        Ok(room) => room,
        Err(e) => {
            return json_error(400, &format!("Cannot join room: {}", e));
        }
    };

    let mut updated_session = player_session;
    updated_session.room_id = room.id.clone();
    updated_session.status = "waiting".into();

    if let Err(e) = session_manager.save_session(&user_id, &updated_session).await {
        error!("Session save failed: {}", e);
        return json_error(500, "Session save failed");
    }

    notification_bus
        .send_to_user(
            user_id,
            &NotificationEvent::SetState {
                user_id,
                state: "waiting".into(),
                room_id: uuid::Uuid::parse_str(&room.id).ok(),
                chess_ws_url: None,
                chess_game_id: None,
            },
        )
        .await;

    info!(
        "[JoinRoom] User {} joined room {} ({}/{})",
        user_id, room.id, room.player_count, room.max_players
    );

    room_service::publish_room_update(redis_pool, notification_bus, &room).await;
    publish_room_update(redis_pool).await;

    json!({
        "status": 200,
        "room_id": room.id,
        "player_count": room.player_count,
        "max_players": room.max_players,
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

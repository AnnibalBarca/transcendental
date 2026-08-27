use std::sync::Arc;

use crate::http::response::json_error;
use crate::services::room as room_service;
use crate::types::{CreateRoomRequest, ServiceRequest};
use crate::user_state::{PlayerSession, RedisSessionManager};
use crate::utils::extract_user_id_from_access_token;
use log::error;
use notification::event::{NotificationBus, NotificationEvent};
use serde_json::json;

pub async fn handle_create_room(
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

    let create_room_req: CreateRoomRequest = match serde_json::from_str(&request.body) {
        Ok(req) => req,
        Err(e) => {
            error!("Failed to parse request body: {}", e);
            return json_error(400, "Invalid request body");
        }
    };

    let allowed_sizes = [2u32, 4, 8, 16];
    if !allowed_sizes.contains(&create_room_req.max_players) {
        return json_error(400, "Player count must be 2, 4, 8 or 16");
    }

    if create_room_req.max_players == 2
        && create_room_req.bot_difficulty.is_none()
        && create_room_req.time_control.is_some()
        && ![5, 10, 15].contains(&create_room_req.time_control.unwrap())
    {
        return json_error(400, "Time control must be 5, 10 or 15 minutes");
    }

    if let Some(ref difficulty) = create_room_req.bot_difficulty {
        if !matches!(difficulty.as_str(), "easy" | "medium" | "hard") {
            return json_error(400, "Bot difficulty must be easy, medium, or hard");
        }
    }

    let mut player_session = match session_manager.get_session(&user_id).await {
        Ok(Some(session)) => session,
        Ok(None) => PlayerSession {
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
        return json_error(409, "Conflict: already in a room or matchmaking");
    }

    let time_control = create_room_req
        .time_control
        .or_else(|| if create_room_req.max_players == 2 { Some(10) } else { None });

    let host_username = match crate::db::user::get_by_id(db_pool, &user_id).await {
        Ok(Some(rec)) => rec.username,
        Ok(None) | Err(_) => "Player".to_string(),
    };

    let room = match room_service::create_room(
        redis_pool,
        user_id,
        create_room_req.title,
        create_room_req.private,
        create_room_req.max_players,
        create_room_req.bot_difficulty,
        &host_username,
        time_control,
    )
    .await
    {
        Ok(room) => room,
        Err(e) => {
            error!("Room creation failed: {}", e);
            return json_error(500, "Room creation failed");
        }
    };

    player_session.room_id = room.id.clone();
    player_session.status = "waiting".into();

    if let Err(e) = session_manager
        .save_session(&user_id, &player_session)
        .await
    {
        error!("Session save failed: {}", e);
        return json_error(500, "Session save failed");
    }

    notification_bus
        .send_to_user(
            user_id,
            &NotificationEvent::SetState {
                user_id,
                state: "waiting".into(),
                room_id: Some(uuid::Uuid::parse_str(&room.id).unwrap_or(user_id)),
                chess_ws_url: None,
                chess_game_id: None,
            },
        )
        .await;

    room_service::publish_room_update(redis_pool, notification_bus, &room).await;
    publish_room_update(redis_pool).await;

    json!({
        "status": 200,
        "room_id": room.id,
        "join_code": room.join_code,
        "private": room.private,
        "player_count": room.player_count,
        "max_players": room.max_players,
        "time_control": room.time_control,
        "game_type": if room.max_players > 2 { "tournament" } else { "game" },
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

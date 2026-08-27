use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cache::room::{self, PlayerData, RoomRecord, RoomStatus, RoomType};
use crate::http::response::{json_error, json_room};
use crate::types::{CreateRoomRequest, RoomPayload, ServiceRequest};
use crate::user_state::{PlayerSession, RedisSessionManager};
use crate::utils::extract_user_id_from_access_token;
use crate::utils::room_id::generate_unique_user_id;
use log::error;
use notification::event::{NotificationBus, NotificationEvent};

pub async fn handle_make_room(
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

    if create_room_req.max_players != 1 && create_room_req.max_players != 2 {
        return json_error(400, "Player count must be 1 or 2");
    }

    if create_room_req.max_players == 1 && create_room_req.bot_difficulty.is_none() {
        return json_error(400, "Bot difficulty required for single player");
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
        return json_error(409, "Conflict: already in a room");
    }

    let room_id = match generate_unique_user_id(redis_pool).await {
        Ok(id) => id,
        Err(e) => {
            error!("Room ID generation failed: {}", e);
            return json_error(500, "Room creation failed");
        }
    };

    player_session.room_id = room_id.to_string();
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
                room_id: Some(room_id),
                chess_ws_url: None,
                chess_game_id: None,
            },
        )
        .await;

    let host_player = PlayerData {
        player_ids: user_id,
        player_number: 1,
        player_profile_picture: "default.png".to_string(),
        player_username: "Player".to_string(),
    };

    let room_record = RoomRecord {
        id: room_id.to_string(),
        room_type: RoomType::Casual,
        private: create_room_req.private,
        join_code: None,
        title: create_room_req.title,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        player_count: 1,
        max_players: create_room_req.max_players,
        host_id: user_id,
        player_ids: vec![host_player],
        bot_difficulty: create_room_req.bot_difficulty,
        status: RoomStatus::Waiting,
        chess_game_id: None,
        chess_ws_url: None,
        time_control: create_room_req.time_control,
        banned_ids: vec![],
    };

    if let Err(e) = room::set(redis_pool, &room_record).await {
        error!("Room save failed: {}", e);
        return json_error(500, "Room save failed");
    }

    json_room(RoomPayload {
        room_id: player_session.room_id,
        status: player_session.status,
    })
}

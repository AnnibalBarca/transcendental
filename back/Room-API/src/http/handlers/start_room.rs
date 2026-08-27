use std::sync::Arc;

use crate::http::response::json_error;
use crate::services::room as room_service;
use crate::types::{ServiceRequest, StartRoomRequest};
use crate::user_state::RedisSessionManager;
use crate::utils::extract_user_id_from_access_token;
use log::error;
use notification::event::NotificationBus;
use serde_json::json;

pub async fn handle_start_room(
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

    let start_req: StartRoomRequest = match serde_json::from_str(&request.body) {
        Ok(req) => req,
        Err(_) => return json_error(400, "Invalid request body"),
    };

    match room_service::start_room(
        redis_pool,
        &session_manager,
        notification_bus,
        &start_req.room_id,
        user_id,
    )
    .await
    {
        Ok(room) => json!({
            "status": 200,
            "room_id": room.id,
            "room_status": "playing",
        }),
        Err(e) => {
            error!("[StartRoom] Failed: {}", e);
            json_error(400, &e)
        }
    }
}

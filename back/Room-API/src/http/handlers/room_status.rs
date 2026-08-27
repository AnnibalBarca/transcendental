use std::sync::Arc;

use crate::http::response::json_error;
use crate::types::ServiceRequest;
use crate::user_state::RedisSessionManager;
use crate::utils::extract_user_id_from_access_token;
use serde_json::json;
use log::error;

pub async fn handle_room_status(
    _redis_pool: &deadpool_redis::Pool,
    request: &ServiceRequest,
    session_manager: Arc<RedisSessionManager>,
) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(token) => token,
        None => return json_error(401, "Missing access token"),
    };

    let user_uuid = match extract_user_id_from_access_token(token) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match session_manager.get_session(&user_uuid).await {
        Ok(Some(session)) => json!({
            "status": 200,
            "room_id": session.room_id,
            "state": session.status,
            "chess_ws_url": session.chess_ws_url
        }),
        Ok(None) => json!({
            "status": 200,
            "room_id": "0",
            "state": "none",
            "chess_ws_url": ""
        }),
        Err(e) => {
            error!("Session retrieval failed: {}", e);
            json_error(500, "Session retrieval failed")
        }
    }
}

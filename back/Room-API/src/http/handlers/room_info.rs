use crate::http::response::json_error;
use crate::services::room as room_service;
use crate::types::ServiceRequest;
use serde_json::json;

pub async fn handle_room_info(
    redis_pool: &deadpool_redis::Pool,
    request: &ServiceRequest,
) -> serde_json::Value {
    let body: serde_json::Value = match serde_json::from_str(&request.body) {
        Ok(v) => v,
        Err(_) => return json_error(400, "Invalid request body"),
    };

    let room_id = match body.get("room_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return json_error(400, "room_id is required"),
    };

    match room_service::get_room_lobby(redis_pool, room_id).await {
        Ok(state) => json!({
            "status": 200,
            "room": state,
        }),
        Err(e) => json_error(400, &e),
    }
}

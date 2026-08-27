use crate::http::response::json_error;
use crate::services::room as room_service;
use serde_json::json;

pub async fn handle_list_rooms(
    redis_pool: &deadpool_redis::Pool,
) -> serde_json::Value {
    match room_service::list_public_rooms(redis_pool).await {
        Ok(rooms) => {
            let count = rooms.len();
            json!({
                "status": 200,
                "rooms": rooms,
                "count": count,
            })
        }
        Err(e) => json_error(500, &format!("Failed to list rooms: {}", e)),
    }
}

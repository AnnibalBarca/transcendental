use crate::cache::matchmaking;
use crate::http::response::json_error;
use log::error;
use serde_json::json;

pub async fn handle_queue_size(
    redis_pool: &deadpool_redis::Pool,
) -> serde_json::Value {
    match matchmaking::queue_size(redis_pool, None).await {
        Ok(size) => {
            json!({
                "status": 200,
                "queue_size": size,
            })
        }
        Err(e) => {
            error!("Failed to get queue size: {}", e);
            json_error(500, "Failed to get queue size")
        }
    }
}

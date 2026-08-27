use crate::db::db::Database;
use crate::http::response::json_error;
use crate::services::friend;
use crate::types::ServiceRequest;
use api_core::auth::validate_access_token;
use deadpool_redis::{Connection, Pool};
use notification::event::NotificationBus;
use serde_json::json;
use tracing::error;
use uuid::Uuid;

pub async fn handle_remove_friend(
    db: &Database,
    conn: &mut Connection,
    _redis_pool: &Pool,
    request: &ServiceRequest,
    friend_id_str: &str,
    notification_bus: &NotificationBus,
) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(token) => token,
        None => return json_error(401, "Missing access token"),
    };

    let claims = match validate_access_token(conn, token).await {
        Ok(claims) => claims,
        Err(e) => {
            tracing::warn!("[User] Failed to validate token for remove friend: {}", e);
            return json_error(401, &e);
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid user ID in token"),
    };

    let friend_id = match Uuid::parse_str(friend_id_str) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid friend ID"),
    };

    match friend::remove_friend(db.get_pool(), notification_bus, &user_id, &friend_id).await {
        Ok(_) => json!({
            "status": 200,
            "message": "Friend removed"
        }),
        Err(e) => {
            error!("[User] Failed to remove friend: {}", e);
            json_error(500, "Failed to remove friend")
        }
    }
}

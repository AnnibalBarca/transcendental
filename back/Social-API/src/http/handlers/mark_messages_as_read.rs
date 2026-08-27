use crate::db::db::Database;
use crate::http::response::json_error;
use crate::services::friend;
use crate::types::ServiceRequest;
use api_core::auth::validate_access_token;
use deadpool_redis::{Connection, Pool};
use serde_json::json;
use tracing::{error, warn};
use uuid::Uuid;

pub async fn handle_mark_messages_as_read(
    db: &Database,
    conn: &mut Connection,
    redis_pool: &Pool,
    request: &ServiceRequest,
    friend_id: &str,
) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(token) => token,
        None => return json_error(401, "Missing access token"),
    };

    let claims = match validate_access_token(conn, token).await {
        Ok(claims) => claims,
        Err(e) => {
            warn!("[User] Failed to validate token for mark read: {}", e);
            return json_error(401, &e);
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid user ID in token"),
    };

    let friend_uuid = match Uuid::parse_str(friend_id) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid friend ID"),
    };

    match friend::mark_messages_as_read(db.get_pool(), &user_id, &friend_uuid).await {
        Ok(count) => json!({
            "status": 200,
            "marked_as_read": count
        }),
        Err(e) => {
            error!("[User] Failed to mark messages as read: {}", e);
            json_error(500, "Failed to mark messages as read")
        }
    }
}

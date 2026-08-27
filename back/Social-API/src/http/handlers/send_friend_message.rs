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

pub async fn handle_send_friend_message(
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
            tracing::warn!("[User] Failed to validate token for send message: {}", e);
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

    let body_data: serde_json::Value = match serde_json::from_str(&request.body) {
        Ok(data) => data,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    let content = match body_data.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.trim(),
        None => return json_error(400, "Missing content field"),
    };

    if content.is_empty() {
        return json_error(400, "Message content cannot be empty");
    }

    match friend::send_message(
        db.get_pool(),
        notification_bus,
        &user_id,
        &friend_id,
        content,
    )
    .await
    {
        Ok(message) => json!({
            "status": 200,
            "message": message
        }),
        Err(e) => {
            error!("[User] Failed to send message: {}", e);
            json_error(500, "Failed to send message")
        }
    }
}

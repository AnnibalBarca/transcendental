use crate::db::db::Database;
use crate::http::response::json_error;
use crate::services::friend;
use crate::types::ServiceRequest;
use api_core::auth::validate_access_token;
use deadpool_redis::{Connection, Pool};
use serde_json::json;
use tracing::error;
use uuid::Uuid;

pub async fn handle_get_friend_messages(
    db: &Database,
    conn: &mut Connection,
    redis_pool: &Pool,
    request: &ServiceRequest,
    friend_id_str: &str,
) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(token) => token,
        None => return json_error(401, "Missing access token"),
    };

    let claims = match validate_access_token(conn, token).await {
        Ok(claims) => claims,
        Err(e) => {
            tracing::warn!("[User] Failed to validate token for get messages: {}", e);
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

    let limit = request
        .body
        .parse::<serde_json::Value>()
        .ok()
        .and_then(|v| v.get("limit").and_then(|l| l.as_i64()))
        .unwrap_or(50);

    let offset = request
        .body
        .parse::<serde_json::Value>()
        .ok()
        .and_then(|v| v.get("offset").and_then(|o| o.as_i64()))
        .unwrap_or(0);

    match friend::get_messages_between(
        db.get_pool(),
        redis_pool,
        &user_id,
        &friend_id,
        limit,
        offset,
    )
    .await
    {
        Ok(messages) => json!({
            "status": 200,
            "messages": messages
        }),
        Err(e) => {
            error!("[User] Failed to get messages: {}", e);
            json_error(500, "Failed to get messages")
        }
    }
}

use crate::db::db::Database;
use crate::http::response::json_error;
use crate::services::friend;
use crate::types::ServiceRequest;
use api_core::auth::validate_access_token;
use deadpool_redis::{Connection, Pool};
use serde_json::json;
use tracing::error;
use uuid::Uuid;

pub async fn handle_get_friends_list(
    db: &Database,
    conn: &mut Connection,
    redis_pool: &Pool,
    request: &ServiceRequest,
) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(token) => token,
        None => return json_error(401, "Missing access token"),
    };

    let claims = match validate_access_token(conn, token).await {
        Ok(claims) => claims,
        Err(e) => {
            tracing::warn!(
                "[User] Failed to validate token for get friends list: {}",
                e
            );
            return json_error(401, &e);
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid user ID in token"),
    };

    match friend::get_friends(db.get_pool(), redis_pool, &user_id).await {
        Ok(friends) => json!({
            "status": 200,
            "friends": friends
        }),
        Err(e) => {
            error!("[User] Failed to get friends list: {}", e);
            json_error(500, "Failed to get friends list")
        }
    }
}

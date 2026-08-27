use crate::db::db::Database;
use crate::db::user;
use crate::http::response::json_error;
use crate::services::friend;
use crate::types::ServiceRequest;
use api_core::auth::validate_access_token;
use deadpool_redis::{Connection, Pool};
use notification::event::NotificationBus;
use serde_json::json;
use tracing::error;
use uuid::Uuid;

pub async fn handle_request_friend(
    db: &Database,
    conn: &mut Connection,
    redis_pool: &Pool,
    request: &ServiceRequest,
    notification_bus: &NotificationBus,
) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(token) => token,
        None => return json_error(401, "Missing access token"),
    };

    let claims = match validate_access_token(conn, token).await {
        Ok(claims) => claims,
        Err(e) => {
            tracing::warn!("[User] Failed to validate token for friend request: {}", e);
            return json_error(401, &e);
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid user ID in token"),
    };

    let body_data: serde_json::Value = match serde_json::from_str(&request.body) {
        Ok(data) => data,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    let friend_username = match body_data.get("friend_username").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => return json_error(400, "Missing friend_username field"),
    };

    let friend_record = match user::get_by_username(db.get_pool(), friend_username).await {
        Ok(Some(record)) => record,
        Ok(None) => return json_error(404, "User not found"),
        Err(e) => {
            error!("[User] DB error looking up username: {}", e);
            return json_error(500, "Database error");
        }
    };

    let friend_id = match Uuid::parse_str(&friend_record.id) {
        Ok(id) => id,
        Err(_) => return json_error(500, "Invalid user ID in database"),
    };

    if user_id == friend_id {
        return json_error(400, "Cannot send friend request to yourself");
    }

    match friend::get_request_status(db.get_pool(), redis_pool, &user_id, &friend_id).await {
        Ok(Some(status)) => {
            if status == "pending" {
                return json_error(409, "Friend request already sent");
            }
            if status == "accepted" {
                return json_error(409, "Already friends");
            }
            if status == "blocked" {
                return json_error(404, "User not found");
            }
        }
        Ok(None) => {}
        Err(e) => {
            error!("[User] DB error checking request status: {}", e);
            return json_error(500, "Database error");
        }
    }

    match friend::get_request_status(db.get_pool(), redis_pool, &friend_id, &user_id).await {
        Ok(Some(status)) => {
            if status == "blocked" {
                return json_error(400, "Cannot send friend request to this user");
            }
        }
        Ok(None) => {}
        Err(e) => {
            error!("[User] DB error checking reverse request status: {}", e);
            return json_error(500, "Database error");
        }
    }

    match friend::send_request(db.get_pool(), notification_bus, &user_id, &friend_id).await {
        Ok(_) => json!({
            "status": 200,
            "message": "Friend request sent"
        }),
        Err(e) => {
            error!("[User] Failed to send friend request: {}", e);
            json_error(500, "Failed to send friend request")
        }
    }
}

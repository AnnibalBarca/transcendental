use api_core::auth::validate_and_get_claims;
use api_core::http::response::json_error;
use serde_json::json;
use log::error;
use uuid::Uuid;

use crate::services::user::{get_user_by_id, name_is_available, update_name};
use crate::AppContext;
use api_core::types::ServiceRequest;

pub async fn handle_change_username(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(token) => token,
        None => return json_error(401, "Missing access token"),
    };

    let mut conn = match ctx.redis_pool.get().await {
        Ok(c) => c,
        Err(_) => return json_error(500, "Internal error"),
    };

    let claims = match validate_and_get_claims(&mut conn, token).await {
        Ok(claims) => claims,
        Err(e) => {
            log::warn!("[User] Failed to validate token for change_username: {}", e);
            return json_error(401, &e);
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid user ID in token"),
    };

    let player_session_option = match ctx.session_manager.get_session(&user_id).await {
        Ok(session) => session,
        Err(e) => {
            error!("[Session] Error fetching session: {}", e);
            return json_error(500, "Internal server error while fetching session");
        }
    };

    if let Some(session) = player_session_option {
        if session.status != "none" {
            return json_error(403, "You cannot change your username while in a game.");
        }
    }

    let body_data: serde_json::Value = match serde_json::from_str(&request.body) {
        Ok(data) => data,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    let new_name = match body_data.get("name").and_then(|n| n.as_str()) {
        Some(name) => name,
        None => return json_error(400, "Missing name field in body"),
    };

    let new_name = match api_core::username::validate_username(new_name) {
        Ok(name) => name,
        Err(msg) => return json_error(400, msg),
    };
    let new_name = new_name.as_str();

    let current_user = match get_user_by_id(&user_id, ctx.db.get_pool(), &ctx.redis_pool).await {
        Ok(Some(user)) => user,
        Ok(None) => return json_error(404, "User not found"),
        Err(e) => {
            error!("[User] DB error while fetching user: {}", e);
            return json_error(500, "Database error");
        }
    };

    if current_user.username.as_deref() == Some(new_name) {
        return json!({
            "status": 200,
            "message": "Username is already set to this value"
        });
    }

    match name_is_available(new_name, ctx.db.get_pool()).await {
        Ok(false) => return json_error(409, "Username is already in use"),
        Ok(true) => {}
        Err(e) => {
            error!(
                "[User] DB error while checking username availability: {}",
                e
            );
            return json_error(500, "Database error");
        }
    }

    match update_name(&user_id, new_name, ctx.db.get_pool(), &ctx.redis_pool).await {
        Ok(_) => json!({
            "status": 200,
            "message": "Username updated successfully"
        }),
        Err(e) => {
            error!("[User] Failed to update username: {}", e);
            json_error(500, "Failed to update username")
        }
    }
}

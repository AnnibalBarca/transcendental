use api_core::auth::validate_and_get_claims;
use api_core::http::response::json_error;
use serde_json::json;
use uuid::Uuid;

use crate::types::UserStatePayload;
use crate::AppContext;
use api_core::types::ServiceRequest;

pub async fn handle_state(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
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
            log::warn!("[User] Failed to validate token for state: {}", e);
            return json_error(401, &e);
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid user ID in token"),
    };

    match ctx.session_manager.get_session(&user_id).await {
        Ok(Some(session)) => {
            let payload = UserStatePayload {
                state: session.status,
                room_id: Uuid::parse_str(&session.room_id).unwrap_or_default(),
            };
            json!({
                "status": 200,
                "state": payload
            })
        }
        Ok(None) => json_error(404, "No active session found for user"),
        Err(e) => {
            log::error!("[User] Failed to get session: {}", e);
            json_error(500, "Internal server error")
        }
    }
}

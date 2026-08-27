use crate::{AppContext, services::cosmetic};
use api_core::{auth::validate_and_get_claims, http::response::json_error, types::ServiceRequest};
use log::{error, warn};
use serde_json::json;
use uuid::Uuid;

pub async fn handle_get_profile_picture(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(token) => token,
        None => return json_error(401, "Missing access token"),
    };

    let mut conn = match ctx.redis_pool.get().await {
        Ok(conn) => conn,
        Err(e) => return json_error(500, &format!("Redis connection error: {}", e)),
    };

    let claims = match validate_and_get_claims(&mut conn, token).await {
        Ok(claims) => claims,
        Err(e) => {
            warn!("[User] Failed to validate token for get inventory: {}", e);
            return json_error(401, &e);
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid user ID in token"),
    };

    match cosmetic::get_profile_picture(ctx.db.get_pool(), &ctx.redis_pool, &user_id).await {
        Ok(Some(url)) => json!({
            "status": 200,
            "picture_id": url
        }),
        Ok(None) => json!({
            "status": 200,
            "picture_id": serde_json::Value::Null
        }),
        Err(e) => {
            error!("[User] Failed to get profile picture: {}", e);
            json_error(500, "Failed to get profile picture")
        }
    }
}

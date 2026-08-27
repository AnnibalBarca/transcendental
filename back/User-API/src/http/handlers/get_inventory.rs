use crate::AppContext;
use crate::services::cosmetic;
use api_core::auth::validate_and_get_claims;
use api_core::http::response::json_error;
use api_core::types::ServiceRequest;

use log::error;
use serde_json::json;
use uuid::Uuid;

pub async fn handle_get_inventory(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
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
            log::warn!("[User] Failed to validate token for get inventory: {}", e);
            return json_error(401, &e);
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid user ID in token"),
    };

    let items = match cosmetic::get_inventory(ctx.db.get_pool(), &ctx.redis_pool, &user_id).await {
        Ok(items) => items,
        Err(e) => {
            error!("[User] Failed to get inventory: {}", e);
            return json_error(500, "Failed to get inventory");
        }
    };

    let items: Vec<serde_json::Value> = items
        .into_iter()
        .map(|item| {
            json!({
                "id": item.id,
                "user_id": item.user_id,
                "item_id": item.item_id,
                "item_type": item.item_type,
                "created_at": item.created_at,
            })
        })
        .collect();

    json!({
        "status": 200,
        "items": items
    })
}

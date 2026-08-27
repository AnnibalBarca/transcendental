use crate::cache;
use crate::db::shop;
use crate::AppContext;
use api_core::auth::validate_and_get_claims;
use api_core::http::response::json_error;
use api_core::types::ServiceRequest;
use log::error;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
struct ItemRequest {
    item_id: String,
    item_type: String,
}

pub async fn handle_remove_item(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(t) => t,
        None => return json_error(401, "Missing access token"),
    };
    let mut conn = match ctx.redis_pool.get().await {
        Ok(c) => c,
        Err(_) => return json_error(500, "Redis connection error"),
    };
    let claims = match validate_and_get_claims(&mut conn, token).await {
        Ok(c) => c,
        Err(e) => return json_error(401, &e),
    };
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid user ID in token"),
    };

    let body: ItemRequest = match serde_json::from_str(&request.body) {
        Ok(b) => b,
        Err(_) => return json_error(400, "Invalid request body"),
    };

    match shop::refund(ctx.db.get_pool(), &user_id, &body.item_id, &body.item_type).await {
        Ok(new_wallet) => {
            let _ = cache::user::invalidate_cached_user(&ctx.redis_pool, &user_id).await;
            let _ = cache::cosmetic::invalidate_inventory(&ctx.redis_pool, &user_id).await;
            json!({
                "status": 200,
                "message": "Item refunded",
                "item_id": body.item_id,
                "item_type": body.item_type,
                "wallet": new_wallet
            })
        }
        Err(e) => {
            error!("[Shop] refund failed: {}", e);
            match e.as_str() {
                "not owned" => json_error(404, "You do not own this item"),
                _ => json_error(500, "Refund failed"),
            }
        }
    }
}

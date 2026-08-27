use api_core::auth::validate_and_get_claims;
use api_core::http::response::json_error;
use api_core::types::ServiceRequest;
use log::{error, info};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::cache;
use crate::db::pack;
use crate::AppContext;

#[derive(Deserialize)]
struct OpenPackRequest {
    pack_type: String,
}

const SKIN_1_PRICE: i64 = 150;
const SKIN_3_PRICE: i64 = 400;
const CARD_2_PRICE: i64 = 250;
const CARD_5_PRICE: i64 = 550;

const SKIN_DUP_REFUND: i64 = 30;
const CARD_DUP_REFUND: i64 = 50;

fn pack_def(pack_type: &str) -> Option<(&'static str, usize, i64, i64)> {
    match pack_type {
        "skin_1" => Some(("skin", 1, SKIN_1_PRICE, SKIN_DUP_REFUND)),
        "skin_3" => Some(("skin", 3, SKIN_3_PRICE, SKIN_DUP_REFUND)),
        "card_2" => Some(("card", 2, CARD_2_PRICE, CARD_DUP_REFUND)),
        "card_5" => Some(("card", 5, CARD_5_PRICE, CARD_DUP_REFUND)),
        _ => None,
    }
}

pub async fn handle_open_pack(ctx: &AppContext, request: &ServiceRequest) -> Value {
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

    let body: OpenPackRequest = match serde_json::from_str(&request.body) {
        Ok(b) => b,
        Err(_) => return json_error(400, "Invalid request body"),
    };

    let (kind, count, price, dup_refund) = match pack_def(&body.pack_type) {
        Some(def) => def,
        None => return json_error(400, "Unknown pack_type"),
    };

    let result = if kind == "skin" {
        pack::open_skins(ctx.db.get_pool(), &user_id, count, price, dup_refund).await
    } else {
        pack::open_cards(ctx.db.get_pool(), &user_id, count, price, dup_refund).await
    };

    match result {
        Ok((wallet, rewards)) => {
            let _ = cache::user::invalidate_cached_user(&ctx.redis_pool, &user_id).await;
            let _ = cache::cosmetic::invalidate_inventory(&ctx.redis_pool, &user_id).await;

            let mut reward_json = Vec::with_capacity(rewards.len());
            for reward in rewards {
                let mut obj = json!({
                    "item_id": reward.item_id,
                    "item_type": reward.item_type,
                    "title": reward.title,
                    "price": reward.price,
                    "is_duplicate": reward.is_duplicate,
                    "refunded": reward.refunded,
                });
                if kind == "card" {
                    obj["rarity"] = json!(reward.rarity);
                }
                reward_json.push(obj);
            }

            info!("[Pack] User {} opened {} pack ({} items)", user_id, body.pack_type, count);

            json!({
                "status": 200,
                "pack_type": body.pack_type,
                "wallet": wallet,
                "rewards": reward_json,
            })
        }
        Err(e) => {
            error!("[Pack] open failed: {}", e);
            match e.as_str() {
                "insufficient funds" => json_error(409, "Insufficient funds"),
                _ => json_error(500, "Failed to open pack"),
            }
        }
    }
}

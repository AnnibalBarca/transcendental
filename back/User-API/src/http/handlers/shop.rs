use api_core::auth::validate_and_get_claims;
use api_core::http::response::json_error;
use api_core::types::ServiceRequest;
use log::{error, info};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::cache;
use crate::db::{collection, shop};
use crate::services::storage::{extension_for, is_valid_slot, Storage, SLOTS};
use crate::services::{cosmetic, user};
use crate::AppContext;

const MAX_IMAGE_BYTES: usize = 2 * 1024 * 1024;

/// GET /api/user/shop — the shop screen's one and only data fetch (called
/// from front's shopService.getShop()). Always returns the public catalog
/// + collections + slots + the hardcoded `packs` list (that last one added
/// later by a teammate, not part of my original handler). If — and only
/// if — the request carries a valid access_token cookie, it *also* merges
/// in the caller's wallet balance and owned-items list, so a logged-out
/// visitor still sees prices but no "owned"/wallet state. That's a
/// deliberate soft-auth pattern (unlike handle_purchase_collection below,
/// which hard-requires a session).
pub async fn handle_get_shop(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
    let mut items = match shop::list_active(ctx.db.get_pool()).await {
        Ok(items) => items,
        Err(e) => {
            error!("[Shop] list_active failed: {}", e);
            return json_error(500, "Failed to load shop");
        }
    };

    let mut collections = match collection::list_all(ctx.db.get_pool()).await {
        Ok(cols) => cols,
        Err(e) => {
            error!("[Shop] list_collections failed: {}", e);
            return json_error(500, "Failed to load shop");
        }
    };

    let mut response = json!({
        "status": 200,
        "items": items,
        "collections": collections,
        "slots": SLOTS,
        "packs": [
            { "type": "skin_1", "kind": "skin", "label": "Pack Skin", "count": 1, "price": 150 },
            { "type": "skin_3", "kind": "skin", "label": "Pack Skin x3", "count": 3, "price": 400 },
            { "type": "card_2", "kind": "card", "label": "Pack Cartes", "count": 2, "price": 250 },
            { "type": "card_5", "kind": "card", "label": "Pack Cartes x5", "count": 5, "price": 550 }
        ],
    });

    if let Some(user_id) = authed_user_id(ctx, request).await {
        if let Ok(Some(record)) =
            user::get_user_by_id(&user_id, ctx.db.get_pool(), &ctx.redis_pool).await
        {
            response["wallet"] = json!(record.wallet);
        }
        if let Ok(inventory) =
            cosmetic::get_inventory(ctx.db.get_pool(), &ctx.redis_pool, &user_id).await
        {
            let owned: Vec<_> = inventory
                .iter()
                .map(|i| json!({ "item_id": i.item_id, "item_type": i.item_type }))
                .collect();
            response["owned"] = json!(owned);
        }
    }

    response
}

// GET /api/user/collections — a lighter, no-auth-fields variant of the
// collections half of handle_get_shop. Registered in router.rs but not
// actually called anywhere in the current front (shopService only calls
// GET /shop, which already includes `collections`) — effectively unused
// today, kept for API completeness / possible future direct use.
pub async fn handle_get_collections(ctx: &AppContext) -> serde_json::Value {
    let collections = match collection::list_all(ctx.db.get_pool()).await {
        Ok(cols) => cols,
        Err(e) => {
            error!("[Shop] list_collections failed: {}", e);
            return json_error(500, "Failed to load collections");
        }
    };

    json!({ "status": 200, "collections": collections })
}

#[derive(Deserialize)]
struct CollectionPurchaseRequest {
    collection_id: String,
}

// POST /api/user/collections/purchase — hard-auth (unlike handle_get_shop):
// no valid session -> immediate 401, never reaches the DB. Delegates all
// the money/ownership logic to db::shop::purchase_collection and only
// translates its Err(&str) reasons into the right HTTP status per case.
// On success it also busts two Redis caches (user + inventory) so the
// *next* request — from this tab or another — doesn't serve stale
// pre-purchase data.
pub async fn handle_purchase_collection(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> serde_json::Value {
    let user_id = match authed_user_id(ctx, request).await {
        Some(id) => id,
        None => return json_error(401, "Missing or invalid access token"),
    };

    let body: CollectionPurchaseRequest = match serde_json::from_str(&request.body) {
        Ok(b) => b,
        Err(_) => return json_error(400, "Invalid request body"),
    };
    let collection_id = match Uuid::parse_str(&body.collection_id) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid collection_id"),
    };

    match shop::purchase_collection(ctx.db.get_pool(), &user_id, &collection_id).await {
        Ok((new_wallet, granted)) => {
            let _ = cache::user::invalidate_cached_user(&ctx.redis_pool, &user_id).await;
            let _ = cache::cosmetic::invalidate_inventory(&ctx.redis_pool, &user_id).await;

            let granted: Vec<_> = granted
                .into_iter()
                .map(|(item_id, item_type)| json!({ "item_id": item_id, "item_type": item_type }))
                .collect();

            json!({
                "status": 200,
                "message": "Collection purchased",
                "collection_id": collection_id.to_string(),
                "granted": granted,
                "wallet": new_wallet,
            })
        }
        Err(e) => {
            error!("[Shop] collection purchase failed: {}", e);
            match e.as_str() {
                "collection not available" => json_error(404, "Collection not available"),
                "empty collection" => json_error(409, "This collection has no items"),
                "already owned" => json_error(409, "You already own every item of this collection"),
                "insufficient funds" => json_error(409, "Insufficient funds"),
                _ => json_error(500, "Purchase failed"),
            }
        }
    }
}

#[derive(Deserialize)]
struct UploadItemRequest {
    item_id: String,
    item_type: String,
    title: String,
    price: i64,
    image_base64: String,
    #[serde(default = "default_content_type")]
    content_type: String,
    #[serde(default)]
    collection_id: Option<String>,
}

fn default_content_type() -> String {
    "image/png".to_string()
}

pub async fn handle_upload_item(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
    let storage = match &ctx.storage {
        Some(s) => s,
        None => return json_error(503, "Object storage is not configured"),
    };

    let body: UploadItemRequest = match serde_json::from_str(&request.body) {
        Ok(b) => b,
        Err(e) => {
            error!("[Shop] upload body parse failed: {}", e);
            return json_error(400, "Invalid request body");
        }
    };

    if !is_valid_slot(&body.item_type) {
        return json_error(
            400,
            &format!("item_type must be one of {}", SLOTS.join(", ")),
        );
    }
    if let Err(e) = validate_slot_id(&body.item_id) {
        return json_error(400, &e);
    }
    if body.price < 0 {
        return json_error(400, "price must be positive");
    }

    let ext = match extension_for(&body.content_type) {
        Ok(ext) => ext,
        Err(e) => return json_error(400, &e),
    };

    let bytes = match decode_base64(&body.image_base64) {
        Ok(b) => b,
        Err(e) => return json_error(400, &e),
    };
    if bytes.is_empty() {
        return json_error(400, "Empty image");
    }
    if bytes.len() > MAX_IMAGE_BYTES {
        return json_error(413, "Image too large (max 2 MB)");
    }

    let key = Storage::object_key(&body.item_type, &body.item_id, ext);
    let image_url = match storage.put_object(&key, bytes, &body.content_type).await {
        Ok(url) => url,
        Err(e) => {
            error!("[Shop] MinIO upload of {} failed: {}", key, e);
            return json_error(502, "Failed to store image");
        }
    };

    if let Err(e) = shop::upsert_item(
        ctx.db.get_pool(),
        &body.item_id,
        &body.item_type,
        &body.title,
        body.price,
        &key,
    )
    .await
    {
        error!("[Shop] upsert_item failed: {}", e);
        return json_error(500, "Failed to register item");
    }

    let mut attached: Option<String> = None;
    if let Some(raw) = body.collection_id.as_deref().filter(|s| !s.is_empty()) {
        let collection_id = match Uuid::parse_str(raw) {
            Ok(id) => id,
            Err(_) => return json_error(400, "Invalid collection_id"),
        };
        let position = SLOTS
            .iter()
            .position(|s| *s == body.item_type)
            .unwrap_or(0) as i16;

        if let Err(e) = collection::add_item(
            ctx.db.get_pool(),
            &collection_id,
            &body.item_id,
            &body.item_type,
            position,
        )
        .await
        {
            error!("[Shop] collection attach failed: {}", e);
            return json_error(500, "Item stored but could not be attached to the collection");
        }
        attached = Some(collection_id.to_string());
    }

    info!(
        "[Shop] Stored {} in bucket '{}' -> {}",
        key,
        storage.bucket(),
        image_url
    );

    json!({
        "status": 200,
        "message": "Item uploaded",
        "item_id": body.item_id,
        "item_type": body.item_type,
        "asset_key": key,
        "collection_id": attached,
    })
}

fn validate_slot_id(item_id: &str) -> Result<(), String> {
    match item_id.parse::<u16>() {
        Ok(v) if (1..=4095).contains(&v) => Ok(()),
        _ => Err("item_id must be a number between 1 and 4095".to_string()),
    }
}

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let payload = match input.find(";base64,") {
        Some(idx) => &input[idx + ";base64,".len()..],
        None => input,
    };

    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut out = Vec::with_capacity(payload.len() / 4 * 3);
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;

    for byte in payload.bytes() {
        if byte.is_ascii_whitespace() || byte == b'=' {
            continue;
        }
        let value = match ALPHABET.iter().position(|c| *c == byte) {
            Some(v) => v as u32,
            None => return Err("Invalid base64 image".to_string()),
        };
        buffer = (buffer << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
        }
    }

    Ok(out)
}

async fn authed_user_id(ctx: &AppContext, request: &ServiceRequest) -> Option<Uuid> {
    let token = request.cookies.get("access_token")?;
    let mut conn = ctx.redis_pool.get().await.ok()?;
    let claims = validate_and_get_claims(&mut conn, token).await.ok()?;
    Uuid::parse_str(&claims.sub).ok()
}

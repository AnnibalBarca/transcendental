use crate::db::cosmetic::{has_equiped_items_in_inventory, InventoryItem};
use crate::services::cosmetic;
use crate::AppContext;
use api_core::auth::validate_and_get_claims;
use api_core::db::Database;
use api_core::http::response::json_error;
use api_core::types::ServiceRequest;
use chrono::DateTime;

use serde_json::json;
use log::error;
use uuid::Uuid;

pub async fn handle_set_profile_picture(
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
            log::warn!(
                "[User] Failed to validate token for set profile picture: {}",
                e
            );
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

    let picture_id = match body_data.get("picture_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return json_error(400, "Missing picture_id field"),
    };

    let id = match base62_to_id(picture_id) {
        Some(id) => id,
        None => return json_error(400, "Invalid picture_id"),
    };

    match check_equipement_validity(id, user_id, &ctx.db).await {
        Ok(true) => {}
        Ok(false) => return json_error(400, "Invalid equipment"),
        Err(e) => {
            error!("[User] Failed to validate equipment: {}", e);
            return json_error(500, "Failed to validate equipment");
        }
    }

    match cosmetic::set_profile_picture(ctx.db.get_pool(), &ctx.redis_pool, &user_id, &picture_id)
        .await
    {
        Ok(_) => json!({
            "status": 200,
            "message": "Profile picture updated"
        }),
        Err(e) => {
            error!("[User] Failed to set profile picture: {}", e);
            json_error(500, "Failed to set profile picture")
        }
    }
}

fn base62_to_id(chaine: &str) -> Option<u64> {
    if chaine.is_empty() || chaine.len() > 11 {
        return None;
    }

    let alphabet = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut id: u64 = 0;

    for c in chaine.chars() {
        let valeur = alphabet.find(c)? as u64;
        id = id.checked_mul(62)?.checked_add(valeur)?;
    }

    return Some(id);
}

async fn check_equipement_validity(id: u64, user_id: Uuid, db: &Database) -> Result<bool, String> {
    let masque_12_bits = 0xFFF;

    let base = (id & masque_12_bits) as u16;
    let hat = ((id >> 12) & masque_12_bits) as u16;
    let mask = ((id >> 24) & masque_12_bits) as u16;
    let clothes = ((id >> 36) & masque_12_bits) as u16;
    let accessory = (id >> 48) as u16;

    let mut items: Vec<InventoryItem> = Vec::with_capacity(5);

    if base != 0 {
        items.push(InventoryItem {
            id: Uuid::nil(),
            user_id,
            item_id: base.to_string(),
            item_type: "base".to_string(),
            created_at: DateTime::UNIX_EPOCH,
        });
    }

    if hat != 0 {
        items.push(InventoryItem {
            id: Uuid::nil(),
            user_id,
            item_id: hat.to_string(),
            item_type: "hat".to_string(),
            created_at: DateTime::UNIX_EPOCH,
        });
    }

    if mask != 0 {
        items.push(InventoryItem {
            id: Uuid::nil(),
            user_id,
            item_id: mask.to_string(),
            item_type: "mask".to_string(),
            created_at: DateTime::UNIX_EPOCH,
        });
    }

    if clothes != 0 {
        items.push(InventoryItem {
            id: Uuid::nil(),
            user_id,
            item_id: clothes.to_string(),
            item_type: "clothes".to_string(),
            created_at: DateTime::UNIX_EPOCH,
        });
    }

    if accessory != 0 {
        items.push(InventoryItem {
            id: Uuid::nil(),
            user_id,
            item_id: accessory.to_string(),
            item_type: "accessory".to_string(),
            created_at: DateTime::UNIX_EPOCH,
        });
    }

    if !has_equiped_items_in_inventory(db.get_pool(), items).await? {
        return Ok(false);
    }

    Ok(true)
}

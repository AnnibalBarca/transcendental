use deadpool_redis::redis::cmd;
use uuid::Uuid;

use crate::db::cosmetic::InventoryItem;

const COSMETIC_CACHE_TTL_SECS: usize = 600;
const PP_CACHE_TTL_SECS: usize = 900;

pub async fn get_inventory(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<Option<Vec<InventoryItem>>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::inventory(user_id);
    let cached: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    if let Some(json_str) = cached {
        if let Ok(items) = serde_json::from_str::<Vec<InventoryItem>>(&json_str) {
            return Ok(Some(items));
        }
    }
    Ok(None)
}

pub async fn set_inventory(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    items: &[InventoryItem],
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::inventory(user_id);
    let json_str =
        serde_json::to_string(items).map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(COSMETIC_CACHE_TTL_SECS)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn invalidate_inventory(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::inventory(user_id);
    let _: () = cmd("DEL")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;

    Ok(())
}

pub async fn get_profile_picture(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<Option<String>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::profile_picture(user_id);
    let cached: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    Ok(cached)
}

pub async fn set_profile_picture(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    picture_id: &str,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::profile_picture(user_id);
    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(PP_CACHE_TTL_SECS)
        .arg(picture_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn invalidate_profile_picture(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::profile_picture(user_id);
    let _: () = cmd("DEL")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;

    Ok(())
}

pub async fn invalidate_social_caches(
    pool: &deadpool_redis::Pool,
    friend_ids: &[Uuid],
) -> Result<(), String> {
    if friend_ids.is_empty() {
        return Ok(());
    }

    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let mut keys = Vec::new();
    for id in friend_ids {
        keys.push(super::keys::friend_list(id));
        keys.push(super::keys::friend_requests(id));
        keys.push(super::keys::friend_sent(id));
        keys.push(super::keys::friend_blocked(id));
    }

    let _: () = cmd("DEL")
        .arg(&keys)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;

    Ok(())
}

pub async fn invalidate_all(pool: &deadpool_redis::Pool, user_id: &Uuid) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let keys = vec![
        super::keys::inventory(user_id),
        super::keys::profile_picture(user_id),
    ];

    let _: () = cmd("DEL")
        .arg(&keys)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;

    Ok(())
}

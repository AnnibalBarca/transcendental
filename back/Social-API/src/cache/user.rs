use deadpool_redis::redis::cmd;
use uuid::Uuid;

use crate::db::db::UserRecord;

const USER_CACHE_TTL_SECS: usize = 900;

pub async fn get(pool: &deadpool_redis::Pool, id: &Uuid) -> Result<Option<UserRecord>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::user_profile(id);

    let cached_data: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    if let Some(json_str) = cached_data {
        if let Ok(user) = serde_json::from_str::<UserRecord>(&json_str) {
            return Ok(Some(user));
        }
    }

    Ok(None)
}

pub async fn set(pool: &deadpool_redis::Pool, user: &UserRecord, id: &Uuid) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::user_profile(id);
    let json_str =
        serde_json::to_string(user).map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(USER_CACHE_TTL_SECS)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn invalidate(pool: &deadpool_redis::Pool, id: &Uuid) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::user_profile(id);

    let _: () = cmd("DEL")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;

    Ok(())
}

use std::fmt::Display;

use deadpool_redis::redis::cmd;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub async fn get_json<T>(pool: &deadpool_redis::Pool, key: &str) -> Result<Option<T>, String>
where
    T: DeserializeOwned,
{
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let cached_data: Option<String> = cmd("GET")
        .arg(key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    if let Some(json_str) = cached_data {
        if let Ok(value) = serde_json::from_str::<T>(&json_str) {
            return Ok(Some(value));
        }
    }

    Ok(None)
}

pub async fn set_json<T>(
    pool: &deadpool_redis::Pool,
    key: &str,
    value: &T,
    ttl_seconds: usize,
) -> Result<(), String>
where
    T: Serialize,
{
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let json_str =
        serde_json::to_string(value).map_err(|e| format!("Serialization failed: {}", e))?;

    cmd("SETEX")
        .arg(key)
        .arg(ttl_seconds)
        .arg(json_str)
        .query_async::<_, ()>(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn invalidate(pool: &deadpool_redis::Pool, key: &str) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    cmd("DEL")
        .arg(key)
        .query_async::<_, ()>(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;

    Ok(())
}

pub fn key<K: Display>(parts: &[K]) -> String {
    parts
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(":")
}

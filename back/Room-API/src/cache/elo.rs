use deadpool_redis::redis::cmd;
use uuid::Uuid;

const ELO_CACHE_TTL_SECS: usize = 300;

pub async fn get(pool: &deadpool_redis::Pool, user_id: &Uuid) -> Result<Option<i32>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = format!("elo:{}", user_id);

    let val: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    Ok(val.and_then(|s| s.parse().ok()))
}

pub async fn set(pool: &deadpool_redis::Pool, user_id: &Uuid, elo: i32) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = format!("elo:{}", user_id);

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(ELO_CACHE_TTL_SECS)
        .arg(elo.to_string())
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

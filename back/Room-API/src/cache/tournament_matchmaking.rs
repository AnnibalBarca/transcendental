use deadpool_redis::redis::cmd;

use crate::cache::tournament::PLAYER_SIZES;

const TOURNAMENT_MATCHMAKING_PREFIX: &str = "tournament_matchmaking:pool";
const TOURNAMENT_MATCHMAKING_META_PREFIX: &str = "tournament_matchmaking:meta";

fn pool_key(player_size: u32) -> String {
    format!("{}:{}", TOURNAMENT_MATCHMAKING_PREFIX, player_size)
}

fn meta_key(player_size: u32) -> String {
    format!("{}:{}", TOURNAMENT_MATCHMAKING_META_PREFIX, player_size)
}

pub fn valid_player_size(size: u32) -> bool {
    PLAYER_SIZES.contains(&size)
}

pub async fn add_player(
    pool: &deadpool_redis::Pool,
    user_id: &str,
    elo: i32,
    player_size: u32,
) -> Result<(), String> {
    if !valid_player_size(player_size) {
        return Err(format!("player_size must be one of {:?}", PLAYER_SIZES));
    }

    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let now = chrono::Utc::now().timestamp();
    let meta = serde_json::json!({
        "user_id": user_id,
        "elo": elo,
        "joined_at": now,
        "player_size": player_size,
    })
    .to_string();

    let _: () = cmd("ZADD")
        .arg(pool_key(player_size))
        .arg(elo)
        .arg(user_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis ZADD failed: {}", e))?;

    let _: () = cmd("HSET")
        .arg(meta_key(player_size))
        .arg(user_id)
        .arg(meta)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis HSET failed: {}", e))?;

    Ok(())
}

pub async fn remove_player(
    pool: &deadpool_redis::Pool,
    user_id: &str,
    player_size: u32,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let _: () = cmd("ZREM")
        .arg(pool_key(player_size))
        .arg(user_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis ZREM failed: {}", e))?;

    let _: () = cmd("HDEL")
        .arg(meta_key(player_size))
        .arg(user_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis HDEL failed: {}", e))?;

    Ok(())
}

pub async fn remove_player_from_all(
    pool: &deadpool_redis::Pool,
    user_id: &str,
) -> Result<(), String> {
    for size in PLAYER_SIZES.iter() {
        let _ = remove_player(pool, user_id, *size).await;
    }
    Ok(())
}

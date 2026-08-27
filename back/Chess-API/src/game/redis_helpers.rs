use log::{info, warn};
use serde_json::json;

pub async fn cleanup_user_session(redis_pool: &deadpool_redis::Pool, user_id: &str) {
    use deadpool_redis::redis::cmd;
    let key = format!("user:session:{}", user_id);
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            warn!("[Chess-WS] Failed to get redis conn for cleanup: {}", e);
            return;
        }
    };
    let _: Result<(), _> = cmd("HDEL")
        .arg(&key)
        .arg("chess_game_id")
        .arg("chess_ws_url")
        .query_async(&mut *conn)
        .await;
    let _: Result<(), _> = cmd("HSET")
        .arg(&key)
        .arg("status")
        .arg("none")
        .query_async(&mut *conn)
        .await;
    info!("[Chess-WS] Cleaned up session for user {}", user_id);
}

pub async fn persist_user_session(redis_pool: &deadpool_redis::Pool, user_id: &str, game_id: &str) {
    use deadpool_redis::redis::cmd;
    let key = format!("user:session:{}", user_id);
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            warn!("[Chess-WS] Failed to get redis conn for persist: {}", e);
            return;
        }
    };
    let _: Result<(), _> = cmd("HSET")
        .arg(&key)
        .arg("chess_game_id")
        .arg(game_id)
        .query_async(&mut *conn)
        .await;
}

pub async fn publish_game_result(
    redis_pool: &deadpool_redis::Pool,
    game_id: &str,
    result: &str,
    winner: Option<&str>,
    white_user_id: Option<&str>,
    black_user_id: Option<&str>,
) {
    use deadpool_redis::redis::cmd;
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            warn!("[Chess-WS] Failed to get redis conn to publish game result: {}", e);
            return;
        }
    };
    let payload = json!({
        "game_id": game_id,
        "result": result,
        "winner": winner,
        "white_user_id": white_user_id,
        "black_user_id": black_user_id,
    });
    let _: deadpool_redis::redis::RedisResult<()> = cmd("XADD")
        .arg("chess:game:events")
        .arg("*")
        .arg("data")
        .arg(payload.to_string())
        .query_async(&mut *conn)
        .await;
    info!("[Chess-WS] Published game result for {} ({})", game_id, result);
}

pub async fn get_profile_picture(redis_pool: &deadpool_redis::Pool, user_id: &str) -> Option<String> {
    if user_id.is_empty() {
        return None;
    }

    let mut conn = redis_pool.get().await.ok()?;
    let key = format!("user:profile:{}", user_id);
    let data: Option<String> = deadpool_redis::redis::cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .ok()?;

    let value: serde_json::Value = serde_json::from_str(&data?).ok()?;
    value
        .get("picture_id")
        .and_then(|p| p.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}
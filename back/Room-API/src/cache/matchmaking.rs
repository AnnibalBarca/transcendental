use deadpool_redis::redis::{cmd, Script};

pub const TIME_CONTROLS: [&str; 3] = ["5", "10", "15"];
pub const DEFAULT_TIME_CONTROL: &str = "10";

const MATCHMAKING_POOL_PREFIX: &str = "matchmaking:ranked:pool";
const MATCHMAKING_META_PREFIX: &str = "matchmaking:ranked:meta";

fn pool_key(time_control: &str) -> String {
    format!("{}:{}", MATCHMAKING_POOL_PREFIX, time_control)
}

fn meta_key(time_control: &str) -> String {
    format!("{}:{}", MATCHMAKING_META_PREFIX, time_control)
}

pub async fn add_player(
    pool: &deadpool_redis::Pool,
    user_id: &str,
    elo: i32,
    time_control: &str,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let now = chrono::Utc::now().timestamp();
    let meta = serde_json::json!({"elo": elo, "joined_at": now, "time_control": time_control}).to_string();

    let script = Script::new(
        r#"
        local added = redis.call('zadd', KEYS[1], ARGV[2], ARGV[1])
        if added == 1 then
            redis.call('hset', KEYS[2], ARGV[1], ARGV[3])
        end
        return added
        "#,
    );

    let _: i32 = script
        .key(pool_key(time_control))
        .key(meta_key(time_control))
        .arg(user_id)
        .arg(elo)
        .arg(meta)
        .invoke_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis add_player script failed: {}", e))?;

    Ok(())
}

pub async fn queue_size(
    pool: &deadpool_redis::Pool,
    time_control: Option<&str>,
) -> Result<usize, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let keys: Vec<String> = match time_control {
        Some(tc) => vec![pool_key(tc)],
        None => TIME_CONTROLS.iter().map(|tc| pool_key(tc)).collect(),
    };

    let mut total = 0usize;
    for key in keys {
        let size: usize = cmd("ZCARD")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| format!("Redis ZCARD failed: {}", e))?;
        total += size;
    }

    Ok(total)
}

pub async fn pop_two_players(
    pool: &deadpool_redis::Pool,
    time_control: &str,
) -> Result<Option<(String, String, i32, i32)>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let flat: Vec<String> = cmd("ZRANGE")
        .arg(pool_key(time_control))
        .arg(0)
        .arg(-1)
        .arg("WITHSCORES")
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis ZRANGE failed: {}", e))?;

    if flat.len() < 4 {
        return Ok(None);
    }

    let now = chrono::Utc::now().timestamp();
    let timeout_secs: i64 = 30;

    let mut entries: Vec<(String, i32, i64)> = Vec::new();
    for chunk in flat.chunks(2) {
        if chunk.len() == 2 {
            let user_id = chunk[0].clone();
            let elo: i32 = chunk[1].parse().unwrap_or(1500);
            let meta_json: String = cmd("HGET")
                .arg(meta_key(time_control))
                .arg(&user_id)
                .query_async(&mut *conn)
                .await
                .unwrap_or_default();
            let joined_at: i64 = serde_json::from_str::<serde_json::Value>(&meta_json)
                .ok()
                .and_then(|v| v.get("joined_at").and_then(|t| t.as_i64()))
                .unwrap_or(now);
            entries.push((user_id, elo, joined_at));
        }
    }

    if entries.len() < 2 {
        return Ok(None);
    }

    let pair = find_best_pair(&entries, now, timeout_secs);

    if let Some((i, j)) = pair {
        let p1 = &entries[i];
        let p2 = &entries[j];

        let _: i32 = cmd("ZREM")
            .arg(pool_key(time_control))
            .arg(&p1.0)
            .arg(&p2.0)
            .query_async(&mut *conn)
            .await
            .map_err(|e| format!("Redis ZREM failed: {}", e))?;

        let _: Result<i32, _> = cmd("HDEL")
            .arg(meta_key(time_control))
            .arg(&p1.0)
            .arg(&p2.0)
            .query_async(&mut *conn)
            .await;

        return Ok(Some((p1.0.clone(), p2.0.clone(), p1.1, p2.1)));
    }

    Ok(None)
}

fn find_best_pair(
    entries: &[(String, i32, i64)],
    now: i64,
    timeout_secs: i64,
) -> Option<(usize, usize)> {
    let timed_out: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| now - e.2 >= timeout_secs)
        .map(|(i, _)| i)
        .collect();

    if timed_out.len() >= 2 {
        return Some((timed_out[0], timed_out[1]));
    }

    if timed_out.len() == 1 && entries.len() >= 2 {
        let t = timed_out[0];
        let others: Vec<usize> = (0..entries.len()).filter(|&i| i != t).collect();
        return Some((t, others[0]));
    }

    let mut best_pair: Option<(usize, usize, i32)> = None;
    for i in 0..entries.len() {
        for j in (i + 1)..entries.len() {
            let diff = (entries[i].1 - entries[j].1).abs();
            match best_pair {
                Some((_, _, best_diff)) if diff < best_diff => {
                    best_pair = Some((i, j, diff));
                }
                None => {
                    best_pair = Some((i, j, diff));
                }
                _ => {}
            }
        }
    }

    best_pair.map(|(i, j, _)| (i, j))
}

pub async fn remove_player(pool: &deadpool_redis::Pool, user_id: &str) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    for tc in TIME_CONTROLS.iter() {
        let _: i32 = cmd("ZREM")
            .arg(pool_key(tc))
            .arg(user_id)
            .query_async(&mut *conn)
            .await
            .map_err(|e| format!("Redis ZREM failed: {}", e))?;

        let _: i32 = cmd("HDEL")
            .arg(meta_key(tc))
            .arg(user_id)
            .query_async(&mut *conn)
            .await
            .map_err(|e| format!("Redis HDEL failed: {}", e))?;
    }

    Ok(())
}

pub async fn get_queue_players_with_elo(
    pool: &deadpool_redis::Pool,
    time_control: &str,
) -> Result<Vec<(String, i32)>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let flat: Vec<String> = cmd("ZRANGE")
        .arg(pool_key(time_control))
        .arg(0)
        .arg(-1)
        .arg("WITHSCORES")
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis ZRANGE failed: {}", e))?;

    let mut result = Vec::new();
    for chunk in flat.chunks(2) {
        if chunk.len() == 2 {
            let elo: i32 = chunk[1].parse().unwrap_or(1500);
            result.push((chunk[0].clone(), elo));
        }
    }

    Ok(result)
}

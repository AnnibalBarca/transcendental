use deadpool_redis::redis::cmd;
use deadpool_redis::Pool;

pub const COUNTER_TTL_SECS: u64 = 90;

// Redis key holding the configured limit for a route: `ratelimit:{METHOD}:{path}`.
pub fn rate_limit_key(method: &str, path: &str) -> String {
    format!("ratelimit:{}:{}", method.to_uppercase(), path)
}

pub fn rate_limit_count_key(identity: &str, method: &str, path: &str, minute: i64) -> String {
    format!(
        "ratelimit:count:{}:{}:{}:{}",
        identity,
        method.to_uppercase(),
        path,
        minute
    )
}

pub fn current_minute() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64 / 60)
        .unwrap_or(0)
}

// `{id}` placeholders stored in the `api_routes` table.
pub fn normalize_path(path: &str) -> String {
    path.split('/')
        .map(|seg| {
            if looks_like_id(seg) {
                "{id}".to_string()
            } else {
                seg.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn looks_like_id(seg: &str) -> bool {
    if seg.is_empty() {
        return false;
    }
    let is_uuid = seg.len() == 36
        && seg
            .chars()
            .enumerate()
            .all(|(i, c)| match i {
                8 | 13 | 18 | 23 => c == '-',
                _ => c.is_ascii_hexdigit(),
            });
    if is_uuid {
        return true;
    }
    seg.chars().all(|c| c.is_ascii_digit())
}

pub async fn set_limit(
    pool: &Pool,
    method: &str,
    path: &str,
    requests_per_minute: i64,
) -> Result<(), String> {
    let mut conn = pool.get().await.map_err(|e| e.to_string())?;
    cmd("SET")
        .arg(rate_limit_key(method, path))
        .arg(requests_per_minute)
        .query_async::<_, ()>(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn get_limit(pool: &Pool, method: &str, path: &str) -> Result<Option<i64>, String> {
    let mut conn = pool.get().await.map_err(|e| e.to_string())?;
    cmd("GET")
        .arg(rate_limit_key(method, path))
        .query_async(&mut *conn)
        .await
        .map_err(|e| e.to_string())
}

pub async fn incr_count(
    pool: &Pool,
    identity: &str,
    method: &str,
    path: &str,
) -> Result<i64, String> {
    let minute = current_minute();
    let key = rate_limit_count_key(identity, method, path, minute);
    let mut conn = pool.get().await.map_err(|e| e.to_string())?;
    let count: i64 = cmd("INCR")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    if count == 1 {
        let _: () = cmd("EXPIRE")
            .arg(&key)
            .arg(COUNTER_TTL_SECS)
            .query_async(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(count)
}

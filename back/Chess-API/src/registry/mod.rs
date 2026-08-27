use deadpool_redis::redis::cmd;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use log::{debug, error};

const INSTANCE_REGISTRY_KEY: &str = "chess:instances";
const GAME_MAPPING_KEY_PREFIX: &str = "chess:game";
const HEARTBEAT_INTERVAL_SECS: u64 = 5;
const INSTANCE_TTL_SECS: usize = 30;
const GAME_MAPPING_TTL_SECS: usize = 3600;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChessInstanceInfo {
    pub id: String,
    pub ws_url: String,
    pub load: usize,
}

pub async fn register_instance(
    pool: &deadpool_redis::Pool,
    instance_id: &str,
    ws_url: &str,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let info = ChessInstanceInfo {
        id: instance_id.to_string(),
        ws_url: ws_url.to_string(),
        load: 0,
    };

    let json_str =
        serde_json::to_string(&info).map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("HSET")
        .arg(INSTANCE_REGISTRY_KEY)
        .arg(instance_id)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis HSET failed: {}", e))?;

    let _: () = cmd("EXPIRE")
        .arg(INSTANCE_REGISTRY_KEY)
        .arg(INSTANCE_TTL_SECS)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis EXPIRE failed: {}", e))?;

    debug!("[Registry] Heartbeat: instance {} at {}", instance_id, ws_url);
    Ok(())
}

pub async fn update_load(
    pool: &deadpool_redis::Pool,
    instance_id: &str,
    load: usize,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let existing: Option<String> = cmd("HGET")
        .arg(INSTANCE_REGISTRY_KEY)
        .arg(instance_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis HGET failed: {}", e))?;

    let mut info: ChessInstanceInfo = match existing {
        Some(s) => serde_json::from_str(&s).unwrap_or_else(|_| ChessInstanceInfo {
            id: instance_id.to_string(),
            ws_url: String::new(),
            load,
        }),
        None => ChessInstanceInfo {
            id: instance_id.to_string(),
            ws_url: String::new(),
            load,
        },
    };

    info.load = load;

    let json_str =
        serde_json::to_string(&info).map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("HSET")
        .arg(INSTANCE_REGISTRY_KEY)
        .arg(instance_id)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis HSET failed: {}", e))?;

    Ok(())
}

pub async fn register_game_mapping(
    pool: &deadpool_redis::Pool,
    game_id: &str,
    instance_id: &str,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = format!("{}:{}", GAME_MAPPING_KEY_PREFIX, game_id);

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(GAME_MAPPING_TTL_SECS)
        .arg(instance_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn start_heartbeat(
    pool: deadpool_redis::Pool,
    instance_id: String,
    ws_url: String,
    load_fn: impl Fn() -> usize + Send + 'static,
) {
    loop {
        if let Err(e) = register_instance(&pool, &instance_id, &ws_url).await {
            error!("[Registry] Heartbeat failed: {}", e);
        }

        if let Err(e) = update_load(&pool, &instance_id, load_fn()).await {
            error!("[Registry] Update load failed: {}", e);
        }

        tokio::time::sleep(Duration::from_secs(HEARTBEAT_INTERVAL_SECS)).await;
    }
}

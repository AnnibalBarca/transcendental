use deadpool_redis::redis::cmd;
use log::{error, info};

const INSTANCE_REGISTRY_KEY: &str = "chess:instances";
const GAME_MAPPING_KEY_PREFIX: &str = "chess:game";
const CHESS_WS_PORT: u16 = 8082;

pub async fn resolve_game_ws_url(
    pool: &deadpool_redis::Pool,
    game_id: &str,
) -> Option<String> {
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("[ChessDiscovery] Redis pool error: {}", e);
            return None;
        }
    };

// 1. Look up which instance hosts this game
    let game_key = format!("{}:{}", GAME_MAPPING_KEY_PREFIX, game_id);
    let instance_id: Option<String> = match cmd("GET")
        .arg(&game_key)
        .query_async(&mut *conn)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            error!(
                "[ChessDiscovery] Failed to GET {}: {}",
                game_key, e
            );
            return None;
        }
    };

    let instance_id = instance_id?;

// 2. Verify the instance is still alive in the registry
    let instance_json: Option<String> = match cmd("HGET")
        .arg(INSTANCE_REGISTRY_KEY)
        .arg(&instance_id)
        .query_async(&mut *conn)
        .await
    {
        Ok(j) => j,
        Err(e) => {
            error!(
                "[ChessDiscovery] Failed to HGET instance {}: {}",
                instance_id, e
            );
            return None;
        }
    };

    if instance_json.is_none() {
        error!(
            "[ChessDiscovery] Instance {} not found in registry (dead?)",
            instance_id
        );
        return None;
    }

// 3. Build internal base URL from instance_id (Docker service name)
    let ws_url = format!("ws://{}:{}", instance_id, CHESS_WS_PORT);

    info!(
        "[ChessDiscovery] Resolved game {} -> instance {} -> {}",
        game_id, instance_id, ws_url
    );

    Some(ws_url)
}

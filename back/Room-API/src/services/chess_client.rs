use std::time::Duration;

use deadpool_redis::redis::cmd;
use serde::{Deserialize, Serialize};
use log::info;
use uuid::Uuid;

const INSTANCE_REGISTRY_KEY: &str = "chess:instances";
const CHESS_REQUEST_STREAM_PREFIX: &str = "chess:requests";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChessInstanceInfo {
    pub id: String,
    pub ws_url: String,
    pub load: usize,
}

pub async fn list_instances(pool: &deadpool_redis::Pool) -> Result<Vec<ChessInstanceInfo>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let raw: Vec<(String, String)> = cmd("HGETALL")
        .arg(INSTANCE_REGISTRY_KEY)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis HGETALL failed: {}", e))?;

    let mut instances = Vec::new();
    for (_, json_str) in raw {
        if let Ok(info) = serde_json::from_str::<ChessInstanceInfo>(&json_str) {
            instances.push(info);
        }
    }

    instances.sort_by_key(|i| i.load);
    Ok(instances)
}

pub async fn create_game(pool: &deadpool_redis::Pool, time_control: &str) -> Result<(String, String), String> {
    let request_id = Uuid::new_v4().to_string();

    let instances = list_instances(pool).await?;
    if instances.is_empty() {
        return Err("No Chess-API instance available".to_string());
    }

    let selected = instances
        .into_iter()
        .min_by_key(|i| i.load)
        .expect("instances not empty");

    info!(
        "[ChessClient] Selected instance {} (load: {}) for new game ({})",
        selected.id, selected.load, time_control
    );

    let request = serde_json::json!({
        "id": request_id,
        "method": "POST",
        "action": "game/create",
        "cookies": {},
        "body": serde_json::json!({
            "time_control": time_control,
        }).to_string(),
        "headers": {}
    });

    let stream_key = format!("{}:{}", CHESS_REQUEST_STREAM_PREFIX, selected.id);

    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let _: () = cmd("XADD")
        .arg(&stream_key)
        .arg("*")
        .arg("data")
        .arg(request.to_string())
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis XADD failed: {}", e))?;

    info!(
        "[ChessClient] Sent create_game request {} to instance {}",
        request_id, selected.id
    );

    let result_key = format!("chess:create_game:result:{}", request_id);

    for _ in 0..50 {
        let mut conn = pool
            .get()
            .await
            .map_err(|e| format!("Redis pool error: {}", e))?;

        let result: Option<String> = cmd("GET")
            .arg(&result_key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| format!("Redis GET failed: {}", e))?;

        if let Some(json_str) = result {
            let parsed: serde_json::Value =
                serde_json::from_str(&json_str).map_err(|e| format!("JSON parse failed: {}", e))?;

            if let Some(game_id) = parsed.get("game_id").and_then(|v| v.as_str()) {
                info!(
                    "[ChessClient] Game created: {} on instance {}",
                    game_id, selected.id
                );
                return Ok((game_id.to_string(), selected.ws_url));
            } else {
                return Err("Missing game_id in Chess-API response".to_string());
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Err("Timeout waiting for Chess-API create_game response".to_string())
}

pub async fn abandon_game(
    pool: &deadpool_redis::Pool,
    game_id: &str,
    user_id: &str,
) -> Result<(), String> {
    let request_id = Uuid::new_v4().to_string();

    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let instance_id: Option<String> = cmd("GET")
        .arg(format!("chess:game:{}", game_id))
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    let instance_id = instance_id.ok_or_else(|| format!("No instance hosting game {}", game_id))?;

    let request = serde_json::json!({
        "id": request_id,
        "method": "POST",
        "action": "game/abandon",
        "cookies": {},
        "body": serde_json::json!({
            "game_id": game_id,
            "user_id": user_id,
        }).to_string(),
        "headers": {}
    });

    let stream_key = format!("{}:{}", CHESS_REQUEST_STREAM_PREFIX, instance_id);

    let _: () = cmd("XADD")
        .arg(&stream_key)
        .arg("*")
        .arg("data")
        .arg(request.to_string())
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis XADD failed: {}", e))?;

    info!(
        "[ChessClient] Sent abandon request {} to instance {} for game {}",
        request_id, instance_id, game_id
    );

    let result_key = format!("chess:abandon:result:{}", request_id);

    for _ in 0..30 {
        let mut conn = pool
            .get()
            .await
            .map_err(|e| format!("Redis pool error: {}", e))?;

        let result: Option<String> = cmd("GET")
            .arg(&result_key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| format!("Redis GET failed: {}", e))?;

        if result.is_some() {
            info!("[ChessClient] Game {} abandoned successfully", game_id);
            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Err("Timeout waiting for Chess-API abandon response".to_string())
}

// Enregistre les deux joueurs légitimes d'une partie. Ce mapping est vérifié par
// le Chess-API au moment de la connexion websocket pour empêcher l'usurpation de slot.
pub async fn set_game_players(
    pool: &deadpool_redis::Pool,
    game_id: &str,
    players: &[&str],
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = format!("chess:game_players:{}", game_id);
    let payload = serde_json::json!(players).to_string();

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(7200)
        .arg(payload)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    info!(
        "[ChessClient] Registered players [{}] for game {}",
        players.join(", "),
        game_id
    );

    Ok(())
}

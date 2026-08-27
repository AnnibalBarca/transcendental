use deadpool_redis::redis::cmd;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomEditionRecord {
    pub id: String,
    pub owner_id: String,
    pub room_name: String,
    pub description: String,
    pub test_button: bool,
}

pub async fn get(
    pool: &deadpool_redis::Pool,
    id: &Uuid,
) -> Result<Option<RoomEditionRecord>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::room_edition(id);

    let cached_data: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    if let Some(json_str) = cached_data {
        if let Ok(room) = serde_json::from_str::<RoomEditionRecord>(&json_str) {
            return Ok(Some(room));
        }
    }

    Ok(None)
}

pub async fn set(
    pool: &deadpool_redis::Pool,
    room: &RoomEditionRecord,
    id: &Uuid,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::room_edition(id);

    let json_str =
        serde_json::to_string(room).map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("SET")
        .arg(&key)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SET failed: {}", e))?;
    Ok(())
}

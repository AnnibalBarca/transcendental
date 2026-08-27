use chrono::Utc;
use deadpool_redis::redis::cmd;
use log::error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::chess_client;

const ROOM_CACHE_TTL_SECS: usize = 7200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomType {
    Casual,
    Ranked,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoomStatus {
    Waiting,
    Playing,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub player_ids: Uuid,
    pub player_number: u32,
    pub player_profile_picture: String,
    pub player_username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomRecord {
    pub id: String,
    pub room_type: RoomType,
    pub private: bool,
    pub join_code: Option<String>,
    pub title: Option<String>,
    pub created_at: i64,
    pub player_count: u32,
    pub max_players: u32,
    pub host_id: Uuid,
    pub player_ids: Vec<PlayerData>,
    pub bot_difficulty: Option<String>,
    pub status: RoomStatus,
    pub chess_game_id: Option<String>,
    pub chess_ws_url: Option<String>,
    #[serde(default)]
    pub time_control: Option<u32>,
    #[serde(default)]
    pub banned_ids: Vec<Uuid>,
}

pub async fn create(
    pool: &deadpool_redis::Pool,
    room_type: RoomType,
    private: bool,
    join_code: Option<String>,
    title: Option<String>,
    max_players: u32,
    host_id: Uuid,
    player_ids: Vec<PlayerData>,
    bot_difficulty: Option<String>,
    status: RoomStatus,
    time_control: Option<u32>,
) -> Result<RoomRecord, String> {
    let room_id = Uuid::new_v4().to_string();
    let current_time = Utc::now().timestamp();
    let player_count = player_ids.len() as u32;

    let room = RoomRecord {
        id: room_id,
        room_type,
        private,
        join_code,
        title,
        created_at: current_time,
        player_count,
        max_players,
        host_id,
        player_ids,
        bot_difficulty,
        status,
        chess_game_id: None,
        chess_ws_url: None,
        time_control,
        banned_ids: Vec::new(),
    };

    set(pool, &room).await?;
    Ok(room)
}

pub async fn create_ranked(
    pool: &deadpool_redis::Pool,
    player1: &str,
    player2: &str,
    chess_game_id: &str,
    chess_ws_url: &str,
) -> Result<RoomRecord, String> {
    let ranked_title = format!("ranked_{}", Uuid::new_v4());
    let host_uuid = Uuid::parse_str(player1).map_err(|e| format!("Invalid UUID: {}", e))?;

    let p1 = PlayerData {
        player_ids: host_uuid,
        player_number: 1,
        player_profile_picture: "default.png".to_string(),
        player_username: "Player 1".to_string(),
    };

    let p2_uuid = Uuid::parse_str(player2).map_err(|e| format!("Invalid UUID: {}", e))?;
    let p2 = PlayerData {
        player_ids: p2_uuid,
        player_number: 2,
        player_profile_picture: "default.png".to_string(),
        player_username: "Player 2".to_string(),
    };

    let mut room = create(
        pool,
        RoomType::Ranked,
        true,
        None,
        Some(ranked_title),
        2,
        host_uuid,
        vec![p1, p2],
        None,
        RoomStatus::Playing,
        None,
    )
    .await?;

    room.chess_game_id = Some(chess_game_id.to_string());
    room.chess_ws_url = Some(format!("{}?game_id={}", chess_ws_url, chess_game_id));
    set(pool, &room).await?;

    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;
    let _: () = cmd("HSET")
        .arg("room:game_index")
        .arg(chess_game_id)
        .arg(&room.id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis HSET failed: {}", e))?;

    let players = [player1, player2];
    if let Err(e) = chess_client::set_game_players(pool, chess_game_id, &players).await {
        error!("[CreateRanked] Failed to register game players: {}", e);
    }

    Ok(room)
}

pub async fn set(pool: &deadpool_redis::Pool, room: &RoomRecord) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::room_profile(&room.id);
    let json_str =
        serde_json::to_string(room).map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(ROOM_CACHE_TTL_SECS)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn get(pool: &deadpool_redis::Pool, id: &str) -> Result<Option<RoomRecord>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::room_profile(id);

    let cached_data: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    if let Some(json_str) = cached_data {
        if let Ok(room) = serde_json::from_str::<RoomRecord>(&json_str) {
            return Ok(Some(room));
        }
    }

    Ok(None)
}

pub async fn delete(pool: &deadpool_redis::Pool, id: &str) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::room_profile(id);

    let _: () = cmd("DEL")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;

    Ok(())
}

pub async fn add_to_public_list(pool: &deadpool_redis::Pool, room_id: &str, created_at: i64) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::room_public_list();

    let _: () = cmd("ZADD")
        .arg(&key)
        .arg(created_at)
        .arg(room_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis ZADD failed: {}", e))?;

    Ok(())
}

pub async fn remove_from_public_list(pool: &deadpool_redis::Pool, room_id: &str) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::room_public_list();

    let _: () = cmd("ZREM")
        .arg(&key)
        .arg(room_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis ZREM failed: {}", e))?;

    Ok(())
}

pub async fn list_public(pool: &deadpool_redis::Pool) -> Result<Vec<RoomRecord>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::room_public_list();

    let room_ids: Vec<String> = cmd("ZRANGE")
        .arg(&key)
        .arg(0)
        .arg(-1)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis ZRANGE failed: {}", e))?;

    let mut rooms = Vec::new();
    for room_id in &room_ids {
        if let Ok(Some(room)) = get(pool, room_id).await {
            rooms.push(room);
        }
    }

    Ok(rooms)
}

pub async fn clean_stale_public(pool: &deadpool_redis::Pool) -> Result<usize, String> {
    let key = super::keys::room_public_list();
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let room_ids: Vec<String> = cmd("ZRANGE")
        .arg(&key)
        .arg(0)
        .arg(-1)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis ZRANGE failed: {}", e))?;

    let mut stale = Vec::new();
    for room_id in &room_ids {
        let room_key = super::keys::room_profile(room_id);
        let exists: bool = cmd("EXISTS")
            .arg(&room_key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| format!("Redis EXISTS failed: {}", e))?;
        if !exists {
            stale.push(room_id.clone());
        }
    }

    if stale.is_empty() {
        return Ok(0);
    }

    for id in &stale {
        let _: () = cmd("ZREM")
            .arg(&key)
            .arg(id)
            .query_async(&mut *conn)
            .await
            .map_err(|e| format!("Redis ZREM failed: {}", e))?;
    }

    Ok(stale.len())
}

pub async fn count_public(pool: &deadpool_redis::Pool) -> Result<usize, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::room_public_list();
    let count: usize = cmd("ZCARD")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis ZCARD failed: {}", e))?;
    Ok(count)
}

pub async fn find_by_game_id(
    pool: &deadpool_redis::Pool,
    game_id: &str,
) -> Result<Option<RoomRecord>, String> {
    let key = "room:game_index";
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let room_id: Option<String> = cmd("HGET")
        .arg(key)
        .arg(game_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis HGET failed: {}", e))?;

    match room_id {
        Some(id) => get(pool, &id).await,
        None => Ok(None),
    }
}

pub async fn set_join_code(pool: &deadpool_redis::Pool, code: &str, room_id: &str) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::room_join_code(code);

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(ROOM_CACHE_TTL_SECS)
        .arg(room_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn get_room_id_by_code(pool: &deadpool_redis::Pool, code: &str) -> Result<Option<String>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::room_join_code(code);

    let room_id: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    Ok(room_id)
}

pub async fn delete_join_code(pool: &deadpool_redis::Pool, code: &str) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::room_join_code(code);

    let _: () = cmd("DEL")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;

    Ok(())
}

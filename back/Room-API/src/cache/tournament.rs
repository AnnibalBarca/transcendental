use chrono::Utc;
use deadpool_redis::redis::cmd;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PLAYER_SIZES: [u32; 3] = [4, 8, 16];
pub const DEFAULT_PLAYER_SIZE: u32 = 4;
pub const TOURNAMENT_TIME_CONTROL: &str = "5";
const TOURNAMENT_TTL_SECS: usize = 21600;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TournamentStatus {
    Waiting,
    Playing,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerEntry {
    pub user_id: Uuid,
    pub username: String,
    pub elo: i32,
    pub picture: String,
    pub alive: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatchStatus {
    Pending,
    Playing,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentMatch {
    pub round: u32,
    pub bracket_index: u32,
    pub player1: Option<String>,
    pub player2: Option<String>,
    pub chess_game_id: Option<String>,
    pub chess_ws_url: Option<String>,
    pub winner: Option<String>,
    pub status: MatchStatus,
    #[serde(default)]
    pub started_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodiumEntry {
    pub rank: u32,
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingEntry {
    pub rank: u32,
    pub user_id: Uuid,
    pub username: String,
    pub elo_change: i32,
    pub xp_gained: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentRecord {
    pub id: String,
    pub name: String,
    pub player_size: u32,
    pub host_id: Uuid,
    pub status: TournamentStatus,
    pub players: Vec<PlayerEntry>,
    pub round: u32,
    pub matches: Vec<TournamentMatch>,
    pub champion: Option<String>,
    pub podium: Vec<PodiumEntry>,
    pub rankings: Vec<RankingEntry>,
    pub created_at: i64,
    pub start_at: Option<i64>,
    #[serde(default)]
    pub is_ranked: bool,
    #[serde(default)]
    pub round_deadline: Option<i64>,
    #[serde(default)]
    pub finished_at: Option<i64>,
    #[serde(default)]
    pub rewards_applied: bool,
}

pub fn total_rounds(player_size: u32) -> u32 {
    match player_size {
        4 => 2,
        8 => 3,
        16 => 4,
        _ => 2,
    }
}

fn key_profile(id: &str) -> String {
    format!("tournament:profile:{}", id)
}

pub fn key_game_index(game_id: &str) -> String {
    format!("tournament:game_index:{}", game_id)
}

pub fn key_user_tournament(user_id: &Uuid) -> String {
    format!("tournament:user:{}", user_id)
}

pub async fn set(pool: &deadpool_redis::Pool, record: &TournamentRecord) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let json_str = serde_json::to_string(record)
        .map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("SETEX")
        .arg(key_profile(&record.id))
        .arg(TOURNAMENT_TTL_SECS)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn get(pool: &deadpool_redis::Pool, id: &str) -> Result<Option<TournamentRecord>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let cached: Option<String> = cmd("GET")
        .arg(key_profile(id))
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    match cached {
        Some(json_str) => match serde_json::from_str::<TournamentRecord>(&json_str) {
            Ok(record) => Ok(Some(record)),
            Err(_) => Ok(None),
        },
        None => Ok(None),
    }
}

pub async fn delete(pool: &deadpool_redis::Pool, id: &str) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let _: () = cmd("DEL")
        .arg(key_profile(id))
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;

    Ok(())
}

pub async fn create(
    pool: &deadpool_redis::Pool,
    name: String,
    player_size: u32,
    host: PlayerEntry,
    is_ranked: bool,
) -> Result<TournamentRecord, String> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().timestamp();

    let record = TournamentRecord {
        id: id.clone(),
        name,
        player_size,
        host_id: host.user_id,
        status: TournamentStatus::Waiting,
        players: vec![host],
        round: 0,
        matches: Vec::new(),
        champion: None,
        podium: Vec::new(),
        rankings: Vec::new(),
        created_at,
        start_at: None,
        is_ranked,
        round_deadline: None,
        finished_at: None,
        rewards_applied: false,
    };

    set(pool, &record).await?;

    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;
    let _: () = cmd("ZADD")
        .arg("tournament:list")
        .arg(created_at)
        .arg(&id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis ZADD failed: {}", e))?;

    Ok(record)
}

pub async fn remove_from_list(pool: &deadpool_redis::Pool, id: &str) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;
    let _: () = cmd("ZREM")
        .arg("tournament:list")
        .arg(id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis ZREM failed: {}", e))?;
    Ok(())
}

pub async fn list_ids(pool: &deadpool_redis::Pool) -> Result<Vec<String>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;
    let ids: Vec<String> = cmd("ZRANGE")
        .arg("tournament:list")
        .arg(0)
        .arg(-1)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis ZRANGE failed: {}", e))?;
    Ok(ids)
}

pub async fn list(pool: &deadpool_redis::Pool) -> Result<Vec<TournamentRecord>, String> {
    let ids = list_ids(pool).await?;
    let mut records = Vec::new();
    for id in ids {
        if let Ok(Some(record)) = get(pool, &id).await {
            records.push(record);
        }
    }
    records.sort_by_key(|r| std::cmp::Reverse(r.created_at));
    Ok(records)
}

pub async fn set_game_index(pool: &deadpool_redis::Pool, game_id: &str, tournament_id: &str) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;
    let _: () = cmd("SETEX")
        .arg(key_game_index(game_id))
        .arg(TOURNAMENT_TTL_SECS)
        .arg(tournament_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;
    Ok(())
}

pub async fn get_tournament_id_by_game(pool: &deadpool_redis::Pool, game_id: &str) -> Result<Option<String>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;
    let id: Option<String> = cmd("GET")
        .arg(key_game_index(game_id))
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;
    Ok(id)
}

pub async fn set_user_tournament(pool: &deadpool_redis::Pool, user_id: &Uuid, tournament_id: &str) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;
    let _: () = cmd("SETEX")
        .arg(key_user_tournament(user_id))
        .arg(TOURNAMENT_TTL_SECS)
        .arg(tournament_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;
    Ok(())
}

pub async fn get_user_tournament(pool: &deadpool_redis::Pool, user_id: &Uuid) -> Result<Option<String>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;
    let id: Option<String> = cmd("GET")
        .arg(key_user_tournament(user_id))
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;
    Ok(id)
}

pub async fn delete_user_tournament(pool: &deadpool_redis::Pool, user_id: &Uuid) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;
    let _: () = cmd("DEL")
        .arg(key_user_tournament(user_id))
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;
    Ok(())
}

pub async fn clean_stale(pool: &deadpool_redis::Pool) -> Result<usize, String> {
    let ids = list_ids(pool).await?;
    let mut stale = Vec::new();
    for id in ids {
        let key = key_profile(&id);
        let mut conn = pool
            .get()
            .await
            .map_err(|e| format!("Redis pool error: {}", e))?;
        let exists: bool = cmd("EXISTS")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| format!("Redis EXISTS failed: {}", e))?;
        if !exists {
            stale.push(id);
        }
    }
    for id in &stale {
        let _ = remove_from_list(pool, id).await;
    }
    Ok(stale.len())
}

use std::collections::HashMap;

use deadpool_redis::redis::cmd;
use deadpool_redis::Pool;
use log::{error, info};
use notification::event::{LiveGame, NotificationBus, NotificationEvent};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::cache::room::{self as room_cache, RoomStatus, RoomType};
use crate::cache::tournament::{self as tournament_cache, TournamentStatus};

const MAX_LIVE_GAMES: usize = 100;
const LIVE_GAMES_INTERVAL_SECS: u64 = 10;

async fn fetch_usernames(db_pool: &PgPool, ids: &[Uuid]) -> HashMap<Uuid, String> {
    if ids.is_empty() {
        return HashMap::new();
    }
    let rows = sqlx::query("SELECT id, username FROM users WHERE id = ANY($1)")
        .bind(ids)
        .fetch_all(db_pool)
        .await;
    let mut map = HashMap::new();
    if let Ok(rows) = rows {
        for row in rows {
            let id: Uuid = row.get("id");
            let username: Option<String> = row.get("username");
            if let Some(username) = username {
                map.insert(id, username);
            }
        }
    }
    map
}

async fn collect_room_games(
    redis_pool: &Pool,
    db_pool: &PgPool,
    games: &mut Vec<LiveGame>,
) -> Result<(), String> {
    let mut conn = redis_pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let entries: HashMap<String, String> = cmd("HGETALL")
        .arg("room:game_index")
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("HGETALL room:game_index failed: {}", e))?;

    let mut uuids: Vec<Uuid> = Vec::new();
    let mut ids_by_room: HashMap<String, (Uuid, Uuid)> = HashMap::new();

    for (game_id, room_id) in &entries {
        if let Ok(Some(room)) = room_cache::get(redis_pool, room_id).await {
            if room.status != RoomStatus::Playing {
                continue;
            }
            if room.chess_game_id.as_deref() != Some(game_id.as_str()) {
                continue;
            }
            if room.player_ids.len() < 2 {
                continue;
            }
            let p1 = room.player_ids[0].player_ids;
            let p2 = room.player_ids[1].player_ids;
            uuids.push(p1);
            uuids.push(p2);
            ids_by_room.insert(room_id.clone(), (p1, p2));
        }
    }

    let usernames = fetch_usernames(db_pool, &uuids).await;

    for (game_id, room_id) in &entries {
        let Some((p1, p2)) = ids_by_room.get(room_id) else {
            continue;
        };
        let Ok(Some(room)) = room_cache::get(redis_pool, room_id).await else {
            continue;
        };
        if room.status != RoomStatus::Playing {
            continue;
        }
        let kind = match room.room_type {
            RoomType::Ranked => "ranked",
            RoomType::Casual => "casual",
        };
        games.push(LiveGame {
            game_id: game_id.clone(),
            kind: kind.to_string(),
            player1: usernames.get(p1).cloned().or_else(|| {
                room.player_ids
                    .iter()
                    .find(|p| p.player_ids == *p1)
                    .map(|p| p.player_username.clone())
            }),
            player2: usernames.get(p2).cloned().or_else(|| {
                room.player_ids
                    .iter()
                    .find(|p| p.player_ids == *p2)
                    .map(|p| p.player_username.clone())
            }),
            label: None,
        });
    }

    Ok(())
}

async fn collect_tournament_games(
    redis_pool: &Pool,
    games: &mut Vec<LiveGame>,
) -> Result<(), String> {
    let mut conn = redis_pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let keys: Vec<String> = cmd("KEYS")
        .arg("tournament:game_index:*")
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("KEYS tournament:game_index failed: {}", e))?;

    for key in keys {
        let Some(game_id) = key.strip_prefix("tournament:game_index:") else {
            continue;
        };
        let tournament_id: Option<String> = cmd("GET")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .unwrap_or(None);

        let Some(tournament_id) = tournament_id else {
            continue;
        };
        let Ok(Some(record)) = tournament_cache::get(redis_pool, &tournament_id).await else {
            continue;
        };
        if record.status != TournamentStatus::Playing {
            continue;
        }

        let username_for = |uid: &Option<String>| {
            uid.as_deref().and_then(|uid| {
                record
                    .players
                    .iter()
                    .find(|p| p.user_id.to_string() == uid)
                    .map(|p| p.username.clone())
            })
        };

        if let Some(m) = record
            .matches
            .iter()
            .find(|m| m.chess_game_id.as_deref() == Some(game_id))
        {
            games.push(LiveGame {
                game_id: game_id.to_string(),
                kind: "tournament".to_string(),
                player1: username_for(&m.player1),
                player2: username_for(&m.player2),
                label: Some(record.name.clone()),
            });
        }
    }

    Ok(())
}

async fn collect_live_games(redis_pool: &Pool, db_pool: &PgPool) -> Result<Vec<LiveGame>, String> {
    let mut games: Vec<LiveGame> = Vec::new();
    collect_room_games(redis_pool, db_pool, &mut games).await?;
    collect_tournament_games(redis_pool, &mut games).await?;

    let mut seen = std::collections::HashSet::new();
    games.retain(|g| seen.insert(g.game_id.clone()));

    games.truncate(MAX_LIVE_GAMES);
    Ok(games)
}

pub async fn run_live_games_loop(
    redis_pool: Pool,
    db_pool: PgPool,
    notification_bus: NotificationBus,
) {
    info!("[LiveGames] Loop started");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(LIVE_GAMES_INTERVAL_SECS)).await;

        let games = match collect_live_games(&redis_pool, &db_pool).await {
            Ok(g) => g,
            Err(e) => {
                error!("[LiveGames] Failed to collect live games: {}", e);
                continue;
            }
        };

        notification_bus
            .broadcast(&NotificationEvent::LiveGames { games })
            .await;
    }
}

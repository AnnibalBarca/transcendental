use std::sync::Arc;

use deadpool_redis::redis::streams::StreamReadReply;
use deadpool_redis::redis::{cmd, FromRedisValue, Value};
use deadpool_redis::Pool;
use log::{error, info};
use notification::event::{NotificationBus, NotificationEvent};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::cache::room::{self, RoomRecord};
use crate::cache::tournament as tournament_cache;
use crate::services::tournament as tournament_service;
use crate::user_state::RedisSessionManager;

const GAME_EVENTS_STREAM: &str = "chess:game:events";

fn calculate_elo_change(player_elo: i32, opponent_elo: i32, score: f64, k: f64) -> i32 {
    let expected = 1.0 / (1.0 + 10_f64.powf((opponent_elo - player_elo) as f64 / 400.0));
    (k * (score - expected)).round() as i32
}

fn xp_gain(won: bool, is_draw: bool) -> i64 {
    if won { 100 } else if is_draw { 50 } else { 20 }
}

fn wallet_gain(won: bool, is_draw: bool) -> i64 {
    if won { 50 } else if is_draw { 25 } else { 10 }
}

async fn update_player_stats(
    redis_pool: &Pool,
    db_pool: &PgPool,
    user_id: &Uuid,
    elo_change: i32,
    xp: i64,
    wallet: i64,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE user_profile
        SET ranked_elo = GREATEST(0, ranked_elo + $1),
            xp = xp + $2
        WHERE user_id = $3
        "#,
    )
    .bind(elo_change)
    .bind(xp)
    .bind(user_id)
    .execute(db_pool)
    .await
    .map_err(|e| format!("Failed to update profile: {}", e))?;

    sqlx::query(
        "UPDATE users SET wallet = wallet + $1 WHERE id = $2 AND wallet + $1 >= 0",
    )
    .bind(wallet)
    .bind(user_id)
    .execute(db_pool)
    .await
    .map_err(|e| format!("Failed to update wallet: {}", e))?;

    invalidate_user_profile_cache(redis_pool, user_id).await;

    Ok(())
}

async fn invalidate_user_profile_cache(redis_pool: &Pool, user_id: &Uuid) {
    let key = format!("user:profile:{}", user_id);
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("[GameResult] Failed to get redis conn to invalidate cache: {}", e);
            return;
        }
    };
    let _: Result<(), _> = cmd("DEL")
        .arg(&key)
        .query_async(&mut *conn)
        .await;
}

pub async fn run_game_result_listener(
    redis_pool: Pool,
    db_pool: PgPool,
    session_manager: Arc<RedisSessionManager>,
    notification_bus: NotificationBus,
) {
    info!("[GameResult] Listener started");

    loop {
        let mut conn = match redis_pool.get().await {
            Ok(c) => c,
            Err(e) => {
                error!("[GameResult] Redis pool error: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        };

        let raw: Result<Value, _> = cmd("XREADGROUP")
            .arg("GROUP")
            .arg("room-game-result-group")
            .arg("room-game-result-consumer")
            .arg("BLOCK")
            .arg(5000)
            .arg("COUNT")
            .arg(1)
            .arg("STREAMS")
            .arg(GAME_EVENTS_STREAM)
            .arg(">")
            .query_async(&mut *conn)
            .await;

        let raw = match raw {
            Ok(v) => v,
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("NOGROUP") {
                    info!("[GameResult] Creating consumer group...");
                    match cmd("XGROUP")
                        .arg("CREATE")
                        .arg(GAME_EVENTS_STREAM)
                        .arg("room-game-result-group")
                        .arg("0")
                        .arg("MKSTREAM")
                        .query_async::<_, ()>(&mut *conn)
                        .await
                    {
                        Ok(_) => info!("[GameResult] Consumer group created"),
                        Err(ce) => error!("[GameResult] Failed to create group: {}", ce),
                    }
                } else {
                    error!("[GameResult] XREADGROUP error: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
                continue;
            }
        };

        if matches!(raw, Value::Nil) {
            continue;
        }

        let reply = match StreamReadReply::from_redis_value(&raw) {
            Ok(r) => r,
            Err(e) => {
                error!("[GameResult] Failed to parse stream reply: {}", e);
                continue;
            }
        };

        for stream in reply.keys {
            for record in stream.ids {
                let msg_id = record.id;
                let mut data_map = std::collections::HashMap::new();
                for (k, v) in record.map {
                    if let Value::Data(bytes) = v {
                        let s = String::from_utf8_lossy(&bytes).into_owned();
                        data_map.insert(k, s);
                    }
                }

                let payload = match data_map.get("data") {
                    Some(p) => p.clone(),
                    None => {
                        error!("[GameResult] Missing 'data' field in message {}", msg_id);
                        let _: Result<(), _> = cmd("XACK")
                            .arg(GAME_EVENTS_STREAM)
                            .arg("room-game-result-group")
                            .arg(&msg_id)
                            .query_async(&mut *conn)
                            .await;
                        continue;
                    }
                };

                match process_game_event(&redis_pool, &db_pool, &session_manager, &notification_bus, &payload).await {
                    Ok(_) => info!("[GameResult] Processed event for message {}", msg_id),
                    Err(e) => error!("[GameResult] Failed to process event: {} (msg_id={})", e, msg_id),
                }

                let _: Result<(), _> = cmd("XACK")
                    .arg(GAME_EVENTS_STREAM)
                    .arg("room-game-result-group")
                    .arg(&msg_id)
                    .query_async(&mut *conn)
                    .await;
            }
        }
    }
}

pub async fn process_game_event(
    redis_pool: &Pool,
    db_pool: &PgPool,
    session_manager: &Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
    payload: &str,
) -> Result<(), String> {
    let event: serde_json::Value =
        serde_json::from_str(payload).map_err(|e| format!("Invalid JSON: {}", e))?;

    let game_id = event["game_id"]
        .as_str()
        .ok_or_else(|| "Missing game_id".to_string())?;
    let result = event["result"]
        .as_str()
        .ok_or_else(|| "Missing result".to_string())?;
    let winner = event["winner"].as_str();

    let white_uid_str = event["white_user_id"].as_str().unwrap_or("");
    let black_uid_str = event["black_user_id"].as_str().unwrap_or("");

    if result != "cancelled" {
        if let Err(e) = record_game(db_pool, game_id, result, winner, white_uid_str, black_uid_str).await {
            error!("[GameResult] Failed to record game {}: {}", game_id, e);
        }
    }

    if let Ok(Some(_tournament_id)) = tournament_cache::get_tournament_id_by_game(redis_pool, game_id).await {
        info!(
            "[GameResult] Game {} belongs to a tournament, delegating to tournament service",
            game_id
        );
        return tournament_service::on_game_result(
            redis_pool,
            db_pool,
            Arc::clone(session_manager),
            notification_bus.clone(),
            game_id,
            result,
            winner,
            white_uid_str.to_string(),
            black_uid_str.to_string(),
        )
        .await;
    }

    let w_uid = Uuid::parse_str(white_uid_str).ok();
    let b_uid = Uuid::parse_str(black_uid_str).ok();

    info!(
        "[GameResult] Processing game {} result={} winner={:?} white={:?} black={:?}",
        game_id, result, winner, white_uid_str, black_uid_str
    );

    let (w_id, b_id, room_data) = match (w_uid, b_uid) {
        (Some(w), Some(b)) => {
            let room = match find_players_by_game(redis_pool, game_id).await {
                Ok(Some((rm, _, _))) => Some(rm),
                _ => None,
            };
            (w, b, room)
        }
        _ => {
            let (rm, p1, p2) = find_players_by_game(redis_pool, game_id)
                .await?
                .ok_or_else(|| format!("No room found for game {}", game_id))?;
            (p1, p2, Some(rm))
        }
    };

    if result == "cancelled" {
        info!("[GameResult] Game cancelled before first move, no ELO change");
        if let Some(ref rm) = room_data {
            cleanup_room_and_index(redis_pool, rm, game_id).await?;
        }
        reset_player_session(session_manager, notification_bus, &w_id).await;
        reset_player_session(session_manager, notification_bus, &b_id).await;
        return Ok(());
    }

    if let Some(win) = winner {
        let (winner_id, loser_id) = if win == "white" {
            (w_id, b_id)
        } else {
            (b_id, w_id)
        };

        let winner_elo = get_elo(db_pool, &winner_id).await;
        let loser_elo = get_elo(db_pool, &loser_id).await;

        let elo_change = calculate_elo_change(winner_elo, loser_elo, 1.0, 32.0);
        let xp = xp_gain(true, false);
        let wallet = wallet_gain(true, false);

        update_player_stats(redis_pool, db_pool, &winner_id, elo_change, xp, wallet).await?;
        update_player_stats(redis_pool, db_pool, &loser_id, -elo_change, xp_gain(false, false), wallet_gain(false, false)).await?;

        if let Some(ref rm) = room_data {
            cleanup_room_and_index(redis_pool, rm, game_id).await?;
        }

        reset_player_session(session_manager, notification_bus, &winner_id).await;
        reset_player_session(session_manager, notification_bus, &loser_id).await;
    } else {
        if let Some(ref rm) = room_data {
            cleanup_room_and_index(redis_pool, rm, game_id).await?;
        }

        update_player_stats(redis_pool, db_pool, &w_id, 0, xp_gain(false, true), wallet_gain(false, true)).await?;
        update_player_stats(redis_pool, db_pool, &b_id, 0, xp_gain(false, true), wallet_gain(false, true)).await?;

        reset_player_session(session_manager, notification_bus, &w_id).await;
        reset_player_session(session_manager, notification_bus, &b_id).await;
    }

    Ok(())
}

async fn find_players_by_game(
    pool: &Pool,
    game_id: &str,
) -> Result<Option<(RoomRecord, Uuid, Uuid)>, String> {
    let room = room::find_by_game_id(pool, game_id)
        .await?
        .ok_or_else(|| format!("No room for game {}", game_id))?;

    if room.player_ids.len() < 2 {
        return Err("Room has fewer than 2 players".to_string());
    }

    let p1_id = room.player_ids[0].player_ids;
    let p2_id = room.player_ids[1].player_ids;

    Ok(Some((room, p1_id, p2_id)))
}

async fn record_game(
    db_pool: &PgPool,
    game_id: &str,
    result: &str,
    winner: Option<&str>,
    white_uid_str: &str,
    black_uid_str: &str,
) -> Result<(), String> {
    let white_uid = Uuid::parse_str(white_uid_str).ok();
    let black_uid = Uuid::parse_str(black_uid_str).ok();

    sqlx::query(
        r#"
        INSERT INTO games (game_id, result, winner, white_user_id, black_user_id)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (game_id) DO NOTHING
        "#,
    )
    .bind(game_id)
    .bind(result)
    .bind(winner)
    .bind(white_uid)
    .bind(black_uid)
    .execute(db_pool)
    .await
    .map_err(|e| format!("Failed to record game: {}", e))?;

    Ok(())
}

async fn get_elo(db_pool: &PgPool, user_id: &Uuid) -> i32 {
    let row = sqlx::query("SELECT ranked_elo FROM user_profile WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(db_pool)
        .await;

    match row {
        Ok(Some(r)) => r.get::<i32, _>("ranked_elo"),
        _ => 1500,
    }
}

async fn cleanup_room_and_index(redis_pool: &Pool, room: &RoomRecord, game_id: &str) -> Result<(), String> {
    let mut conn = redis_pool.get().await.map_err(|e| e.to_string())?;
    let _: () = cmd("HDEL")
        .arg("room:game_index")
        .arg(game_id)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Failed to remove game index: {}", e))?;
    room::delete(redis_pool, &room.id).await?;
    Ok(())
}

async fn reset_player_session(
    session_manager: &Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
) {
    if let Err(e) = session_manager.delete_session(user_id).await {
        error!("[GameResult] Failed to delete session for {}: {}", user_id, e);
    }

    notification_bus
        .send_to_user(
            *user_id,
            &NotificationEvent::SetState {
                user_id: *user_id,
                state: "none".into(),
                room_id: None,
                chess_ws_url: None,
                chess_game_id: None,
            },
        )
        .await;
}

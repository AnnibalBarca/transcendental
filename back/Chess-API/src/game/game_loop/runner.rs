use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tokio::sync::Mutex as AsyncMutex;

use crate::game::board::Color;
use crate::game::game_loop::{GameLoop, GameTimer};
use crate::game::manager::GameManager;
use crate::game::redis_helpers::{cleanup_user_session, publish_game_result};
use crate::websocket::lobby::{LobbyState, OutgoingMessage};
use log::info;
use serde_json::json;

const STOP_GRACE_DURATION: Duration = Duration::from_secs(5 * 60);

async fn clear_game_resources(
    redis_pool: &deadpool_redis::Pool,
    game_id: &str,
    player_user_ids: &AsyncMutex<Vec<String>>,
) {
    use deadpool_redis::redis::cmd;

    let mut ids = player_user_ids.lock().await.clone();

    let players_key = format!("chess:game_players:{}", game_id);
    if let Ok(mut conn) = redis_pool.get().await {
        let data: Option<String> = cmd("GET")
            .arg(&players_key)
            .query_async(&mut *conn)
            .await
            .ok()
            .flatten();
        if let Some(data) = data {
            let players: Vec<String> = serde_json::from_str(&data).unwrap_or_default();
            for p in players {
                if !p.is_empty() && !ids.contains(&p) {
                    ids.push(p);
                }
            }
        }
    }

    for uid in ids {
        if !uid.is_empty() {
            cleanup_user_session(redis_pool, &uid).await;
        }
    }
}

async fn delete_game_key(redis_pool: &deadpool_redis::Pool, game_id: &str) {
    use deadpool_redis::redis::cmd;
    let key = format!("chess:game:{}", game_id);
    if let Ok(mut conn) = redis_pool.get().await {
        let _: deadpool_redis::redis::RedisResult<()> =
            cmd("DEL").arg(&key).query_async(&mut *conn).await;
    }
}

pub async fn run_game_loop(
    game: Arc<GameLoop>,
    lobby: Arc<AsyncMutex<LobbyState>>,
    redis_pool: deadpool_redis::Pool,
    player_user_ids: Arc<AsyncMutex<Vec<String>>>,
    game_id: String,
    manager: Arc<GameManager>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let mut stopped_at: Option<Instant> = None;
    loop {
        interval.tick().await;
        if game.ended.load(Ordering::Relaxed) {
            break;
        }
        if !game.running.load(Ordering::Relaxed) {
            if game.started.load(Ordering::Relaxed) {
                stopped_at = None;
                continue;
            }

            let players_connected = {
                let lobby = lobby.lock().await;
                lobby.players.iter().flatten().next().is_some()
            };
            if players_connected {
                stopped_at = None;
                continue;
            }
            let now = Instant::now();
            let since = stopped_at.get_or_insert(now);
            if now.duration_since(*since) >= STOP_GRACE_DURATION {
                info!(
                    "[GameLoop] Game {} removed after grace period of {}s",
                    game_id,
                    STOP_GRACE_DURATION.as_secs()
                );
                let (white_uid, black_uid) = {
                    let lobby = lobby.lock().await;
                    lobby.color_user_ids()
                };
                publish_game_result(
                    &redis_pool,
                    &game_id,
                    "cancelled",
                    None,
                    white_uid.as_deref(),
                    black_uid.as_deref(),
                )
                .await;
                manager.remove_game(&game_id).await;
                clear_game_resources(&redis_pool, &game_id, &player_user_ids).await;
                delete_game_key(&redis_pool, &game_id).await;
                break;
            }
            continue;
        }
        stopped_at = None;

        if !game.first_move_played.load(Ordering::Relaxed) {
            let deadline = *game.first_move_deadline.lock().unwrap();
            if std::time::Instant::now() > deadline {
                info!("[GameLoop] First move timeout - game cancelled");
                let lobby = lobby.lock().await;
                lobby.broadcast(OutgoingMessage {
                    action: "game_cancelled".to_string(),
                    from: None,
                    message: json!({"reason": "White did not play within 20 seconds"}),
                });
                let (white_uid, black_uid) = lobby.color_user_ids();
                publish_game_result(
                    &redis_pool,
                    &game_id,
                    "cancelled",
                    None,
                    white_uid.as_deref(),
                    black_uid.as_deref(),
                )
                .await;
                drop(lobby);

                clear_game_resources(&redis_pool, &game_id, &player_user_ids).await;

                game.end();
                break;
            }
        }

        if let Some(loser) = game.tick() {
            let winner = match loser {
                Color::White => "black",
                Color::Black => "white",
            };
            info!(
                "[GameLoop] Timeout! {:?} ran out of time. Winner: {}",
                loser, winner
            );

            let lobby = lobby.lock().await;
            lobby.broadcast(OutgoingMessage {
                action: "timeout".to_string(),
                from: None,
                message: json!({"loser": format!("{:?}", loser).to_lowercase(), "winner": winner}),
            });
            let (white_uid, black_uid) = lobby.color_user_ids();
            publish_game_result(
                &redis_pool,
                &game_id,
                winner,
                Some(winner),
                white_uid.as_deref(),
                black_uid.as_deref(),
            )
            .await;
            drop(lobby);

            clear_game_resources(&redis_pool, &game_id, &player_user_ids).await;

            game.end();
            break;
        }
    }

    manager.remove_game(&game_id).await;
}

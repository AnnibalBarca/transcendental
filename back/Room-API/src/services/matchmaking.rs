use std::sync::Arc;
use std::time::Duration;

use deadpool_redis::Pool;
use log::{error, info};
use notification::event::{NotificationBus, NotificationEvent};
use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::{elo as elo_cache, matchmaking as matchmaking_cache, room};
use crate::db::profile;
use crate::services::chess_client;
use crate::user_state::{PlayerSession, RedisSessionManager};

pub async fn run_matchmaking_loop(
    pool: Pool,
    session_manager: Arc<RedisSessionManager>,
    notification_bus: NotificationBus,
    _db_pool: PgPool,
) {
    info!("[Matchmaking] Loop started");

    loop {
        for tc in matchmaking_cache::TIME_CONTROLS.iter() {
            match matchmaking_cache::queue_size(&pool, Some(tc)).await {
                Ok(size) => {
                    if size >= 2 {
                        match matchmaking_cache::pop_two_players(&pool, tc).await {
                            Ok(Some((player1, player2, elo1, elo2))) => {
                                if player1 == player2 {
                                    continue;
                                }
                                info!(
                                    "[Matchmaking] Paired players: {} (elo={}) and {} (elo={}) on {} min",
                                    player1, elo1, player2, elo2, tc
                                );

                                let (chess_game_id, chess_ws_url) =
                                    match chess_client::create_game(&pool, tc).await {
                                        Ok(result) => result,
                                        Err(e) => {
                                            error!("[Matchmaking] Failed to create chess game: {}", e);
                                            continue;
                                        }
                                    };

                                match room::create_ranked(
                                    &pool,
                                    &player1,
                                    &player2,
                                    &chess_game_id,
                                    &chess_ws_url,
                                )
                                .await
                                {
                                    Ok(room) => {
                                        info!(
                                            "[Matchmaking] Ranked room created: {} (game: {})",
                                            room.id,
                                            room.chess_game_id.as_deref().unwrap_or("?")
                                        );

                                        for player_id in [&player1, &player2] {
                                            let player_uuid = match Uuid::parse_str(player_id) {
                                                Ok(uuid) => uuid,
                                                Err(e) => {
                                                    error!(
                                                        "[Matchmaking] Invalid player UUID {}: {}",
                                                        player_id, e
                                                    );
                                                    continue;
                                                }
                                            };

                                            let session = PlayerSession {
                                                room_id: room.id.clone(),
                                                status: "playing".into(),
                                                chess_ws_url: room.chess_ws_url.clone().unwrap_or_default(),
                                                chess_game_id: room.chess_game_id.clone().unwrap_or_default(),
                                            };
                                            if let Err(e) = session_manager
                                                .save_session(&player_uuid, &session)
                                                .await
                                            {
                                                error!(
                                                    "[Matchmaking] Failed to update session for {}: {}",
                                                    player_id, e
                                                );
                                            }

                                            notification_bus
                                                .send_to_user(
                                                    player_uuid,
                                                    &NotificationEvent::SetState {
                                                        user_id: player_uuid,
                                                        state: "playing".into(),
                                                        room_id: Uuid::parse_str(&room.id).ok(),
                                                        chess_ws_url: room.chess_ws_url.clone(),
                                                        chess_game_id: room.chess_game_id.clone(),
                                                    },
                                                )
                                                .await;
                                        }
                                    }
                                    Err(e) => {
                                        error!("[Matchmaking] Failed to create ranked room: {}", e);
                                    }
                                }
                            }
                            Ok(None) => {
                                info!("[Matchmaking] Could not pair players, retrying...");
                            }
                            Err(e) => {
                                error!("[Matchmaking] Failed to pop players: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("[Matchmaking] Failed to check queue size: {}", e);
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

pub async fn get_or_fetch_elo(
    pool: &deadpool_redis::Pool,
    db_pool: &PgPool,
    user_id: &Uuid,
) -> i32 {
    if let Ok(Some(elo)) = elo_cache::get(pool, user_id).await {
        return elo;
    }
    let elo = profile::get_elo(db_pool, user_id).await.unwrap_or(1500);
    let _ = elo_cache::set(pool, user_id, elo).await;
    elo
}

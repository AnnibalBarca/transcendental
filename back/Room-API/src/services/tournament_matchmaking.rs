use std::sync::Arc;

use chrono::Utc;
use deadpool_redis::Pool;
use log::{error, info};
use notification::event::{NotificationBus, NotificationEvent};
use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::tournament::{TournamentRecord, TournamentStatus};
use crate::cache::tournament_matchmaking as tournament_mm_cache;
use crate::services::tournament as tournament_service;
use crate::user_state::{PlayerSession, RedisSessionManager};

const ELO_TOLERANCE: i32 = 250;

pub async fn queue_for_tournament(
    pool: &Pool,
    db_pool: &PgPool,
    session_manager: &Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
    player_size: u32,
) -> Result<TournamentRecord, String> {
    if !tournament_mm_cache::valid_player_size(player_size) {
        return Err(format!(
            "player_size must be one of {:?}",
            crate::cache::tournament::PLAYER_SIZES
        ));
    }

    if let Some(existing) = tournament_service::get_user_tournament(pool, user_id).await? {
        if existing.status != TournamentStatus::Finished {
            return Ok(existing);
        }
    }

    let user_id_str = user_id.to_string();
    let _ = tournament_mm_cache::remove_player_from_all(pool, &user_id_str).await;

    let elo = crate::services::matchmaking::get_or_fetch_elo(pool, db_pool, user_id).await;
    if let Err(e) = tournament_mm_cache::add_player(pool, &user_id_str, elo, player_size).await {
        error!("[TournamentMatchmaking] Failed to add player to queue: {}", e);
    }

    let result = find_or_create_ranked_tournament(pool, db_pool, session_manager, notification_bus, user_id, player_size).await;

    let _ = tournament_mm_cache::remove_player_from_all(pool, &user_id_str).await;

    match &result {
        Ok(record) => {
            let session = PlayerSession {
                room_id: record.id.clone(),
                status: "tournament_lobby".into(),
                chess_ws_url: String::new(),
                chess_game_id: String::new(),
            };
            if let Err(e) = session_manager.save_session(user_id, &session).await {
                error!("[TournamentMatchmaking] Failed to save session for {}: {}", user_id, e);
            }
            notification_bus
                .send_to_user(
                    *user_id,
                    &NotificationEvent::SetState {
                        user_id: *user_id,
                        state: "tournament_lobby".into(),
                        room_id: Some(Uuid::parse_str(&record.id).unwrap_or_default()),
                        chess_ws_url: None,
                        chess_game_id: None,
                    },
                )
                .await;
            info!(
                "[TournamentMatchmaking] User {} queued for {}-player ranked tournament ({})",
                user_id, player_size, record.id
            );
        }
        Err(e) => {
            error!("[TournamentMatchmaking] Failed to find/create tournament for {}: {}", user_id, e);
        }
    }

    result
}

async fn find_or_create_ranked_tournament(
    pool: &Pool,
    db_pool: &PgPool,
    session_manager: &Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
    player_size: u32,
) -> Result<TournamentRecord, String> {
    let user_elo = crate::services::matchmaking::get_or_fetch_elo(pool, db_pool, user_id).await;

    let tournaments = tournament_service::list_tournaments(pool).await?;
    let mut candidates: Vec<TournamentRecord> = tournaments
        .into_iter()
        .filter(|t| {
            t.is_ranked
                && t.status == TournamentStatus::Waiting
                && t.player_size == player_size
                && t.players.len() < player_size as usize
                && !t.players.iter().any(|p| p.user_id == *user_id)
        })
        .collect();

    candidates.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    for record in candidates {
        let avg_elo = if record.players.is_empty() {
            user_elo
        } else {
            record.players.iter().map(|p| p.elo).sum::<i32>() / record.players.len() as i32
        };

        let waiting_seconds = Utc::now().timestamp() - record.created_at;
        let tolerance = ELO_TOLERANCE + (waiting_seconds / 60).min(300) as i32;

        if (user_elo - avg_elo).abs() <= tolerance {
            let tournament_id = record.id.clone();
            match tournament_service::join_tournament(pool, db_pool, user_id, &tournament_id).await {
                Ok(updated) => {
                    if updated.players.len() as u32 == updated.player_size {
                        let pool_clone = pool.clone();
                        let db_pool_clone = db_pool.clone();
                        let sm_clone = Arc::clone(session_manager);
                        let bus_clone = notification_bus.clone();
                        let tid = tournament_id.clone();
                        tokio::spawn(async move {
                            let _ = tournament_service::start_tournament_internal(
                                &pool_clone, &db_pool_clone, &sm_clone, &bus_clone, &tid,
                            )
                            .await;
                        });
                    }
                    return Ok(updated);
                }
                Err(e) => {
                    error!(
                        "[TournamentMatchmaking] Failed to join tournament {} for {}: {}",
                        tournament_id, user_id, e
                    );
                    continue;
                }
            }
        }
    }

    let name = format!("Tournoi classé {} joueurs", player_size);
    tournament_service::create_ranked_tournament(pool, db_pool, user_id, name, player_size).await
}

pub async fn cancel_queue(
    pool: &Pool,
    user_id: &Uuid,
) -> Result<(), String> {
    let user_id_str = user_id.to_string();
    let _ = tournament_mm_cache::remove_player_from_all(pool, &user_id_str).await;

    if let Some(record) = tournament_service::get_user_tournament(pool, user_id).await? {
        if record.status == TournamentStatus::Waiting && record.is_ranked {
            tournament_service::leave_tournament(pool, user_id, &record.id).await?;
        }
    }

    Ok(())
}

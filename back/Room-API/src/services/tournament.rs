use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::{Mutex as StdMutex, OnceLock};
use std::time::Duration;

use chrono::Utc;
use deadpool_redis::Pool;
use log::{error, info, warn};
use notification::event::{NotificationBus, NotificationEvent};
use sqlx::PgPool;
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;

use crate::cache::tournament as tournament_cache;
use crate::cache::tournament::{
    MatchStatus, PlayerEntry, PodiumEntry, RankingEntry, TournamentMatch, TournamentRecord,
    TournamentStatus, PLAYER_SIZES,
};
use crate::db::profile as db_profile;
use crate::db::user as db_user;
use crate::services::chess_client;
use crate::user_state::{PlayerSession, RedisSessionManager};

fn tournament_locks() -> &'static StdMutex<HashMap<String, Arc<AsyncMutex<()>>>> {
    static LOCKS: OnceLock<StdMutex<HashMap<String, Arc<AsyncMutex<()>>>>> = OnceLock::new();
    LOCKS.get_or_init(|| StdMutex::new(HashMap::new()))
}

async fn acquire_lock(id: &str) -> tokio::sync::OwnedMutexGuard<()> {
    let lock = {
        let mut map = tournament_locks().lock().unwrap();
        map.entry(id.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    };
    lock.lock_owned().await
}

fn in_flight_scheduling() -> &'static StdMutex<HashSet<(String, u32, u32)>> {
    static IN_FLIGHT: OnceLock<StdMutex<HashSet<(String, u32, u32)>>> = OnceLock::new();
    IN_FLIGHT.get_or_init(|| StdMutex::new(HashSet::new()))
}

pub fn total_rounds(player_size: u32) -> u32 {
    tournament_cache::total_rounds(player_size)
}

fn seed_positions(n: usize) -> Vec<usize> {
    if n == 2 {
        return vec![0, 1];
    }
    let half = n / 2;
    let sub = seed_positions(half);
    let mut result = vec![0usize; n];
    for (i, p) in sub.iter().enumerate() {
        result[2 * i] = *p;
        result[2 * i + 1] = n - 1 - *p;
    }
    result
}

fn find_match_mut(
    record: &mut TournamentRecord,
    round: u32,
    bracket_index: u32,
) -> Option<&mut TournamentMatch> {
    record
        .matches
        .iter_mut()
        .find(|m| m.round == round && m.bracket_index == bracket_index)
}

async fn player_entry(
    pool: &Pool,
    db_pool: &PgPool,
    user_id: &Uuid,
) -> Result<PlayerEntry, String> {
    let username = db_user::get_by_id(db_pool, user_id)
        .await?
        .map(|u| u.username)
        .unwrap_or_else(|| "Player".to_string());
    let elo = crate::services::matchmaking::get_or_fetch_elo(pool, db_pool, user_id).await;
    Ok(PlayerEntry {
        user_id: *user_id,
        username,
        elo,
        picture: "default.png".into(),
        alive: true,
    })
}

async fn cleanup_tournament(pool: &Pool, record: &TournamentRecord) -> Result<(), String> {
    for p in &record.players {
        let _ = tournament_cache::delete_user_tournament(pool, &p.user_id).await;
    }
    tournament_cache::delete(pool, &record.id).await?;
    tournament_cache::remove_from_list(pool, &record.id).await?;
    Ok(())
}

async fn reset_player_session(
    session_manager: &Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
) {
    if let Err(e) = session_manager.delete_session(user_id).await {
        error!("[Tournament] Failed to delete session for {}: {}", user_id, e);
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

async fn set_playing_session(
    session_manager: &Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
    tournament_id: &str,
    chess_ws_url: &str,
    chess_game_id: &str,
) {
    let session = PlayerSession {
        room_id: tournament_id.to_string(),
        status: "playing".into(),
        chess_ws_url: chess_ws_url.to_string(),
        chess_game_id: chess_game_id.to_string(),
    };
    if let Err(e) = session_manager.save_session(user_id, &session).await {
        error!("[Tournament] Failed to save session for {}: {}", user_id, e);
    }
    notification_bus
        .send_to_user(
            *user_id,
            &NotificationEvent::SetState {
                user_id: *user_id,
                state: "playing".into(),
                room_id: Some(Uuid::parse_str(tournament_id).unwrap_or_default()),
                chess_ws_url: Some(chess_ws_url.to_string()),
                chess_game_id: Some(chess_game_id.to_string()),
            },
        )
        .await;
}

async fn set_bracket_session(
    session_manager: &Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
    tournament_id: &str,
) {
    let session = PlayerSession {
        room_id: tournament_id.to_string(),
        status: "tournament_bracket".into(),
        chess_ws_url: String::new(),
        chess_game_id: String::new(),
    };
    if let Err(e) = session_manager.save_session(user_id, &session).await {
        error!("[Tournament] Failed to save session for {}: {}", user_id, e);
    }
    notification_bus
        .send_to_user(
            *user_id,
            &NotificationEvent::SetState {
                user_id: *user_id,
                state: "tournament_bracket".into(),
                room_id: Some(Uuid::parse_str(tournament_id).unwrap_or_default()),
                chess_ws_url: None,
                chess_game_id: None,
            },
        )
        .await;
}

async fn schedule_match_game(
    pool: Pool,
    session_manager: Arc<RedisSessionManager>,
    notification_bus: NotificationBus,
    tournament_id: String,
    round: u32,
    bracket_index: u32,
) -> Result<(), String> {
    let key = (tournament_id.clone(), round, bracket_index);
    {
        let mut set = in_flight_scheduling().lock().unwrap();
        if !set.insert(key.clone()) {
            return Ok(());
        }
    }

    let result = schedule_match_game_inner(
        &pool,
        &session_manager,
        &notification_bus,
        &tournament_id,
        round,
        bracket_index,
    )
    .await;

    {
        let mut set = in_flight_scheduling().lock().unwrap();
        set.remove(&key);
    }

    result
}

async fn schedule_match_game_inner(
    pool: &Pool,
    session_manager: &Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
    tournament_id: &str,
    round: u32,
    bracket_index: u32,
) -> Result<(), String> {
    let (game_id, ws_url) =
        chess_client::create_game(pool, tournament_cache::TOURNAMENT_TIME_CONTROL).await?;

    let _guard = acquire_lock(tournament_id).await;

    let mut record = tournament_cache::get(pool, tournament_id)
        .await?
        .ok_or_else(|| format!("Tournament {} not found", tournament_id))?;

    let (player1, player2, already) = {
        let m = find_match_mut(&mut record, round, bracket_index)
            .ok_or_else(|| "Match not found".to_string())?;
        if m.status != MatchStatus::Pending || m.chess_game_id.is_some() {
            (None, None, true)
        } else {
            (m.player1.clone(), m.player2.clone(), false)
        }
    };

    if already {
        return Ok(());
    }

    let player1 = player1.ok_or_else(|| "Match has no player 1".to_string())?;
    let player2 = player2.ok_or_else(|| "Match has no player 2".to_string())?;

    let players = [player1.as_str(), player2.as_str()];
    if let Err(e) = chess_client::set_game_players(pool, &game_id, &players).await {
        error!("[Tournament] Failed to register game players: {}", e);
    }

    {
        let m = find_match_mut(&mut record, round, bracket_index)
            .ok_or_else(|| "Match not found".to_string())?;
        m.chess_game_id = Some(game_id.clone());
        m.chess_ws_url = Some(ws_url.clone());
        m.status = MatchStatus::Playing;
        m.started_at = Some(Utc::now().timestamp());
    }
    record.round = round;
    tournament_cache::set(pool, &record).await?;

    tournament_cache::set_game_index(pool, &game_id, tournament_id).await?;

    let full_ws_url = format!("{}?game_id={}", ws_url, game_id);

    for uid_str in [player1, player2] {
        if let Ok(uid) = Uuid::parse_str(&uid_str) {
            set_playing_session(
                session_manager,
                notification_bus,
                &uid,
                tournament_id,
                &full_ws_url,
                &game_id,
            )
            .await;
        }
    }

    info!(
        "[Tournament] Scheduled match r{} b{} of {} -> game {}",
        round, bracket_index, tournament_id, game_id
    );
    Ok(())
}

pub async fn on_game_result(
    pool: &deadpool_redis::Pool,
    db_pool: &PgPool,
    session_manager: Arc<RedisSessionManager>,
    notification_bus: NotificationBus,
    game_id: &str,
    result: &str,
    winner_color: Option<&str>,
    white_uid: String,
    black_uid: String,
) -> Result<(), String> {
    let tournament_id = tournament_cache::get_tournament_id_by_game(pool, game_id)
        .await?
        .ok_or_else(|| format!("No tournament found for game {}", game_id))?;

    let _guard = acquire_lock(&tournament_id).await;

    let mut record = tournament_cache::get(pool, &tournament_id)
        .await?
        .ok_or_else(|| format!("Tournament {} not found", tournament_id))?;

    if record.status != TournamentStatus::Playing {
        return Ok(());
    }

    let match_idx = record
        .matches
        .iter()
        .position(|m| m.chess_game_id.as_deref() == Some(game_id))
        .ok_or_else(|| format!("Match for game {} not found", game_id))?;

    let winner_uid = if result == "cancelled" {
        black_uid.clone()
    } else if winner_color == Some("white") {
        white_uid.clone()
    } else {
        black_uid.clone()
    };

    if winner_uid.is_empty() {
        return Err("Cannot determine tournament match winner".to_string());
    }

    info!(
        "[Tournament] Game {} of {} finished: winner={}",
        game_id, tournament_id, winner_uid
    );

    let (round, bracket_index) = {
        let m = &record.matches[match_idx];
        (m.round, m.bracket_index)
    };

    let loser_uid = {
        let m = &record.matches[match_idx];
        if m.player1.as_deref() == Some(winner_uid.as_str()) {
            m.player2.clone().unwrap_or_default()
        } else if m.player2.as_deref() == Some(winner_uid.as_str()) {
            m.player1.clone().unwrap_or_default()
        } else {
            String::new()
        }
    };

    {
        let m = &mut record.matches[match_idx];
        m.status = MatchStatus::Finished;
        m.winner = Some(winner_uid.clone());
    }

    for p in &mut record.players {
        if p.user_id.to_string() == winner_uid {
            p.alive = true;
        }
        if p.user_id.to_string() == loser_uid {
            p.alive = false;
        }
    }

    let rounds = total_rounds(record.player_size);
    let is_last_round = round + 1 == rounds;
    let is_semi_round = round + 2 == rounds;

    if is_last_round {

        let all_last_round_done = record
            .matches
            .iter()
            .filter(|m| m.round == round)
            .all(|m| m.status == MatchStatus::Finished);

        if all_last_round_done {
            finish_tournament(&mut record);
            record.finished_at = Some(Utc::now().timestamp());
            tournament_cache::set(pool, &record).await?;
            info!("[Tournament] {} finished, champion={}", tournament_id, winner_uid);

            let uuids: Vec<Uuid> = record.players.iter().map(|p| p.user_id).collect();
            for uid in uuids {
                set_bracket_session(&session_manager, &notification_bus, &uid, &tournament_id).await;
            }

            if let Err(e) = apply_rewards(db_pool, &record).await {
                error!("[Tournament] Failed to apply rewards for {}: {}", tournament_id, e);
            } else {
                record.rewards_applied = true;
                let _ = tournament_cache::set(pool, &record).await;
            }
        } else {

            tournament_cache::set(pool, &record).await?;
            for uid_str in [&winner_uid, &loser_uid] {
                if let Ok(uid) = Uuid::parse_str(uid_str) {
                    set_bracket_session(&session_manager, &notification_bus, &uid, &tournament_id).await;
                }
            }
        }
        return Ok(());
    }

    if is_semi_round {

        let final_match = find_match_mut(&mut record, round + 1, 0)
            .ok_or_else(|| "Final match not found".to_string())?;
        if bracket_index == 0 {
            final_match.player1 = Some(winner_uid.clone());
        } else {
            final_match.player2 = Some(winner_uid.clone());
        }

        let small_final = find_match_mut(&mut record, round + 1, 1)
            .ok_or_else(|| "Small final match not found".to_string())?;
        if bracket_index == 0 {
            small_final.player1 = Some(loser_uid.clone());
        } else {
            small_final.player2 = Some(loser_uid.clone());
        }
    } else {

        let parent_round = round + 1;
        let parent_index = bracket_index / 2;
        let feeds_player1 = bracket_index % 2 == 0;

        let parent = find_match_mut(&mut record, parent_round, parent_index)
            .ok_or_else(|| "Parent match not found".to_string())?;
        if feeds_player1 {
            parent.player1 = Some(winner_uid.clone());
        } else {
            parent.player2 = Some(winner_uid.clone());
        }
    }

    let round_complete = record
        .matches
        .iter()
        .filter(|m| m.round == round)
        .all(|m| m.status == MatchStatus::Finished);

    if round_complete && record.round_deadline.is_none() {
        record.round = round + 1;
        record.round_deadline = Some(Utc::now().timestamp() + 5);
        info!(
            "[Tournament] Round {} of {} complete, next round in 5s",
            round, tournament_id
        );
    }

    tournament_cache::set(pool, &record).await?;

    for uid_str in [&winner_uid, &loser_uid] {
        if let Ok(uid) = Uuid::parse_str(uid_str) {
            set_bracket_session(&session_manager, &notification_bus, &uid, &tournament_id).await;
        }
    }

    Ok(())
}

fn finish_tournament(record: &mut TournamentRecord) {
    record.status = TournamentStatus::Finished;

    let final_match = record
        .matches
        .iter()
        .find(|m| m.round + 1 == total_rounds(record.player_size) && m.bracket_index == 0)
        .cloned();
    let small_final_match = record
        .matches
        .iter()
        .find(|m| m.round + 1 == total_rounds(record.player_size) && m.bracket_index == 1)
        .cloned();

    let mut podium: Vec<PodiumEntry> = Vec::new();

    if let Some(final_match) = final_match {
        let champion_uid = final_match.winner.clone().unwrap_or_default();
        record.champion = Some(champion_uid.clone());

        let champion_name = player_name(record, &champion_uid);
        podium.push(PodiumEntry {
            rank: 1,
            user_id: Uuid::parse_str(&champion_uid).unwrap_or_default(),
            username: champion_name,
        });

        let runner_uid = if final_match.player1.as_deref() == Some(&champion_uid) {
            final_match.player2.clone().unwrap_or_default()
        } else {
            final_match.player1.clone().unwrap_or_default()
        };
        if !runner_uid.is_empty() {
            podium.push(PodiumEntry {
                rank: 2,
                user_id: Uuid::parse_str(&runner_uid).unwrap_or_default(),
                username: player_name(record, &runner_uid),
            });
        }
    }

    if let Some(small_final) = small_final_match {
        let third_uid = small_final.winner.clone().unwrap_or_default();
        if !third_uid.is_empty() {
            podium.push(PodiumEntry {
                rank: 3,
                user_id: Uuid::parse_str(&third_uid).unwrap_or_default(),
                username: player_name(record, &third_uid),
            });
        }

        let fourth_uid = if small_final.player1.as_deref() == small_final.winner.as_deref() {
            small_final.player2.clone().unwrap_or_default()
        } else {
            small_final.player1.clone().unwrap_or_default()
        };
        if !fourth_uid.is_empty() {
            podium.push(PodiumEntry {
                rank: 4,
                user_id: Uuid::parse_str(&fourth_uid).unwrap_or_default(),
                username: player_name(record, &fourth_uid),
            });
        }
    }

    record.podium = podium;
    record.rankings = compute_rankings(record);
}

fn player_name(record: &TournamentRecord, user_id: &str) -> String {
    record
        .players
        .iter()
        .find(|p| p.user_id.to_string() == user_id)
        .map(|p| p.username.clone())
        .unwrap_or_else(|| "Player".to_string())
}

fn compute_rankings(record: &TournamentRecord) -> Vec<RankingEntry> {
    let mut entries: Vec<(String, u32)> = Vec::new();

// 1st-4th from podium
    for p in &record.podium {
        entries.push((p.user_id.to_string(), p.rank));
    }

// Remaining players ranked by the round they lost in (later = better)
    let rounds = total_rounds(record.player_size);
    let semi_round = rounds.saturating_sub(2);

    for r in 0..semi_round {
        for m in record.matches.iter().filter(|m| m.round == r) {
            let loser = if m.player1.as_deref() == m.winner.as_deref() {
                m.player2.clone()
            } else {
                m.player1.clone()
            };
            if let Some(loser) = loser {
                if entries.iter().any(|(uid, _)| uid == &loser) {
                    continue;
                }

                let rank = (record.player_size >> (r + 1)) + 1;
                entries.push((loser, rank));
            }
        }
    }

    let mut ranked: Vec<RankingEntry> = Vec::new();
    let mut by_rank: HashMap<u32, Vec<(String, u32)>> = HashMap::new();
    for (uid, rank) in entries {
        by_rank.entry(rank).or_default().push((uid, rank));
    }

    let mut sorted_ranks: Vec<u32> = by_rank.keys().copied().collect();
    sorted_ranks.sort();

    for rank in sorted_ranks {
        let mut group = by_rank.remove(&rank).unwrap_or_default();
        group.sort_by(|(a, _), (b, _)| {
            let elo_a = record.players.iter().find(|p| p.user_id.to_string() == *a).map(|p| p.elo).unwrap_or(0);
            let elo_b = record.players.iter().find(|p| p.user_id.to_string() == *b).map(|p| p.elo).unwrap_or(0);
            elo_b.cmp(&elo_a)
        });
        for (uid, base_rank) in group {
            let username = player_name(record, &uid);
            let user_uuid = Uuid::parse_str(&uid).unwrap_or_default();
            let elo_change = calculate_elo_change(record, &uid, base_rank);
            let xp_gained = calculate_xp(record.player_size, base_rank);
            ranked.push(RankingEntry {
                rank: base_rank,
                user_id: user_uuid,
                username,
                elo_change,
                xp_gained,
            });
        }
    }

    ranked.sort_by(|a, b| a.rank.cmp(&b.rank));
    ranked
}

fn calculate_elo_change(record: &TournamentRecord, user_id: &str, final_rank: u32) -> i32 {
    let player_elo = record
        .players
        .iter()
        .find(|p| p.user_id.to_string() == user_id)
        .map(|p| p.elo)
        .unwrap_or(1500);

    let mut sorted_elos: Vec<i32> = record.players.iter().map(|p| p.elo).collect();
    sorted_elos.sort_by(|a, b| b.cmp(a));

    let expected_rank = sorted_elos
        .iter()
        .position(|&elo| elo == player_elo)
        .map(|pos| pos as u32 + 1)
        .unwrap_or(record.player_size);

    let k = 32_i32;
    let denominator = (record.player_size.saturating_sub(1)).max(1) as i32;
    let diff = expected_rank as i32 - final_rank as i32;
    (k * diff) / denominator
}

fn calculate_xp(player_size: u32, rank: u32) -> i64 {
    let base: i64 = 100;
    let bonus: i64 = 30;
    base + bonus * (player_size.saturating_sub(rank)) as i64
}

async fn apply_rewards(db_pool: &PgPool, record: &TournamentRecord) -> Result<(), String> {
    for entry in &record.rankings {
        db_profile::update_tournament_elo(db_pool, &entry.user_id, entry.elo_change).await?;
        db_profile::add_xp(db_pool, &entry.user_id, entry.xp_gained).await?;
    }
    Ok(())
}

pub async fn create_ranked_tournament(
    pool: &Pool,
    db_pool: &PgPool,
    user_id: &Uuid,
    name: String,
    player_size: u32,
) -> Result<TournamentRecord, String> {
    create_tournament_with_ranked(pool, db_pool, user_id, name, player_size, true).await
}

async fn create_tournament_with_ranked(
    pool: &Pool,
    db_pool: &PgPool,
    user_id: &Uuid,
    name: String,
    player_size: u32,
    is_ranked: bool,
) -> Result<TournamentRecord, String> {
    if !PLAYER_SIZES.contains(&player_size) {
        return Err(format!("player_size must be one of {:?}", PLAYER_SIZES));
    }

    if let Some(existing) = tournament_cache::get_user_tournament(pool, user_id).await? {
        if let Ok(Some(existing_record)) = tournament_cache::get(pool, &existing).await {
            if existing_record.status != TournamentStatus::Finished {
                return Err("You already have an active tournament".to_string());
            }
        }
    }

    let host = player_entry(pool, db_pool, user_id).await?;
    let name = if name.trim().is_empty() {
        format!("Tournoi {} joueurs", player_size)
    } else {
        name
    };
    let record = tournament_cache::create(pool, name, player_size, host, is_ranked).await?;
    tournament_cache::set_user_tournament(pool, user_id, &record.id).await?;
    Ok(record)
}

pub async fn join_tournament(
    pool: &Pool,
    db_pool: &PgPool,
    user_id: &Uuid,
    tournament_id: &str,
) -> Result<TournamentRecord, String> {
    let _guard = acquire_lock(tournament_id).await;

    let mut record = tournament_cache::get(pool, tournament_id)
        .await?
        .ok_or_else(|| "Tournament not found".to_string())?;

    if record.status != TournamentStatus::Waiting {
        return Err("Tournament has already started".to_string());
    }
    if record.players.iter().any(|p| p.user_id == *user_id) {
        return Ok(record);
    }
    if record.players.len() as u32 >= record.player_size {
        return Err("Tournament is full".to_string());
    }

    if let Some(existing) = tournament_cache::get_user_tournament(pool, user_id).await? {
        if existing != tournament_id {
            if let Ok(Some(_)) = tournament_cache::get(pool, &existing).await {
                return Err("You already have an active tournament".to_string());
            }
        }
    }

    let entry = player_entry(pool, db_pool, user_id).await?;
    record.players.push(entry);

    tournament_cache::set(pool, &record).await?;
    tournament_cache::set_user_tournament(pool, user_id, tournament_id).await?;
    Ok(record)
}

pub async fn leave_tournament(
    pool: &Pool,
    user_id: &Uuid,
    tournament_id: &str,
) -> Result<(), String> {
    let _guard = acquire_lock(tournament_id).await;

    let mut record = tournament_cache::get(pool, tournament_id)
        .await?
        .ok_or_else(|| "Tournament not found".to_string())?;

    if record.status != TournamentStatus::Waiting {
        return Err("Cannot leave a tournament that has already started".to_string());
    }

    record.players.retain(|p| p.user_id != *user_id);
    tournament_cache::delete_user_tournament(pool, user_id).await?;

    if record.players.is_empty() {
        tournament_cache::delete(pool, &record.id).await?;
        tournament_cache::remove_from_list(pool, &record.id).await?;
        return Ok(());
    }

    if record.host_id == *user_id {
        record.host_id = record.players[0].user_id;
    }
    record.start_at = None;
    tournament_cache::set(pool, &record).await?;
    Ok(())
}

pub(crate) async fn start_tournament_internal(
    pool: &Pool,
    _db_pool: &PgPool,
    session_manager: &Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
    tournament_id: &str,
) -> Result<(), String> {
    let _guard = acquire_lock(tournament_id).await;

    let mut record = tournament_cache::get(pool, tournament_id)
        .await?
        .ok_or_else(|| "Tournament not found".to_string())?;

    if record.status != TournamentStatus::Waiting {
        return Ok(());
    }

    let n = record.player_size as usize;
    let mut players = record.players.clone();
    players.sort_by(|a, b| b.elo.cmp(&a.elo));

    let positions = seed_positions(n);
    let mut leaves: Vec<Option<PlayerEntry>> = (0..n).map(|_| None).collect();
    for (idx, &pos) in positions.iter().enumerate() {
        if let Some(p) = players.get(idx) {
            leaves[pos] = Some(p.clone());
        }
    }

    let rounds = total_rounds(record.player_size);
    let semi_round = rounds.saturating_sub(2);
    let mut matches: Vec<TournamentMatch> = Vec::new();
    for round in 0..rounds {
        let count = if round == rounds - 1 {
            2
        } else {
            n >> (round + 1)
        };
        for i in 0..count {
            let (p1, p2) = if round == 0 {
                (
                    leaves[2 * i].as_ref().map(|e| e.user_id.to_string()),
                    leaves[2 * i + 1].as_ref().map(|e| e.user_id.to_string()),
                )
            } else {
                (None, None)
            };
            matches.push(TournamentMatch {
                round,
                bracket_index: i as u32,
                player1: p1,
                player2: p2,
                chess_game_id: None,
                chess_ws_url: None,
                winner: None,
                status: MatchStatus::Pending,
                started_at: None,
            });
        }
    }

    record.matches = matches;
    record.status = TournamentStatus::Playing;
    record.round = 0;
    record.start_at = None;
    record.round_deadline = None;
    tournament_cache::set(pool, &record).await?;

    info!("[Tournament] Starting {} with small final (semi_round={})", tournament_id, semi_round);

    let first_round_matches = n >> 1;
    for i in 0..first_round_matches {
        let pool = pool.clone();
        let sm = Arc::clone(session_manager);
        let bus = notification_bus.clone();
        let tid = record.id.clone();
        tokio::spawn(async move {
            if let Err(e) = schedule_match_game(pool, sm, bus, tid, 0, i as u32).await {
                warn!("[Tournament] Failed to schedule first-round match: {}", e);
            }
        });
    }

    Ok(())
}

pub async fn get_tournament(pool: &Pool, tournament_id: &str) -> Result<TournamentRecord, String> {
    tournament_cache::get(pool, tournament_id)
        .await?
        .ok_or_else(|| "Tournament not found".to_string())
}

pub async fn get_user_tournament(
    pool: &Pool,
    user_id: &Uuid,
) -> Result<Option<TournamentRecord>, String> {
    if let Some(id) = tournament_cache::get_user_tournament(pool, user_id).await? {
        return Ok(tournament_cache::get(pool, &id).await?);
    }
    Ok(None)
}

// this leaves the lobby; for playing tournaments it forfeits the current match
// (if any); for finished tournaments it just clears the user association.
pub async fn abandon_tournament(
    pool: &Pool,
    db_pool: &PgPool,
    session_manager: &Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
) -> Result<(), String> {
    let record = match get_user_tournament(pool, user_id).await? {
        Some(r) => r,
        None => return Ok(()),
    };

    let tournament_id = record.id.clone();
    let _guard = acquire_lock(&tournament_id).await;

    let mut record = tournament_cache::get(pool, &tournament_id)
        .await?
        .ok_or_else(|| "Tournament not found".to_string())?;

    match record.status {
        TournamentStatus::Waiting => {
            leave_tournament(pool, user_id, &tournament_id).await?;
        }
        TournamentStatus::Playing => {

            let maybe_match = record.matches.iter().find(|m| {
                m.status == MatchStatus::Playing
                    && (m.player1.as_deref() == Some(user_id.to_string().as_str())
                        || m.player2.as_deref() == Some(user_id.to_string().as_str()))
            }).cloned();

            if let Some(m) = maybe_match {
                let winner_uid = if m.player1.as_deref() == Some(user_id.to_string().as_str()) {
                    m.player2.clone().unwrap_or_default()
                } else {
                    m.player1.clone().unwrap_or_default()
                };
                let winner_uid = winner_uid;
                let game_id = m.chess_game_id.clone().unwrap_or_default();
                let white_uid = m.player1.clone().unwrap_or_default();
                let black_uid = m.player2.clone().unwrap_or_default();
                drop(_guard);

                if !winner_uid.is_empty() && !game_id.is_empty() {
                    info!(
                        "[Tournament] User {} forfeits match r{} b{} in {}, winner={}",
                        user_id, m.round, m.bracket_index, tournament_id, winner_uid
                    );
                    on_game_result(
                        pool,
                        db_pool,
                        Arc::clone(session_manager),
                        notification_bus.clone(),
                        &game_id,
                        "finished",
                        None,
                        white_uid,
                        black_uid,
                    )
                    .await?;
                }
            } else {
// No current match; just mark player as eliminated.
                for p in &mut record.players {
                    if p.user_id == *user_id {
                        p.alive = false;
                    }
                }
                tournament_cache::set(pool, &record).await?;
            }
        }
        TournamentStatus::Finished => {

        }
    }

    tournament_cache::delete_user_tournament(pool, user_id).await?;
    reset_player_session(session_manager, notification_bus, user_id).await;
    Ok(())
}

pub async fn list_tournaments(pool: &Pool) -> Result<Vec<TournamentRecord>, String> {
    tournament_cache::list(pool).await
}

const MATCH_INACTIVITY_TIMEOUT_SECS: i64 = 180;

async fn player_present(
    session_manager: &Arc<RedisSessionManager>,
    uid_str: &str,
    expected_status: &str,
) -> bool {
    let uid = match Uuid::parse_str(uid_str) {
        Ok(u) => u,
        Err(_) => return false,
    };
    match session_manager.get_session(&uid).await {
        Ok(Some(s)) => s.status == expected_status,
        _ => false,
    }
}

async fn forfeit_stuck_match(
    pool: &Pool,
    db_pool: &PgPool,
    session_manager: &Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
    record: &TournamentRecord,
    m: &TournamentMatch,
) -> Result<(), String> {
    let now = Utc::now().timestamp();
    let started = match m.started_at {
        Some(s) => s,
        None => return Ok(()),
    };
    if now - started < MATCH_INACTIVITY_TIMEOUT_SECS {
        return Ok(());
    }

    let p1 = m.player1.clone().unwrap_or_default();
    let p2 = m.player2.clone().unwrap_or_default();
    let game_id = m.chess_game_id.clone().unwrap_or_default();
    if p1.is_empty() || p2.is_empty() || game_id.is_empty() {
        return Ok(());
    }

    let p1_present = player_present(session_manager, &p1, "playing").await;
    let p2_present = player_present(session_manager, &p2, "playing").await;

    if !p1_present && !p2_present {
        warn!(
            "[Tournament] Both players absent in match r{} b{} of {}, forfeiting player1",
            m.round, m.bracket_index, record.id
        );
    } else if p1_present && p2_present {
        return Ok(());
    }

    let winner_uid = if p1_present && !p2_present {
        info!(
            "[Tournament] Player2 absent in match r{} b{} of {}, forfeiting",
            m.round, m.bracket_index, record.id
        );
        p1.clone()
    } else if !p1_present && p2_present {
        info!(
            "[Tournament] Player1 absent in match r{} b{} of {}, forfeiting",
            m.round, m.bracket_index, record.id
        );
        p2.clone()
    } else {

        p1.clone()
    };

    let white_uid = m.player1.clone().unwrap_or_default();
    let black_uid = m.player2.clone().unwrap_or_default();

    let winner_color = if winner_uid == black_uid {
        Some("black")
    } else {
        Some("white")
    };

    on_game_result(
        pool,
        db_pool,
        Arc::clone(session_manager),
        notification_bus.clone(),
        &game_id,
        "finished",
        winner_color,
        white_uid,
        black_uid,
    )
    .await
}

pub async fn run_tournament_loop(
    pool: Pool,
    db_pool: PgPool,
    session_manager: Arc<RedisSessionManager>,
    notification_bus: NotificationBus,
) {
    info!("[Tournament] Loop started");

    loop {
        let _ = tournament_cache::clean_stale(&pool).await;

        let records = match tournament_cache::list(&pool).await {
            Ok(r) => r,
            Err(e) => {
                error!("[Tournament] Failed to list tournaments: {}", e);
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        for record in records {
            match record.status {
                TournamentStatus::Waiting => {
                    let now = Utc::now().timestamp();
                    let full = record.players.len() as u32 == record.player_size;
                    if full {
                        if let Some(deadline) = record.start_at {
                            if now >= deadline {
                                info!(
                                    "[Tournament] Auto-starting {} (countdown reached)",
                                    record.id
                                );
                                let pool = pool.clone();
                                let db_pool = db_pool.clone();
                                let sm = Arc::clone(&session_manager);
                                let bus = notification_bus.clone();
                                let tid = record.id.clone();
                                tokio::spawn(async move {
                                    let _ = start_tournament_internal(
                                        &pool, &db_pool, &sm, &bus, &tid,
                                    )
                                    .await;
                                });
                            }
                        }
                    } else if now - record.created_at > 1800 {
                        info!("[Tournament] Cleaning stale waiting tournament {}", record.id);
                        let _ = cleanup_tournament(&pool, &record).await;
                    }
                }
                TournamentStatus::Playing => {
                    let now = Utc::now().timestamp();

                    for m in &record.matches {
                        if m.status == MatchStatus::Playing {
                            if let Err(e) = forfeit_stuck_match(
                                &pool,
                                &db_pool,
                                &session_manager,
                                &notification_bus,
                                &record,
                                m,
                            )
                            .await
                            {
                                warn!(
                                    "[Tournament] Failed to forfeit stuck match r{} b{} of {}: {}",
                                    m.round, m.bracket_index, record.id, e
                                );
                            }
                        }
                    }

                    let deadline_passed = match record.round_deadline {
                        Some(deadline) => now >= deadline,
                        None => true,
                    };

                    if deadline_passed {
                        let mut to_schedule: Vec<(u32, u32)> = Vec::new();
                        for m in &record.matches {
                            if m.status == MatchStatus::Pending
                                && m.player1.is_some()
                                && m.player2.is_some()
                                && m.chess_game_id.is_none()
                            {
                                to_schedule.push((m.round, m.bracket_index));
                            }
                        }
                        for (round, bracket_index) in to_schedule {
                            let pool = pool.clone();
                            let sm = Arc::clone(&session_manager);
                            let bus = notification_bus.clone();
                            let tid = record.id.clone();
                            tokio::spawn(async move {
                                if let Err(e) = schedule_match_game(
                                    pool, sm, bus, tid, round, bracket_index,
                                )
                                .await
                                {
                                    warn!("[Tournament] Loop failed to schedule match: {}", e);
                                }
                            });
                        }
                    }
                }
                TournamentStatus::Finished => {
                    let now = Utc::now().timestamp();
                    if now - record.created_at > 7200 {
                        info!("[Tournament] Cleaning stale finished tournament {}", record.id);
                        let _ = cleanup_tournament(&pool, &record).await;
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

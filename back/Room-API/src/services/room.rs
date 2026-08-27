use deadpool_redis::Pool;
use log::{error, info};
use notification::event::{
    NotificationBus, NotificationEvent, RoomPlayerInfo, RoomStatePayload,
};
use rand::Rng;
use uuid::Uuid;

use crate::cache::room::{self as cache_room, PlayerData, RoomRecord, RoomStatus, RoomType};
use crate::services::chess_client;
use crate::types::RoomListItem;
use crate::user_state::{PlayerSession, RedisSessionManager};

const PUBLIC_ROOM_TITLE_PREFIXES: &[&str] = &[
    "Partie rapide", "Défi express", "Match amical",
    "Partie classique", "Échiquier libre", "JcJ ouvert",
    "Duel spontané", "Partie éclair", "Match fair-play",
    "Salle d'attente",
];

pub async fn create_room(
    pool: &deadpool_redis::Pool,
    host_id: Uuid,
    title: Option<String>,
    private: bool,
    max_players: u32,
    bot_difficulty: Option<String>,
    host_username: &str,
    time_control: Option<u32>,
) -> Result<RoomRecord, String> {
    let join_code = if private {
        Some(generate_join_code())
    } else {
        None
    };

    let display_title = title.or_else(|| {
        let idx = rand::thread_rng().gen_range(0..PUBLIC_ROOM_TITLE_PREFIXES.len());
        Some(format!("{} #{}", PUBLIC_ROOM_TITLE_PREFIXES[idx], &Uuid::new_v4().to_string()[..6]))
    });

    let host_player = PlayerData {
        player_ids: host_id,
        player_number: 1,
        player_profile_picture: "default.png".to_string(),
        player_username: host_username.to_string(),
    };

    let room = cache_room::create(
        pool,
        RoomType::Casual,
        private,
        join_code.clone(),
        display_title,
        max_players,
        host_id,
        vec![host_player],
        bot_difficulty,
        RoomStatus::Waiting,
        time_control,
    )
    .await?;

    if !private {
        cache_room::add_to_public_list(pool, &room.id, room.created_at).await?;
    }

    if let Some(ref code) = room.join_code {
        cache_room::set_join_code(pool, code, &room.id).await?;
    }

    Ok(room)
}

pub fn room_state(room: &RoomRecord) -> RoomStatePayload {
    RoomStatePayload {
        room_id: Uuid::parse_str(&room.id).unwrap_or_default(),
        title: room.title.clone(),
        private: room.private,
        join_code: room.join_code.clone(),
        host_id: room.host_id,
        players: room
            .player_ids
            .iter()
            .map(|p| RoomPlayerInfo {
                user_id: p.player_ids,
                username: p.player_username.clone(),
            })
            .collect(),
        max_players: room.max_players,
        status: match room.status {
            RoomStatus::Waiting => "waiting".into(),
            RoomStatus::Playing => "playing".into(),
            RoomStatus::Finished => "finished".into(),
        },
        time_control: room.time_control,
        game_type: if room.max_players > 2 { "tournament".into() } else { "game".into() },
    }
}

pub async fn get_room_lobby(pool: &deadpool_redis::Pool, room_id: &str) -> Result<RoomStatePayload, String> {
    let room = cache_room::get(pool, room_id)
        .await?
        .ok_or_else(|| "Room not found".to_string())?;
    Ok(room_state(&room))
}

pub async fn publish_room_update(
    pool: &deadpool_redis::Pool,
    notification_bus: &NotificationBus,
    room: &RoomRecord,
) {
    let state = room_state(room);
    let event = NotificationEvent::RoomUpdate { room: state };
    for player in &room.player_ids {
        notification_bus.send_to_user(player.player_ids, &event).await;
    }
}

pub async fn start_room(
    pool: &deadpool_redis::Pool,
    session_manager: &RedisSessionManager,
    notification_bus: &NotificationBus,
    room_id: &str,
    host_id: Uuid,
) -> Result<RoomRecord, String> {
    let mut room = cache_room::get(pool, room_id)
        .await?
        .ok_or_else(|| "Room not found".to_string())?;

    if room.host_id != host_id {
        return Err("Only the host can start the game".to_string());
    }
    if room.status != RoomStatus::Waiting {
        return Err("Room is not waiting".to_string());
    }
    if room.player_count < room.max_players {
        return Err("Room is not full yet".to_string());
    }

    let time_control = room.time_control.unwrap_or(10);
    let (game_id, ws_url) = chess_client::create_game(pool, &time_control.to_string()).await?;
    let ws_url_with_gid = format!("{}?game_id={}", ws_url, game_id);

    let player_ids: Vec<String> = room
        .player_ids
        .iter()
        .map(|p| p.player_ids.to_string())
        .collect();
    let player_refs: Vec<&str> = player_ids.iter().map(|s| s.as_str()).collect();
    if let Err(e) = chess_client::set_game_players(pool, &game_id, &player_refs).await {
        error!("[StartRoom] Failed to register game players: {}", e);
    }

    room.chess_game_id = Some(game_id.clone());
    room.chess_ws_url = Some(ws_url_with_gid.clone());
    room.status = RoomStatus::Playing;

    if room.private {
        cache_room::remove_from_public_list(pool, room_id).await?;
    }
    cache_room::set(pool, &room).await?;

    for player in &room.player_ids {
        let session = PlayerSession {
            room_id: room.id.clone(),
            status: "playing".into(),
            chess_ws_url: ws_url_with_gid.clone(),
            chess_game_id: game_id.clone(),
        };
        if let Err(e) = session_manager.save_session(&player.player_ids, &session).await {
            error!("[StartRoom] Failed to save session for {}: {}", player.player_ids, e);
        }
        notification_bus
            .send_to_user(
                player.player_ids,
                &NotificationEvent::SetState {
                    user_id: player.player_ids,
                    state: "playing".into(),
                    room_id: Uuid::parse_str(&room.id).ok(),
                    chess_ws_url: Some(ws_url_with_gid.clone()),
                    chess_game_id: Some(game_id.clone()),
                },
            )
            .await;
    }

    publish_room_update(pool, notification_bus, &room).await;

    info!(
        "[StartRoom] Room {} started: game={} ws={} tc={}",
        room.id, game_id, ws_url_with_gid, time_control
    );

    Ok(room)
}

pub async fn kick_player(
    pool: &deadpool_redis::Pool,
    session_manager: &RedisSessionManager,
    notification_bus: &NotificationBus,
    room_id: &str,
    host_id: Uuid,
    target_id: Uuid,
    ban: bool,
) -> Result<RoomRecord, String> {
    let mut room = cache_room::get(pool, room_id)
        .await?
        .ok_or_else(|| "Room not found".to_string())?;

    if room.host_id != host_id {
        return Err("Only the host can kick players".to_string());
    }
    if target_id == host_id {
        return Err("The host cannot kick themselves".to_string());
    }

    let was_member = room.player_ids.iter().any(|p| p.player_ids == target_id);
    if !was_member {
        return Err("Player is not in this room".to_string());
    }

    room.player_ids.retain(|p| p.player_ids != target_id);
    room.player_count = room.player_ids.len() as u32;

    if ban && !room.banned_ids.contains(&target_id) {
        room.banned_ids.push(target_id);
    }

    cache_room::set(pool, &room).await?;

    let cleared_session = PlayerSession {
        room_id: "0".into(),
        status: "none".into(),
        chess_ws_url: String::new(),
        chess_game_id: String::new(),
    };
    let _ = session_manager.save_session(&target_id, &cleared_session).await;

    notification_bus
        .send_to_user(
            target_id,
            &NotificationEvent::SetState {
                user_id: target_id,
                state: "none".into(),
                room_id: None,
                chess_ws_url: None,
                chess_game_id: None,
            },
        )
        .await;

    publish_room_update(pool, notification_bus, &room).await;
    publish_public_update(pool).await;

    info!(
        "[KickRoom] {} kicked player {} from room {} (ban={})",
        host_id, target_id, room_id, ban
    );

    Ok(room)
}

async fn publish_public_update(pool: &deadpool_redis::Pool) {
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(_) => return,
    };
    let _: redis::RedisResult<()> = redis::cmd("PUBLISH")
        .arg("room:public:updates")
        .arg("updated")
        .query_async(&mut *conn)
        .await;
}

pub async fn join_room(
    pool: &deadpool_redis::Pool,
    room_id: &str,
    user_id: Uuid,
    username: &str,
) -> Result<RoomRecord, String> {
    let mut room = cache_room::get(pool, room_id)
        .await?
        .ok_or_else(|| "Room not found".to_string())?;

    if room.status != RoomStatus::Waiting {
        return Err("Room is not available for joining".to_string());
    }

    if room.banned_ids.contains(&user_id) {
        return Err("You are banned from this room".to_string());
    }

    if room.player_count >= room.max_players {
        return Err("Room is full".to_string());
    }

    if room.player_ids.iter().any(|p| p.player_ids == user_id) {
        return Err("Already in this room".to_string());
    }

    let player_number = room.player_count + 1;
    let player = PlayerData {
        player_ids: user_id,
        player_number,
        player_profile_picture: "default.png".to_string(),
        player_username: username.to_string(),
    };

    room.player_ids.push(player);
    room.player_count += 1;

    if room.host_id == Uuid::nil() {
        room.host_id = user_id;
    }

    cache_room::set(pool, &room).await?;

    Ok(room)
}

pub async fn leave_room(
    pool: &deadpool_redis::Pool,
    room_id: &str,
    user_id: Uuid,
) -> Result<Option<RoomRecord>, String> {
    let mut room = cache_room::get(pool, room_id)
        .await?
        .ok_or_else(|| "Room not found".to_string())?;

    room.player_ids.retain(|p| p.player_ids != user_id);
    room.player_count = room.player_ids.len() as u32;

    if room.player_ids.is_empty() {
        cache_room::remove_from_public_list(pool, room_id).await?;
        if let Some(ref code) = room.join_code {
            cache_room::delete_join_code(pool, code).await?;
        }
        cache_room::delete(pool, room_id).await?;
        return Ok(None);
    }

    if room.host_id == user_id && !room.player_ids.is_empty() {
        room.host_id = room.player_ids[0].player_ids;
    }

    cache_room::set(pool, &room).await?;
    Ok(Some(room))
}

pub async fn list_public_rooms(pool: &deadpool_redis::Pool) -> Result<Vec<RoomListItem>, String> {
    let rooms = cache_room::list_public(pool).await?;

    let mut filtered: Vec<RoomListItem> = rooms
        .into_iter()
        .filter(|r| {
            !r.private
                && r.status == RoomStatus::Waiting
                && r.player_count < r.max_players
        })
        .map(|r| {
            let host_username = r.player_ids
                .iter()
                .find(|p| p.player_ids == r.host_id)
                .map(|p| p.player_username.clone())
                .unwrap_or_default();

            let mode = if r.max_players > 2 {
                format!("Tournoi {}j", r.max_players)
            } else {
                "1v1".to_string()
            };

            RoomListItem {
                id: r.id,
                title: r.title,
                host_username,
                player_count: r.player_count,
                max_players: r.max_players,
                created_at: r.created_at,
                private: false,
                join_code: None,
                mode,
            }
        })
        .collect();

    filtered.sort_by_key(|r| r.created_at);
    Ok(filtered)
}

pub async fn ensure_min_public_rooms(
    pool: &deadpool_redis::Pool,
    min_count: usize,
) -> Result<(), String> {
    let stale_count = cache_room::clean_stale_public(pool).await?;
    if stale_count > 0 {
        info!("[AutoFill] Cleaned {} stale public room(s) from listing", stale_count);
    }
    let actual_rooms = cache_room::list_public(pool).await?;
    let current_count = actual_rooms.len();
    if current_count >= min_count {
        return Ok(());
    }
    let to_create = min_count - current_count;

    info!("[AutoFill] Creating {} public 1v1 room(s) to reach minimum of {}", to_create, min_count);

    for _ in 0..to_create {
        let title = format!("1v1 - {}",
            PUBLIC_ROOM_TITLE_PREFIXES[rand::thread_rng().gen_range(0..PUBLIC_ROOM_TITLE_PREFIXES.len())]);

        let room = cache_room::create(
            pool,
            RoomType::Casual,
            false,
            None,
            Some(title),
            2,
            Uuid::nil(),
            vec![],
            None,
            RoomStatus::Waiting,
            None,
        )
        .await?;

        cache_room::add_to_public_list(pool, &room.id, room.created_at).await?;
        info!("[AutoFill] Created public 1v1 room: {}", room.id);
    }

    Ok(())
}

pub async fn room_to_room_list_item(room: &RoomRecord) -> RoomListItem {
    let host_username = room.player_ids
        .iter()
        .find(|p| p.player_ids == room.host_id)
        .map(|p| p.player_username.clone())
        .unwrap_or_default();

    let mode = if room.max_players > 2 {
        format!("Tournoi {}j", room.max_players)
    } else {
        "1v1".to_string()
    };

    RoomListItem {
        id: room.id.clone(),
        title: room.title.clone(),
        host_username,
        player_count: room.player_count,
        max_players: room.max_players,
        created_at: room.created_at,
        private: room.private,
        join_code: room.join_code.clone(),
        mode,
    }
}

fn generate_join_code() -> String {
    let mut rng = rand::thread_rng();
    let code: String = (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx as u8) as char
            } else {
                (b'A' + (idx - 10) as u8) as char
            }
        })
        .collect();
    code
}

pub async fn run_auto_fill_loop(
    pool: Pool,
    _notification_bus: NotificationBus,
) {
    info!("[Room-AutoFill] Auto-fill loop started");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));

    loop {
        interval.tick().await;

        if let Err(e) = ensure_min_public_rooms(&pool, 10).await {
            error!("[Room-AutoFill] Failed to ensure minimum rooms: {}", e);
        }
    }
}

pub async fn run_public_rooms_loop(pool: Pool, notification_bus: NotificationBus) {
    info!("[PublicRooms] SSE broadcast loop started");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        let rooms = match list_public_rooms(&pool).await {
            Ok(r) => r,
            Err(e) => {
                error!("[PublicRooms] Failed to collect public rooms: {}", e);
                continue;
            }
        };

        let event_rooms: Vec<notification::event::PublicRoom> = rooms
            .into_iter()
            .map(|r| notification::event::PublicRoom {
                id: r.id,
                title: r.title,
                host_username: r.host_username,
                player_count: r.player_count,
                max_players: r.max_players,
                created_at: r.created_at,
                private: r.private,
                join_code: r.join_code,
                mode: r.mode,
            })
            .collect();

        notification_bus
            .broadcast(&NotificationEvent::PublicRooms {
                rooms: event_rooms,
            })
            .await;
    }
}

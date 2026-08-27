use std::sync::Arc;

use crate::cache::matchmaking;
use crate::db::db::Database;
use crate::http::response::json_error;
use crate::services::matchmaking as matchmaking_service;
use crate::types::ServiceRequest;
use crate::user_state::{PlayerSession, RedisSessionManager};
use crate::utils::extract_user_id_from_access_token;
use log::{error, info};
use notification::event::{NotificationBus, NotificationEvent};
use serde_json::json;

pub async fn handle_play_ranked(
    db: &Database,
    redis_pool: &deadpool_redis::Pool,
    request: &ServiceRequest,
    session_manager: Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(token) => token,
        None => return json_error(401, "Missing access token"),
    };

    let user_uuid = match extract_user_id_from_access_token(token) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let user_id = user_uuid.to_string();

    let mut time_control = crate::cache::matchmaking::DEFAULT_TIME_CONTROL.to_string();
    if let Ok(body_value) = serde_json::from_str::<serde_json::Value>(&request.body) {
        if let Some(tc) = body_value.get("time_control").and_then(|v| v.as_str()) {
            if crate::cache::matchmaking::TIME_CONTROLS.contains(&tc) {
                time_control = tc.to_string();
            }
        }
    }

    let mut player_session = match session_manager.get_session(&user_uuid).await {
        Ok(Some(session)) => session,
        Ok(None) => PlayerSession {
            room_id: "0".into(),
            status: "none".into(),
            chess_ws_url: String::new(),
            chess_game_id: String::new(),
        },
        Err(e) => {
            error!("Session retrieval failed: {}", e);
            return json_error(500, "Session retrieval failed");
        }
    };

    if player_session.status.as_str() != "none" {
        return json_error(409, "Conflict: is already playing");
    }

    let elo = matchmaking_service::get_or_fetch_elo(
        redis_pool,
        db.get_pool(),
        &user_uuid,
    )
    .await;

    if let Err(e) = matchmaking::add_player(redis_pool, &user_id, elo, &time_control).await {
        error!("Matchmaking add failed: {}", e);
        return json_error(500, "Matchmaking failed");
    }

    player_session.room_id = "matchmaking".into();
    player_session.status = "matchmaking".into();

    if let Err(e) = session_manager
        .save_session(&user_uuid, &player_session)
        .await
    {
        error!("Session save failed: {}", e);
        let _ = matchmaking::remove_player(redis_pool, &user_id).await;
        return json_error(500, "Session save failed");
    }

    notification_bus
        .send_to_user(
            user_uuid,
            &NotificationEvent::SetState {
                user_id: user_uuid,
                state: "matchmaking".into(),
                room_id: None,
                chess_ws_url: None,
                chess_game_id: None,
            },
        )
        .await;

    let queue_size = match matchmaking::queue_size(redis_pool, None).await {
        Ok(size) => size,
        Err(e) => {
            error!("Queue size check failed: {}", e);
            0
        }
    };

    info!(
        "[PlayRanked] User {} joined matchmaking queue (elo={}, queue_size={})",
        user_id, elo, queue_size
    );

    json!({
        "status": 200,
        "room_status": "matchmaking",
        "queue_size": queue_size,
        "elo": elo,
    })
}

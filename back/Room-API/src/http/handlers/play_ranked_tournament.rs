use std::sync::Arc;

use crate::cache::tournament_matchmaking;
use crate::db::db::Database;
use crate::http::response::json_error;
use crate::services::tournament_matchmaking as tournament_mm_service;
use crate::types::ServiceRequest;
use crate::user_state::{PlayerSession, RedisSessionManager};
use crate::utils::extract_user_id_from_access_token;
use log::error;
use notification::event::NotificationBus;
use serde_json::json;

pub async fn handle_play_ranked_tournament(
    db: &Database,
    redis_pool: &deadpool_redis::Pool,
    request: &ServiceRequest,
    session_manager: Arc<RedisSessionManager>,
    notification_bus: &NotificationBus,
) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(t) => t,
        None => return json_error(401, "Missing access token"),
    };
    let user_id = match extract_user_id_from_access_token(token) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let body = serde_json::from_str::<serde_json::Value>(&request.body)
        .unwrap_or(serde_json::Value::Null);
    let player_size = body
        .get("player_size")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(crate::cache::tournament::DEFAULT_PLAYER_SIZE);

    if !tournament_matchmaking::valid_player_size(player_size) {
        return json_error(400, &format!("player_size must be one of {:?}", crate::cache::tournament::PLAYER_SIZES));
    }

    let player_session = match session_manager.get_session(&user_id).await {
        Ok(Some(session)) => session,
        Ok(None) => PlayerSession {
            room_id: "0".into(),
            status: "none".into(),
            chess_ws_url: String::new(),
            chess_game_id: String::new(),
        },
        Err(e) => {
            error!("[PlayRankedTournament] Session retrieval failed: {}", e);
            return json_error(500, "Session retrieval failed");
        }
    };

    if player_session.status.as_str() != "none" && player_session.status.as_str() != "tournament_lobby" {
        return json_error(409, "Conflict: is already playing or in another queue");
    }

    match tournament_mm_service::queue_for_tournament(
        redis_pool,
        db.get_pool(),
        &session_manager,
        notification_bus,
        &user_id,
        player_size,
    )
    .await
    {
        Ok(record) => {
            json!({
                "status": 200,
                "tournament": record,
                "room_status": "tournament_lobby",
            })
        }
        Err(e) => {
            error!("[PlayRankedTournament] Failed to queue for tournament: {}", e);
            json_error(400, &e)
        }
    }
}

use std::sync::Arc;

use crate::http::response::{json_error, json_ok};
use crate::services::tournament_matchmaking as tournament_mm_service;
use crate::types::ServiceRequest;
use crate::user_state::{PlayerSession, RedisSessionManager};
use crate::utils::extract_user_id_from_access_token;
use log::error;
use notification::event::{NotificationBus, NotificationEvent};

pub async fn handle_cancel_ranked_tournament(
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

    let player_session = match session_manager.get_session(&user_id).await {
        Ok(Some(session)) => session,
        Ok(None) => return json_error(409, "Not in tournament matchmaking"),
        Err(e) => {
            error!("[CancelRankedTournament] Session retrieval failed: {}", e);
            return json_error(500, "Session retrieval failed");
        }
    };

    if player_session.status.as_str() != "tournament_lobby" {
        return json_error(409, "Not in tournament matchmaking");
    }

    if let Err(e) = tournament_mm_service::cancel_queue(redis_pool, &user_id).await {
        error!("[CancelRankedTournament] Cancel queue failed: {}", e);
        return json_error(500, "Failed to cancel tournament matchmaking");
    }

    let cleared_session = PlayerSession {
        room_id: "0".into(),
        status: "none".into(),
        chess_ws_url: String::new(),
        chess_game_id: String::new(),
    };

    if let Err(e) = session_manager.save_session(&user_id, &cleared_session).await {
        error!("[CancelRankedTournament] Session save failed: {}", e);
        return json_error(500, "Session save failed");
    }

    notification_bus
        .send_to_user(
            user_id,
            &NotificationEvent::SetState {
                user_id,
                state: "none".into(),
                room_id: None,
                chess_ws_url: None,
                chess_game_id: None,
            },
        )
        .await;

    json_ok("Left tournament matchmaking")
}

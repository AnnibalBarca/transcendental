use std::sync::Arc;

use crate::cache::matchmaking;
use crate::http::response::{json_error, json_ok};
use crate::types::ServiceRequest;
use crate::user_state::{PlayerSession, RedisSessionManager};
use crate::utils::extract_user_id_from_access_token;
use notification::event::{NotificationBus, NotificationEvent};
use log::error;

pub async fn handle_cancel_ranked(
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

    let player_session = match session_manager.get_session(&user_uuid).await {
        Ok(Some(session)) => session,
        Ok(None) => return json_error(409, "Not in matchmaking"),
        Err(e) => {
            error!("Session retrieval failed: {}", e);
            return json_error(500, "Session retrieval failed");
        }
    };

    if player_session.status.as_str() != "matchmaking" {
        return json_error(409, "Not in matchmaking");
    }

    if let Err(e) = matchmaking::remove_player(redis_pool, &user_id).await {
        error!("Matchmaking remove failed: {}", e);
        return json_error(500, "Matchmaking remove failed");
    }

    let cleared_session = PlayerSession {
        room_id: "0".into(),
        status: "none".into(),
        chess_ws_url: String::new(),
        chess_game_id: String::new(),
    };

    if let Err(e) = session_manager
        .save_session(&user_uuid, &cleared_session)
        .await
    {
        error!("Session save failed: {}", e);
        return json_error(500, "Session save failed");
    }

    notification_bus
        .send_to_user(
            user_uuid,
            &NotificationEvent::SetState {
                user_id: user_uuid,
                state: "none".into(),
                room_id: None,
                chess_ws_url: None,
                chess_game_id: None,
            },
        )
        .await;

    json_ok("Left matchmaking")
}

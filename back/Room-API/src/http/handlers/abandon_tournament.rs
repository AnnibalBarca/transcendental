use std::sync::Arc;

use crate::db::db::Database;
use crate::http::response::{json_error, json_ok};
use crate::services::tournament as tournament_service;
use crate::types::ServiceRequest;
use crate::user_state::RedisSessionManager;
use crate::utils::extract_user_id_from_access_token;
use log::error;
use notification::event::NotificationBus;

pub async fn handle_abandon_tournament(
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

    match tournament_service::abandon_tournament(
        redis_pool,
        db.get_pool(),
        &session_manager,
        notification_bus,
        &user_id,
    )
    .await
    {
        Ok(_) => json_ok("Tournament abandoned"),
        Err(e) => {
            error!("[AbandonTournament] Failed to abandon tournament: {}", e);
            json_error(500, "Failed to abandon tournament")
        }
    }
}

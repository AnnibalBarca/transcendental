use crate::http::router::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use log::error;
use serde_json::json;

pub async fn stats_handler(State(state): State<AppState>) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let mut is_error = false;

    let matches_played = match state.database.game_count().await {
        Ok(count) => count,
        Err(e) => {
            error!("[Auth] Failed to count games: {}", e);
            is_error = true;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to load stats" })),
            );
        }
    };

    let users_online = match state.database.online_user_count().await {
        Ok(count) => count,
        Err(e) => {
            error!("[Auth] Failed to count online users: {}", e);
            is_error = true;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to load stats" })),
            );
        }
    };

    let active_rooms = match count_keys_matching(&state.redis_pool, "room:profile:*").await {
        Ok(count) => count,
        Err(e) => {
            error!("[Auth] Failed to count active rooms: {}", e);
            is_error = true;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to load stats" })),
            );
        }
    };

    state.metrics.record(
        "stats",
        start.elapsed().as_millis() as u64,
        is_error,
    );

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "matches_played": matches_played,
            "users_online": users_online,
            "active_rooms": active_rooms,
        })),
    )
}

async fn count_keys_matching(
    pool: &deadpool_redis::Pool,
    pattern: &str,
) -> Result<u64, String> {
    let mut conn = pool.get().await.map_err(|e| e.to_string())?;

    let mut cursor: u64 = 0;
    let mut total: u64 = 0;

    loop {
        let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(500)
            .query_async(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        total += keys.len() as u64;
        cursor = next_cursor;

        if cursor == 0 {
            break;
        }
    }

    Ok(total)
}

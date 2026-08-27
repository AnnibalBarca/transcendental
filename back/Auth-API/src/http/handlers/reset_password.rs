use crate::http::router::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

fn record_reset_password_metric(state: &AppState, start: &Instant, is_error: bool) {
    let latency_ms = start.elapsed().as_millis() as u64;
    state.metrics.record("reset_password", latency_ms, is_error);
}

fn reset_password_error(
    state: &AppState,
    start: &Instant,
    status: StatusCode,
    message: &str,
) -> Response {
    record_reset_password_metric(state, start, true);
    (status, Json(json!({ "error": message }))).into_response()
}

pub async fn reset_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    let token = payload.token.trim();
    let new_password = payload.new_password.trim();

    if token.is_empty() {
        return reset_password_error(
            &state,
            &start,
            StatusCode::BAD_REQUEST,
            "Missing reset token",
        );
    }

    if new_password.len() < 8 || new_password.len() > 255 {
        return reset_password_error(
            &state,
            &start,
            StatusCode::BAD_REQUEST,
            "Password must be between 8 and 255 characters",
        );
    }

    let user_id = match state.cache.get_password_reset_user_id(token).await {
        Ok(Some(user_id)) => user_id,
        Ok(None) => {
            return reset_password_error(
                &state,
                &start,
                StatusCode::BAD_REQUEST,
                "Invalid or expired reset token",
            );
        }
        Err(e) => {
            error!("[Auth] Failed to fetch reset token from Redis: {}", e);
            return reset_password_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        }
    };

    if let Err(e) = state.database.change_password(&user_id, new_password).await {
        error!("[Auth] Failed to update password: {}", e);
        return reset_password_error(
            &state,
            &start,
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        );
    }

    if let Err(e) = state.cache.delete_password_reset_token(token).await {
        error!("[Auth] Failed to delete reset token from Redis: {}", e);
    }

    record_reset_password_metric(&state, &start, false);

    (
        StatusCode::OK,
        Json(ResetPasswordResponse {
            message: "Password reset successfully".to_string(),
        }),
    )
        .into_response()
}

use crate::http::router::AppState;
use crate::services::email::send_password_reset_email;
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
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

fn record_forgot_password_metric(state: &AppState, start: &Instant, is_error: bool) {
    let latency_ms = start.elapsed().as_millis() as u64;
    state
        .metrics
        .record("forgot_password", latency_ms, is_error);
}

fn forgot_password_error(
    state: &AppState,
    start: &Instant,
    status: StatusCode,
    message: &str,
) -> Response {
    record_forgot_password_metric(state, start, true);
    (status, Json(json!({ "error": message }))).into_response()
}

pub async fn forgot_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return forgot_password_error(
            &state,
            &start,
            StatusCode::BAD_REQUEST,
            "Invalid email address",
        );
    }

    let user = match state.database.get_user_by_email(&email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            record_forgot_password_metric(&state, &start, false);
            return (
                StatusCode::OK,
                Json(ForgotPasswordResponse {
                    message: "If this email is registered, a reset link has been sent.".to_string(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            error!("[Auth] Failed to fetch user by email: {}", e);
            return forgot_password_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        }
    };

    if user.auth_provider == "email" {
        let token = Uuid::new_v4().to_string();
        const TOKEN_TTL_SECONDS: usize = 900;

        if let Err(e) = state
            .cache
            .set_password_reset_token(&token, &user.id, TOKEN_TTL_SECONDS)
            .await
        {
            error!(
                "[Auth] Failed to store password reset token in Redis: {}",
                e
            );
            return forgot_password_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        }

        let base_url = state.config.domain_name.trim_end_matches('/');
        let reset_url = format!("{}/reset-password?token={}", base_url, token);

        if let Err(e) = send_password_reset_email(user.email, "en".to_string(), reset_url).await {
            error!("[Auth] Failed to send password reset email: {}", e);
            return forgot_password_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to send reset email",
            );
        }
    }

    record_forgot_password_metric(&state, &start, false);

    (
        StatusCode::OK,
        Json(ForgotPasswordResponse {
            message: "If this email is registered, a reset link has been sent.".to_string(),
        }),
    )
        .into_response()
}

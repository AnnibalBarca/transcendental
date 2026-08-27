use crate::auth::utils::get_cookie_value;
use crate::auth::{jwt_manager, utils::get_bearer_token};
use crate::http::router::AppState;
use axum::{
    extract::State,
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use log::error;
use rand::Rng;
use serde::Serialize;
use serde_json::json;
use std::time::Instant;

use crate::services::email::send_validation_email;

#[derive(Serialize)]
pub struct UserDataResponse {
    pub id: String,
    pub username: Option<String>,
    pub email: String,
    pub account_validated: bool,
    pub email_validated: bool,
}

#[derive(Serialize)]
pub struct SendValidationCodeResponse {
    pub message: String,
    pub user: UserDataResponse,
}

fn record_finish_account_metric(state: &AppState, start: &Instant, is_error: bool) {
    let latency_ms = start.elapsed().as_millis() as u64;
    state.metrics.record("finish_account", latency_ms, is_error);
}

fn finish_account_error(
    state: &AppState,
    start: &Instant,
    status: StatusCode,
    message: &str,
) -> Response {
    record_finish_account_metric(state, start, true);
    (status, Json(json!({ "error": message }))).into_response()
}

pub async fn send_validation_email_code(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let start = Instant::now();

    let token =
        match get_bearer_token(&headers).or_else(|| get_cookie_value(&headers, "access_token")) {
            Some(t) => t,
            None => {
                return finish_account_error(
                    &state,
                    &start,
                    StatusCode::UNAUTHORIZED,
                    "Missing or invalid access token",
                );
            }
        };

    let claims = match jwt_manager().validate_token(&token) {
        Ok(claims) => claims,
        Err(e) => {
            error!("[Auth] Failed to validate token: {}", e);
            return finish_account_error(
                &state,
                &start,
                StatusCode::UNAUTHORIZED,
                "Invalid or expired access token",
            );
        }
    };

    let user = match state.database.get_user_by_id(&claims.sub).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!("[Auth] User not found: {}", claims.sub);
            return finish_account_error(&state, &start, StatusCode::NOT_FOUND, "User not found");
        }
        Err(e) => {
            error!("[Auth] Failed to fetch user: {}", e);
            return finish_account_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        }
    };

    if user.email_validated {
        return finish_account_error(
            &state,
            &start,
            StatusCode::CONFLICT,
            "Email is already validated",
        );
    }

    const RATE_LIMIT_SECONDS: i64 = 60;
    const CODE_TTL_SECONDS: usize = 600;

    match state.cache.get_email_validation_code(&user.id).await {
        Ok(Some((_, timestamp))) => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            if now - timestamp < RATE_LIMIT_SECONDS {
                return finish_account_error(
                    &state,
                    &start,
                    StatusCode::TOO_MANY_REQUESTS,
                    "Validation code already sent. Please wait before requesting a new one.",
                );
            }
        }
        Ok(None) => {}
        Err(e) => {
            error!("[Auth] Failed to check validation code in Redis: {}", e);
            return finish_account_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        }
    }

    let code = rand::thread_rng().gen_range(100000..=999999).to_string();

    if let Err(e) = state
        .cache
        .set_email_validation_code(&user.id, &code, CODE_TTL_SECONDS)
        .await
    {
        error!("[Auth] Failed to store validation code in Redis: {}", e);
        return finish_account_error(
            &state,
            &start,
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        );
    }

    if let Err(e) = send_validation_email(user.email.clone(), "en".to_string(), code).await {
        error!("[Auth] Failed to send validation email: {}", e);
        return finish_account_error(
            &state,
            &start,
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to send validation email",
        );
    }

    record_finish_account_metric(&state, &start, false);

    let response = SendValidationCodeResponse {
        message: "Validation code sent".to_string(),
        user: UserDataResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            account_validated: user.account_validated,
            email_validated: user.email_validated,
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}

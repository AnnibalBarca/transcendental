use crate::auth::{
    jwt_manager,
    utils::{get_bearer_token, get_cookie_value},
};
use crate::http::router::AppState;
use axum::{
    extract::State,
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;

#[derive(Deserialize)]
pub struct ValidateEmailRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct UserDataResponse {
    pub id: String,
    pub username: Option<String>,
    pub email: String,
    pub account_validated: bool,
    pub email_validated: bool,
}

#[derive(Serialize)]
pub struct ValidateEmailResponse {
    pub message: String,
    pub user: UserDataResponse,
}

fn record_validate_email_metric(state: &AppState, start: &Instant, is_error: bool) {
    let latency_ms = start.elapsed().as_millis() as u64;
    state.metrics.record("validate_email", latency_ms, is_error);
}

fn validate_email_error(
    state: &AppState,
    start: &Instant,
    status: StatusCode,
    message: &str,
) -> Response {
    record_validate_email_metric(state, start, true);
    (status, Json(json!({ "error": message }))).into_response()
}

pub async fn validate_email_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ValidateEmailRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    let token =
        match get_bearer_token(&headers).or_else(|| get_cookie_value(&headers, "access_token")) {
            Some(t) => t,
            None => {
                return validate_email_error(
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
            return validate_email_error(
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
            return validate_email_error(&state, &start, StatusCode::NOT_FOUND, "User not found");
        }
        Err(e) => {
            error!("[Auth] Failed to fetch user: {}", e);
            return validate_email_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        }
    };

    if user.email_validated {
        return validate_email_error(
            &state,
            &start,
            StatusCode::CONFLICT,
            "Email is already validated",
        );
    }

    let stored_code = match state.cache.get_email_validation_code(&user.id).await {
        Ok(Some((code, _))) => code,
        Ok(None) => {
            return validate_email_error(
                &state,
                &start,
                StatusCode::BAD_REQUEST,
                "No validation code found or expired",
            );
        }
        Err(e) => {
            error!("[Auth] Failed to fetch validation code from Redis: {}", e);
            return validate_email_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        }
    };

    if payload.code.trim() != stored_code {
        return validate_email_error(
            &state,
            &start,
            StatusCode::BAD_REQUEST,
            "Invalid validation code",
        );
    }

    if let Err(e) = state.database.validate_email(&user.id).await {
        error!("[Auth] Failed to validate email in database: {}", e);
        return validate_email_error(
            &state,
            &start,
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        );
    }

    if let Ok(user_uuid) = uuid::Uuid::parse_str(&user.id) {
        if let Err(e) = state.cache.invalidate_user_profile_cache(&user_uuid).await {
            error!("[Auth] Failed to invalidate user profile cache: {}", e);
        }
    }

    if let Err(e) = state.cache.delete_email_validation_code(&user.id).await {
        error!("[Auth] Failed to delete validation code from Redis: {}", e);
    }

    if let Err(e) = state.cache.set_email_validated(&user.id, 86400).await {
        error!("[Auth] Failed to set email validated flag in Redis: {}", e);
    }

    record_validate_email_metric(&state, &start, false);

    let response = ValidateEmailResponse {
        message: "Email validated successfully".to_string(),
        user: UserDataResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            account_validated: user.account_validated,
            email_validated: true,
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}

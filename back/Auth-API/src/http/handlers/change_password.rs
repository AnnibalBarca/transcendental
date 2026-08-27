use crate::auth::utils::get_cookie_value;
use crate::auth::{hash_refresh_token, jwt_manager, utils::get_bearer_token};
use crate::http::router::AppState;
use axum::{
    extract::State,
    http::{
        header::{HeaderMap, HeaderValue, SET_COOKIE},
        StatusCode,
    },
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct UserDataResponse {
    pub id: String,
    pub username: Option<String>,
    pub email: String,
    pub account_validated: bool,
}

#[derive(Serialize)]
pub struct ChangePasswordResponse {
    pub message: String,
    pub user: UserDataResponse,
}

fn record_change_password_metric(state: &AppState, start: &Instant, is_error: bool) {
    let latency_ms = start.elapsed().as_millis() as u64;
    state
        .metrics
        .record("change_password", latency_ms, is_error);
}

fn change_password_error(
    state: &AppState,
    start: &Instant,
    status: StatusCode,
    message: &str,
) -> Response {
    record_change_password_metric(state, start, true);
    (status, Json(json!({ "error": message }))).into_response()
}

pub async fn change_password_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    let token =
        match get_bearer_token(&headers).or_else(|| get_cookie_value(&headers, "access_token")) {
            Some(t) => t,
            None => {
                return change_password_error(
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
            return change_password_error(
                &state,
                &start,
                StatusCode::UNAUTHORIZED,
                "Invalid or expired access token",
            );
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return change_password_error(
                &state,
                &start,
                StatusCode::BAD_REQUEST,
                "Invalid user ID in token",
            );
        }
    };

    let old_password = payload.old_password.trim();
    let new_password = payload.new_password.trim();
    if old_password.is_empty() || new_password.is_empty() {
        return change_password_error(
            &state,
            &start,
            StatusCode::BAD_REQUEST,
            "Old and new passwords are required",
        );
    }

    if old_password == new_password {
        return change_password_error(
            &state,
            &start,
            StatusCode::BAD_REQUEST,
            "The new password must be different from the old password",
        );
    }

    let user = match state.database.get_user_by_id(&claims.sub).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!("[Auth] User not found: {}", claims.sub);
            return change_password_error(&state, &start, StatusCode::NOT_FOUND, "User not found");
        }
        Err(e) => {
            error!("[Auth] Failed to fetch user: {}", e);
            return change_password_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            );
        }
    };

    if user.auth_provider != "email" {
        return change_password_error(
            &state,
            &start,
            StatusCode::FORBIDDEN,
            "Only email-based users can change their password",
        );
    }

    let is_valid = match bcrypt::verify(old_password, &user.password_hash) {
        Ok(valid) => valid,
        Err(e) => {
            error!("[Auth] Password verification failed: {}", e);
            return change_password_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        }
    };

    if !is_valid {
        return change_password_error(
            &state,
            &start,
            StatusCode::UNAUTHORIZED,
            "Old password is incorrect",
        );
    }

    match state
        .database
        .change_password(&claims.sub, new_password)
        .await
    {
        Ok(true) => {}
        Ok(false) => {
            error!("[Auth] Failed to update password for user: {}", claims.sub);
            return change_password_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update password",
            );
        }
        Err(e) => {
            error!("[Auth] Failed to update password: {}", e);
            return change_password_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update password",
            );
        }
    }

    info!("[Auth] User {} changed password successfully", user_id);

    let token_response = match jwt_manager().generate_token_pair(
        &claims.sub,
        user.username.as_deref().unwrap_or(""),
        &user.email,
    ) {
        Ok(tokens) => tokens,
        Err(e) => {
            error!("[Auth] Failed to generate new token pair: {}", e);
            record_change_password_metric(&state, &start, false);
            return (
                StatusCode::OK,
                Json(ChangePasswordResponse {
                    message: "Password changed, but session refresh failed. Please log in again."
                        .to_string(),
                    user: UserDataResponse {
                        id: user.id,
                        username: user.username,
                        email: user.email,
                        account_validated: user.account_validated,
                    },
                }),
            )
                .into_response();
        }
    };

    let refresh_hash = hash_refresh_token(&token_response.refresh_token);
    let refresh_expires_at =
        Utc::now() + Duration::seconds(token_response.refresh_token_expires_in);

    if let Err(e) = state
        .database
        .store_refresh_token(&user_id, &refresh_hash, refresh_expires_at)
        .await
    {
        error!("[Auth] Failed to store refresh token: {}", e);
    }

    let access_cookie = format!(
        "access_token={}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
        token_response.access_token, token_response.access_token_expires_in
    );
    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly; SameSite=Strict; Path=/api/auth/refresh; Max-Age={}",
        token_response.refresh_token, token_response.refresh_token_expires_in
    );

    record_change_password_metric(&state, &start, false);

    let response_body = ChangePasswordResponse {
        message: "Password changed successfully".to_string(),
        user: UserDataResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            account_validated: user.account_validated,
        },
    };

    let mut response = (StatusCode::OK, Json(response_body)).into_response();

    let response_headers = response.headers_mut();
    if let Ok(v) = HeaderValue::from_str(&access_cookie) {
        response_headers.append(SET_COOKIE, v);
    }
    if let Ok(v) = HeaderValue::from_str(&refresh_cookie) {
        response_headers.append(SET_COOKIE, v);
    }

    response
}

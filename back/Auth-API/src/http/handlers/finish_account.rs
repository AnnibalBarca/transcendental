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
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FinishAccountRequest {
    pub username: String,
}

#[derive(Serialize)]
pub struct UserDataResponse {
    pub id: String,
    pub username: Option<String>,
    pub email: String,
    pub account_validated: bool,
}

#[derive(Serialize)]
pub struct FinishAccountResponse {
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

pub async fn finish_account_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<FinishAccountRequest>,
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

    let username = match api_core::username::validate_username(&payload.username) {
        Ok(name) => name,
        Err(msg) => return finish_account_error(&state, &start, StatusCode::BAD_REQUEST, msg),
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

    if user.account_validated {
        return finish_account_error(
            &state,
            &start,
            StatusCode::CONFLICT,
            "Account is already validated",
        );
    }

    if !user.email_validated {
        return finish_account_error(
            &state,
            &start,
            StatusCode::FORBIDDEN,
            "Email not validated. Please validate your email before finishing account setup.",
        );
    }

    match state.database.username_exists(&username).await {
        Ok(true) => {
            return finish_account_error(
                &state,
                &start,
                StatusCode::CONFLICT,
                "Username already taken",
            );
        }
        Ok(false) => {}
        Err(e) => {
            error!("[Auth] Failed to check username: {}", e);
            return finish_account_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            );
        }
    }

    match state
        .database
        .set_username_and_validate(&claims.sub, &username)
        .await
    {
        Ok(true) => {
            if let Ok(user_uuid) = Uuid::parse_str(&claims.sub) {
                if let Err(e) = state.cache.invalidate_user_profile_cache(&user_uuid).await {
                    warn!("[Auth] Failed to invalidate user profile cache: {}", e);
                }
            }
        }
        Ok(false) => {
            error!("[Auth] Failed to update username for user: {}", claims.sub);
            return finish_account_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to complete account validation",
            );
        }
        Err(e) => {
            error!("[Auth] Failed to update username: {}", e);
            return finish_account_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        }
    }

    let updated_user = match state.database.get_user_by_id(&claims.sub).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!("[Auth] User not found after update: {}", claims.sub);
            return finish_account_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "User not found",
            );
        }
        Err(e) => {
            error!("[Auth] Failed to fetch updated user: {}", e);
            return finish_account_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        }
    };

    let token_response =
        match jwt_manager().generate_token_pair(&claims.sub, &username, &updated_user.email) {
            Ok(tokens) => tokens,
            Err(e) => {
                error!("[Auth] Failed to generate new token: {}", e);
                return finish_account_error(
                    &state,
                    &start,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to generate new token",
                );
            }
        };

    let user_uuid = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return finish_account_error(
                &state,
                &start,
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error",
            );
        }
    };

    let refresh_hash = hash_refresh_token(&token_response.refresh_token);
    let refresh_expires_at =
        Utc::now() + Duration::seconds(token_response.refresh_token_expires_in);

    if let Err(e) = state
        .database
        .store_refresh_token(&user_uuid, &refresh_hash, refresh_expires_at)
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

    info!(
        "[Auth] User {} finished account setup with username: {} (new token generated)",
        claims.sub, username
    );

    record_finish_account_metric(&state, &start, false);

    let response_body = FinishAccountResponse {
        message: "Account validation completed successfully".to_string(),
        user: UserDataResponse {
            id: updated_user.id,
            username: updated_user.username,
            email: updated_user.email,
            account_validated: updated_user.account_validated,
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

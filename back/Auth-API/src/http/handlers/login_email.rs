use crate::auth::{hash_refresh_token, jwt_manager};
use crate::http::response::{error_response, internal_error_responce};
use crate::http::router::AppState;
use axum::{
    extract::State,
    http::{
        header::{HeaderValue, SET_COOKIE},
        StatusCode,
    },
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use log::{error, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LoginEmailRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserData {
    pub id: String,
    pub username: Option<String>,
    pub email: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub message: String,
    pub user: UserData,
    pub access_token_expires_in: i64,
    pub refresh_token_expires_in: i64,
    pub token_type: String,
}

pub async fn login_email_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginEmailRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    if payload.email.trim().is_empty() || payload.password.is_empty() {
        let latency_ms = start.elapsed().as_millis() as u64;
        state.metrics.record("login", latency_ms, true);
        return error_response(StatusCode::UNAUTHORIZED, "Email and password are required");
    }

    let user = match state.database.get_user_by_email(payload.email.trim()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("login", latency_ms, true);
            return error_response(StatusCode::UNAUTHORIZED, "Invalid email or password");
        }
        Err(e) => {
            error!("[Auth] Failed to fetch user: {}", e);
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("login", latency_ms, true);
            return internal_error_responce();
        }
    };

    if user.is_banned {
        let latency_ms = start.elapsed().as_millis() as u64;
        state.metrics.record("login", latency_ms, true);
        return error_response(StatusCode::FORBIDDEN, "This account has been banned");
    }

    if user.auth_provider != "email" {
        let latency_ms = start.elapsed().as_millis() as u64;
        state.metrics.record("login", latency_ms, true);
        return error_response(StatusCode::UNAUTHORIZED, "Invalid email or password");
    }

    let is_valid = match bcrypt::verify(&payload.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(e) => {
            error!("[Auth] Password verification failed: {}", e);
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("login", latency_ms, true);
            return internal_error_responce();
        }
    };

    if !is_valid {
        let latency_ms = start.elapsed().as_millis() as u64;
        state.metrics.record("login", latency_ms, true);
        return error_response(StatusCode::UNAUTHORIZED, "Invalid email or password");
    }

    let token_response = match jwt_manager().generate_token_pair(
        &user.id,
        user.username.as_deref().unwrap_or(""),
        &user.email,
    ) {
        Ok(tokens) => tokens,
        Err(e) => {
            error!("[Auth] Failed to generate token pair: {}", e);
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("login", latency_ms, true);
            return internal_error_responce();
        }
    };

    let user_id = match Uuid::parse_str(&user.id) {
        Ok(id) => id,
        Err(e) => {
            error!("[Auth] Invalid user id: {}", e);
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("login", latency_ms, true);
            return internal_error_responce();
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
        let latency_ms = start.elapsed().as_millis() as u64;
        state.metrics.record("login", latency_ms, true);
        return internal_error_responce();
    }

    info!(
        "[Auth] User {} logged in successfully",
        user.username.as_deref().unwrap_or("unknown")
    );

    let access_cookie = format!(
        "access_token={}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
        token_response.access_token, token_response.access_token_expires_in
    );

    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly; SameSite=Strict; Path=/api/auth/refresh; Max-Age={}",
        token_response.refresh_token, token_response.refresh_token_expires_in
    );

    let latency_ms = start.elapsed().as_millis() as u64;
    state.metrics.record("login", latency_ms, false);

    let mut response = (
        StatusCode::OK,
        Json(AuthResponse {
            message: "Login successful".to_string(),
            user: UserData {
                id: user.id,
                username: user.username,
                email: user.email,
            },
            access_token_expires_in: token_response.access_token_expires_in,
            refresh_token_expires_in: token_response.refresh_token_expires_in,
            token_type: token_response.token_type,
        }),
    )
        .into_response();

    let headers = response.headers_mut();
    if let Ok(cookie_val) = HeaderValue::from_str(&access_cookie) {
        headers.append(SET_COOKIE, cookie_val);
    }
    if let Ok(cookie_val) = HeaderValue::from_str(&refresh_cookie) {
        headers.append(SET_COOKIE, cookie_val);
    }

    response
}

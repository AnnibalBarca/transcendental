use crate::auth::{hash_refresh_token, jwt_manager};
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
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserData {
    pub id: String,
    pub username: Option<String>,
    pub email: String,
    pub account_validated: bool,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub message: String,
    pub user: UserData,
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_in: i64,
    pub refresh_token_expires_in: i64,
    pub token_type: String,
}

fn password_entropy(password: String) -> bool {
    if password.is_empty() {
        return false;
    }

    let volume = (if password.chars().any(|c| c.is_ascii_uppercase()) {
        26
    } else {
        0
    }) + (if password.chars().any(|c| c.is_ascii_lowercase()) {
        26
    } else {
        0
    }) + (if password.chars().any(|c| c.is_ascii_digit()) {
        10
    } else {
        0
    }) + (if password.chars().any(|c| !c.is_ascii_alphanumeric()) {
        32
    } else {
        0
    });

    return (password.len() as f64 * (volume as f64).ln()) >= 48.0f64;
}

pub async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    if !payload.email.contains('@') || payload.email.len() > 255 {
        let latency_ms = start.elapsed().as_millis() as u64;
        state.metrics.record("register", latency_ms, true);
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid email format"
            })),
        )
            .into_response();
    }

    if !password_entropy(payload.password.clone()) {
        let latency_ms = start.elapsed().as_millis() as u64;
        state.metrics.record("register", latency_ms, true);
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Password is invalid"
            })),
        )
            .into_response();
    }

    match state.database.get_user_by_email(&payload.email).await {
        Ok(Some(user)) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("register", latency_ms, true);
            if user.is_banned {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "This email has been banned"
                    })),
                )
                    .into_response();
            }
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Email already registered"
                })),
            )
                .into_response();
        }
        Ok(None) => {}
        Err(e) => {
            error!("[Auth] Error checking email: {}", e);
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("register", latency_ms, true);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Database error"
                })),
            )
                .into_response();
        }
    }

    let user = match state
        .database
        .create_user(&payload.email, &payload.password)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            error!("[Auth] Error creating user: {}", e);
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("register", latency_ms, true);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to create user"
                })),
            )
                .into_response();
        }
    };

    let token_response = match jwt_manager().generate_token_pair(&user.id, "", &user.email) {
        Ok(tokens) => tokens,
        Err(e) => {
            error!("[Auth] Failed to generate tokens: {}", e);
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("register", latency_ms, true);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to generate authentication tokens"
                })),
            )
                .into_response();
        }
    };

    let user_id = match Uuid::parse_str(&user.id) {
        Ok(id) => id,
        Err(e) => {
            error!("[Auth] Invalid user id: {}", e);
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("register", latency_ms, true);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal server error"
                })),
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
        let latency_ms = start.elapsed().as_millis() as u64;
        state.metrics.record("register", latency_ms, true);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to store refresh token"
            })),
        )
            .into_response();
    }

    info!("[Auth] User {} registered successfully", payload.email);

    let access_cookie = format!(
        "access_token={}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
        token_response.access_token, token_response.access_token_expires_in
    );

    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly; SameSite=Strict; Path=/api/auth/refresh; Max-Age={}",
        token_response.refresh_token, token_response.refresh_token_expires_in
    );

    let latency_ms = start.elapsed().as_millis() as u64;
    state.metrics.record("register", latency_ms, false);

    let mut response = (
        StatusCode::CREATED,
        Json(RegisterResponse {
            message: "Register success".to_string(),
            user: UserData {
                id: user.id,
                username: user.username,
                email: user.email,
                account_validated: user.account_validated,
            },
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
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

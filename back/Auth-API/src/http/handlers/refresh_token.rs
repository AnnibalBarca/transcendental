use crate::auth::{hash_refresh_token, jwt_manager};
use crate::http::router::AppState;
use axum::{
    extract::State,
    http::{
        header::{HeaderValue, SET_COOKIE},
        HeaderMap, StatusCode,
    },
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use log::{error, info};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Serialize)]
pub struct RefreshTokenResponse {
    pub message: String,
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_in: i64,
    pub refresh_token_expires_in: i64,
    pub token_type: String,
}

pub async fn refresh_token_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    let refresh_token = match get_cookie_value(&headers, "refresh_token") {
        Some(token) => token,
        None => {
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("refresh", latency_ms, true);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Missing refresh token cookie"
                })),
            )
                .into_response();
        }
    };

    let token_hash = hash_refresh_token(&refresh_token);
    let stored = match state.database.get_refresh_token(&token_hash).await {
        Ok(Some(token)) => token,
        Ok(None) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("refresh", latency_ms, true);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid refresh token"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!("[Auth] Failed to load refresh token: {}", e);
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("refresh", latency_ms, true);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal server error"
                })),
            )
                .into_response();
        }
    };

    if stored.revoked || Utc::now() > stored.expires_at {
        let latency_ms = start.elapsed().as_millis() as u64;
        state.metrics.record("refresh", latency_ms, true);
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Refresh token expired"
            })),
        )
            .into_response();
    }

    let user = match state.database.get_user_by_id(&stored.user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("refresh", latency_ms, true);
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "User not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!("[Auth] Failed to load user: {}", e);
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("refresh", latency_ms, true);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal server error"
                })),
            )
                .into_response();
        }
    };

    let token_response = match jwt_manager().generate_token_pair(
        &user.id,
        user.username.as_deref().unwrap_or(""),
        &user.email,
    ) {
        Ok(tokens) => tokens,
        Err(e) => {
            error!("[Auth] Token generation failed: {}", e);
            let latency_ms = start.elapsed().as_millis() as u64;
            state.metrics.record("refresh", latency_ms, true);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Token generation failed"
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
            state.metrics.record("refresh", latency_ms, true);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal server error"
                })),
            )
                .into_response();
        }
    };

    let new_refresh_hash = hash_refresh_token(&token_response.refresh_token);
    let refresh_expires_at =
        Utc::now() + Duration::seconds(token_response.refresh_token_expires_in);

    if let Err(e) = state
        .database
        .store_refresh_token(&user_id, &new_refresh_hash, refresh_expires_at)
        .await
    {
        error!("[Auth] Failed to store refresh token: {}", e);
        let latency_ms = start.elapsed().as_millis() as u64;
        state.metrics.record("refresh", latency_ms, true);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Internal server error"
            })),
        )
            .into_response();
    }

    info!(
        "[Auth] Refresh token rotated for user {}",
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
    state.metrics.record("refresh", latency_ms, false);

    let mut response = (
        StatusCode::OK,
        Json(RefreshTokenResponse {
            message: "Token refreshed".to_string(),
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

fn get_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(axum::http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').map(|c| c.trim()).find_map(|cookie| {
                let mut parts = cookie.splitn(2, '=');
                let key = parts.next()?;
                let value = parts.next()?;
                if key == name {
                    Some(value.to_string())
                } else {
                    None
                }
            })
        })
}

use crate::auth::{hash_refresh_token, utils::get_cookie_value};
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
use log::{error, info};
use serde_json::json;

pub async fn logout_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let mut is_error = false;

    if let Some(refresh_token) = get_cookie_value(&headers, "refresh_token") {
        let token_hash = hash_refresh_token(&refresh_token);
        if let Err(e) = state.database.revoke_refresh_token(&token_hash).await {
            error!("[Auth] Failed to revoke refresh token: {}", e);
            is_error = true;
        } else {
            info!("[Auth] Refresh token revoked");
        }
    }

    if let Some(access_token) = get_cookie_value(&headers, "access_token") {
        if let Err(e) = state.cache.blacklist_token(&access_token, 900).await {
            error!("[Auth] Failed to blacklist access token: {}", e);
            is_error = true;
        } else {
            info!("[Auth] Access token blacklisted");
        }
    }

    info!("[Auth] User logged out");

    let access_cookie = "access_token=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0".to_string();
    let refresh_cookie =
        "refresh_token=; HttpOnly; SameSite=Strict; Path=/api/auth/refresh; Max-Age=0".to_string();

    let mut response = (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": "You have been logged out."
        })),
    )
        .into_response();

    let headers = response.headers_mut();
    if let Ok(cookie_val) = HeaderValue::from_str(&access_cookie) {
        headers.append(SET_COOKIE, cookie_val);
    }
    if let Ok(cookie_val) = HeaderValue::from_str(&refresh_cookie) {
        headers.append(SET_COOKIE, cookie_val);
    }

    let latency_ms = start.elapsed().as_millis() as u64;
    state.metrics.record("logout", latency_ms, is_error);

    response
}

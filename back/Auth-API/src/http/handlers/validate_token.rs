use crate::auth::jwt_manager;
use crate::http::router::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use log::error;
use serde_json::json;

pub async fn validate_token_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    let token = get_bearer_token(&headers).or_else(|| get_cookie_value(&headers, "access_token"));

    let response = if let Some(token_val) = token {
        match state.cache.is_token_blacklisted(&token_val).await {
            Ok(true) => {
                log::info!("[Auth] Token is blacklisted");
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Token has been revoked"
                    })),
                )
                    .into_response()
            }
            Ok(false) => match jwt_manager().validate_access_token(&token_val) {
                Ok(claims) => (
                    StatusCode::OK,
                    Json(json!({
                        "valid": true,
                        "sub": claims.sub,
                        "exp": claims.exp,
                        "username": claims.username,
                        "email": claims.email
                    })),
                )
                    .into_response(),
                Err(e) => {
                    error!("[Auth] Invalid access token: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "error": "Invalid token"
                        })),
                    )
                        .into_response()
                }
            },
            Err(e) => {
                error!("[Auth] Error checking token blacklist: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Internal server error"
                    })),
                )
                    .into_response()
            }
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Missing session cookie"
            })),
        )
            .into_response()
    };

    let latency_ms = start.elapsed().as_millis() as u64;
    let is_error = !response.status().is_success();
    state.metrics.record("validate", latency_ms, is_error);

    response
}

fn get_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|token| token.to_string())
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

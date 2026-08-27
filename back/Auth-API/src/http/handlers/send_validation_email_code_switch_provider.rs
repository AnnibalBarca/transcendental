use crate::auth::utils::get_cookie_value;
use crate::auth::{jwt_manager, utils::get_bearer_token};
use crate::http::response::{error_response, internal_error_responce};
use crate::http::router::AppState;
use axum::{
    extract::State,
    http::{header::HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use log::error;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use std::time::Instant;

use crate::services::email::send_validation_email;

#[derive(Deserialize)]
pub struct SwitchProviderEmailRequest {
    pub email: String,
}

#[derive(Serialize)]
pub struct SwitchProviderEmailResponse {
    pub message: String,
}

pub async fn send_validation_email_code_switch_provider_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SwitchProviderEmailRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    let token =
        match get_bearer_token(&headers).or_else(|| get_cookie_value(&headers, "access_token")) {
            Some(t) => t,
            None => {
                return error_response(StatusCode::UNAUTHORIZED, "Missing or invalid access token");
            }
        };

    let claims = match jwt_manager().validate_token(&token) {
        Ok(claims) => claims,
        Err(e) => {
            error!("[Auth] Failed to validate token: {}", e);
            return error_response(StatusCode::UNAUTHORIZED, "Invalid or expired access token");
        }
    };

    let trimmed_email = payload.email.trim().to_lowercase();
    if trimmed_email.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Email is required");
    }

    match state.database.get_user_by_email(&trimmed_email).await {
        Ok(Some(existing_user)) => {
            if existing_user.id != claims.sub {
                return error_response(StatusCode::CONFLICT, "Email is already in use");
            }
        }
        Ok(None) => {}
        Err(e) => {
            error!("[Auth] Failed to check email existence: {}", e);
            return internal_error_responce();
        }
    };

    const RATE_LIMIT_SECONDS: i64 = 60;
    const CODE_TTL_SECONDS: usize = 600;

    match state.cache.get_email_validation_code(&claims.sub).await {
        Ok(Some((_, timestamp))) => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            if now - timestamp < RATE_LIMIT_SECONDS {
                return error_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    "Validation code already sent. Please wait before requesting a new one.",
                );
            }
        }
        Ok(None) => {}
        Err(e) => {
            error!("[Auth] Failed to check validation code in Redis: {}", e);
            return internal_error_responce();
        }
    }

    let code = rand::thread_rng().gen_range(100000..=999999).to_string();

    if let Err(e) = state
        .cache
        .set_email_validation_code(&claims.sub, &code, CODE_TTL_SECONDS)
        .await
    {
        error!("[Auth] Failed to store validation code in Redis: {}", e);
        return internal_error_responce();
    }

    if let Err(e) = send_validation_email(trimmed_email, "en".to_string(), code).await {
        error!("[Auth] Failed to send validation email: {}", e);
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to send validation email",
        );
    }

    let latency_ms = start.elapsed().as_millis() as u64;
    state
        .metrics
        .record("send_switch_provider_code", latency_ms, false);

    (
        StatusCode::OK,
        Json(SwitchProviderEmailResponse {
            message: "Validation code sent successfully".to_string(),
        }),
    )
        .into_response()
}

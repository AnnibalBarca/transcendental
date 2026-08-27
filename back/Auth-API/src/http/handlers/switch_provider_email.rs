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
use serde::Deserialize;
use serde::Serialize;
use std::time::Instant;

#[derive(Deserialize)]
pub struct SwitchProviderToEmailRequest {
    pub email: String,
    pub code: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct SwitchProviderToEmailResponse {
    pub message: String,
}

pub async fn switch_provider_to_email_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SwitchProviderToEmailRequest>,
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

    let user_uuid = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(e) => {
            error!("[Auth] Invalid user UUID in token: {}", e);
            return error_response(StatusCode::BAD_REQUEST, "Invalid user ID");
        }
    };

    let trimmed_email = payload.email.trim().to_lowercase();
    let trimmed_code = payload.code.trim();

    if trimmed_email.is_empty() || trimmed_code.is_empty() || payload.password.is_empty() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "Email, code, and password are required",
        );
    }

    if payload.password.len() < 8 {
        return error_response(
            StatusCode::BAD_REQUEST,
            "Password must be at least 8 characters long",
        );
    }

    match state.cache.get_email_validation_code(&claims.sub).await {
        Ok(Some((stored_code, _))) => {
            if stored_code != trimmed_code {
                return error_response(StatusCode::BAD_REQUEST, "Invalid verification code");
            }
        }
        Ok(None) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "Verification code expired or not requested",
            );
        }
        Err(e) => {
            error!(
                "[Auth] Failed to retrieve validation code from Redis: {}",
                e
            );
            return internal_error_responce();
        }
    }

    match state.database.get_user_by_email(&trimmed_email).await {
        Ok(Some(existing_user)) => {
            if existing_user.id != claims.sub {
                return error_response(StatusCode::CONFLICT, "Email is already in use");
            }
        }
        Ok(None) => {}
        Err(e) => {
            error!("[Auth] Failed to check email: {}", e);
            return internal_error_responce();
        }
    }

    let password_hash = match bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            error!("[Auth] Failed to hash password: {}", e);
            return internal_error_responce();
        }
    };

    if let Err(e) = state
        .database
        .change_provider_to_email(&claims.sub, &trimmed_email, &password_hash)
        .await
    {
        error!(
            "[Auth] Failed to change provider to email in database: {}",
            e
        );
        return internal_error_responce();
    }

    let _ = state.cache.delete_email_validation_code(&claims.sub).await;
    let _ = state.cache.invalidate_user_profile_cache(&user_uuid).await;

    let latency_ms = start.elapsed().as_millis() as u64;
    state
        .metrics
        .record("switch_provider_to_email", latency_ms, false);

    (
        StatusCode::OK,
        Json(SwitchProviderToEmailResponse {
            message: "Successfully switched provider to email".to_string(),
        }),
    )
        .into_response()
}

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
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Deserialize)]
pub struct SwitchProviderToGoogleRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct SwitchProviderToGoogleResponse {
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct GoogleTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
}

#[derive(Deserialize, Serialize, Debug)]
struct GoogleUserInfo {
    id: String,
    email: String,
    name: Option<String>,
    picture: Option<String>,
}

async fn exchange_code_for_token(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<GoogleTokenResponse, String> {
    let client = reqwest::Client::new();
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Google token error: {}", body));
    }

    res.json::<GoogleTokenResponse>()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))
}

async fn fetch_google_user_info(access_token: &str) -> Result<GoogleUserInfo, String> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Google userinfo error: {}", body));
    }

    res.json::<GoogleUserInfo>()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))
}

pub async fn switch_provider_to_google_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SwitchProviderToGoogleRequest>,
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

    if payload.code.trim().is_empty() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "Google authorization code is required",
        );
    }

    let client_id = match &state.config.google_client_id {
        Some(id) => id.clone(),
        None => {
            return error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "Google OAuth not configured",
            )
        }
    };

    let client_secret = match &state.config.google_client_secret {
        Some(secret) => secret.clone(),
        None => {
            return error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "Google OAuth not configured",
            )
        }
    };

    let token_data =
        match exchange_code_for_token(&client_id, &client_secret, "postmessage", &payload.code)
            .await
        {
            Ok(data) => data,
            Err(e) => {
                error!("[GoogleOAuth] Failed to exchange code for switch: {}", e);
                return error_response(StatusCode::UNAUTHORIZED, "Failed to exchange Google code");
            }
        };

    let google_user = match fetch_google_user_info(&token_data.access_token).await {
        Ok(user) => user,
        Err(e) => {
            error!("[GoogleOAuth] Failed to fetch user info for switch: {}", e);
            return error_response(StatusCode::UNAUTHORIZED, "Failed to fetch Google user info");
        }
    };

    let google_email = google_user.email.trim().to_lowercase();

    match state.database.get_user_by_email(&google_email).await {
        Ok(Some(existing_user)) => {
            if existing_user.id != claims.sub {
                return error_response(
                    StatusCode::CONFLICT,
                    "Email is already in use by another account",
                );
            }
        }
        Ok(None) => {}
        Err(e) => {
            error!("[Auth] Failed to check email in DB: {}", e);
            return internal_error_responce();
        }
    }

    if let Err(e) = state
        .database
        .change_provider_to_google(&claims.sub, &google_email)
        .await
    {
        error!(
            "[Auth] Failed to change provider to google in database: {}",
            e
        );
        return internal_error_responce();
    }

    let _ = state.cache.invalidate_user_profile_cache(&user_uuid).await;

    let latency_ms = start.elapsed().as_millis() as u64;
    state
        .metrics
        .record("switch_provider_to_google", latency_ms, false);

    (
        StatusCode::OK,
        Json(SwitchProviderToGoogleResponse {
            message: "Successfully switched provider to Google".to_string(),
        }),
    )
        .into_response()
}

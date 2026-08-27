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
pub struct GoogleCodeRequest {
    pub code: String,
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

pub async fn google_code_handler(
    State(state): State<AppState>,
    Json(payload): Json<GoogleCodeRequest>,
) -> impl IntoResponse {
    info!("[GoogleOAuth] Received authorization code for exchange");

    let client_id = match &state.config.google_client_id {
        Some(id) => id.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": "Google OAuth not configured"})),
            )
                .into_response();
        }
    };

    let client_secret = match &state.config.google_client_secret {
        Some(secret) => secret.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": "Google OAuth not configured"})),
            )
                .into_response();
        }
    };

    let token_data =
        match exchange_code_for_token(&client_id, &client_secret, "postmessage", &payload.code)
            .await
        {
            Ok(data) => data,
            Err(e) => {
                error!("[GoogleOAuth] Failed to exchange code: {}", e);
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Failed to exchange Google code", "details": e})),
                )
                    .into_response();
            }
        };

    let google_user = match fetch_google_user_info(&token_data.access_token).await {
        Ok(user) => user,
        Err(e) => {
            error!("[GoogleOAuth] Failed to fetch user info: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Failed to fetch user info"})),
            )
                .into_response();
        }
    };

    let existing_user = match state.database.get_user_by_email(&google_user.email).await {
        Ok(existing) => existing,
        Err(e) => {
            error!("[Auth] DB error checking email: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
                .into_response();
        }
    };

    if let Some(u) = &existing_user {
        if u.is_banned {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "This email has been banned"})),
            )
                .into_response();
        }
    }

    let needs_creation = match &existing_user {
        None => true,
        Some(u) => u.auth_provider != "google" && !u.email_validated,
    };

    if let Some(u) = &existing_user {
        if u.auth_provider != "google" && u.email_validated {
            return (
                StatusCode::CONFLICT,
                Json(json!({"error": "This email is already registered with a different sign-in method"})),
            )
                .into_response();
        }
    }

    let user = if needs_creation {
        if let Some(old_user) = &existing_user {
            if let Err(e) = state.database.delete_user(&old_user.id).await {
                error!("[Auth] Failed to delete stale unvalidated user: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to process account"})),
                )
                    .into_response();
            }
        }

        let user_id = Uuid::new_v4();
        match state
            .database
            .create_user_from_google(&user_id.to_string(), &google_user.email)
            .await
        {
            Ok(u) => u,
            Err(e) => {
                error!("[Auth] Failed to create Google user: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to create user"})),
                )
                    .into_response();
            }
        }
    } else {
        existing_user.expect("existing_user must be Some when needs_creation is false")
    };

    let token_response = match jwt_manager().generate_token_pair(
        &user.id,
        user.username.as_deref().unwrap_or(""),
        &user.email,
    ) {
        Ok(tokens) => tokens,
        Err(e) => {
            error!("[Auth] Failed to generate tokens: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Token generation failed"})),
            )
                .into_response();
        }
    };

    let user_uuid = match Uuid::parse_str(&user.id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Internal error"})),
            )
                .into_response();
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

    info!(
        "[Auth] Google user {} logged in successfully",
        google_user.email
    );

    let access_cookie = format!(
        "access_token={}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
        token_response.access_token, token_response.access_token_expires_in
    );
    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly; SameSite=Strict; Path=/api/auth/refresh; Max-Age={}",
        token_response.refresh_token, token_response.refresh_token_expires_in
    );

    let response_body = json!({
        "message": "Google login successful",
        "user": {
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "account_validated": user.account_validated
        },
        "account_validated": user.account_validated
    });

    let mut response = (StatusCode::OK, Json(response_body)).into_response();

    let headers = response.headers_mut();
    if let Ok(v) = HeaderValue::from_str(&access_cookie) {
        headers.append(SET_COOKIE, v);
    }
    if let Ok(v) = HeaderValue::from_str(&refresh_cookie) {
        headers.append(SET_COOKIE, v);
    }

    response
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

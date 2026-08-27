use crate::auth::utils::get_cookie_value;
use crate::auth::{hash_refresh_token, jwt_manager, utils::get_bearer_token};
use crate::http::router::AppState;
use axum::{
    extract::{Query, State},
    http::{
        header::{HeaderValue, SET_COOKIE},
        StatusCode,
    },
    response::IntoResponse,
};
use chrono::{Duration, Utc};
use log::{error, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FtCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct FtTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    scope: String,
    created_at: i64,
}

#[derive(Deserialize, Serialize, Debug)]
struct FtUser {
    id: i64,
    email: String,
    login: String,
}

pub async fn ft_login_handler(State(state): State<AppState>) -> impl IntoResponse {
    let config = &state.config;

    let client_id = match &config.ft_client_id {
        Some(id) => id.clone(),
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, "42 OAuth not configured").into_response();
        }
    };

    let state_value = Uuid::new_v4().to_string();

    let ft_auth_url = format!(
        "https://api.intra.42.fr/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope=public&state={}",
        urlencoding::encode(&client_id),
        urlencoding::encode(&config.ft_redirect_uri),
        urlencoding::encode(&state_value)
    );

    let state_cookie = format!(
        "ft_oauth_state={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=300",
        state_value
    );

    info!("[42OAuth] Login/Switch hit: redirecting to {}", ft_auth_url);

    let mut response = (
        StatusCode::FOUND,
        axum::response::AppendHeaders([("Location", ft_auth_url)]),
        "Redirecting to 42...",
    )
        .into_response();

    if let Ok(cookie_val) = HeaderValue::from_str(&state_cookie) {
        response.headers_mut().append(SET_COOKIE, cookie_val);
    }

    response
}

pub async fn ft_callback_handler(
    State(state): State<AppState>,
    Query(params): Query<FtCallbackQuery>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    info!(
        "[42OAuth] Callback hit: code={:?} state={:?} error={:?}",
        params.code, params.state, params.error
    );

    let config = &state.config;

    if let Some(ref error) = params.error {
        let desc = params.error_description.as_deref().unwrap_or(error);
        error!("[42OAuth] 42 returned error: {} (desc: {})", error, desc);
        return redirect_with_error(&config.frontend_redirect_url, error);
    }

    let code = match params.code {
        Some(c) => c,
        None => {
            error!("[42OAuth] Missing authorization code");
            return redirect_with_error(&config.frontend_redirect_url, "missing_code");
        }
    };

    let state_value = match params.state {
        Some(s) => s,
        None => {
            error!("[42OAuth] Missing state parameter");
            return redirect_with_error(&config.frontend_redirect_url, "missing_state");
        }
    };

    let cookie_state = extract_cookie_value(&headers, "ft_oauth_state");
    if cookie_state.as_deref() != Some(state_value.as_str()) {
        error!("[42OAuth] State mismatch");
        return redirect_with_error(&config.frontend_redirect_url, "invalid_state");
    }

    let client_id = match &config.ft_client_id {
        Some(id) => id.clone(),
        None => return redirect_with_error(&config.frontend_redirect_url, "42_not_configured"),
    };

    let client_secret = match &config.ft_client_secret {
        Some(secret) => secret.clone(),
        None => return redirect_with_error(&config.frontend_redirect_url, "42_not_configured"),
    };

    let token_data =
        match exchange_ft_code(&client_id, &client_secret, &config.ft_redirect_uri, &code).await {
            Ok(data) => data,
            Err(e) => {
                error!("[42OAuth] Token exchange failed: {}", e);
                return redirect_with_error(&config.frontend_redirect_url, "token_exchange_failed");
            }
        };

    let ft_user = match fetch_ft_user_info(&token_data.access_token).await {
        Ok(user) => user,
        Err(e) => {
            error!("[42OAuth] User info failed: {}", e);
            return redirect_with_error(&config.frontend_redirect_url, "user_info_failed");
        }
    };

    let ft_email = ft_user.email.trim().to_lowercase();
    let base = config.frontend_redirect_url.trim_end_matches('/');
    let base = if base.is_empty() { "/" } else { base };

    let existing_token =
        get_bearer_token(&headers).or_else(|| get_cookie_value(&headers, "access_token"));

    if let Some(token) = existing_token {
        if let Ok(claims) = jwt_manager().validate_token(&token) {
            info!(
                "[42OAuth] Switch provider mode detected for user {}",
                claims.sub
            );

            if let Ok(Some(existing_user)) = state.database.get_user_by_email(&ft_email).await {
                if existing_user.id != claims.sub {
                    return redirect_with_error(base, "email_already_used");
                }
            }

            if let Err(e) = state
                .database
                .change_provider_to_42(&claims.sub, &ft_email)
                .await
            {
                error!("[42OAuth] Failed to change provider to 42: {}", e);
                return redirect_with_error(base, "database_error");
            }

            if let Ok(uuid) = Uuid::parse_str(&claims.sub) {
                let _ = state.cache.invalidate_user_profile_cache(&uuid).await;
            }

            let success_url = format!("/settings?success=provider_changed");
            return (
                StatusCode::FOUND,
                axum::response::AppendHeaders([("Location", success_url)]),
                "Redirecting...",
            )
                .into_response();
        }
    }

    let existing_user = match state.database.get_user_by_email(&ft_email).await {
        Ok(existing) => existing,
        Err(_) => return redirect_with_error(base, "database_error"),
    };

    if let Some(ref u) = existing_user {
        if u.is_banned {
            return redirect_with_error(base, "account_banned");
        }
    }

    let needs_creation = if let Some(ref u) = existing_user {
        if u.auth_provider != "42" {
            if u.email_validated {
                return redirect_with_error(base, "email_already_used");
            } else {
                true
            }
        } else {
            false
        }
    } else {
        true
    };

    let user = if needs_creation {
        if let Some(old_user) = &existing_user {
            let _ = state.database.delete_user(&old_user.id).await;
        }
        let user_id = Uuid::new_v4();
        match state
            .database
            .create_user_from_42(&user_id.to_string(), &ft_email)
            .await
        {
            Ok(u) => u,
            Err(_) => return redirect_with_error(base, "user_creation_failed"),
        }
    } else {
        existing_user.expect("existing_user must be Some")
    };

    let token_response = match jwt_manager().generate_token_pair(
        &user.id,
        user.username.as_deref().unwrap_or(""),
        &user.email,
    ) {
        Ok(tokens) => tokens,
        Err(_) => return redirect_with_error(base, "token_generation_failed"),
    };

    let user_uuid = Uuid::parse_str(&user.id).unwrap_or_default();
    let refresh_hash = hash_refresh_token(&token_response.refresh_token);
    let refresh_expires_at =
        Utc::now() + Duration::seconds(token_response.refresh_token_expires_in);
    let _ = state
        .database
        .store_refresh_token(&user_uuid, &refresh_hash, refresh_expires_at)
        .await;

    let access_cookie = format!(
        "access_token={}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}",
        token_response.access_token, token_response.access_token_expires_in
    );
    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly; SameSite=Strict; Path=/api/auth/refresh; Max-Age={}",
        token_response.refresh_token, token_response.refresh_token_expires_in
    );
    let clear_state_cookie = "ft_oauth_state=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0";

    let redirect_url = if user.account_validated {
        base.to_string()
    } else {
        format!("{}?setup=1", base)
    };

    let mut response = (
        StatusCode::FOUND,
        axum::response::AppendHeaders([("Location", redirect_url)]),
        "Redirecting...",
    )
        .into_response();

    let headers = response.headers_mut();
    if let Ok(v) = HeaderValue::from_str(&access_cookie) {
        headers.append(SET_COOKIE, v);
    }
    if let Ok(v) = HeaderValue::from_str(&refresh_cookie) {
        headers.append(SET_COOKIE, v);
    }
    if let Ok(v) = HeaderValue::from_str(clear_state_cookie) {
        headers.append(SET_COOKIE, v);
    }

    response
}

async fn exchange_ft_code(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<FtTokenResponse, String> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("redirect_uri", redirect_uri),
    ];
    let res = client
        .post("https://api.intra.42.fr/oauth/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;
    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(format!("42 token error: {}", body));
    }
    res.json::<FtTokenResponse>()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))
}

async fn fetch_ft_user_info(access_token: &str) -> Result<FtUser, String> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.intra.42.fr/v2/me")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;
    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(format!("42 userinfo error: {}", body));
    }
    res.json::<FtUser>()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))
}

fn redirect_with_error(base: &str, error: &str) -> axum::response::Response {
    let base = base.trim_end_matches('/');
    let base = if base.is_empty() { "/" } else { base };
    let url = format!("{}?error={}", base, error);
    (
        StatusCode::FOUND,
        axum::response::AppendHeaders([("Location", url)]),
        "Redirecting...",
    )
        .into_response()
}

fn extract_cookie_value(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
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

use api_core::auth::validate_and_get_claims;
use api_core::http::response::json_error;
use api_core::types::ServiceRequest;
use log::error;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppContext;
use crate::services::user::{get_user_by_id, update_profile_settings};

#[derive(Deserialize, Default)]
struct SettingsUpdate {
    bio: Option<String>,
    github: Option<String>,
    discord: Option<String>,
    twitter: Option<String>,
    is_private: Option<bool>,
    theme: Option<String>,
    lang: Option<String>,
}

async fn auth_user_id(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> Result<Uuid, Value> {
    let token = match request.cookies.get("access_token") {
        Some(token) => token,
        None => return Err(json_error(401, "Missing access token")),
    };

    let mut conn = match ctx.redis_pool.get().await {
        Ok(c) => c,
        Err(_) => return Err(json_error(500, "Internal error")),
    };

    let claims = match validate_and_get_claims(&mut conn, token).await {
        Ok(claims) => claims,
        Err(e) => {
            log::warn!("[Settings] Failed to validate token: {}", e);
            return Err(json_error(401, &e));
        }
    };

    Uuid::parse_str(&claims.sub).map_err(|_| json_error(400, "Invalid user ID in token"))
}

pub async fn handle_get_settings(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> serde_json::Value {
    let user_id = match auth_user_id(ctx, request).await {
        Ok(id) => id,
        Err(e) => return e,
    };

    let user = match get_user_by_id(&user_id, ctx.db.get_pool(), &ctx.redis_pool).await {
        Ok(Some(user)) => user,
        Ok(None) => return json_error(404, "User not found"),
        Err(e) => {
            error!("[Settings] DB error: {}", e);
            return json_error(500, "Database error");
        }
    };

    json!({
        "status": 200,
        "settings": {
            "username": user.username,
            "email": user.email,
            "bio": user.bio,
            "github": user.github,
            "discord": user.discord,
            "twitter": user.twitter,
            "is_private": user.is_private,
            "theme": user.theme,
            "lang": user.lang,
        }
    })
}

pub async fn handle_update_settings(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> serde_json::Value {
    let user_id = match auth_user_id(ctx, request).await {
        Ok(id) => id,
        Err(e) => return e,
    };

    let payload: SettingsUpdate = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    let theme = payload.theme.as_deref();
    if let Some(t) = theme {
        if t != "dark" && t != "light" {
            return json_error(400, "Theme must be 'dark' or 'light'");
        }
    }

    match update_profile_settings(
        &user_id,
        payload.bio.as_deref(),
        payload.github.as_deref(),
        payload.discord.as_deref(),
        payload.twitter.as_deref(),
        payload.is_private,
        theme,
        payload.lang.as_deref(),
        ctx.db.get_pool(),
        &ctx.redis_pool,
    )
    .await
    {
        Ok(_) => json!({
            "status": 200,
            "message": "Settings updated successfully"
        }),
        Err(e) => {
            error!("[Settings] Failed to update settings: {}", e);
            json_error(500, "Failed to update settings")
        }
    }
}
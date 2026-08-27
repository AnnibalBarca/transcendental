use api_core::auth::validate_and_get_claims;
use api_core::http::response::{json_error, json_user};
use log::error;
use uuid::Uuid;

use crate::AppContext;
use crate::services::user::get_user_by_id;
use crate::types::UserPayload;
use api_core::types::ServiceRequest;

pub async fn handle_me(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
    let start_time = std::time::Instant::now();
    log::debug!("[User-Me] Starting handle_me for request id={}", request.id);

    let token = match request.cookies.get("access_token") {
        Some(token) => token,
        None => {
            log::warn!(
                "[User-Me] Missing access token for request id={}",
                request.id
            );
            return json_error(401, "Missing access token");
        }
    };

    let mut conn = match ctx.redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("[User-Me] Redis error: {}", e);
            return json_error(500, "Internal error");
        }
    };

    let claims = match validate_and_get_claims(&mut conn, token).await {
        Ok(claims) => claims,
        Err(e) => {
            log::warn!("[User-Me] Failed to validate token: {}", e);
            return json_error(401, &e);
        }
    };

    let now = chrono::Utc::now().timestamp();
    let access_token_expires_in = (claims.exp - now).max(0);

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return json_error(400, "Invalid user ID in token"),
    };

    log::debug!("[User-Me] Getting user from DB for user_id={}", user_id);
    let user = match get_user_by_id(&user_id, ctx.db.get_pool(), &ctx.redis_pool).await {
        Ok(Some(user)) => {
            log::debug!(
                "[User-Me] Found user {} in {:?}",
                user_id,
                start_time.elapsed()
            );
            user
        }
        Ok(None) => {
            log::warn!("[User-Me] User {} not found", user_id);
            return json_error(404, "User not found");
        }
        Err(e) => {
            error!("[User-Me] DB error for user_id={}: {}", user_id, e);
            return json_error(500, "Database error");
        }
    };

    if user.is_banned {
        log::warn!("[User-Me] Banned user {} attempted access", user_id);
        return json_error(403, "This account has been banned");
    }

    let (state, room_id, chess_game_id, chess_ws_url) =
        match ctx.session_manager.get_session(&user_id).await {
            Ok(Some(session)) => (
                session.status,
                Some(session.room_id),
                Some(session.chess_game_id).filter(|s| !s.is_empty()),
                Some(session.chess_ws_url).filter(|s| !s.is_empty()),
            ),
            _ => ("none".to_string(), None, None, None),
        };

    let level = crate::xp::level_from_xp(user.xp) as i32;
    let xp_progress = crate::xp::level_progress(user.xp, level as i64);

    let permissions = match crate::db::user::permissions_of_user(ctx.db.get_pool(), &user_id).await
    {
        Ok(p) => p,
        Err(e) => {
            error!("[User-Me] Failed to load permissions for user {}: {}", user_id, e);
            Vec::new()
        }
    };

    let has_panel_permission = match crate::db::user::user_has_route_permission(
        ctx.db.get_pool(),
        &user_id,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("[User-Me] Failed to check route permissions for user {}: {}", user_id, e);
            false
        }
    };

    let has_panel_access = user.roles.iter().any(|r| r == "admin")
        || permissions.iter().any(|p| p == "panel.access")
        || has_panel_permission;

    let _ = crate::services::permission::sync_user_permissions(ctx, &user_id).await;

    let payload = UserPayload {
        id: user.id,
        username: user.username,
        email: Some(user.email),
        state: Some(state),
        room_id,
        chess_game_id,
        chess_ws_url,
        account_validated: user.account_validated,
        email_validated: user.email_validated,
        access_token_expires_in: Some(access_token_expires_in),
        auth_provider: user.auth_provider,
        wallet: user.wallet,
        ranked_elo: user.ranked_elo,
        level,
        xp: user.xp,
        xp_progress,
        picture_id: user.picture_id,
        has_panel_access,
        bio: user.bio,
        github: user.github,
        discord: user.discord,
        twitter: user.twitter,
        is_private: user.is_private,
        theme: user.theme,
        lang: user.lang,
    };

    json_user(payload)
}

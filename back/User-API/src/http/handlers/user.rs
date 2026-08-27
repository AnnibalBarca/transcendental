use api_core::auth::validate_and_get_claims;
use api_core::http::response::{json_error, json_user};
use log::error;
use uuid::Uuid;

use crate::AppContext;
use crate::http::utils::parse_user_id;
use crate::services::user::{get_user_by_id, get_user_by_username};
use crate::types::UserPayload;
use api_core::types::ServiceRequest;

pub async fn handle_user(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
    let parts: Vec<&str> = request.action.split('/').collect();
    let id_str = match parts.get(1) {
        Some(id) => id,
        None => return json_error(400, "Missing user id"),
    };

    let user_id_to_find = match parse_user_id(id_str) {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    let user = match user_id_to_find {
        Some(uuid) => {
            get_user_by_id(&uuid, ctx.db.get_pool(), &ctx.redis_pool).await
        }
        None => {
            if id_str.is_empty() {
                return json_error(400, "Missing user id");
            }
            get_user_by_username(id_str, ctx.db.get_pool()).await
        }
    };

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => return json_error(404, "User not found"),
        Err(e) => {
            error!("[User] Repository error: {}", e);
            return json_error(500, "Internal server error");
        }
    };

    let user_id_to_find = match Uuid::parse_str(&user.id) {
        Ok(id) => id,
        Err(_) => return json_error(500, "Invalid user id"),
    };

    let requester_id = match request.cookies.get("access_token") {
        Some(token) => {
            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(_) => return json_error(500, "Internal error"),
            };
            match validate_and_get_claims(&mut conn, token).await {
                Ok(claims) => Some(claims.sub),
                Err(_) => None,
            }
        }
        None => None,
    };

    let is_owner = requester_id.as_deref() == Some(&user.id);

    let is_friend = if is_owner {
        false
    } else {
        let requester_uuid = requester_id
            .as_deref()
            .and_then(|id| Uuid::parse_str(id).ok());

        match requester_uuid {
            Some(id) => {
                match crate::db::user::are_friends(ctx.db.get_pool(), &id, &user_id_to_find).await
                {
                    Ok(f) => f,
                    Err(e) => {
                        error!("[User] Friend check error: {}", e);
                        false
                    }
                }
            }
            None => false,
        }
    };

    if user.is_private && !is_owner && !is_friend {
        return json_error(403, "This profile is private");
    }

    let public_visible = is_owner || is_friend || !user.is_private;

    let level = crate::xp::level_from_xp(user.xp) as i32;
    let xp_progress = crate::xp::level_progress(user.xp, level as i64);

    let payload = UserPayload {
        id: user.id,
        username: user.username,
        email: if is_owner { Some(user.email) } else { None },
        state: None,
        room_id: None,
        chess_game_id: None,
        chess_ws_url: None,
        account_validated: user.account_validated,
        email_validated: if is_owner {
            user.email_validated
        } else {
            false
        },
        access_token_expires_in: None,
        auth_provider: user.auth_provider,
        wallet: if is_owner { user.wallet } else { 0 },
        ranked_elo: if public_visible {
            user.ranked_elo
        } else {
            0
        },
        level: if public_visible { level } else { 0 },
        xp: if public_visible { user.xp } else { 0 },
        xp_progress: if public_visible { xp_progress } else { 0.0 },
        picture_id: user.picture_id,
        has_panel_access: false,
        bio: if public_visible { user.bio } else { String::new() },
        github: if public_visible { user.github } else { String::new() },
        discord: if public_visible { user.discord } else { String::new() },
        twitter: if public_visible { user.twitter } else { String::new() },
        is_private: user.is_private,
        theme: String::new(),
        lang: String::new(),
    };

    json_user(payload)
}

use api_core::auth::validate_and_get_claims;
use api_core::http::response::json_error;
use api_core::types::ServiceRequest;
use log::{error, info};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppContext;
use crate::db::{cards as cards_repo, permissions as perm_repo, roles as roles_repo, routes as routes_repo, user_roles as user_roles_repo};
use crate::http::utils::parse_user_id;
use crate::services::user::{admin_delete_user, admin_update_user, get_user_by_id, list_users};

#[derive(Deserialize)]
struct ListPayload {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

#[derive(Deserialize, Default)]
struct UpdatePayload {
    username: Option<String>,
    email: Option<String>,
    account_validated: Option<bool>,
    email_validated: Option<bool>,
    is_banned: Option<bool>,
    wallet: Option<i64>,
    ranked_elo: Option<i32>,
    xp: Option<i64>,
}

fn default_limit() -> i64 {
    50
}

async fn require_permission(
    ctx: &AppContext,
    request: &ServiceRequest,
    required: &[&str],
) -> Result<(), Value> {
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
            log::warn!("[Admin] Failed to validate token: {}", e);
            return Err(json_error(401, &e));
        }
    };

    let requester_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Err(json_error(400, "Invalid user ID in token")),
    };

    let requester = match get_user_by_id(&requester_id, ctx.db.get_pool(), &ctx.redis_pool).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(json_error(404, "Requester not found")),
        Err(e) => {
            error!("[Admin] DB error while fetching requester: {}", e);
            return Err(json_error(500, "Database error"));
        }
    };

    let permissions = match crate::db::user::permissions_of_user(ctx.db.get_pool(), &requester_id)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            error!("[Admin] Failed to load permissions for requester {}: {}", requester_id, e);
            return Err(json_error(500, "Database error"));
        }
    };

    let has_admin_role = requester.roles.iter().any(|r| r == "admin");
    let has_required = required.iter().any(|r| permissions.iter().any(|p| p == r));
    let has_panel = permissions.iter().any(|p| p == "panel.access");

    if !has_admin_role && !has_panel && !has_required {
        return Err(json_error(403, "Forbidden: missing required permission"));
    }

    Ok(())
}

fn parse_i32(id_str: &str) -> Result<i32, Value> {
    id_str
        .parse::<i32>()
        .map_err(|_| json_error(400, "Invalid id"))
}

fn id_from_parts(parts: &[&str], index: usize) -> Result<i32, Value> {
    match parts.get(index) {
        Some(id) => parse_i32(id),
        None => Err(json_error(400, "Missing id")),
    }
}

#[derive(Deserialize)]
struct NamePayload {
    name: String,
    #[serde(default)]
    description: String,
}

#[derive(Deserialize)]
struct IdPayload {
    id: i32,
}

#[derive(Deserialize)]
struct RateLimitPayload {
    #[serde(rename = "requests_per_minute")]
    requests_per_minute: i64,
}

pub async fn handle_admin_list(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
    if let Err(e) = require_permission(ctx, request, &["users.view"]).await {
        return e;
    }

    let payload: ListPayload = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(_) => ListPayload {
            limit: default_limit(),
            offset: 0,
        },
    };

    match list_users(ctx.db.get_pool(), payload.limit, payload.offset).await {
        Ok((users, total)) => json!({
            "status": 200,
            "users": users,
            "total": total
        }),
        Err(e) => {
            error!("[Admin] Failed to list users: {}", e);
            json_error(500, "Failed to list users")
        }
    }
}

pub async fn handle_admin_update(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
    if let Err(e) = require_permission(ctx, request, &["users.edit"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let id_str = match parts.get(2) {
        Some(id) => id,
        None => return json_error(400, "Missing user id"),
    };
    let user_id = match parse_user_id(id_str) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let payload: UpdatePayload = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    match admin_update_user(
        &user_id,
        payload.username.as_deref(),
        payload.email.as_deref(),
        payload.account_validated,
        payload.email_validated,
        payload.is_banned,
        payload.wallet,
        payload.ranked_elo,
        payload.xp,
        ctx.db.get_pool(),
        &ctx.redis_pool,
    )
    .await
    {
        Ok(_) => {
            info!("[Admin] Updated user {}", user_id);
            json!({
                "status": 200,
                "message": "User updated successfully",
                "user_id": user_id.to_string()
            })
        }
        Err(e) => {
            error!("[Admin] Failed to update user {}: {}", user_id, e);
            json_error(500, "Failed to update user")
        }
    }
}

pub async fn handle_admin_delete(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
    if let Err(e) = require_permission(ctx, request, &["users.delete"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let id_str = match parts.get(2) {
        Some(id) => id,
        None => return json_error(400, "Missing user id"),
    };
    let user_id = match parse_user_id(id_str) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match admin_delete_user(&user_id, ctx.db.get_pool(), &ctx.redis_pool).await {
        Ok(true) => {
            info!("[Admin] Deleted user {}", user_id);
            json!({
                "status": 200,
                "message": "User deleted successfully",
                "user_id": user_id.to_string()
            })
        }
        Ok(false) => json_error(404, "User not found"),
        Err(e) => {
            error!("[Admin] Failed to delete user {}: {}", user_id, e);
            json_error(500, "Failed to delete user")
        }
    }
}

pub async fn handle_admin_roles_list(ctx: &AppContext, request: &ServiceRequest) -> Value {
    if let Err(e) = require_permission(ctx, request, &["roles.manage"]).await {
        return e;
    }

    match roles_repo::list(ctx.db.get_pool()).await {
        Ok(roles) => json!({ "status": 200, "roles": roles }),
        Err(e) => {
            error!("[Admin] Failed to list roles: {}", e);
            json_error(500, "Failed to list roles")
        }
    }
}

pub async fn handle_admin_roles_create(ctx: &AppContext, request: &ServiceRequest) -> Value {
    if let Err(e) = require_permission(ctx, request, &["roles.manage"]).await {
        return e;
    }

    let payload: NamePayload = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    if payload.name.trim().is_empty() || payload.name.len() > 50 {
        return json_error(400, "Invalid role name");
    }

    match roles_repo::create(ctx.db.get_pool(), &payload.name.trim(), &payload.description).await {
        Ok(role) => {
                let _ = crate::services::permission::sync_permissions_to_redis(ctx).await;
                json!({ "status": 200, "role": role })
            },
        Err(e) => {
            error!("[Admin] Failed to create role: {}", e);
            json_error(500, "Failed to create role")
        }
    }
}

pub async fn handle_admin_roles_update(ctx: &AppContext, request: &ServiceRequest) -> Value {
    if let Err(e) = require_permission(ctx, request, &["roles.manage"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let id = match id_from_parts(&parts, 2) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let payload: NamePayload = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    if payload.name.trim().is_empty() || payload.name.len() > 50 {
        return json_error(400, "Invalid role name");
    }

    match roles_repo::update(ctx.db.get_pool(), id, &payload.name.trim(), &payload.description).await {
        Ok(true) => {
                let _ = crate::services::permission::sync_permissions_to_redis(ctx).await;
                json!({ "status": 200, "message": "Role updated", "role_id": id })
            },
        Ok(false) => json_error(404, "Role not found"),
        Err(e) => {
            error!("[Admin] Failed to update role {}: {}", id, e);
            json_error(500, "Failed to update role")
        }
    }
}

pub async fn handle_admin_roles_delete(ctx: &AppContext, request: &ServiceRequest) -> Value {
    if let Err(e) = require_permission(ctx, request, &["roles.manage"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let id = match id_from_parts(&parts, 2) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match roles_repo::delete(ctx.db.get_pool(), id).await {
        Ok(true) => {
                let _ = crate::services::permission::sync_permissions_to_redis(ctx).await;
                json!({ "status": 200, "message": "Role deleted", "role_id": id })
            },
        Ok(false) => json_error(404, "Role not found"),
        Err(e) => {
            error!("[Admin] Failed to delete role {}: {}", id, e);
            json_error(500, "Failed to delete role")
        }
    }
}

pub async fn handle_admin_roles_add_permission(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> Value {
    if let Err(e) = require_permission(ctx, request, &["roles.manage"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let role_id = match id_from_parts(&parts, 2) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let payload: IdPayload = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    match perm_repo::get_by_id(ctx.db.get_pool(), payload.id).await {
        Ok(Some(_)) => {}
        Ok(None) => return json_error(404, "Permission not found"),
        Err(e) => {
            error!("[Admin] Failed to fetch permission: {}", e);
            return json_error(500, "Database error");
        }
    }

    match roles_repo::add_permission(ctx.db.get_pool(), role_id, payload.id).await {
        Ok(_) => {
                let _ = crate::services::permission::sync_permissions_to_redis(ctx).await;
                json!({ "status": 200, "message": "Permission granted", "role_id": role_id })
            },
        Err(e) => {
            error!("[Admin] Failed to add permission to role: {}", e);
            json_error(500, "Failed to add permission")
        }
    }
}

pub async fn handle_admin_roles_remove_permission(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> Value {
    if let Err(e) = require_permission(ctx, request, &["roles.manage"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let role_id = match id_from_parts(&parts, 2) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let permission_id = match id_from_parts(&parts, 4) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match roles_repo::remove_permission(ctx.db.get_pool(), role_id, permission_id).await {
        Ok(_) => {
                let _ = crate::services::permission::sync_permissions_to_redis(ctx).await;
                json!({ "status": 200, "message": "Permission removed", "role_id": role_id })
            },
        Err(e) => {
            error!("[Admin] Failed to remove permission from role: {}", e);
            json_error(500, "Failed to remove permission")
        }
    }
}

pub async fn handle_admin_permissions_list(ctx: &AppContext, request: &ServiceRequest) -> Value {
    if let Err(e) = require_permission(ctx, request, &["permissions.manage"]).await {
        return e;
    }

    match perm_repo::list(ctx.db.get_pool()).await {
        Ok(permissions) => json!({ "status": 200, "permissions": permissions }),
        Err(e) => {
            error!("[Admin] Failed to list permissions: {}", e);
            json_error(500, "Failed to list permissions")
        }
    }
}

pub async fn handle_admin_permissions_create(ctx: &AppContext, request: &ServiceRequest) -> Value {
    if let Err(e) = require_permission(ctx, request, &["permissions.manage"]).await {
        return e;
    }

    let payload: NamePayload = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    if payload.name.trim().is_empty() || payload.name.len() > 100 {
        return json_error(400, "Invalid permission name");
    }

    match perm_repo::create(ctx.db.get_pool(), &payload.name.trim(), &payload.description).await {
        Ok(permission) => {
            // Le rôle admin reçoit automatiquement toute nouvelle permission.
            let _ = sqlx::query(
                "INSERT INTO role_permissions (role_id, permission_id) \
                 SELECT r.id, $1 FROM roles r WHERE r.name = 'admin' ON CONFLICT DO NOTHING",
            )
            .bind(permission.id)
            .execute(ctx.db.get_pool())
            .await;

            let _ = crate::services::permission::sync_permissions_to_redis(ctx).await;
            json!({ "status": 200, "permission": permission })
        },
        Err(e) => {
            error!("[Admin] Failed to create permission: {}", e);
            json_error(500, "Failed to create permission")
        }
    }
}

pub async fn handle_admin_permissions_update(ctx: &AppContext, request: &ServiceRequest) -> Value {
    if let Err(e) = require_permission(ctx, request, &["permissions.manage"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let id = match id_from_parts(&parts, 2) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let payload: NamePayload = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    if payload.name.trim().is_empty() || payload.name.len() > 100 {
        return json_error(400, "Invalid permission name");
    }

    match perm_repo::update(ctx.db.get_pool(), id, &payload.name.trim(), &payload.description).await {
        Ok(true) => {
                let _ = crate::services::permission::sync_permissions_to_redis(ctx).await;
                json!({ "status": 200, "message": "Permission updated", "permission_id": id })
            },
        Ok(false) => json_error(404, "Permission not found"),
        Err(e) => {
            error!("[Admin] Failed to update permission {}: {}", id, e);
            json_error(500, "Failed to update permission")
        }
    }
}

pub async fn handle_admin_permissions_delete(ctx: &AppContext, request: &ServiceRequest) -> Value {
    if let Err(e) = require_permission(ctx, request, &["permissions.manage"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let id = match id_from_parts(&parts, 2) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match perm_repo::delete(ctx.db.get_pool(), id).await {
        Ok(true) => {
                let _ = crate::services::permission::sync_permissions_to_redis(ctx).await;
                json!({ "status": 200, "message": "Permission deleted", "permission_id": id })
            },
        Ok(false) => json_error(404, "Permission not found"),
        Err(e) => {
            error!("[Admin] Failed to delete permission {}: {}", id, e);
            json_error(500, "Failed to delete permission")
        }
    }
}

pub async fn handle_admin_permissions_add_route(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> Value {
    if let Err(e) = require_permission(ctx, request, &["permissions.manage"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let permission_id = match id_from_parts(&parts, 2) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let payload: IdPayload = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    match routes_repo::exists(ctx.db.get_pool(), payload.id).await {
        Ok(true) => {}
        Ok(false) => return json_error(404, "Route not found"),
        Err(e) => {
            error!("[Admin] Failed to check route: {}", e);
            return json_error(500, "Database error");
        }
    }

    match perm_repo::add_route(ctx.db.get_pool(), permission_id, payload.id).await {
        Ok(_) => {
                let _ = crate::services::permission::sync_permissions_to_redis(ctx).await;
                json!({ "status": 200, "message": "Route linked", "permission_id": permission_id })
            },
        Err(e) => {
            error!("[Admin] Failed to link route to permission: {}", e);
            json_error(500, "Failed to link route")
        }
    }
}

pub async fn handle_admin_permissions_remove_route(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> Value {
    if let Err(e) = require_permission(ctx, request, &["permissions.manage"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let permission_id = match id_from_parts(&parts, 2) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let route_id = match id_from_parts(&parts, 4) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match perm_repo::remove_route(ctx.db.get_pool(), permission_id, route_id).await {
        Ok(_) => {
                let _ = crate::services::permission::sync_permissions_to_redis(ctx).await;
                json!({ "status": 200, "message": "Route unlinked", "permission_id": permission_id })
            },
        Err(e) => {
            error!("[Admin] Failed to unlink route from permission: {}", e);
            json_error(500, "Failed to unlink route")
        }
    }
}

pub async fn handle_admin_routes_list(ctx: &AppContext, request: &ServiceRequest) -> Value {
    if let Err(e) = require_permission(ctx, request, &["routes.manage"]).await {
        return e;
    }

    match routes_repo::list(ctx.db.get_pool()).await {
        Ok(routes) => json!({ "status": 200, "routes": routes }),
        Err(e) => {
            error!("[Admin] Failed to list routes: {}", e);
            json_error(500, "Failed to list routes")
        }
    }
}

pub async fn handle_admin_routes_set_rate_limit(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> Value {
    if let Err(e) = require_permission(ctx, request, &["rate-limits.manage"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let route_id = match id_from_parts(&parts, 2) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let payload: RateLimitPayload = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    if payload.requests_per_minute < 0 {
        return json_error(400, "requests_per_minute must be >= 0");
    }

    let route = match routes_repo::get_by_id(ctx.db.get_pool(), route_id).await {
        Ok(Some(route)) => route,
        Ok(None) => return json_error(404, "Route not found"),
        Err(e) => {
            error!("[Admin] Failed to check route: {}", e);
            return json_error(500, "Database error");
        }
    };

    match routes_repo::set_rate_limit(ctx.db.get_pool(), route_id, payload.requests_per_minute).await
    {
        Ok(_) => {
            let _ = api_core::ratelimit::set_limit(
                &ctx.redis_pool,
                &route.method,
                &route.path,
                payload.requests_per_minute,
            )
            .await;

            json!({
                "status": 200,
                "message": "Rate limit updated",
                "route_id": route_id,
                "requests_per_minute": payload.requests_per_minute
            })
        }
        Err(e) => {
            error!("[Admin] Failed to set rate limit: {}", e);
            json_error(500, "Failed to set rate limit")
        }
    }
}

pub async fn handle_admin_users_add_role(ctx: &AppContext, request: &ServiceRequest) -> Value {
    if let Err(e) = require_permission(ctx, request, &["users.edit"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let user_id = match parts.get(2) {
        Some(id_str) => match parse_user_id(id_str) {
            Ok(uuid) => uuid,
            Err(e) => return e,
        },
        None => return json_error(400, "Missing user id"),
    };

    let payload: IdPayload = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    match roles_repo::get_by_id(ctx.db.get_pool(), payload.id).await {
        Ok(Some(_)) => {}
        Ok(None) => return json_error(404, "Role not found"),
        Err(e) => {
            error!("[Admin] Failed to fetch role: {}", e);
            return json_error(500, "Database error");
        }
    }

    match user_roles_repo::add(ctx.db.get_pool(), &user_id, payload.id).await {
        Ok(_) => {
            let _ = crate::cache::user::invalidate_cached_user(&ctx.redis_pool, &user_id).await;
            let _ = crate::services::permission::sync_user_permissions(ctx, &user_id).await;
            json!({ "status": 200, "message": "Role assigned", "user_id": user_id.to_string() })
        }
        Err(e) => {
            error!("[Admin] Failed to assign role: {}", e);
            json_error(500, "Failed to assign role")
        }
    }
}

pub async fn handle_admin_users_remove_role(ctx: &AppContext, request: &ServiceRequest) -> Value {
    if let Err(e) = require_permission(ctx, request, &["users.edit"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let user_id = match parts.get(2) {
        Some(id_str) => match parse_user_id(id_str) {
            Ok(uuid) => uuid,
            Err(e) => return e,
        },
        None => return json_error(400, "Missing user id"),
    };
    let role_id = match id_from_parts(&parts, 4) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match user_roles_repo::remove(ctx.db.get_pool(), &user_id, role_id).await {
        Ok(_) => {
            let _ = crate::cache::user::invalidate_cached_user(&ctx.redis_pool, &user_id).await;
            let _ = crate::services::permission::sync_user_permissions(ctx, &user_id).await;
            json!({ "status": 200, "message": "Role removed", "user_id": user_id.to_string() })
        }
        Err(e) => {
            error!("[Admin] Failed to remove role: {}", e);
            json_error(500, "Failed to remove role")
        }
    }
}

#[derive(Deserialize)]
struct GrantCardPayload {
    card_id: String,
    #[serde(default)]
    rarity: Option<i16>,
}

pub async fn handle_admin_player_cards_list(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> Value {
    if let Err(e) = require_permission(ctx, request, &["users.edit"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let user_id = match parts.get(2) {
        Some(id_str) => match parse_user_id(id_str) {
            Ok(uuid) => uuid,
            Err(e) => return e,
        },
        None => return json_error(400, "Missing user id"),
    };

    match cards_repo::get_player_cards(ctx.db.get_pool(), &user_id).await {
        Ok(cards) => json!({ "status": 200, "cards": cards }),
        Err(e) => {
            error!("[Admin] Failed to list player cards: {}", e);
            json_error(500, "Failed to list player cards")
        }
    }
}

pub async fn handle_admin_grant_card(ctx: &AppContext, request: &ServiceRequest) -> Value {
    if let Err(e) = require_permission(ctx, request, &["users.edit"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let user_id = match parts.get(2) {
        Some(id_str) => match parse_user_id(id_str) {
            Ok(uuid) => uuid,
            Err(e) => return e,
        },
        None => return json_error(400, "Missing user id"),
    };

    let payload: GrantCardPayload = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(_) => return json_error(400, "Invalid JSON body"),
    };

    if payload.card_id.trim().is_empty() {
        return json_error(400, "card_id is required");
    }

    let rarity = payload.rarity.unwrap_or(0);
    if !(0..=2).contains(&rarity) {
        return json_error(400, "rarity must be between 0 and 2");
    }

    match cards_repo::card_price(ctx.db.get_pool(), &payload.card_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return json_error(404, "Card not available"),
        Err(e) => {
            error!("[Admin] Failed to fetch card price: {}", e);
            return json_error(500, "Database error");
        }
    }
    if rarity > crate::db::pack::max_rarity_for_card(&payload.card_id) {
        return json_error(400, "This card cannot have this rarity");
    }

    match cards_repo::grant_card(ctx.db.get_pool(), &user_id, &payload.card_id, rarity).await {
        Ok(()) => {
            info!(
                "[Admin] Granted card {} (rarity {}) to user {}",
                payload.card_id, rarity, user_id
            );
            json!({
                "status": 200,
                "message": "Card granted",
                "user_id": user_id.to_string(),
                "card_id": payload.card_id,
                "rarity": rarity,
            })
        }
        Err(e) => {
            error!("[Admin] Failed to grant card: {}", e);
            json_error(500, "Failed to grant card")
        }
    }
}

pub async fn handle_admin_remove_card_rarity(
    ctx: &AppContext,
    request: &ServiceRequest,
) -> Value {
    if let Err(e) = require_permission(ctx, request, &["users.edit"]).await {
        return e;
    }

    let parts: Vec<&str> = request.action.split('/').collect();
    let user_id = match parts.get(2) {
        Some(id_str) => match parse_user_id(id_str) {
            Ok(uuid) => uuid,
            Err(e) => return e,
        },
        None => return json_error(400, "Missing user id"),
    };
    let card_id = match parts.get(4) {
        Some(id) => *id,
        None => return json_error(400, "Missing card_id"),
    };
    let rarity = match parts.get(5).and_then(|r| r.parse::<i16>().ok()) {
        Some(r) => r,
        None => return json_error(400, "Missing or invalid rarity"),
    };

    match cards_repo::remove_card_rarity(ctx.db.get_pool(), &user_id, card_id, rarity).await {
        Ok(()) => {
            info!(
                "[Admin] Removed card {} (rarity {}) from user {}",
                card_id, rarity, user_id
            );
            json!({
                "status": 200,
                "message": "Card rarity removed",
                "user_id": user_id.to_string(),
                "card_id": card_id,
                "rarity": rarity,
            })
        }
        Err(e) => {
            error!("[Admin] Failed to remove card rarity: {}", e);
            json_error(500, "Failed to remove card rarity")
        }
    }
}

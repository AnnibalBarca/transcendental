use axum::Json;
use axum::http::StatusCode;
use axum::http::header::COOKIE;
use axum::response::{IntoResponse, Response};
use cookie::Cookie;
use log::warn;

use crate::http::handlers::ErrorResponse;

pub async fn enforce_access(
    redis_pool: &deadpool_redis::Pool,
    headers: &axum::http::HeaderMap,
    method: &axum::http::Method,
    service: &str,
    path: &str,
) -> Result<(), Response> {
    let access_token = extract_cookie(headers, "access_token");

    let Some(user_id) =
        crate::http::token_validator::extract_user_sub(redis_pool, access_token.as_deref()).await
    else {
        return Ok(());
    };

    let full_path = format!("/api/{}/{}", service, path);
    let normalized = api_core::ratelimit::normalize_path(&full_path);

// 1) Rate limit per user per route.
    if let Some(limit) = api_core::ratelimit::get_limit(redis_pool, method.as_str(), &normalized)
        .await
        .ok()
        .flatten()
    {
        let count =
            match api_core::ratelimit::incr_count(redis_pool, &user_id, method.as_str(), &normalized)
                .await
            {
                Ok(count) => count,
                Err(_) => return Ok(()),
            };

        if count > limit {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(ErrorResponse {
                    error: "Rate limit exceeded".to_string(),
                    status: 429,
                }),
            )
                .into_response());
        }
    }

// 2) Permission check: the user must hold a permission linked to the route.
    enforce_permission(redis_pool, &user_id, method, &normalized).await
}

async fn enforce_permission(
    redis_pool: &deadpool_redis::Pool,
    user_id: &str,
    method: &axum::http::Method,
    normalized_path: &str,
) -> Result<(), Response> {
    let route_key = api_core::permission::route_permission_key(method.as_str(), normalized_path);
    let required: Vec<String> = api_core::cache::get_json(redis_pool, &route_key)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    if required.is_empty() {
        return Ok(());
    }

    let user_key = api_core::permission::user_permission_key(user_id);
    let user_perms: Vec<String> = api_core::cache::get_json(redis_pool, &user_key)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    if user_perms.is_empty() {
        warn!(
            "[Gateway] No cached permissions for user {} on {} {} — allowing",
            user_id,
            method,
            normalized_path
        );
        return Ok(());
    }

    let allowed = user_perms
        .iter()
        .any(|perm| required.iter().any(|req| req == perm));
    if allowed {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Forbidden: missing permission".to_string(),
                status: 403,
            }),
        )
            .into_response())
    }
}

fn extract_cookie(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    let value = headers.get(COOKIE)?.to_str().ok()?;
    Cookie::split_parse(value)
        .filter_map(|c| c.ok())
        .find(|c| c.name() == name)
        .map(|c| c.value().to_string())
}

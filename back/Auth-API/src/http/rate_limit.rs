use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use crate::http::router::AppState;

// the admin panel (`ratelimit:{METHOD}:{path}` in Redis).
pub async fn rate_limit_middleware(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let full_path = format!("/api/auth{}", path);
    let normalized = api_core::ratelimit::normalize_path(&full_path);

    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    if let Some(limit) = api_core::ratelimit::get_limit(&state.redis_pool, method.as_str(), &normalized)
        .await
        .ok()
        .flatten()
    {
        let count =
            match api_core::ratelimit::incr_count(&state.redis_pool, &format!("ip:{}", ip), method.as_str(), &normalized)
                .await
            {
                Ok(count) => count,
                Err(_) => return next.run(req).await,
            };

        if count > limit {
            log::warn!(
                "[Auth] Rate limit exceeded for {} {} ({}): {}/{}",
                method,
                path,
                ip,
                count,
                limit
            );
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "error": "Rate limit exceeded",
                    "status": 429
                })),
            )
                .into_response();
        }
    }

    next.run(req).await
}

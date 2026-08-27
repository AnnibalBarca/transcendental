use axum::{
    Json, Router,
    extract::ws::rejection::WebSocketUpgradeRejection,
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{any, get},
};
use std::time::Instant;

use crate::http::handlers::{AppState, ErrorResponse, HealthResponse};
use crate::http::handlers::{proxy_http, proxy_redis, proxy_websocket};
use log::error;

pub fn create_gateway_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/*rest", any(api_handler))
        .with_state(state)
}

fn parse_service_path(rest: &str) -> (String, String) {
    let rest = rest.trim_start_matches('/');
    let mut parts = rest.splitn(2, '/');
    let service = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();
    (service, path)
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let redis_ok = match state.redis_pool.get().await {
        Ok(mut conn) => deadpool_redis::redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .is_ok(),
        Err(_) => false,
    };

    let status_code = if redis_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(HealthResponse {
            status: if redis_ok { "healthy" } else { "unhealthy" }.to_string(),
            service: "gateway".to_string(),
        }),
    )
}

async fn api_handler(
    ws: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
    State(state): State<AppState>,
    Path(rest): Path<String>,
    axum::extract::RawQuery(query): axum::extract::RawQuery,
    method: axum::http::Method,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let (service, path) = parse_service_path(&rest);
    if let Ok(ws_upgrade) = ws {
        let service_url = if service == "chess" {
            // Dynamic discovery: resolve which Chess-API instance hosts this game
            let game_id = query.as_deref().and_then(|q| {
                q.split('&')
                    .find(|part| part.starts_with("game_id="))
                    .and_then(|part| part.split('=').nth(1))
            });

            if let Some(gid) = game_id {
                match crate::chess_discovery::resolve_game_ws_url(&state.redis_pool, gid).await {
                    Some(url) => Some(url),
                    None => {
                        error!(
                            "[Gateway] Chess discovery failed for game_id={}, falling back to static config",
                            gid
                        );
                        state.config.game_api_url.clone()
                    }
                }
            } else {
                state.config.game_api_url.clone()
            }
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Unknown WebSocket service: {}", service),
                    status: 404,
                }),
            )
                .into_response();
        };

        let ws_path = if service == "chess" && path.is_empty() {
            "chess".to_string()
        } else {
            path
        };

        return match service_url {
            Some(url) => {
                proxy_websocket(ws_upgrade, state, &url, &ws_path, query.as_deref(), headers)
            }
            None => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: format!("Service {} not configured", service),
                    status: 503,
                }),
            )
                .into_response(),
        };
    }

    let start_time = Instant::now();

    if service == "notifications" {
        let service_url = state.config.notification_api_url.as_ref().unwrap();
        let path = if path.is_empty() {
            "notifications"
        } else {
            &path
        };
        return proxy_http(
            &state,
            service_url,
            "notification",
            path,
            query.as_deref(),
            method,
            headers,
            body,
            start_time,
        )
        .await;
    }

    if service == "auth" {
        if let Err(resp) = crate::http::rate_limit::enforce_access(
            &state.redis_pool,
            &headers,
            &method,
            &service,
            &path,
        )
        .await
        {
            return resp;
        }
        let service_url = state.config.auth_api_url.as_ref().unwrap();
        return proxy_http(
            &state,
            service_url,
            &service,
            &path,
            query.as_deref(),
            method,
            headers,
            body,
            start_time,
        )
        .await;
    }

    if service == "user" {
        if let Err(resp) = crate::http::rate_limit::enforce_access(
            &state.redis_pool,
            &headers,
            &method,
            &service,
            &path,
        )
        .await
        {
            return resp;
        }
        return proxy_redis(&state, &service, &path, method, headers, body, start_time).await;
    }

    if service == "chess" || service == "room" || service == "social" || service == "permission" {
        if let Err(resp) = crate::http::rate_limit::enforce_access(
            &state.redis_pool,
            &headers,
            &method,
            &service,
            &path,
        )
        .await
        {
            return resp;
        }
        return proxy_redis(&state, &service, &path, method, headers, body, start_time).await;
    }

    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Unknown service".to_string(),
            status: 404,
        }),
    )
        .into_response()
}

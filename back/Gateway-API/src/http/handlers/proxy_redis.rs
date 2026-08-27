use api_core::types::ServiceRequest;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use cookie::Cookie;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};
use log::{error, info};
use uuid::Uuid;

use crate::http::handlers::AppState;
use crate::http::handlers::ErrorResponse;
use crate::http::token_validator::validate_token_for_service;

pub async fn proxy_redis(
    state: &AppState,
    service: &str,
    path: &str,
    method: axum::http::Method,
    headers: HeaderMap,
    body: axum::body::Bytes,
    start_time: Instant,
) -> Response {
    let request_id = Uuid::new_v4().to_string();
    info!(
        "[Gateway] Queueing Redis request id={} service={} path={} method={}",
        request_id, service, path, method
    );

    let cookies = headers
        .get(axum::http::header::COOKIE)
        .and_then(|header| header.to_str().ok())
        .map(|header_value| {
            Cookie::split_parse(header_value)
                .filter_map(|c| c.ok())
                .map(|c| (c.name().to_string(), c.value().to_string()))
                .collect::<std::collections::HashMap<String, String>>()
        })
        .unwrap_or_default();

    if let Err(e) = validate_token_for_service(service, path, &cookies, &state.redis_pool).await {
        state.metrics.record(service, 0, true);
        return e;
    }

    let body_str = String::from_utf8(body.to_vec()).unwrap_or_default();

    let service_req = ServiceRequest {
        id: request_id.clone(),
        method: method.to_string(),
        action: path.to_string(),
        cookies,
        body: body_str,
        headers: Default::default(),
        internal: false,
    };

    if let Err(e) = push_to_queue(
        &state.redis_pool,
        &format!("{}:requests", service),
        &service_req,
    )
    .await
    {
        state.metrics.record(service, 0, true);
        error!("[Gateway] Failed to queue request: {}", e);
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Failed to queue request".to_string(),
                status: 503,
            }),
        )
            .into_response();
    }

    let (tx, mut rx) = mpsc::unbounded_channel();
    state.request_router.register(request_id.clone(), tx);
    info!(
        "[Gateway] Registered request id={} waiting for response",
        request_id
    );

    let res = match timeout(Duration::from_secs(30), rx.recv()).await {
        Ok(Some(response)) => {
            info!("[Gateway] Received response for request id={}", request_id);
            let is_error =
                response.status().is_client_error() || response.status().is_server_error();
            state
                .metrics
                .record(service, start_time.elapsed().as_millis() as u64, is_error);
            state.request_router.cleanup(&request_id);
            info!("[Gateway] Cleaned up request id={}", request_id);
            Ok(response)
        }
        _ => {
            info!("[Gateway] Timeout waiting for request id={}", request_id);
            state
                .metrics
                .record(service, start_time.elapsed().as_millis() as u64, true);
            state.request_router.cleanup(&request_id);
            info!(
                "[Gateway] Cleaned up request id={} after timeout",
                request_id
            );
            let status = if start_time.elapsed().as_secs() >= 30 {
                StatusCode::GATEWAY_TIMEOUT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((
                status,
                Json(ErrorResponse {
                    error: "Service timeout or unavailable".to_string(),
                    status: status.as_u16(),
                }),
            ))
        }
    };

    drop(rx);

    match res {
        Ok(response) => response.into_response(),
        Err((status, json)) => (status, json).into_response(),
    }
}

pub async fn push_to_queue(
    redis_pool: &deadpool_redis::Pool,
    channel: &str,
    service_request: &ServiceRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = redis_pool.get().await?;
    let payload = serde_json::to_string(service_request)?;

    let _: String = redis::cmd("XADD")
        .arg(channel)
        .arg("*")
        .arg("data")
        .arg(&payload)
        .query_async(&mut *conn)
        .await?;

    Ok(())
}

use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use futures::StreamExt;
use std::time::Instant;
use log::{error, info};

use crate::http::handlers::AppState;
use crate::http::handlers::ErrorResponse;

pub async fn proxy_http(
    state: &AppState,
    service_url: &str,
    service: &str,
    path: &str,
    query: Option<&str>,
    method: axum::http::Method,
    headers: HeaderMap,
    body: axum::body::Bytes,
    start_time: Instant,
) -> Response {
    let target_url = if let Some(q) = query {
        format!("{}/{}?{}", service_url, path, q)
    } else {
        format!("{}/{}", service_url, path)
    };
    info!(
        "[Gateway] Proxying HTTP {} {} -> {}",
        method, service, target_url
    );

    let mut req = match method {
        axum::http::Method::GET => state.http_client.get(&target_url),
        axum::http::Method::POST => state.http_client.post(&target_url),
        axum::http::Method::PUT => state.http_client.put(&target_url),
        axum::http::Method::DELETE => state.http_client.delete(&target_url),
        axum::http::Method::PATCH => state.http_client.patch(&target_url),
        axum::http::Method::HEAD => state.http_client.head(&target_url),
        _ => {
            let latency_ms = start_time.elapsed().as_millis() as u64;
            state.metrics.record(service, latency_ms, true);
            return (
                StatusCode::METHOD_NOT_ALLOWED,
                axum::Json(ErrorResponse {
                    error: "Method not allowed".to_string(),
                    status: 405,
                }),
            )
                .into_response();
        }
    };

    for (name, value) in headers.iter() {
        if should_forward_header(name) {
            if let Ok(val) = value.to_str() {
                req = req.header(name.as_str(), val);
            }
        }
    }

    if !body.is_empty() {
        req = req.body(body.to_vec());
    }

    match req.send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let axum_status =
                StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let is_error = status_code >= 400;
            let latency_ms = start_time.elapsed().as_millis() as u64;
            state.metrics.record(service, latency_ms, is_error);

            info!("[Gateway] {} {} - Status: {}", method, service, status_code);

            let is_sse = response
                .headers()
                .get("content-type")
                .and_then(|ct| ct.to_str().ok())
                .map(|ct| ct.contains("text/event-stream"))
                .unwrap_or(false);

            let mut response_builder = axum::response::Response::builder().status(axum_status);

            for (name, value) in response.headers().iter() {
                if should_forward_header_str(name.as_str()) {
                    if let Ok(val) = value.to_str() {
                        response_builder = response_builder.header(name.as_str(), val);
                    }
                }
            }

            if is_sse {
                let stream = response.bytes_stream().map(|result| match result {
                    Ok(bytes) => Ok::<_, std::convert::Infallible>(bytes),
                    Err(_) => Ok::<_, std::convert::Infallible>(bytes::Bytes::new()),
                });

                response_builder
                    .body(Body::from_stream(stream))
                    .unwrap_or_else(|_| {
                        axum::response::Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from("Failed to build SSE stream"))
                            .unwrap()
                    })
                    .into_response()
            } else {
                match response.bytes().await {
                    Ok(bytes) => response_builder
                        .body(axum::body::Body::from(bytes))
                        .unwrap_or_else(|_| {
                            axum::response::Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(axum::body::Body::from("Failed to build response"))
                                .unwrap()
                        })
                        .into_response(),
                    Err(e) => {
                        error!("[Gateway] Failed to read response body: {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            axum::Json(ErrorResponse {
                                error: "Failed to read service response".to_string(),
                                status: 500,
                            }),
                        )
                            .into_response()
                    }
                }
            }
        }
        Err(e) => {
            let latency_ms = start_time.elapsed().as_millis() as u64;
            state.metrics.record(service, latency_ms, true);

            error!("[Gateway] Failed to proxy request to {}: {}", service, e);

            (
                StatusCode::SERVICE_UNAVAILABLE,
                axum::Json(ErrorResponse {
                    error: "Service unavailable".to_string(),
                    status: 503,
                }),
            )
                .into_response()
        }
    }
}

fn should_forward_header(name: &axum::http::HeaderName) -> bool {
    matches!(
        name.as_str().to_lowercase().as_str(),
        "content-type"
            | "content-length"
            | "content-encoding"
            | "vary"
            | "authorization"
            | "cookie"
            | "set-cookie"
            | "accept"
            | "accept-encoding"
            | "user-agent"
            | "location"
            | "cache-control"
            | "connection"
            | "keep-alive"
            | "transfer-encoding"
            | "x-accel-buffering"
            | "x-forwarded-for"
            | "x-forwarded-proto"
            | "x-real-ip"
    )
}

fn should_forward_header_str(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "content-type"
            | "content-length"
            | "content-encoding"
            | "vary"
            | "authorization"
            | "cookie"
            | "set-cookie"
            | "accept"
            | "accept-encoding"
            | "user-agent"
            | "location"
            | "cache-control"
            | "connection"
            | "keep-alive"
            | "transfer-encoding"
            | "x-accel-buffering"
    )
}

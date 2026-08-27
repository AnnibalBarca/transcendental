use std::sync::Arc;

use crate::AppState;
use crate::event::NotificationMetadata;
use axum::extract::{Path, State};
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use futures::stream::{BoxStream, Stream};
use futures::StreamExt;
use log::{error, info, warn};
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::interval;
use uuid::Uuid;

pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let redis_ok = match state.redis_pool.get().await {
        Ok(mut conn) => deadpool_redis::redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .is_ok(),
        Err(_) => false,
    };

    let status_code = if redis_ok {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        axum::Json(serde_json::json!({
            "status": if redis_ok { "healthy" } else { "unhealthy" },
            "service": "notification",
            "dependencies": {
                "redis": { "status": if redis_ok { "healthy" } else { "unhealthy" } }
            }
        })),
    )
}
pub async fn sse_connect(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> axum::response::Response {
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            warn!("[Notification-API] Invalid user_id for SSE: {}", user_id);
            let stream: BoxStream<'static, Result<Event, Infallible>> = Box::pin(async_stream::stream! {
                loop {
                    yield Ok(Event::default().data("{\"error\":\"invalid user_id\"}"));
                }
            });
            return add_sse_headers(Sse::new(stream).into_response());
        }
    };

    let metadata = NotificationMetadata { user_id: user_uuid };
    let (mut rx, connection_id, guard) = state.notification_service.manager().connect(metadata);
    info!(
        "[Notification-API] SSE connection established for user {} (conn_id: {})",
        user_uuid, connection_id
    );

    let stream: BoxStream<'static, Result<Event, Infallible>> = Box::pin(async_stream::stream! {
        let _guard = guard;
        let mut ping_interval = interval(Duration::from_secs(15));
        ping_interval.tick().await;

        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok(notification) => yield Ok(Event::default().data(notification)),
                        Err(_) => break,
                    }
                }
                _ = ping_interval.tick() => {
                    yield Ok(Event::default().comment("ping"));
                }
            }
        }
    });
    add_sse_headers(Sse::new(stream).into_response())
}

fn add_sse_headers(mut response: axum::response::Response) -> axum::response::Response {
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-cache"),
    );
    response.headers_mut().insert(
        axum::http::HeaderName::from_static("x-accel-buffering"),
        axum::http::HeaderValue::from_static("no"),
    );
    response
}

pub async fn sse_rooms(
    State(state): State<AppState>,
) -> Sse<BoxStream<'static, Result<Event, Infallible>>> {
    let (tx, mut rx) = broadcast::channel::<String>(32);

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    tokio::spawn(async move {
        loop {
            let client = match redis::Client::open(redis_url.as_str()) {
                Ok(c) => c,
                Err(e) => {
                    error!("[RoomSSE] Failed to create redis client: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let mut pubsub = match client.get_async_pubsub().await {
                Ok(p) => p,
                Err(e) => {
                    error!("[RoomSSE] Failed to get async pubsub: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            if let Err(e) = pubsub.subscribe("room:public:updates").await {
                error!("[RoomSSE] Failed to subscribe: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            let mut stream = pubsub.on_message();
            while let Some(msg) = stream.next().await {
                let payload: String = msg.get_payload().unwrap_or_default();
                if tx.send(payload).is_err() {
                    break;
                }
            }
        }
    });

    let stream = async_stream::stream! {
        let mut ping_interval = interval(Duration::from_secs(30));
        ping_interval.tick().await;

        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok(_) => {
                            yield Ok(Event::default().data("{\"type\":\"rooms_updated\"}"));
                        }
                        Err(_) => break,
                    }
                }
                _ = ping_interval.tick() => {
                    yield Ok(Event::default().comment("ping"));
                }
            }
        }
    };
    Sse::new(Box::pin(stream))
}

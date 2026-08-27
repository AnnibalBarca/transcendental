use log::{debug, error, info, warn};

use super::router::Router;
use api_core::redis::stream::RedisStreamManager;

pub struct ResponseListener {
    redis_pool: deadpool_redis::Pool,
    request_router: Router,
}

impl ResponseListener {
    pub fn new(redis_pool: deadpool_redis::Pool, request_router: Router) -> Self {
        Self {
            redis_pool,
            request_router,
        }
    }

    pub async fn run(self) {
        let stream_manager = RedisStreamManager::new(
            self.redis_pool.clone(),
            "gateway:responses",
            "gateway-response-group",
            "gateway-consumer-1",
        );

        let router = self.request_router.clone();

        let result = stream_manager.listen_concurrently(move |_msg_id, data| {
            let router = router.clone();
            async move {
                if let (Some(id_field), Some(response_data)) = (data.get("id"), data.get("data")) {
                    debug!("[Gateway] Decoding response message for request={}", id_field);
                    debug!("[Gateway] Raw response payload: {}", response_data);

                    match serde_json::from_str::<serde_json::Value>(response_data) {
                        Ok(json_response) => {
                            debug!("[Gateway] Successfully parsed JSON for request={}", id_field);

                            let status_code = json_response
                                .get("status")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(200)
                                as u16;

                            let axum_status = axum::http::StatusCode::from_u16(status_code)
                                .unwrap_or_else(|_| {
                                    warn!("[Gateway] Invalid status code {} in response for request={}, defaulting to 500", status_code, id_field);
                                    axum::http::StatusCode::INTERNAL_SERVER_ERROR
                                });

                            debug!("[Gateway] Building Axum response for request={}", id_field);
                            let response = axum::response::Response::builder()
                                .status(axum_status)
                                .header("content-type", "application/json")
                                .body(axum::body::Body::from(response_data.clone()))
                                .unwrap_or_else(|e| {
                                    error!("[Gateway] Error building Axum response: {}", e);
                                    axum::response::Response::builder()
                                        .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                                        .body(axum::body::Body::from("Error building response"))
                                        .unwrap()
                                });

                            debug!("[Gateway] Sending response to router for request={}", id_field);
                            match router.send_response(&id_field, response) {
                                Ok(()) => {
                                    debug!("[Gateway] Successfully sent response for request={}", id_field);
                                }
                                Err(e) => {
                                    if e.to_string().contains("not found") {
                                        warn!("[Gateway] No waiting request found for id={}, response will be discarded", id_field);
                                    } else {
                                        error!("[Gateway] Error sending response for request={}: {}", id_field, e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("[Gateway] Failed to parse response JSON for request {}: {}", id_field, e);
                        }
                    }
                } else {
                    warn!("[Gateway] Malformed message: Missing 'id' or 'data' field");
                }
                Ok(())
            }
        }).await;

        match result {
            Ok(_) => info!("[Gateway] Response listener stopped"),
            Err(e) => error!("[Gateway] Response listener error: {}", e),
        }
    }
}

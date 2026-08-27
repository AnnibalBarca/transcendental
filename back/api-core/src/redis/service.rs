use std::future::Future;
use std::time::Duration;

use deadpool_redis::redis::cmd;
use log::{error, info};
use serde_json::Value;

use crate::redis::stream::RedisStreamManager;

pub struct ServiceConfig {
    pub request_stream: String,
    pub response_stream: String,
    pub group_name: String,
    pub consumer_name: String,
}

pub struct IncomingRequest {
    pub request_id: String,
    pub payload: String,
}

pub struct OutgoingResponse {
    pub request_id: String,
    pub payload: Value,
}

pub struct RedisService {
    pool: deadpool_redis::Pool,
    config: ServiceConfig,
}

impl RedisService {
    pub fn new(pool: deadpool_redis::Pool, config: ServiceConfig) -> Self {
        Self { pool, config }
    }

    pub async fn listen<F, Fut>(
        &self,
        handler: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(IncomingRequest) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = Result<OutgoingResponse, Box<dyn std::error::Error + Send + Sync>>>
            + Send,
    {
        let manager = RedisStreamManager::new(
            self.pool.clone(),
            &self.config.request_stream,
            &self.config.group_name,
            &self.config.consumer_name,
        );

        let response_stream = self.config.response_stream.clone();
        let pool = self.pool.clone();

        manager
            .listen_concurrently(move |_msg_id, data| {
                let response_stream = response_stream.clone();
                let pool = pool.clone();
                let handler = handler.clone();

                async move {
                    let request_id = data.get("id").cloned().unwrap_or_default();
                    let payload = data.get("data").cloned().unwrap_or_default();

                    if payload.is_empty() {
                        return Ok(());
                    }

                    let response = handler(IncomingRequest {
                        request_id: request_id.clone(),
                        payload,
                    })
                    .await?;

                    send_response(
                        &pool,
                        &response.request_id,
                        response.payload,
                        &response_stream,
                    )
                    .await
                }
            })
            .await
    }

    pub async fn listen_forever<F, Fut>(&self, handler: F)
    where
        F: Fn(IncomingRequest) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = Result<OutgoingResponse, Box<dyn std::error::Error + Send + Sync>>>
            + Send,
    {
        loop {
            match self.listen(handler.clone()).await {
                Ok(()) => {
                    error!("Redis service listener exited unexpectedly. Restarting in 5s...");
                }
                Err(e) => {
                    error!("Redis service listener failed: {}. Restarting in 5s...", e);
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn send_response(
    pool: &deadpool_redis::Pool,
    request_id: &str,
    response_json: Value,
    response_stream: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = pool.get().await?;
    let response_str = response_json.to_string();

    let message_id: String = cmd("XADD")
        .arg(response_stream)
        .arg("*")
        .arg("id")
        .arg(request_id)
        .arg("data")
        .arg(response_str)
        .query_async(&mut *conn)
        .await?;

    info!(
        "Response sent to Redis at message_id={} for request_id={}",
        message_id, request_id
    );
    Ok(())
}

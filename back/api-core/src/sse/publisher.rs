use std::marker::PhantomData;

use deadpool_redis::Pool;
use log::{error, warn};
use serde::Serialize;

use crate::sse::envelope::SseEnvelope;
use crate::sse::traits::SseEvent;

pub struct SsePublisher<E, T>
where
    E: SseEvent,
    T: Serialize + Send + Sync + 'static,
{
    redis_pool: Pool,
    channel: String,
    _event: PhantomData<E>,
    _target: PhantomData<T>,
}

impl<E, T> SsePublisher<E, T>
where
    E: SseEvent,
    T: Serialize + Send + Sync + 'static,
{
    pub fn new(redis_pool: Pool, channel: impl Into<String>) -> Self {
        Self {
            redis_pool,
            channel: channel.into(),
            _event: PhantomData,
            _target: PhantomData,
        }
    }

    pub async fn publish(&self, target: T, event: &E) {
        let envelope = SseEnvelope {
            target,
            event: event.clone(),
        };

        let payload = match serde_json::to_string(&envelope) {
            Ok(p) => p,
            Err(e) => {
                error!("[SsePublisher] Failed to serialize envelope: {}", e);
                return;
            }
        };

        let mut conn = match self.redis_pool.get().await {
            Ok(c) => c,
            Err(e) => {
                error!("[SsePublisher] Failed to get Redis connection: {}", e);
                return;
            }
        };

        let result: redis::RedisResult<()> = redis::cmd("PUBLISH")
            .arg(&self.channel)
            .arg(payload)
            .query_async(&mut *conn)
            .await;

        if let Err(e) = result {
            warn!("[SsePublisher] Failed to publish event: {}", e);
        }
    }
}

impl<E, T> Clone for SsePublisher<E, T>
where
    E: SseEvent,
    T: Serialize + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            redis_pool: self.redis_pool.clone(),
            channel: self.channel.clone(),
            _event: PhantomData,
            _target: PhantomData,
        }
    }
}

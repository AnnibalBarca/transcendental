use deadpool_redis::redis::streams::StreamReadReply;
use deadpool_redis::redis::{FromRedisValue, Value, cmd};
use log::{debug, error, info};
use std::collections::HashMap;

pub struct RedisStreamManager {
    pool: deadpool_redis::Pool,
    stream_name: String,
    group_name: String,
    consumer_name: String,
}

impl RedisStreamManager {
    pub fn new(
        pool: deadpool_redis::Pool,
        stream_name: &str,
        group_name: &str,
        consumer_name: &str,
    ) -> Self {
        Self {
            pool,
            stream_name: stream_name.to_string(),
            group_name: group_name.to_string(),
            consumer_name: consumer_name.to_string(),
        }
    }

    pub async fn ensure_group_exists(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.pool.get().await?;
        let result: redis::RedisResult<()> = cmd("XGROUP")
            .arg("CREATE")
            .arg(&self.stream_name)
            .arg(&self.group_name)
            .arg("0")
            .arg("MKSTREAM")
            .query_async(&mut *conn)
            .await;

        if let Err(e) = result {
            if !e.to_string().contains("BUSYGROUP") {
                return Err(e.into());
            }
        }
        Ok(())
    }

    pub async fn listen_concurrently<F, Fut>(
        &self,
        handler: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(String, HashMap<String, String>) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
            + Send,
    {
        self.ensure_group_exists().await?;

        info!(
            "Starting Redis stream listener for stream: {}, group: {}, consumer: {}",
            self.stream_name, self.group_name, self.consumer_name
        );

        loop {
            let mut conn = match self.pool.get().await {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to get redis connection: {}. Retrying in 5s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            let raw: Value = match cmd("XREADGROUP")
                .arg("GROUP")
                .arg(&self.group_name)
                .arg(&self.consumer_name)
                .arg("BLOCK")
                .arg(2000)
                .arg("COUNT")
                .arg(10)
                .arg("STREAMS")
                .arg(&self.stream_name)
                .arg(">")
                .query_async(&mut *conn)
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    error!("XREADGROUP error: {}. Retrying in 5s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            if matches!(raw, Value::Nil) {
                continue;
            }

            let reply = match StreamReadReply::from_redis_value(&raw) {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to parse stream reply: {}", e);
                    continue;
                }
            };

            for stream in reply.keys {
                for record in stream.ids {
                    let message_id = record.id;

                    let mut data = HashMap::new();
                    for (k_str, v) in record.map {
                        if let Value::Data(v_bytes) = v {
                            let val_str = String::from_utf8_lossy(&v_bytes).into_owned();
                            data.insert(k_str, val_str);
                        }
                    }

                    let handler_clone = handler.clone();
                    let msg_id_clone = message_id.clone();
                    let pool = self.pool.clone();
                    let stream_name = self.stream_name.clone();
                    let group_name = self.group_name.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handler_clone(msg_id_clone.clone(), data).await {
                            error!("Handler failed for message {}: {}", msg_id_clone, e);
                        } else {
                            if let Ok(mut ack_conn) = pool.get().await {
                                let _: redis::RedisResult<()> = cmd("XACK")
                                    .arg(&stream_name)
                                    .arg(&group_name)
                                    .arg(&msg_id_clone)
                                    .query_async(&mut *ack_conn)
                                    .await;
                                debug!("Acknowledged message {}", msg_id_clone);
                            }
                        }
                    });
                }
            }
        }
    }
}

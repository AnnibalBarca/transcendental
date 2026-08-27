use std::time::Duration;

use log::{error, info};

pub async fn create_redis_pool(
    redis_url: &str,
) -> Result<deadpool_redis::Pool, Box<dyn std::error::Error>> {
    use deadpool_redis::{Config, Runtime};

    let cfg = Config::from_url(redis_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;

    let mut conn = pool.get().await?;
    redis::cmd("PING").query_async::<_, ()>(&mut *conn).await?;

    Ok(pool)
}

pub async fn get_redis_pool(redis_url: &str) -> deadpool_redis::Pool {
    loop {
        match create_redis_pool(redis_url).await {
            Ok(pool) => {
                info!("Redis pool created");
                return pool;
            }
            Err(e) => {
                error!("Failed to connect to Redis: {}. Retrying in 5s...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

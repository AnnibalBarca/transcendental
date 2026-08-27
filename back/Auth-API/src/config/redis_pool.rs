use deadpool_redis::{Config, Pool, Runtime};
use once_cell::sync::Lazy;
use std::env;

pub static REDIS_POOL: Lazy<Pool> = Lazy::new(|| {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set in the environment");

    let cfg = Config::from_url(redis_url);

    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis connection pool.")
});

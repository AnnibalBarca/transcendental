use serde_json::{json, Value};
use crate::AppContext;

pub async fn handle_health(ctx: &AppContext) -> Value {
    let mut all_healthy = true;

    // Check Postgres
    let pg_status = match sqlx::query("SELECT 1").fetch_one(ctx.db.get_pool()).await {
        Ok(_) => "healthy",
        Err(_) => {
            all_healthy = false;
            "unhealthy"
        }
    };

    // Check Redis
    let redis_status = match ctx.redis_pool.get().await {
        Ok(mut conn) => {
            match deadpool_redis::redis::cmd("PING")
                .query_async::<_, String>(&mut conn)
                .await
            {
                Ok(_) => "healthy",
                Err(_) => {
                    all_healthy = false;
                    "unhealthy"
                }
            }
        }
        Err(_) => {
            all_healthy = false;
            "unhealthy"
        }
    };

    // Check MinIO/Storage
    let storage_status = match &ctx.storage {
        Some(storage) => {
            if storage.ping().await {
                "healthy"
            } else {
                all_healthy = false;
                "unhealthy"
            }
        }
        None => "not_configured",
    };

    json!({
        "status": if all_healthy { "healthy" } else { "unhealthy" },
        "service": "user",
        "dependencies": {
            "postgres": { "status": pg_status },
            "redis": { "status": redis_status },
            "storage": { "status": storage_status }
        }
    })
}
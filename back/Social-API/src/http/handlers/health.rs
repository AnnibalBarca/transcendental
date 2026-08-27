use serde_json::{json, Value};
use crate::http::router::ServiceContext;

pub async fn handle_health(ctx: &ServiceContext) -> Value {
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

    json!({
        "status": if all_healthy { "healthy" } else { "unhealthy" },
        "service": "social",
        "dependencies": {
            "postgres": { "status": pg_status },
            "redis": { "status": redis_status }
        }
    })
}
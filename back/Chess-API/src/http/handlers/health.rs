use api_core::types::ServiceRequest;
use serde_json::{json, Value};

use crate::service::ServiceContext;

pub async fn handle_health(ctx: ServiceContext, _request: ServiceRequest) -> Value {
    let redis_status = match ctx.redis_pool.get().await {
        Ok(mut conn) => {
            match deadpool_redis::redis::cmd("PING")
                .query_async::<_, String>(&mut conn)
                .await
            {
                Ok(_) => "healthy",
                Err(_) => "unhealthy",
            }
        }
        Err(_) => "unhealthy",
    };

    let all_healthy = redis_status == "healthy";

    json!({
        "status": if all_healthy { "healthy" } else { "unhealthy" },
        "service": "chess",
        "instance": ctx.instance_id,
        "dependencies": {
            "redis": { "status": redis_status }
        }
    })
}
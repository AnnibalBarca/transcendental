use api_core::http::response::json_error;
use api_core::types::ServiceRequest;
use deadpool_redis::redis::cmd;
use serde_json::json;
use uuid::Uuid;

use crate::registry;
use crate::service::ServiceContext;

pub async fn handle_create_game(ctx: ServiceContext, request: ServiceRequest) -> serde_json::Value {
    let game_id = Uuid::new_v4().to_string();

    let mut initial_time_ms: u64 = 600_000;
    if let Ok(body_value) = serde_json::from_str::<serde_json::Value>(&request.body) {
        if let Some(tc) = body_value.get("time_control").and_then(|v| v.as_str()) {
            if let Ok(minutes) = tc.parse::<u64>() {
                if matches!(minutes, 5 | 10 | 15) {
                    initial_time_ms = minutes * 60_000;
                }
            }
        }
    }

    let instance = ctx
        .game_manager
        .create_game(game_id.clone(), ctx.redis_pool.clone(), initial_time_ms)
        .await;
    instance.game_loop.restart();

    if let Err(e) =
        registry::register_game_mapping(&ctx.redis_pool, &game_id, &ctx.instance_id).await
    {
        return json_error(500, &format!("Failed to register game mapping: {}", e));
    }

    let result_key = format!("chess:create_game:result:{}", request.id);
    let result_json = json!({
        "game_id": &game_id,
        "status": "created"
    });

    let mut conn = match ctx.redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return json_error(500, &format!("Redis pool error: {}", e));
        }
    };

    let _: deadpool_redis::redis::RedisResult<()> = cmd("SETEX")
        .arg(&result_key)
        .arg(10)
        .arg(result_json.to_string())
        .query_async(&mut *conn)
        .await;

    json!({
        "status": 200,
        "game_id": game_id,
        "message": "Game created"
    })
}

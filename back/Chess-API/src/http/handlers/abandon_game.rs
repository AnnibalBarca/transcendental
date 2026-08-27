use api_core::http::response::json_error;
use api_core::types::ServiceRequest;
use deadpool_redis::redis::cmd;
use serde_json::json;

use crate::game::board::Color;
use crate::game::redis_helpers::publish_game_result;
use crate::service::ServiceContext;
use crate::websocket::lobby::{OutgoingMessage, PlayerId};

pub async fn handle_abandon_game(ctx: ServiceContext, request: ServiceRequest) -> serde_json::Value {
    #[derive(serde::Deserialize)]
    struct AbandonBody {
        game_id: String,
        user_id: String,
    }

    let parsed = serde_json::from_str::<AbandonBody>(&request.body);
    let (game_id, user_id) = match parsed {
        Ok(body) => (body.game_id, body.user_id),
        Err(e) => {
            set_abandon_result(&ctx.redis_pool, &request.id, "invalid body").await;
            return json_error(400, &format!("Invalid abandon body: {}", e));
        }
    };

    let Some(instance) = ctx.game_manager.get_game(&game_id).await else {
        set_abandon_result(&ctx.redis_pool, &request.id, "game not found").await;
        return json_error(404, "Game not found");
    };

    let abandoner = {
        let lobby = instance.lobby.lock().await;
        lobby.user_slot_map.get(&user_id).cloned()
    };

    let winner = match abandoner {
        Some(PlayerId::Player1) => Color::Black,
        Some(PlayerId::Player2) => Color::White,
        None => Color::Black,
    };
    let winner_label = format!("{:?}", winner).to_lowercase();

    {
        let lobby = instance.lobby.lock().await;
        lobby.broadcast(OutgoingMessage {
            action: "checkmate".to_string(),
            from: None,
            message: json!({"winner": winner_label, "reason": "resignation"}),
        });
        let (white_uid, black_uid) = lobby.color_user_ids();
        publish_game_result(
            &ctx.redis_pool,
            &game_id,
            &winner_label,
            Some(&winner_label),
            white_uid.as_deref(),
            black_uid.as_deref(),
        )
        .await;
    }

    instance.game_loop.end();
    instance.end_game_cleanup().await;

    set_abandon_result(&ctx.redis_pool, &request.id, "ok").await;

    json!({
        "status": 200,
        "winner": winner_label,
        "message": "Game abandoned"
    })
}

async fn set_abandon_result(redis_pool: &deadpool_redis::Pool, request_id: &str, status: &str) {
    let key = format!("chess:abandon:result:{}", request_id);
    let result = json!({
        "status": status,
        "abandoned": true
    });
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(_) => return,
    };
    let _: deadpool_redis::redis::RedisResult<()> = cmd("SETEX")
        .arg(&key)
        .arg(10)
        .arg(result.to_string())
        .query_async(&mut *conn)
        .await;
}

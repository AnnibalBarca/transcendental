use std::sync::Arc;

use api_core::http::response::json_error;
use api_core::http::router::Router;
use api_core::redis::service::{IncomingRequest, OutgoingResponse, RedisService, ServiceConfig};
use api_core::types::ServiceRequest;

use crate::game::manager::GameManager;
use crate::http::handlers::health::handle_health;
use crate::http::handlers::abandon_game::handle_abandon_game;
use crate::http::handlers::create_game::handle_create_game;

#[derive(Clone)]
pub struct ServiceContext {
    pub game_manager: Arc<GameManager>,
    pub redis_pool: deadpool_redis::Pool,
    pub instance_id: String,
}

pub async fn listen_for_requests(
    pool: deadpool_redis::Pool,
    game_manager: Arc<GameManager>,
    instance_id: String,
) {
    let mut router = Router::<ServiceContext>::new();
    router.register("POST", "game/create", handle_create_game);
    router.register("POST", "game/abandon", handle_abandon_game);
    router.register("GET", "health", handle_health);

    let metrics_port = std::env::var("ROUTER_METRICS_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok());
    if let Some(port) = router.start_metrics_server(metrics_port).await {
        log::info!(
            "[Chess] Router metrics available at http://localhost:{}/metrics",
            port
        );
    }

    let ctx = ServiceContext {
        game_manager,
        redis_pool: pool.clone(),
        instance_id: instance_id.clone(),
    };

    let config = ServiceConfig {
        request_stream: format!("chess:requests:{}", instance_id),
        response_stream: "gateway:responses".to_string(),
        group_name: format!("chess-service-group-{}", instance_id),
        consumer_name: format!("chess-consumer-{}", instance_id),
    };

    let service = RedisService::new(pool, config);

    service
        .listen_forever(move |req: IncomingRequest| {
            let router = router.clone();
            let ctx = ctx.clone();

            async move {
                let request = match serde_json::from_str::<ServiceRequest>(&req.payload) {
                    Ok(request) => request,
                    Err(e) => {
                        return Ok(OutgoingResponse {
                            request_id: req.request_id,
                            payload: json_error(400, &format!("Invalid request JSON: {}", e)),
                        });
                    }
                };

                let response = router.route(ctx, request).await;

                Ok(OutgoingResponse {
                    request_id: req.request_id,
                    payload: response,
                })
            }
        })
        .await;
}

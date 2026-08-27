use crate::http::router::{build_router, ServiceContext};
use crate::types::ServiceRequest;
use crate::user_state::RedisSessionManager;
use crate::db::db::Database;
use api_core::http::router::Router;
use api_core::redis::stream::RedisStreamManager;
use notification::event::NotificationBus;
use std::sync::Arc;
use log::{error, info};

const REQUEST_STREAM: &str = "room:requests";
const RESPONSE_STREAM: &str = "gateway:responses";

pub async fn listen_for_requests(
    db: Arc<Database>,
    pool: &deadpool_redis::Pool,
    session_manager: Arc<RedisSessionManager>,
    notification_bus: NotificationBus,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let router = build_router();

    let metrics_port = std::env::var("ROUTER_METRICS_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok());
    if let Some(port) = router.start_metrics_server(metrics_port).await {
        info!(
            "[Room] Router metrics available at http://localhost:{}/metrics",
            port
        );
    }

    let ctx = ServiceContext {
        db,
        redis_pool: pool.clone(),
        session_manager,
        notification_bus,
    };

    let stream_manager = RedisStreamManager::new(
        pool.clone(),
        REQUEST_STREAM,
        "room-service-group",
        "room-consumer-1",
    );

    stream_manager
        .listen_concurrently(move |_msg_id, data| {
            let ctx = ctx.clone();
            let router = router.clone();

            async move {
                if let Some(payload_str) = data.get("data") {
                    if let Err(e) = process_single_request(ctx, router, payload_str.clone()).await {
                        error!(
                            "[Room-Listener] Error processing request in background: {}",
                            e
                        );
                    }
                }
                Ok(())
            }
        })
        .await
}

async fn process_single_request(
    ctx: ServiceContext,
    router: Router<ServiceContext>,
    payload: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match serde_json::from_str::<ServiceRequest>(&payload) {
        Ok(request) => {
            let request_id = request.id.clone();
            let redis_pool = ctx.redis_pool.clone();

            let response_json = router.route(ctx, request).await;

            send_response(&redis_pool, &request_id, response_json).await?;
        }
        Err(e) => {
            error!("[Room] Failed to parse request JSON: {}", e);
        }
    }

    Ok(())
}

async fn send_response(
    pool: &deadpool_redis::Pool,
    request_id: &str,
    response_json: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = pool.get().await?;

    let response_str = response_json.to_string();

    let message_id: String = redis::cmd("XADD")
        .arg("gateway:responses")
        .arg("*")
        .arg("id")
        .arg(request_id)
        .arg("data")
        .arg(response_str)
        .query_async(&mut *conn)
        .await?;

    info!(
        "[Room] Response sent to Redis at message_id={} for request_id={}",
        message_id, request_id
    );

    Ok(())
}

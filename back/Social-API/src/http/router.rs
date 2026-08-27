use std::sync::Arc;

use api_core::http::router::Router;
use notification::event::NotificationBus;

use crate::db::db::Database;
use crate::http::handlers::{
    accept_friend_request, block_friend, cancel_friend_request, get_blocked_list,
    get_friend_message, get_friend_request, get_friends_list, get_sent_requests,
    health, mark_messages_as_read, refuse_friend_request, remove_friend, send_friend_message,
    send_friend_request, unblock_friend,
};
use crate::http::response::json_error;
use crate::types::ServiceRequest;

#[derive(Clone)]
pub struct ServiceContext {
    pub db: Arc<Database>,
    pub redis_pool: deadpool_redis::Pool,
    pub notification_bus: NotificationBus,
}

fn fid_from(req: &ServiceRequest) -> &str {
    req.action.split('/').nth(1).unwrap_or_default()
}

pub fn build_router() -> Router<ServiceContext> {
    let mut router = Router::new();

    router.register("GET", "health", |ctx: ServiceContext, _req: ServiceRequest| async move {
        health::handle_health(&ctx).await
    });

    router.register(
        "POST",
        "friend-requests",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            send_friend_request::handle_request_friend(
                &ctx.db,
                &mut conn,
                &ctx.redis_pool,
                &req,
                &ctx.notification_bus,
            )
            .await
        },
    );

    router.register(
        "GET",
        "friend-requests",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            get_friend_request::handle_get_friend_requests(&ctx.db, &mut conn, &ctx.redis_pool, &req)
                .await
        },
    );

    router.register(
        "GET",
        "friend-requests/sent",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            get_sent_requests::handle_get_sent_requests(&ctx.db, &mut conn, &ctx.redis_pool, &req)
                .await
        },
    );

    router.register(
        "PATCH",
        "friend-requests/*/accept",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let fid = fid_from(&req);

            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            accept_friend_request::handle_accept_friend_request(
                &ctx.db,
                &mut conn,
                &ctx.redis_pool,
                &req,
                fid,
                &ctx.notification_bus,
            )
            .await
        },
    );

    router.register(
        "PATCH",
        "friend-requests/*/refuse",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let fid = fid_from(&req);

            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            refuse_friend_request::handle_refuse_friend_request(
                &ctx.db,
                &mut conn,
                &ctx.redis_pool,
                &req,
                fid,
                &ctx.notification_bus,
            )
            .await
        },
    );

    router.register(
        "DELETE",
        "friend-requests/*",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let fid = fid_from(&req);

            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            cancel_friend_request::handle_cancel_friend_request(
                &ctx.db,
                &mut conn,
                &ctx.redis_pool,
                &req,
                fid,
                &ctx.notification_bus,
            )
            .await
        },
    );

    router.register(
        "GET",
        "friends",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            get_friends_list::handle_get_friends_list(&ctx.db, &mut conn, &ctx.redis_pool, &req)
                .await
        },
    );

    router.register(
        "GET",
        "friends/blocked",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            get_blocked_list::handle_get_blocked_list(&ctx.db, &mut conn, &ctx.redis_pool, &req)
                .await
        },
    );

    router.register(
        "DELETE",
        "friends/*",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let fid = fid_from(&req);

            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            remove_friend::handle_remove_friend(
                &ctx.db,
                &mut conn,
                &ctx.redis_pool,
                &req,
                fid,
                &ctx.notification_bus,
            )
            .await
        },
    );

    router.register(
        "POST",
        "friends/*/block",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let fid = fid_from(&req);

            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            block_friend::handle_block_friend(
                &ctx.db,
                &mut conn,
                &ctx.redis_pool,
                &req,
                fid,
                &ctx.notification_bus,
            )
            .await
        },
    );

    router.register(
        "DELETE",
        "friends/*/block",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let fid = fid_from(&req);

            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            unblock_friend::handle_unblock_friend(&ctx.db, &mut conn, &ctx.redis_pool, &req, fid)
                .await
        },
    );

    router.register(
        "POST",
        "friends/*/messages",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let fid = fid_from(&req);

            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            send_friend_message::handle_send_friend_message(
                &ctx.db,
                &mut conn,
                &ctx.redis_pool,
                &req,
                fid,
                &ctx.notification_bus,
            )
            .await
        },
    );

    router.register(
        "GET",
        "friends/*/messages",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let fid = fid_from(&req);

            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            get_friend_message::handle_get_friend_messages(
                &ctx.db,
                &mut conn,
                &ctx.redis_pool,
                &req,
                fid,
            )
            .await
        },
    );

    router.register(
        "POST",
        "friends/*/messages/read",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            let fid = fid_from(&req);

            let mut conn = match ctx.redis_pool.get().await {
                Ok(c) => c,
                Err(e) => return json_error(500, &format!("Redis connection failed: {}", e)),
            };

            mark_messages_as_read::handle_mark_messages_as_read(
                &ctx.db, &mut conn, &ctx.redis_pool, &req, fid,
            )
            .await
        },
    );

    router
}

use std::sync::Arc;

use api_core::http::router::Router;
use notification::event::NotificationBus;

use crate::db::db::Database;
use crate::http::handlers::{
    abandon_tournament, cancel_ranked, cancel_ranked_tournament, create_room, health, join_room,
    kick_room, leave_room, list_rooms, play_ranked, play_ranked_tournament, queue_size, room_info,
    room_status, start_room, tournament,
};
use crate::http::response::json_error;
use crate::types::ServiceRequest;
use crate::user_state::RedisSessionManager;

#[derive(Clone)]
pub struct ServiceContext {
    pub db: Arc<Database>,
    pub redis_pool: deadpool_redis::Pool,
    pub session_manager: Arc<RedisSessionManager>,
    pub notification_bus: NotificationBus,
}

pub fn build_router() -> Router<ServiceContext> {
    let mut router = Router::new();

    router.register("GET", "health", |ctx: ServiceContext, _req: ServiceRequest| async move {
        health::handle_health(&ctx).await
    });

    router.register("POST", "create_room", |ctx: ServiceContext, req: ServiceRequest| async move {
        create_room::handle_create_room(
            ctx.db.get_pool(),
            &ctx.redis_pool,
            &req,
            ctx.session_manager,
            &ctx.notification_bus,
        )
        .await
    });

    router.register("POST", "play_ranked", |ctx: ServiceContext, req: ServiceRequest| async move {
        play_ranked::handle_play_ranked(
            &ctx.db,
            &ctx.redis_pool,
            &req,
            ctx.session_manager,
            &ctx.notification_bus,
        )
        .await
    });

    router.register(
        "POST",
        "play_ranked_tournament",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            play_ranked_tournament::handle_play_ranked_tournament(
                &ctx.db,
                &ctx.redis_pool,
                &req,
                ctx.session_manager,
                &ctx.notification_bus,
            )
            .await
        },
    );

    router.register(
        "POST",
        "cancel_ranked_tournament",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            cancel_ranked_tournament::handle_cancel_ranked_tournament(
                &ctx.redis_pool,
                &req,
                ctx.session_manager,
                &ctx.notification_bus,
            )
            .await
        },
    );

    router.register(
        "POST",
        "abandon_tournament",
        |ctx: ServiceContext, req: ServiceRequest| async move {
            abandon_tournament::handle_abandon_tournament(
                &ctx.db,
                &ctx.redis_pool,
                &req,
                ctx.session_manager,
                &ctx.notification_bus,
            )
            .await
        },
    );

    router.register("GET", "queue_size", |ctx: ServiceContext, _req: ServiceRequest| async move {
        queue_size::handle_queue_size(&ctx.redis_pool).await
    });

    router.register("POST", "cancel_ranked", |ctx: ServiceContext, req: ServiceRequest| async move {
        cancel_ranked::handle_cancel_ranked(
            &ctx.redis_pool,
            &req,
            ctx.session_manager,
            &ctx.notification_bus,
        )
        .await
    });

    router.register("GET", "status", |ctx: ServiceContext, req: ServiceRequest| async move {
        room_status::handle_room_status(&ctx.redis_pool, &req, ctx.session_manager).await
    });

    router.register("GET", "list_public", |ctx: ServiceContext, _req: ServiceRequest| async move {
        list_rooms::handle_list_rooms(&ctx.redis_pool).await
    });

    router.register("POST", "join_room", |ctx: ServiceContext, req: ServiceRequest| async move {
        join_room::handle_join_room(
            ctx.db.get_pool(),
            &ctx.redis_pool,
            &req,
            ctx.session_manager,
            &ctx.notification_bus,
        )
        .await
    });

    router.register("POST", "leave_room", |ctx: ServiceContext, req: ServiceRequest| async move {
        leave_room::handle_leave_room(
            &ctx.redis_pool,
            &req,
            ctx.session_manager,
            &ctx.notification_bus,
        )
        .await
    });

    router.register("POST", "start_room", |ctx: ServiceContext, req: ServiceRequest| async move {
        start_room::handle_start_room(
            &ctx.redis_pool,
            &req,
            ctx.session_manager,
            &ctx.notification_bus,
        )
        .await
    });

    router.register("POST", "kick_room", |ctx: ServiceContext, req: ServiceRequest| async move {
        kick_room::handle_kick_room(
            &ctx.redis_pool,
            &req,
            ctx.session_manager,
            &ctx.notification_bus,
        )
        .await
    });

    router.register("POST", "room_info", |ctx: ServiceContext, req: ServiceRequest| async move {
        room_info::handle_room_info(&ctx.redis_pool, &req).await
    });

    router.register("GET", "tournament/*", |ctx: ServiceContext, req: ServiceRequest| async move {
        let parts: Vec<&str> = req.action.split('/').collect();
        let sub = parts.get(1).copied().unwrap_or_default();

        match sub {
            "list" => tournament::handle_list(&ctx.redis_pool).await,
            "my" => tournament::handle_my(&ctx.redis_pool, &req).await,
            "status" => {
                if let Some(id) = parts.get(2) {
                    tournament::handle_status(&ctx.redis_pool, id).await
                } else {
                    json_error(400, "Missing tournament id")
                }
            }
            _ => json_error(404, "Unknown tournament route"),
        }
    });

    router
}

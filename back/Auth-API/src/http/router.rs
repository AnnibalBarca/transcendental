use api_core::http::router::Router as ServiceRouter;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::http::handlers::{
    change_password::change_password_handler,
    change_provider_google::switch_provider_to_google_handler,
    delete_user::delete_user_handler,
    finish_account::finish_account_handler,
    forgot_password::forgot_password_handler,
    ft_oauth_42::{ft_callback_handler, ft_login_handler},
    google_code::google_code_handler,
    login_email::login_email_handler,
    logout::logout_handler,
    refresh_token::refresh_token_handler,
    register::register_handler,
    reset_password::reset_password_handler,
    send_validation_email_code::send_validation_email_code,
    send_validation_email_code_switch_provider::send_validation_email_code_switch_provider_handler,
    stats::stats_handler,
    switch_provider_email::switch_provider_to_email_handler,
    validate_email::validate_email_handler,
    validate_token::validate_token_handler,
};
use crate::{cache::redis::RedisCache, db::Database, metrics::app_metrics::AppMetrics};

use crate::config::service::ServiceConfig;

#[derive(Clone)]
pub struct AppState {
    pub metrics: Arc<AppMetrics>,
    pub database: Arc<Database>,
    pub cache: Arc<RedisCache>,
    pub config: Arc<ServiceConfig>,
    pub redis_pool: deadpool_redis::Pool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DependencyStatus {
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub dependencies: std::collections::HashMap<String, DependencyStatus>,
}

pub fn create_auth_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/login/email", post(login_email_handler))
        .route("/google/code", post(google_code_handler))
        .route("/42/login", get(ft_login_handler))
        .route("/42/callback", get(ft_callback_handler))
        .route("/finish_account", post(finish_account_handler))
        .route("/register", post(register_handler))
        .route(
            "/send_validation_email_code",
            post(send_validation_email_code),
        )
        .route("/delete_user", delete(delete_user_handler))
        .route("/validate_email", post(validate_email_handler))
        .route("/forgot_password", post(forgot_password_handler))
        .route("/reset_password", post(reset_password_handler))
        .route("/refresh", get(refresh_token_handler))
        .route("/logout", post(logout_handler))
        .route("/validate", post(validate_token_handler))
        .route("/change_password", post(change_password_handler))
        .route(
            "/change_provider/email/send_validation_code",
            post(send_validation_email_code_switch_provider_handler),
        )
        .route(
            "/change_provider/email/switch",
            post(switch_provider_to_email_handler),
        )
        .route(
            "/change_provider/google/switch",
            post(switch_provider_to_google_handler),
        )
        .route("/stats", get(stats_handler))
        .with_state(state)
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let mut dependencies = std::collections::HashMap::new();
    let mut all_healthy = true;

    // Check Postgres
    let db_status = match sqlx::query("SELECT 1")
        .fetch_one(state.database.get_pool())
        .await
    {
        Ok(_) => "healthy".to_string(),
        Err(_) => {
            all_healthy = false;
            "unhealthy".to_string()
        }
    };
    dependencies.insert(
        "postgres".to_string(),
        DependencyStatus { status: db_status },
    );

    // Check Redis
    let redis_status = match state.redis_pool.get().await {
        Ok(mut conn) => {
            match deadpool_redis::redis::cmd("PING")
                .query_async::<_, String>(&mut conn)
                .await
            {
                Ok(_) => "healthy".to_string(),
                Err(_) => {
                    all_healthy = false;
                    "unhealthy".to_string()
                }
            }
        }
        Err(_) => {
            all_healthy = false;
            "unhealthy".to_string()
        }
    };
    dependencies.insert(
        "redis".to_string(),
        DependencyStatus {
            status: redis_status,
        },
    );

    let overall_status = if all_healthy { "healthy" } else { "unhealthy" };
    let status_code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(HealthResponse {
            status: overall_status.to_string(),
            service: "auth".to_string(),
            dependencies,
        }),
    )
}

pub fn build_redis_router() -> ServiceRouter<AppState> {
    let mut router = ServiceRouter::new();

    // public
    // router.register_public("GET", "health", |_ctx: AppContext, _req: ServiceRequest| {
    //     Box::pin(health::handle_health())
    // });

    // router.register_public("GET", "me", |ctx: AppContext, req: ServiceRequest| {
    //     Box::pin(async move { me::handle_me(&ctx, &req).await })
    // });

    // router.register_public("GET", "users/*", |ctx: AppContext, req: ServiceRequest| {
    //     Box::pin(async move { user::handle_user(&ctx, &req).await })
    // });

    // router.register_public("GET", "state", |ctx: AppContext, req: ServiceRequest| {
    //     Box::pin(async move { state::handle_state(&ctx, &req).await })
    // });

    // router.register_public(
    //     "GET",
    //     "inventory",
    //     |ctx: AppContext, req: ServiceRequest| {
    //         Box::pin(async move { get_inventory::handle_get_inventory(&ctx, &req).await })
    //     },
    // );

    // router.register_public(
    //     "GET",
    //     "profile-picture",
    //     |ctx: AppContext, req: ServiceRequest| {
    //         Box::pin(
    //             async move { get_profile_picture::handle_get_profile_picture(&ctx, &req).await },
    //         )
    //     },
    // );

    // router.register_public(
    //     "PATCH",
    //     "change-email",
    //     |ctx: AppContext, req: ServiceRequest| {
    //         Box::pin(async move { change_email::handle_change_email(&ctx, &req).await })
    //     },
    // );

    // router.register_public(
    //     "PATCH",
    //     "change-username",
    //     |ctx: AppContext, req: ServiceRequest| {
    //         Box::pin(async move { change_username::handle_change_username(&ctx, &req).await })
    //     },
    // );

    // router.register_public(
    //     "POST",
    //     "inventory",
    //     |ctx: AppContext, req: ServiceRequest| {
    //         Box::pin(async move { add_item::handle_add_item(&ctx, &req).await })
    //     },
    // );

    // router.register_public(
    //     "DELETE",
    //     "inventory",
    //     |ctx: AppContext, req: ServiceRequest| {
    //         Box::pin(async move { remove_item::handle_remove_item(&ctx, &req).await })
    //     },
    // );

    // router.register_public(
    //     "POST",
    //     "profile-picture",
    //     |ctx: AppContext, req: ServiceRequest| {
    //         Box::pin(
    //             async move { set_profile_picture::handle_set_profile_picture(&ctx, &req).await },
    //         )
    //     },
    // );

    // internal

    router
}

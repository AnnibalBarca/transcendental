use axum::{routing::get, Router};

use crate::{http::handlers, AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/sse/rooms", get(handlers::sse_rooms))
        .route("/sse/:user_id", get(handlers::sse_connect))
        .with_state(state)
}

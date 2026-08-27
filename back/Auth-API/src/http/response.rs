use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub fn error_response(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorResponse {
            error: message.into(),
        }),
    )
        .into_response()
}

pub fn internal_error_responce() -> Response {
    error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error".to_string(),
    )
}

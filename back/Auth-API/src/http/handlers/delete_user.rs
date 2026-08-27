use crate::auth::utils::get_cookie_value;
use crate::auth::{hash_refresh_token, jwt_manager, utils::get_bearer_token};
use crate::http::router::AppState;
use api_core::types::ServiceRequest;
use axum::{
    extract::State,
    http::{
        header::{HeaderMap, HeaderValue, SET_COOKIE},
        StatusCode,
    },
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

#[derive(Serialize)]
pub struct UserDataResponse {
    pub id: String,
    pub username: Option<String>,
    pub email: String,
    pub account_validated: bool,
}

#[derive(Serialize)]
pub struct DeleteUserResponse {
    pub message: String,
}

fn clear_auth_cookies(headers: &mut HeaderMap) {
    let cookies = [
        "access_token=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0",
        "refresh_token=; HttpOnly; SameSite=Strict; Path=/api/auth/refresh; Max-Age=0",
    ];

    for cookie in cookies {
        if let Ok(v) = HeaderValue::from_str(cookie) {
            headers.append(SET_COOKIE, v);
        }
    }
}

fn delete_user_error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

pub async fn delete_user_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = match get_bearer_token(&headers)
        .or_else(|| get_cookie_value(&headers, "access_token"))
    {
        Some(t) => t,
        None => {
            return delete_user_error(StatusCode::UNAUTHORIZED, "Missing or invalid access token");
        }
    };

    let claims = match jwt_manager().validate_token(&token) {
        Ok(claims) => claims,
        Err(e) => {
            error!("[Auth] Failed to validate token: {}", e);
            return delete_user_error(StatusCode::UNAUTHORIZED, "Invalid or expired access token");
        }
    };

    info!("send delete message");

    match state.database.delete_user(&claims.sub).await {
        Ok(true) => {
            let custom_req = ServiceRequest {
                id: Uuid::new_v4().to_string(),
                method: "POST".to_string(),
                action: "/users/cleanup".to_string(),
                cookies: Default::default(),
                body: json!({ "user_id": claims.sub }).to_string(),
                headers: Default::default(),
                internal: true,
            };

            if let Err(e) = push_to_queue(&state.redis_pool, "user:requests", &custom_req).await {
                error!(
                    "[Auth] Failed to push cleanup request for user {}: {}",
                    claims.sub, e
                );
            }
        }
        Ok(false) => {
            error!("[Auth] User not found: {}", claims.sub);
            return delete_user_error(StatusCode::NOT_FOUND, "User not found");
        }
        Err(e) => {
            error!("[Auth] Failed to delete user: {}", e);
            return delete_user_error(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error");
        }
    };

    let response_body = DeleteUserResponse {
        message: "Account deleted successfully".to_string(),
    };
    let mut response = (StatusCode::OK, Json(response_body)).into_response();
    clear_auth_cookies(response.headers_mut());
    response
}

pub async fn push_to_queue(
    redis_pool: &deadpool_redis::Pool,
    channel: &str,
    service_request: &ServiceRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = redis_pool.get().await?;
    let payload = serde_json::to_string(service_request)?;

    let _: String = redis::cmd("XADD")
        .arg(channel)
        .arg("*")
        .arg("data")
        .arg(&payload)
        .query_async(&mut *conn)
        .await?;

    Ok(())
}

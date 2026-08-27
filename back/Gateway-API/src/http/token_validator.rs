use crate::http::handlers::ErrorResponse;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub username: String,
    pub email: String,
    pub token_type: String,
}

pub async fn validate_token_for_service(
    service: &str,
    path: &str,
    cookies: &HashMap<String, String>,
    redis_pool: &deadpool_redis::Pool,
) -> Result<(), Response> {    if service == "auth" {
        return Ok(());
    }

    let token = cookies.get("access_token").ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Missing access token".to_string(),
                status: 401,
            }),
        )
            .into_response()
    })?;

    if is_token_blacklisted(redis_pool, token).await.map_err(|e| {
        log::error!("[Gateway] Error checking token blacklist: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Internal server error".to_string(),
                status: 500,
            }),
        )
            .into_response()
    })? {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Token has been revoked".to_string(),
                status: 401,
            }),
        )
            .into_response());
    }

    let claims = decode_and_validate_token(redis_pool, token).await?;

    if is_account_validation_required(service, path) && claims.username.is_empty() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Account not validated. Please complete your profile.",
                "status": 403,
                "account_validated": false
            })),
        )
            .into_response());
    }

    Ok(())
}

pub async fn extract_user_sub(
    redis_pool: &deadpool_redis::Pool,
    access_token: Option<&str>,
) -> Option<String> {
    let token = access_token?;
    decode_and_validate_token(redis_pool, token).await.ok().map(|c| c.sub)
}

fn is_account_validation_required(service: &str, path: &str) -> bool {
    if service == "user" {
        if path == "me"
            || path.starts_with("me/")
            || path == "finish_account"
            || path.starts_with("finish_account/")
        {
            return false;
        }
        return true;
    }
    service == "room"
}

async fn decode_and_validate_token(
    redis_pool: &deadpool_redis::Pool,
    token: &str,
) -> Result<TokenClaims, Response> {
    let public_pem = get_jwt_public_key(redis_pool).await.map_err(|e| {
        log::error!("[Gateway] Failed to get JWT public key: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Internal server error".to_string(),
                status: 500,
            }),
        )
            .into_response()
    })?;

    let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes()).map_err(|e| {
        log::error!("[Gateway] Failed to load RSA key: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Internal server error".to_string(),
                status: 500,
            }),
        )
            .into_response()
    })?;

    let claims = decode::<TokenClaims>(
        token,
        &decoding_key,
        &Validation::new(jsonwebtoken::Algorithm::RS256),
    )
    .map_err(|e| {
        log::warn!("[Gateway] Invalid or expired token: {}", e);
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid or expired token".to_string(),
                status: 401,
            }),
        )
            .into_response()
    })?
    .claims;

    Ok(claims)
}

async fn get_jwt_public_key(redis_pool: &deadpool_redis::Pool) -> Result<String, String> {
    let mut conn = redis_pool
        .get()
        .await
        .map_err(|e| format!("Redis connection error: {}", e))?;

    let public_pem: Option<String> = redis::cmd("GET")
        .arg("auth:jwt:public_pem")
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis error: {}", e))?;

    public_pem.ok_or_else(|| "JWT public key not found".to_string())
}

async fn is_token_blacklisted(
    redis_pool: &deadpool_redis::Pool,
    token: &str,
) -> Result<bool, String> {
    let key = format!("blacklist:{}", token);
    let mut conn = redis_pool
        .get()
        .await
        .map_err(|e| format!("Redis connection error: {}", e))?;

    let exists: bool = redis::cmd("EXISTS")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis error: {}", e))?;

    Ok(exists)
}

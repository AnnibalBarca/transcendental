use api_core::auth::jwt_manager;
use serde_json::json;
use uuid::Uuid;

pub fn extract_user_id_from_token_sub(sub: &str) -> Result<Uuid, serde_json::Value> {
    Uuid::parse_str(sub).map_err(|_| json!({"status": 400, "error": "Invalid user id in token"}))
}

pub fn parse_user_id(id_str: &str) -> Result<Uuid, serde_json::Value> {
    Uuid::parse_str(id_str).map_err(|_| json!({"status": 400, "error": "Invalid user id"}))
}

pub fn extract_user_id_from_access_token(token: &str) -> Result<Uuid, serde_json::Value> {
    let claims = jwt_manager()
        .decode_token_claims(token)
        .map_err(|_| json!({"status": 401, "error": "Invalid or expired token"}))?;

    extract_user_id_from_token_sub(&claims.sub)
}

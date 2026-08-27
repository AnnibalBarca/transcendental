use chrono::Utc;
use deadpool_redis::Connection;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

const REDIS_JWT_PUBLIC_KEY: &str = "auth:jwt:public_pem";

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub username: String,
    pub email: String,
    pub token_type: String,
}

pub async fn validate_and_get_claims(
    conn: &mut Connection,
    token: &str,
) -> Result<TokenClaims, String> {
    let public_pem: Option<String> = redis::cmd("GET")
        .arg(REDIS_JWT_PUBLIC_KEY)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    let public_pem = public_pem.ok_or_else(|| "JWT public key missing".to_string())?;
    let public_pem = normalize_pem(&public_pem);

    let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes())
        .map_err(|e| format!("Invalid public key: {}", e))?;

    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.leeway = 60;

    let data = decode::<TokenClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("Token decode failed: {}", e))?;

    if data.claims.exp < Utc::now().timestamp() {
        return Err("Token expired".to_string());
    }

    if data.claims.token_type != "access" {
        return Err("Invalid token type".to_string());
    }

    Ok(data.claims)
}

pub async fn validate_access_token(
    conn: &mut Connection,
    token: &str,
) -> Result<TokenClaims, String> {
    let public_pem: Option<String> = redis::cmd("GET")
        .arg(REDIS_JWT_PUBLIC_KEY)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    let public_pem = public_pem.ok_or_else(|| "JWT public key missing".to_string())?;
    let public_pem = normalize_pem(&public_pem);

    let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes())
        .map_err(|e| format!("Invalid public key: {}", e))?;

    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.leeway = 60;

    let data = decode::<TokenClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("Token decode failed: {}", e))?;

    if data.claims.exp < Utc::now().timestamp() {
        return Err("Token expired".to_string());
    }

    Ok(data.claims)
}

fn normalize_pem(pem: &str) -> String {
    pem.replace("\\n", "\n")
}

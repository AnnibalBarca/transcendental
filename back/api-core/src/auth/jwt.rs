use chrono::{Duration, Utc};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const REDIS_JWT_PRIVATE_KEY: &str = "auth:jwt:private_pem";
const REDIS_JWT_PUBLIC_KEY: &str = "auth:jwt:public_pem";
const REDIS_JWT_READY_FLAG: &str = "auth:jwt:ready";

#[derive(Error, Debug, Clone)]
pub enum JwtError {
    #[error("Failed to initialize JWT: {0}")]
    InitError(String),

    #[error("JWT encode error: {0}")]
    EncodeError(String),

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token type")]
    InvalidTokenType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub username: String,
    pub email: String,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_in: i64,
    pub refresh_token_expires_in: i64,
    pub token_type: String,
}

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtManager {
    #[allow(dead_code)]
    pub fn init() -> Result<Self, JwtError> {
        if let (Ok(private_key_str), Ok(public_key_str)) = (
            std::env::var("JWT_PRIVATE_KEY_PEM"),
            std::env::var("JWT_PUBLIC_KEY_PEM"),
        ) {
            println!("[JWT] Loading keys from environment");
            let private_pem = normalize_pem(&private_key_str);
            let public_pem = normalize_pem(&public_key_str);
            Self::from_pem_strings(&private_pem, &public_pem)
        } else {
            println!("[JWT] Generating new RSA key pair for JWT signing...");
            let (manager, _, _) = Self::generate_keys()?;
            Ok(manager)
        }
    }

    pub fn from_pem_strings(private_pem: &str, public_pem: &str) -> Result<Self, JwtError> {
        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes())
            .map_err(|e| JwtError::InitError(format!("Failed to load private key: {}", e)))?;
        let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes())
            .map_err(|e| JwtError::InitError(format!("Failed to load public key: {}", e)))?;

        Ok(Self {
            encoding_key,
            decoding_key,
        })
    }

    fn generate_keys() -> Result<(Self, String, String), JwtError> {
        use rsa::pkcs1::EncodeRsaPrivateKey;
        use rsa::{RsaPrivateKey, RsaPublicKey};

        let mut rng = rand::thread_rng();
        let bits = 2048;
        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| JwtError::InitError(format!("RSA key generation failed: {}", e)))?;

        let public_key = RsaPublicKey::from(&private_key);

        let private_pem = private_key
            .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
            .map_err(|e| JwtError::InitError(format!("Private key PEM encoding failed: {}", e)))?
            .to_string();

        use rsa::pkcs1::EncodeRsaPublicKey;
        let public_pem = public_key
            .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
            .map_err(|e| JwtError::InitError(format!("Public key PEM encoding failed: {}", e)))?;

        let manager = Self::from_pem_strings(&private_pem, &public_pem)?;

        println!("[JWT] Generated new RSA key pair.");

        Ok((manager, private_pem, public_pem))
    }

    pub fn generate_access_token(
        &self,
        user_id: &str,
        username: &str,
        email: &str,
    ) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now + Duration::minutes(15);

        let claims = TokenClaims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            username: username.to_string(),
            email: email.to_string(),
            token_type: "access".to_string(),
        };

        encode(
            &Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &self.encoding_key,
        )
        .map_err(|e| JwtError::EncodeError(e.to_string()))
    }

    pub fn generate_refresh_token(
        &self,
        user_id: &str,
        username: &str,
        email: &str,
    ) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now + Duration::days(7);

        let claims = TokenClaims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            username: username.to_string(),
            email: email.to_string(),
            token_type: "refresh".to_string(),
        };

        encode(
            &Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &self.encoding_key,
        )
        .map_err(|e| JwtError::EncodeError(e.to_string()))
    }

    pub fn generate_token_pair(
        &self,
        user_id: &str,
        username: &str,
        email: &str,
    ) -> Result<TokenResponse, JwtError> {
        let access_token = self.generate_access_token(user_id, username, email)?;
        let refresh_token = self.generate_refresh_token(user_id, username, email)?;

        Ok(TokenResponse {
            access_token,
            refresh_token,
            access_token_expires_in: 15 * 60,
            refresh_token_expires_in: 7 * 24 * 60 * 60,
            token_type: "Bearer".to_string(),
        })
    }

    pub fn decode_token_claims(&self, token: &str) -> Result<TokenClaims, JwtError> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.leeway = 60;

        decode::<TokenClaims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                if e.kind() == &ErrorKind::ExpiredSignature {
                    JwtError::TokenExpired
                } else {
                    JwtError::InvalidToken
                }
            })
    }

    pub fn decode_and_validate_access(&self, token: &str) -> Result<TokenClaims, JwtError> {
        let claims = self.decode_token_claims(token)?;

        if claims.token_type != "access" {
            return Err(JwtError::InvalidTokenType);
        }

        Ok(claims)
    }

    pub fn validate_token(&self, token: &str) -> Result<TokenClaims, JwtError> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.leeway = 60;

        decode::<TokenClaims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                if e.kind() == &ErrorKind::ExpiredSignature {
                    JwtError::TokenExpired
                } else if e.kind() == &ErrorKind::InvalidSignature {
                    JwtError::InvalidToken
                } else {
                    JwtError::InvalidToken
                }
            })
    }

    pub fn validate_access_token(&self, token: &str) -> Result<TokenClaims, JwtError> {
        let claims = self.validate_token(token)?;

        if claims.token_type != "access" {
            return Err(JwtError::InvalidTokenType);
        }

        Ok(claims)
    }
}

pub static JWT_MANAGER: OnceCell<JwtManager> = OnceCell::new();

pub fn jwt_manager() -> &'static JwtManager {
    JWT_MANAGER
        .get()
        .expect("JWT manager not initialized. Call init_jwt_manager first.")
}

pub async fn init_jwt_discover(pool: &deadpool_redis::Pool) -> Result<(), JwtError> {
    use deadpool_redis::redis::cmd;
    use tokio::time::{Duration, sleep};

    let mut conn = pool
        .get()
        .await
        .map_err(|e| JwtError::InitError(format!("Redis pool error: {}", e)))?;

    println!("[JWT] Waiting for JWT keys to be ready...");

    let mut attempts = 0;
    let max_attempts = 30;

    loop {
        let ready_flag: Option<String> = cmd("GET")
            .arg(REDIS_JWT_READY_FLAG)
            .query_async(&mut *conn)
            .await
            .map_err(|e| JwtError::InitError(format!("Redis GET ready flag failed: {}", e)))?;

        if ready_flag.is_some() {
            break;
        }

        attempts += 1;
        if attempts >= max_attempts {
            return Err(JwtError::InitError(
                "JWT Discovery failed: Ready flag not set after 30 seconds. Shutting down."
                    .to_string(),
            ));
        }

        println!(
            "[JWT] Waiting for JWT keys to be ready... ({}/{})",
            attempts, max_attempts
        );
        sleep(Duration::from_secs(1)).await;
    }

    println!("[JWT] JWT keys are ready, fetching from Redis...");

    let private_pem: Option<String> = cmd("GET")
        .arg(REDIS_JWT_PRIVATE_KEY)
        .query_async(&mut *conn)
        .await
        .map_err(|e| JwtError::InitError(format!("Redis GET private key failed: {}", e)))?;

    let public_pem: Option<String> = cmd("GET")
        .arg(REDIS_JWT_PUBLIC_KEY)
        .query_async(&mut *conn)
        .await
        .map_err(|e| JwtError::InitError(format!("Redis GET public key failed: {}", e)))?;

    let private_pem =
        private_pem.ok_or_else(|| JwtError::InitError("JWT private key missing".to_string()))?;
    let public_pem =
        public_pem.ok_or_else(|| JwtError::InitError("JWT public key missing".to_string()))?;

    let manager = JwtManager::from_pem_strings(&private_pem, &public_pem)?;

    JWT_MANAGER
        .set(manager)
        .map_err(|_| JwtError::InitError("JWT manager already initialized".to_string()))?;

    println!("[JWT] Successfully loaded RSA keys from Redis.");
    Ok(())
}

pub async fn init_jwt_manager(pool: &deadpool_redis::Pool) -> Result<(), JwtError> {
    use deadpool_redis::redis::cmd;

    let mut conn = pool
        .get()
        .await
        .map_err(|e| JwtError::InitError(format!("Redis pool error: {}", e)))?;

    let ready_flag: Option<String> = cmd("GET")
        .arg(REDIS_JWT_READY_FLAG)
        .query_async(&mut *conn)
        .await
        .map_err(|e| JwtError::InitError(format!("Redis GET ready flag failed: {}", e)))?;

    if ready_flag.is_some() {
        println!("[JWT] Existing RSA keys found in Redis, reusing them (no rotation)");
        let private_pem: Option<String> = cmd("GET")
            .arg(REDIS_JWT_PRIVATE_KEY)
            .query_async(&mut *conn)
            .await
            .map_err(|e| JwtError::InitError(format!("Redis GET private key failed: {}", e)))?;
        let public_pem: Option<String> = cmd("GET")
            .arg(REDIS_JWT_PUBLIC_KEY)
            .query_async(&mut *conn)
            .await
            .map_err(|e| JwtError::InitError(format!("Redis GET public key failed: {}", e)))?;

        if let (Some(private_pem), Some(public_pem)) = (private_pem, public_pem) {
            let manager = JwtManager::from_pem_strings(&private_pem, &public_pem)?;
            JWT_MANAGER
                .set(manager)
                .map_err(|_| JwtError::InitError("JWT manager already initialized".to_string()))?;
            println!("[JWT] Successfully reused RSA keys from Redis.");
            return Ok(());
        }
    }

    println!("[JWT] Rotating RSA keys on startup (forced)");
    let (manager, private_pem, public_pem) = JwtManager::generate_keys()?;

    cmd("SET")
        .arg(REDIS_JWT_PRIVATE_KEY)
        .arg(&private_pem)
        .query_async::<_, ()>(&mut *conn)
        .await
        .map_err(|e| JwtError::InitError(format!("Redis SET failed: {}", e)))?;

    cmd("SET")
        .arg(REDIS_JWT_PUBLIC_KEY)
        .arg(&public_pem)
        .query_async::<_, ()>(&mut *conn)
        .await
        .map_err(|e| JwtError::InitError(format!("Redis SET failed: {}", e)))?;

    cmd("SET")
        .arg(REDIS_JWT_READY_FLAG)
        .arg("1")
        .query_async::<_, ()>(&mut *conn)
        .await
        .map_err(|e| JwtError::InitError(format!("Redis SET ready flag failed: {}", e)))?;

    println!("[JWT] RSA keys published to Redis with ready flag");

    JWT_MANAGER
        .set(manager)
        .map_err(|_| JwtError::InitError("JWT manager already initialized".to_string()))?;

    Ok(())
}

fn normalize_pem(pem: &str) -> String {
    pem.replace("\\n", "\n")
}

pub fn hash_refresh_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let manager = JwtManager::init().expect("Failed to init JWT manager");
        let token_pair = manager
            .generate_token_pair("user123", "alice", "alice@example.com")
            .expect("Failed to generate token pair");

        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert_eq!(token_pair.access_token_expires_in, 15 * 60);
        assert_eq!(token_pair.refresh_token_expires_in, 7 * 24 * 60 * 60);
    }

    #[test]
    fn test_token_validation() {
        let manager = JwtManager::init().expect("Failed to init JWT manager");
        let token_pair = manager
            .generate_token_pair("user123", "alice", "alice@example.com")
            .expect("Failed to generate token pair");

        let claims = manager
            .validate_access_token(&token_pair.access_token)
            .expect("Failed to validate access token");

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, "alice");
        assert_eq!(claims.email, "alice@example.com");
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_refresh_token_hash() {
        let token = "test_refresh_token_12345";
        let hash = hash_refresh_token(token);

        assert!(!hash.is_empty());
    }
}

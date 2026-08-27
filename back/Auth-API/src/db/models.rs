use api_core::db::Database as CoreDatabase;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: Option<String>,
    pub email: String,
    pub password_hash: String,
    pub account_validated: bool,
    pub email_validated: bool,
    pub auth_provider: String,
    #[serde(default)]
    pub is_banned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub ranked_elo: i32,
    pub picture_id: Option<String>,
    pub picture_updated_at: Option<DateTime<Utc>>,
}

pub struct Database {
    core_db: CoreDatabase,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, String> {
        let migrations = super::migrations::get_migrations();
        let core_db = CoreDatabase::new(database_url, &migrations).await?;

        println!("[DB] Database initialized with migrations");

        Ok(Self { core_db })
    }

    pub fn get_pool(&self) -> &PgPool {
        self.core_db.get_pool()
    }

    async fn grant_default_inventory(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: uuid::Uuid,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO player_inventory (user_id, item_id, item_type, item_rarity)
            VALUES ($1, '99', 'base', '0')
            ON CONFLICT (user_id, item_id, item_type) DO NOTHING
            "#,
        )
        .bind(user_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn create_user(&self, email: &str, password: &str) -> Result<User, String> {
        let password_hash = bcrypt::hash(password, 12).map_err(|e| e.to_string())?;

        let user_id = uuid::Uuid::new_v4();
        let now = Utc::now();

        let mut tx = self.get_pool().begin().await.map_err(|e| e.to_string())?;

        let row = sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, account_validated, email_validated, auth_provider, created_at, updated_at)
            VALUES ($1, $2, $3, FALSE, FALSE, $4, $5, $6)
            RETURNING id, username, email, password_hash, account_validated, email_validated, is_banned, auth_provider, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(email)
        .bind(&password_hash)
        .bind("email")
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            INSERT INTO user_profile (user_id, ranked_elo, picture_id)
            VALUES ($1, 1500, '1B')
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        self.grant_default_inventory(&mut tx, user_id).await?;

        tx.commit().await.map_err(|e| e.to_string())?;

        let user = User {
            id: row.get::<uuid::Uuid, _>("id").to_string(),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            account_validated: row.get("account_validated"),
            email_validated: row.get("email_validated"),
            is_banned: row.get("is_banned"),
            auth_provider: row.get("auth_provider"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        println!("[DB] User created: {} ({})", email, user.id);

        Ok(user)
    }

    pub async fn create_user_from_google(&self, id: &str, email: &str) -> Result<User, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;
        let now = Utc::now();
        let placeholder_hash = "GOOGLE_OAUTH_NO_PASSWORD";

        let mut tx = self.get_pool().begin().await.map_err(|e| e.to_string())?;

        let row = sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, account_validated, email_validated, auth_provider, created_at, updated_at)
            VALUES ($1, $2, $3, FALSE, TRUE, $4, $5, $6)
            RETURNING id, username, email, password_hash, account_validated, email_validated, is_banned, auth_provider, created_at, updated_at
            "#,
        )
        .bind(user_uuid)
        .bind(email)
        .bind(placeholder_hash)
        .bind("google")
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            INSERT INTO user_profile (user_id, ranked_elo, picture_id)
            VALUES ($1, 1500, '1B')
            "#,
        )
        .bind(user_uuid)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        self.grant_default_inventory(&mut tx, user_uuid).await?;

        tx.commit().await.map_err(|e| e.to_string())?;

        let user = User {
            id: row.get::<uuid::Uuid, _>("id").to_string(),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            account_validated: row.get("account_validated"),
            email_validated: row.get("email_validated"),
            is_banned: row.get("is_banned"),
            auth_provider: row.get("auth_provider"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        println!("[DB] Google user created: {} ({})", email, user.id);

        Ok(user)
    }

    pub async fn create_user_from_42(&self, id: &str, email: &str) -> Result<User, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;
        let now = Utc::now();
        let placeholder_hash = "FT_OAUTH_NO_PASSWORD";

        let mut tx = self.get_pool().begin().await.map_err(|e| e.to_string())?;

        let row = sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, account_validated, email_validated, auth_provider, created_at, updated_at)
            VALUES ($1, $2, $3, FALSE, TRUE, $4, $5, $6)
            RETURNING id, username, email, password_hash, account_validated, email_validated, is_banned, auth_provider, created_at, updated_at
            "#,
        )
        .bind(user_uuid)
        .bind(email)
        .bind(placeholder_hash)
        .bind("42")
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            INSERT INTO user_profile (user_id, ranked_elo, picture_id)
            VALUES ($1, 1500, '1B')
            "#,
        )
        .bind(user_uuid)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        self.grant_default_inventory(&mut tx, user_uuid).await?;

        tx.commit().await.map_err(|e| e.to_string())?;

        let user = User {
            id: row.get::<uuid::Uuid, _>("id").to_string(),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            account_validated: row.get("account_validated"),
            email_validated: row.get("email_validated"),
            is_banned: row.get("is_banned"),
            auth_provider: row.get("auth_provider"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        println!("[DB] 42 user created: {} ({})", email, user.id);

        Ok(user)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, account_validated, email_validated, is_banned, auth_provider, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(row.map(|r| User {
            id: r.get::<uuid::Uuid, _>("id").to_string(),
            username: r.get("username"),
            email: r.get("email"),
            password_hash: r.get("password_hash"),
            account_validated: r.get("account_validated"),
            email_validated: r.get("email_validated"),
            is_banned: r.get("is_banned"),
            auth_provider: r.get("auth_provider"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<Option<User>, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;

        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, account_validated, email_validated, is_banned, auth_provider, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_uuid)
        .fetch_optional(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(row.map(|r| User {
            id: r.get::<uuid::Uuid, _>("id").to_string(),
            username: r.get("username"),
            email: r.get("email"),
            password_hash: r.get("password_hash"),
            account_validated: r.get("account_validated"),
            email_validated: r.get("email_validated"),
            is_banned: r.get("is_banned"),
            auth_provider: r.get("auth_provider"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, account_validated, email_validated, is_banned, auth_provider, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(row.map(|r| User {
            id: r.get::<uuid::Uuid, _>("id").to_string(),
            username: r.get("username"),
            email: r.get("email"),
            password_hash: r.get("password_hash"),
            account_validated: r.get("account_validated"),
            email_validated: r.get("email_validated"),
            is_banned: r.get("is_banned"),
            auth_provider: r.get("auth_provider"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, account_validated, email_validated, is_banned, auth_provider, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows
            .into_iter()
            .map(|r| User {
                id: r.get::<uuid::Uuid, _>("id").to_string(),
                username: r.get("username"),
                email: r.get("email"),
                password_hash: r.get("password_hash"),
                account_validated: r.get("account_validated"),
                email_validated: r.get("email_validated"),
                is_banned: r.get("is_banned"),
                auth_provider: r.get("auth_provider"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    pub async fn change_provider_to_email(
        &self,
        id: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<bool, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;

        let result = sqlx::query(
            r#"
                UPDATE users
                SET email = $1,
                    password_hash = $2,
                    auth_provider = 'email',
                    email_validated = TRUE,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = $3
                "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(user_uuid)
        .execute(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn change_provider_to_google(&self, id: &str, email: &str) -> Result<bool, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;
        let placeholder_hash = "GOOGLE_OAUTH_NO_PASSWORD";

        let result = sqlx::query(
            r#"
                UPDATE users
                SET email = $1,
                    password_hash = $2,
                    auth_provider = 'google',
                    email_validated = TRUE,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = $3
                "#,
        )
        .bind(email)
        .bind(placeholder_hash)
        .bind(user_uuid)
        .execute(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn change_provider_to_42(&self, id: &str, email: &str) -> Result<bool, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;
        let placeholder_hash = "FT_OAUTH_NO_PASSWORD";

        let result = sqlx::query(
            r#"
                    UPDATE users
                    SET email = $1,
                        password_hash = $2,
                        auth_provider = '42',
                        email_validated = TRUE,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE id = $3
                    "#,
        )
        .bind(email)
        .bind(placeholder_hash)
        .bind(user_uuid)
        .execute(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn verify_password(&self, username: &str, password: &str) -> Result<bool, String> {
        match self.get_user_by_username(username).await? {
            Some(user) => {
                let is_valid =
                    bcrypt::verify(password, &user.password_hash).map_err(|e| e.to_string())?;
                Ok(is_valid)
            }
            None => Ok(false),
        }
    }

    pub async fn update_email(&self, id: &str, email: &str) -> Result<bool, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;

        let result = sqlx::query(
            r#"
            UPDATE users
            SET email = $1, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(email)
        .bind(user_uuid)
        .execute(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn change_password(&self, id: &str, new_password: &str) -> Result<bool, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;
        let password_hash = bcrypt::hash(new_password, 12).map_err(|e| e.to_string())?;

        let result = sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $1, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(&password_hash)
        .bind(user_uuid)
        .execute(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_username(&self, id: &str, username: &str) -> Result<bool, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;

        let result = sqlx::query(
            r#"
            UPDATE users
            SET username = $1, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(username)
        .bind(user_uuid)
        .execute(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn validate_email(&self, id: &str) -> Result<bool, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;

        let result = sqlx::query(
            r#"
            UPDATE users
            SET email_validated = TRUE, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(user_uuid)
        .execute(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn validate_account(&self, id: &str) -> Result<bool, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;

        let result = sqlx::query(
            r#"
            UPDATE users
            SET account_validated = TRUE, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(user_uuid)
        .execute(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn set_username_and_validate(
        &self,
        id: &str,
        username: &str,
    ) -> Result<bool, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;

        let result = sqlx::query(
            r#"
            UPDATE users
            SET username = $1, account_validated = TRUE, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(username)
        .bind(user_uuid)
        .execute(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_user(&self, id: &str) -> Result<bool, String> {
        let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;

        let result_1 = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_uuid)
            .execute(self.get_pool())
            .await
            .map_err(|e| e.to_string())?;

        let result_2 = sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
            .bind(user_uuid)
            .execute(self.get_pool())
            .await
            .map_err(|e| e.to_string())?;

        let result_3 = sqlx::query("DELETE FROM user_profile WHERE user_id = $1")
            .bind(user_uuid)
            .execute(self.get_pool())
            .await
            .map_err(|e| e.to_string())?;

        Ok(result_1.rows_affected() + result_2.rows_affected() + result_3.rows_affected() > 0)
    }

    pub async fn username_exists(&self, username: &str) -> Result<bool, String> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.0 > 0)
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool, String> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.0 > 0)
    }

    pub async fn user_count(&self) -> Result<i64, String> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.0)
    }

    pub async fn online_user_count(&self) -> Result<i64, String> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM refresh_tokens WHERE revoked = FALSE AND expires_at > CURRENT_TIMESTAMP",
        )
        .fetch_one(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.0)
    }

    pub async fn game_count(&self) -> Result<i64, String> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM games")
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.0)
    }

    pub async fn store_refresh_token(
        &self,
        user_id: &uuid::Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<RefreshToken, String> {
        let id = uuid::Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at, revoked)
            VALUES ($1, $2, $3, $4, $5, FALSE)
            RETURNING id, user_id, token_hash, expires_at, created_at, revoked
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .bind(now)
        .fetch_one(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(RefreshToken {
            id: row.get::<uuid::Uuid, _>("id").to_string(),
            user_id: row.get::<uuid::Uuid, _>("user_id").to_string(),
            token_hash: row.get("token_hash"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
            revoked: row.get("revoked"),
        })
    }

    pub async fn get_refresh_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, token_hash, expires_at, created_at, revoked
            FROM refresh_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(row.map(|r| RefreshToken {
            id: r.get::<uuid::Uuid, _>("id").to_string(),
            user_id: r.get::<uuid::Uuid, _>("user_id").to_string(),
            token_hash: r.get("token_hash"),
            expires_at: r.get("expires_at"),
            created_at: r.get("created_at"),
            revoked: r.get("revoked"),
        }))
    }

    pub async fn get_user_refresh_tokens(
        &self,
        user_id: &str,
    ) -> Result<Vec<RefreshToken>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, token_hash, expires_at, created_at, revoked
            FROM refresh_tokens
            WHERE user_id = $1 AND revoked = FALSE AND expires_at > CURRENT_TIMESTAMP
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows
            .into_iter()
            .map(|r| RefreshToken {
                id: r.get::<uuid::Uuid, _>("id").to_string(),
                user_id: r.get::<uuid::Uuid, _>("user_id").to_string(),
                token_hash: r.get("token_hash"),
                expires_at: r.get("expires_at"),
                created_at: r.get("created_at"),
                revoked: r.get("revoked"),
            })
            .collect())
    }

    pub async fn revoke_refresh_token(&self, token_hash: &str) -> Result<bool, String> {
        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = TRUE
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .execute(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn revoke_all_user_tokens(&self, user_id: &str) -> Result<u64, String> {
        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = TRUE
            WHERE user_id = $1 AND revoked = FALSE
            "#,
        )
        .bind(user_id)
        .execute(self.get_pool())
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected())
    }

    pub async fn cleanup_expired_tokens(&self) -> Result<u64, String> {
        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at < CURRENT_TIMESTAMP OR (revoked = TRUE AND expires_at < CURRENT_TIMESTAMP - INTERVAL '7 days')
            "#,
        )
        .execute(self.get_pool())
        .await.map_err(|e| e.to_string())?;

        Ok(result.rows_affected())
    }

    pub async fn is_refresh_token_valid(&self, token_hash: &str) -> Result<bool, String> {
        match self.get_refresh_token(token_hash).await? {
            Some(token) => Ok(!token.revoked && token.expires_at > Utc::now()),
            None => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_create_user() {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let db = Database::new(&db_url).await.unwrap();

        let user = db
            .create_user("alice@example.com", "password123")
            .await
            .unwrap();

        assert_eq!(user.username, None);
        assert_eq!(user.email, "alice@example.com");
        assert!(!user.account_validated);
    }

    #[tokio::test]
    #[ignore]
    async fn test_verify_password() {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let db = Database::new(&db_url).await.unwrap();

        db.create_user("bob@example.com", "secret123")
            .await
            .unwrap();

        let is_valid = db
            .verify_password("bob@example.com", "secret123")
            .await
            .unwrap();
        assert!(is_valid);

        let is_invalid = db
            .verify_password("bob@example.com", "wrongpassword")
            .await
            .unwrap();
        assert!(!is_invalid);
    }
}

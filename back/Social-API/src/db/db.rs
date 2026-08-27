use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub struct Database {
    pool: PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    pub email: String,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, String> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}

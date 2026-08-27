pub mod migration;

use migration::Migration;
use sqlx::PgPool;

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str, migrations: &[Migration]) -> Result<Self, String> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| e.to_string())?;

        Self::ensure_migration_table(&pool).await?;
        Self::run_migrations(&pool, migrations).await?;

        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }

    async fn ensure_migration_table(pool: &PgPool) -> Result<(), String> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _migrations (
                name VARCHAR(255) PRIMARY KEY,
                applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn run_migrations(pool: &PgPool, migrations: &[Migration]) -> Result<(), String> {
        for migration in migrations {
            let already_applied: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _migrations WHERE name = $1)")
                    .bind(migration.name)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

            if already_applied {
                continue;
            }

            sqlx::raw_sql(migration.sql)
                .execute(pool)
                .await
                .map_err(|e| format!("Migration {} failed: {}", migration.name, e))?;

            sqlx::query("INSERT INTO _migrations (name) VALUES ($1)")
                .bind(migration.name)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;

            log::info!("Applied migration: {}", migration.name);
        }

        Ok(())
    }
}

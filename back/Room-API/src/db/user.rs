use crate::db::db::UserRecord;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

pub async fn get_by_id(db_pool: &Pool<Postgres>, id: &Uuid) -> Result<Option<UserRecord>, String> {
    let row = sqlx::query(
        r#"
        SELECT id, username, email
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|r| UserRecord {
        id: r.get::<Uuid, _>("id").to_string(),
        username: r.get("username"),
        email: r.get("email"),
    }))
}

pub async fn update_email(
    db_pool: &Pool<Postgres>,
    id: &Uuid,
    new_email: &str,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE users
        SET email = $1
        WHERE id = $2
        "#,
    )
    .bind(new_email)
    .bind(id)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn email_exists(db_pool: &Pool<Postgres>, email: &str) -> Result<bool, String> {
    let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
        .bind(email)
        .fetch_one(db_pool)
        .await
        .map_err(|e| e.to_string())?;

    let exists: bool = row.get(0);
    Ok(exists)
}

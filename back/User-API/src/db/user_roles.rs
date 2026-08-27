use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

pub async fn list_for_user(db: &Pool<Postgres>, user_id: &Uuid) -> Result<Vec<String>, String> {
    let rows = sqlx::query(
        r#"
        SELECT r.name
        FROM roles r
        JOIN user_roles ur ON ur.role_id = r.id
        WHERE ur.user_id = $1
        ORDER BY r.name
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| r.get::<String, _>("name"))
        .collect())
}

pub async fn add(db: &Pool<Postgres>, user_id: &Uuid, role_id: i32) -> Result<bool, String> {
    let result = sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(role_id)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.rows_affected() > 0)
}

pub async fn remove(db: &Pool<Postgres>, user_id: &Uuid, role_id: i32) -> Result<bool, String> {
    let result = sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
        .bind(user_id)
        .bind(role_id)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.rows_affected() > 0)
}

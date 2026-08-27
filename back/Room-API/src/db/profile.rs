use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

pub async fn get_elo(db_pool: &Pool<Postgres>, user_id: &Uuid) -> Result<i32, String> {
    let row = sqlx::query(
        r#"
        SELECT ranked_elo
        FROM user_profile
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| format!("DB query failed: {}", e))?;

    Ok(row.map(|r| r.get::<i32, _>("ranked_elo")).unwrap_or(1500))
}

pub async fn update_tournament_elo(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    delta: i32,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE user_profile
        SET tournament_elo = tournament_elo + $2
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .bind(delta)
    .execute(db_pool)
    .await
    .map_err(|e| format!("DB update tournament_elo failed: {}", e))?;

    Ok(())
}

pub async fn add_xp(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    xp: i64,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE user_profile
        SET xp = xp + $2
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .bind(xp)
    .execute(db_pool)
    .await
    .map_err(|e| format!("DB add xp failed: {}", e))?;

    Ok(())
}

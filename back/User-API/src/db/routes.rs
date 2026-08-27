use sqlx::{Pool, Postgres, Row};

use crate::types::RouteRecord;

pub async fn list(db: &Pool<Postgres>) -> Result<Vec<RouteRecord>, String> {
    let rows = sqlx::query(
        r#"
        SELECT
            ar.id, ar.method, ar.path, ar.name, ar.description,
            rl.requests_per_minute
        FROM api_routes ar
        LEFT JOIN rate_limits rl ON rl.route_id = ar.id
        ORDER BY ar.method, ar.path
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| RouteRecord {
            id: r.get("id"),
            method: r.get("method"),
            path: r.get("path"),
            name: r.get("name"),
            description: r.get("description"),
            requests_per_minute: r.get("requests_per_minute"),
        })
        .collect())
}

pub async fn get_by_id(db: &Pool<Postgres>, id: i32) -> Result<Option<RouteRecord>, String> {
    let row = sqlx::query(
        r#"
        SELECT
            ar.id, ar.method, ar.path, ar.name, ar.description,
            rl.requests_per_minute
        FROM api_routes ar
        LEFT JOIN rate_limits rl ON rl.route_id = ar.id
        WHERE ar.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some(r) => Ok(Some(RouteRecord {
            id: r.get("id"),
            method: r.get("method"),
            path: r.get("path"),
            name: r.get("name"),
            description: r.get("description"),
            requests_per_minute: r.get("requests_per_minute"),
        })),
        None => Ok(None),
    }
}

pub async fn exists(db: &Pool<Postgres>, id: i32) -> Result<bool, String> {
    let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM api_routes WHERE id = $1)")
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(|e| e.to_string())?;
    let exists: bool = row.get(0);
    Ok(exists)
}

pub async fn set_rate_limit(
    db: &Pool<Postgres>,
    route_id: i32,
    requests_per_minute: i64,
) -> Result<(), String> {
    let rpm = requests_per_minute.max(0);
    sqlx::query(
        r#"
        INSERT INTO rate_limits (route_id, requests_per_minute)
        VALUES ($1, $2)
        ON CONFLICT (route_id)
        DO UPDATE SET requests_per_minute = EXCLUDED.requests_per_minute
        "#,
    )
    .bind(route_id)
    .bind(rpm)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

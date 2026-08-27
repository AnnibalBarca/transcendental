use sqlx::{Pool, Postgres, Row};

use crate::types::PermissionRecord;

fn route_label(method: &str, path: &str) -> String {
    format!("{} {}", method, path)
}

pub async fn list(db: &Pool<Postgres>) -> Result<Vec<PermissionRecord>, String> {
    let rows = sqlx::query("SELECT id, name, description FROM permissions ORDER BY id")
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut permissions = Vec::new();
    for row in rows {
        let id: i32 = row.get("id");
        permissions.push(PermissionRecord {
            id,
            name: row.get("name"),
            description: row.get("description"),
            routes: routes_of_permission(db, id).await?,
        });
    }
    Ok(permissions)
}

pub async fn get_by_id(db: &Pool<Postgres>, id: i32) -> Result<Option<PermissionRecord>, String> {
    let row = sqlx::query("SELECT id, name, description FROM permissions WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| e.to_string())?;

    match row {
        Some(row) => Ok(Some(PermissionRecord {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            routes: routes_of_permission(db, id).await?,
        })),
        None => Ok(None),
    }
}

pub async fn routes_of_permission(
    db: &Pool<Postgres>,
    permission_id: i32,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query(
        r#"
        SELECT ar.method, ar.path
        FROM api_routes ar
        JOIN permission_routes pr ON pr.route_id = ar.id
        WHERE pr.permission_id = $1
        ORDER BY ar.method, ar.path
        "#,
    )
    .bind(permission_id)
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| route_label(&r.get::<String, _>("method"), &r.get::<String, _>("path")))
        .collect())
}

pub async fn create(
    db: &Pool<Postgres>,
    name: &str,
    description: &str,
) -> Result<PermissionRecord, String> {
    let row = sqlx::query(
        "INSERT INTO permissions (name, description) VALUES ($1, $2) RETURNING id, name, description",
    )
    .bind(name)
    .bind(description)
    .fetch_one(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(PermissionRecord {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        routes: Vec::new(),
    })
}

pub async fn update(
    db: &Pool<Postgres>,
    id: i32,
    name: &str,
    description: &str,
) -> Result<bool, String> {
    let result =
        sqlx::query("UPDATE permissions SET name = $1, description = $2 WHERE id = $3")
            .bind(name)
            .bind(description)
            .bind(id)
            .execute(db)
            .await
            .map_err(|e| e.to_string())?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete(db: &Pool<Postgres>, id: i32) -> Result<bool, String> {
    let result = sqlx::query("DELETE FROM permissions WHERE id = $1")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.rows_affected() > 0)
}

pub async fn add_route(
    db: &Pool<Postgres>,
    permission_id: i32,
    route_id: i32,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO permission_routes (permission_id, route_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(permission_id)
    .bind(route_id)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn remove_route(
    db: &Pool<Postgres>,
    permission_id: i32,
    route_id: i32,
) -> Result<(), String> {
    sqlx::query("DELETE FROM permission_routes WHERE permission_id = $1 AND route_id = $2")
        .bind(permission_id)
        .bind(route_id)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn routes_with_permissions(
    db: &Pool<Postgres>,
) -> Result<Vec<(String, String, Vec<String>)>, String> {
    let rows = sqlx::query(
        r#"
        SELECT ar.method, ar.path,
               COALESCE(array_agg(p.name ORDER BY p.name) FILTER (WHERE p.name IS NOT NULL),
                        ARRAY[]::text[]) AS perms
        FROM api_routes ar
        LEFT JOIN permission_routes pr ON pr.route_id = ar.id
        LEFT JOIN permissions p ON p.id = pr.permission_id
        GROUP BY ar.id
        ORDER BY ar.method, ar.path
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.get::<String, _>("method"),
                r.get::<String, _>("path"),
                r.get::<Vec<String>, _>("perms"),
            )
        })
        .collect())
}

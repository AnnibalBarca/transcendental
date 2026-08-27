use sqlx::{Pool, Postgres, Row};

use crate::types::RoleRecord;

pub async fn list(db: &Pool<Postgres>) -> Result<Vec<RoleRecord>, String> {
    let rows = sqlx::query("SELECT id, name, description FROM roles ORDER BY id")
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut roles = Vec::new();
    for row in rows {
        let id: i32 = row.get("id");
        roles.push(RoleRecord {
            id,
            name: row.get("name"),
            description: row.get("description"),
            permissions: permissions_of_role(db, id).await?,
        });
    }
    Ok(roles)
}

pub async fn get_by_id(db: &Pool<Postgres>, id: i32) -> Result<Option<RoleRecord>, String> {
    let row = sqlx::query("SELECT id, name, description FROM roles WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| e.to_string())?;

    match row {
        Some(row) => Ok(Some(RoleRecord {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            permissions: permissions_of_role(db, id).await?,
        })),
        None => Ok(None),
    }
}

pub async fn permissions_of_role(
    db: &Pool<Postgres>,
    role_id: i32,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query(
        r#"
        SELECT p.name
        FROM permissions p
        JOIN role_permissions rp ON rp.permission_id = p.id
        WHERE rp.role_id = $1
        ORDER BY p.name
        "#,
    )
    .bind(role_id)
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| r.get::<String, _>("name"))
        .collect())
}

pub async fn create(
    db: &Pool<Postgres>,
    name: &str,
    description: &str,
) -> Result<RoleRecord, String> {
    let row = sqlx::query(
        "INSERT INTO roles (name, description) VALUES ($1, $2) RETURNING id, name, description",
    )
    .bind(name)
    .bind(description)
    .fetch_one(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(RoleRecord {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        permissions: Vec::new(),
    })
}

pub async fn update(
    db: &Pool<Postgres>,
    id: i32,
    name: &str,
    description: &str,
) -> Result<bool, String> {
    let result = sqlx::query("UPDATE roles SET name = $1, description = $2 WHERE id = $3")
        .bind(name)
        .bind(description)
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete(db: &Pool<Postgres>, id: i32) -> Result<bool, String> {
    let result = sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.rows_affected() > 0)
}

pub async fn add_permission(
    db: &Pool<Postgres>,
    role_id: i32,
    permission_id: i32,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(role_id)
    .bind(permission_id)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn remove_permission(
    db: &Pool<Postgres>,
    role_id: i32,
    permission_id: i32,
) -> Result<(), String> {
    sqlx::query("DELETE FROM role_permissions WHERE role_id = $1 AND permission_id = $2")
        .bind(role_id)
        .bind(permission_id)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

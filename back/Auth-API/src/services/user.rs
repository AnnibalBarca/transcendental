use api_core::{cached_get, cached_update};
use uuid::Uuid;

use crate::cache::user::{get_cached_user, invalidate_cached_user, set_cached_user};
use crate::db::models::Database;
use crate::types::UserRecord;

type DbPool = sqlx::Pool<sqlx::Postgres>;
type RedisPool = deadpool_redis::Pool;

pub async fn get_user_by_id(
    id: &Uuid,
    db: &Database,
    redis_pool: &RedisPool,
) -> Result<Option<UserRecord>, String> {
    cached_get!(
        get_cached_user(redis_pool, id),
        async {
            let user = db.get_user_by_id(&id.to_string()).await?;
            Ok(user.map(UserRecord::from))
        },
        |user| set_cached_user(redis_pool, id, user)
    )
}

pub async fn email_is_available(email: &str, db: &Database) -> Result<bool, String> {
    Ok(!db.email_exists(email).await?)
}

pub async fn bootstrap_admin_user(db: &Database) -> Result<(), String> {
    let admin_email = match std::env::var("ADMIN_EMAIL") {
        Ok(v) if !v.trim().is_empty() => v.trim().to_lowercase(),
        _ => return Ok(()),
    };
    let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or_default();
    if admin_password.trim().len() < 8 {
        log::warn!(
            "[Bootstrap] ADMIN_EMAIL is set but ADMIN_PASSWORD is too short (min. 8) — admin account ignored"
        );
        return Ok(());
    }

    let user = match db.get_user_by_email(&admin_email).await? {
        Some(u) => u,
        None => db.create_user(&admin_email, &admin_password).await?,
    };

    if user
        .username
        .as_deref()
        .map_or(true, |u| u.trim().is_empty())
    {
        if db
            .set_username_and_validate(&user.id, "admin")
            .await
            .is_err()
        {
            let _ = db.validate_account(&user.id).await;
        }
    } else {
        let _ = db.validate_account(&user.id).await;
    }
    let _ = db.validate_email(&user.id).await;

    let user_id = match uuid::Uuid::parse_str(&user.id) {
        Ok(id) => id,
        Err(e) => return Err(format!("Invalid admin user id: {}", e)),
    };

    sqlx::query(
        r#"
        INSERT INTO user_roles (user_id, role_id)
        SELECT $1, r.id FROM roles r WHERE r.name = 'admin'
        ON CONFLICT (user_id, role_id) DO NOTHING
        "#,
    )
    .bind(&user_id)
    .execute(db.get_pool())
    .await
    .map_err(|e| format!("role assignment failed: {}", e))?;

    let admin_bio = std::env::var("ADMIN_BIO").unwrap_or_default();
    if !admin_bio.trim().is_empty() {
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO user_profile (user_id, bio)
            VALUES ($1, $2)
            ON CONFLICT (user_id) DO UPDATE
                SET bio = CASE WHEN user_profile.bio = '' THEN EXCLUDED.bio ELSE user_profile.bio END
            "#,
        )
        .bind(&user_id)
        .bind(&admin_bio)
        .execute(db.get_pool())
        .await
        {
            log::warn!("[Bootstrap] Failed to set admin default bio: {}", e);
        }
    }

    log::info!("[Bootstrap] Admin account ready: {}", admin_email);
    Ok(())
}

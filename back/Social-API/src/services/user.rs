use super::super::{cache, db};
use crate::db::db::UserRecord;
use uuid::Uuid;

pub async fn get_user_by_id(
    id: &Uuid,
    db_pool: &sqlx::Pool<sqlx::Postgres>,
    redis_pool: &deadpool_redis::Pool,
) -> Result<Option<UserRecord>, String> {
    if let Ok(Some(user)) = cache::user::get(redis_pool, id).await {
        return Ok(Some(user));
    }

    let db_result = db::user::get_by_id(db_pool, id).await?;

    if let Some(ref user) = db_result {
        let _ = cache::user::set(redis_pool, user, id).await;
    }

    Ok(db_result)
}

pub async fn update_email(
    id: &Uuid,
    new_email: &str,
    db_pool: &sqlx::Pool<sqlx::Postgres>,
    redis_pool: &deadpool_redis::Pool,
) -> Result<(), String> {
    db::user::update_email(db_pool, id, new_email).await?;
    cache::user::invalidate(redis_pool, id).await?;
    Ok(())
}

pub async fn email_is_available(
    email: &str,
    db_pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<bool, String> {
    let exists = db::user::email_exists(db_pool, email).await?;
    Ok(!exists)
}

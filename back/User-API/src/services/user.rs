use api_core::{cached_get, cached_update};
use uuid::Uuid;

use crate::cache::user::{get_cached_user, invalidate_cached_user, set_cached_user};
use crate::db::user as user_repo;
use crate::types::UserRecord;

type DbPool = sqlx::Pool<sqlx::Postgres>;
type RedisPool = deadpool_redis::Pool;

pub async fn get_user_by_id(
    id: &Uuid,
    db_pool: &DbPool,
    redis_pool: &RedisPool,
) -> Result<Option<UserRecord>, String> {
    cached_get!(
        get_cached_user(redis_pool, id),
        user_repo::get_by_id(db_pool, id),
        |user| set_cached_user(redis_pool, id, user)
    )
}

pub async fn get_user_by_username(
    username: &str,
    db_pool: &DbPool,
) -> Result<Option<UserRecord>, String> {
    user_repo::get_by_username(db_pool, username).await
}

pub async fn update_name(
    id: &Uuid,
    new_name: &str,
    db_pool: &DbPool,
    redis_pool: &RedisPool,
) -> Result<(), String> {
    cached_update!(
        user_repo::update_name(db_pool, id, new_name),
        invalidate_cached_user(redis_pool, id)
    )
}

pub async fn name_is_available(name: &str, db_pool: &DbPool) -> Result<bool, String> {
    Ok(!user_repo::name_exists(db_pool, name).await?)
}

pub async fn list_users(
    db_pool: &DbPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<UserRecord>, i64), String> {
    user_repo::list_users(db_pool, limit, offset).await
}

#[allow(clippy::too_many_arguments)]
pub async fn admin_update_user(
    id: &Uuid,
    username: Option<&str>,
    email: Option<&str>,
    account_validated: Option<bool>,
    email_validated: Option<bool>,
    is_banned: Option<bool>,
    wallet: Option<i64>,
    ranked_elo: Option<i32>,
    xp: Option<i64>,
    db_pool: &DbPool,
    redis_pool: &RedisPool,
) -> Result<(), String> {
    cached_update!(
        user_repo::update_user(
            db_pool,
            id,
            username,
            email,
            account_validated,
            email_validated,
            is_banned,
            wallet,
            ranked_elo,
            xp
        ),
        invalidate_cached_user(redis_pool, id)
    )
}

pub async fn admin_delete_user(
    id: &Uuid,
    db_pool: &DbPool,
    redis_pool: &RedisPool,
) -> Result<bool, String> {
    let deleted = user_repo::admin_delete_user(db_pool, id).await?;
    let _ = invalidate_cached_user(redis_pool, id).await;
    Ok(deleted)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_profile_settings(
    id: &Uuid,
    bio: Option<&str>,
    github: Option<&str>,
    discord: Option<&str>,
    twitter: Option<&str>,
    is_private: Option<bool>,
    theme: Option<&str>,
    lang: Option<&str>,
    db_pool: &DbPool,
    redis_pool: &RedisPool,
) -> Result<(), String> {
    cached_update!(
        user_repo::update_profile_settings(
            db_pool, id, bio, github, discord, twitter, is_private, theme, lang
        ),
        invalidate_cached_user(redis_pool, id)
    )
}

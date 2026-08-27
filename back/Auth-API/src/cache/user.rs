use crate::types::UserRecord;
use api_core::cache::{get_json, invalidate, key, set_json};
use uuid::Uuid;

const USER_CACHE_TTL_SECS: usize = 900;

type DbPool = sqlx::Pool<sqlx::Postgres>;
type RedisPool = deadpool_redis::Pool;

pub fn user_profile_key(id: &Uuid) -> String {
    key(&["user:profile", &id.to_string()])
}

pub async fn get_cached_user(
    redis_pool: &RedisPool,
    id: &Uuid,
) -> Result<Option<UserRecord>, String> {
    get_json(redis_pool, &user_profile_key(id)).await
}

pub async fn set_cached_user(
    redis_pool: &RedisPool,
    id: &Uuid,
    user: &UserRecord,
) -> Result<(), String> {
    set_json(redis_pool, &user_profile_key(id), user, USER_CACHE_TTL_SECS).await
}

pub async fn invalidate_cached_user(redis_pool: &RedisPool, id: &Uuid) -> Result<(), String> {
    invalidate(redis_pool, &user_profile_key(id)).await
}

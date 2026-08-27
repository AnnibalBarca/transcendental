use deadpool_redis::redis::cmd;
use uuid::Uuid;

use crate::db::friend::{FriendRequestView, FriendView};

const FRIEND_CACHE_TTL_SECS: usize = 600;
const FRIEND_STATUS_TTL_SECS: usize = 300;

pub async fn get_friends(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<Option<Vec<FriendView>>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_list(user_id);
    let cached: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    if let Some(json_str) = cached {
        if let Ok(friends) = serde_json::from_str::<Vec<FriendView>>(&json_str) {
            return Ok(Some(friends));
        }
    }
    Ok(None)
}

pub async fn set_friends(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    friends: &[FriendView],
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_list(user_id);
    let json_str =
        serde_json::to_string(friends).map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(FRIEND_CACHE_TTL_SECS)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn invalidate_friends(pool: &deadpool_redis::Pool, user_id: &Uuid) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_list(user_id);
    let _: () = cmd("DEL")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;
    Ok(())
}

pub async fn get_pending_requests(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<Option<Vec<FriendRequestView>>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_requests(user_id);
    let cached: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    if let Some(json_str) = cached {
        if let Ok(requests) = serde_json::from_str::<Vec<FriendRequestView>>(&json_str) {
            return Ok(Some(requests));
        }
    }
    Ok(None)
}

pub async fn set_pending_requests(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    requests: &[FriendRequestView],
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_requests(user_id);
    let json_str =
        serde_json::to_string(requests).map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(FRIEND_CACHE_TTL_SECS)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn invalidate_pending_requests(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_requests(user_id);
    let _: () = cmd("DEL")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;
    Ok(())
}

pub async fn get_sent_requests(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<Option<Vec<FriendRequestView>>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_sent(user_id);
    let cached: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    if let Some(json_str) = cached {
        if let Ok(requests) = serde_json::from_str::<Vec<FriendRequestView>>(&json_str) {
            return Ok(Some(requests));
        }
    }
    Ok(None)
}

pub async fn set_sent_requests(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    requests: &[FriendRequestView],
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_sent(user_id);
    let json_str =
        serde_json::to_string(requests).map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(FRIEND_CACHE_TTL_SECS)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn invalidate_sent_requests(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_sent(user_id);
    let _: () = cmd("DEL")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;
    Ok(())
}

pub async fn get_status(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<Option<String>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_status(user_id, friend_id);
    let cached: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    Ok(cached)
}

pub async fn set_status(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    friend_id: &Uuid,
    status: &str,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_status(user_id, friend_id);
    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(FRIEND_STATUS_TTL_SECS)
        .arg(status)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn invalidate_status(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_status(user_id, friend_id);
    let _: () = cmd("DEL")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;
    Ok(())
}

pub async fn get_blocked_users(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<Option<Vec<FriendView>>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_blocked(user_id);
    let cached: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    if let Some(json_str) = cached {
        if let Ok(blocked) = serde_json::from_str::<Vec<FriendView>>(&json_str) {
            return Ok(Some(blocked));
        }
    }
    Ok(None)
}

pub async fn set_blocked_users(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    blocked: &[FriendView],
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_blocked(user_id);
    let json_str =
        serde_json::to_string(blocked).map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(FRIEND_CACHE_TTL_SECS)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn invalidate_blocked_users(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::friend_blocked(user_id);
    let _: () = cmd("DEL")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;
    Ok(())
}

pub async fn invalidate_all_friend_cache(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let keys = vec![
        super::keys::friend_list(user_id),
        super::keys::friend_list(friend_id),
        super::keys::friend_requests(user_id),
        super::keys::friend_requests(friend_id),
        super::keys::friend_sent(user_id),
        super::keys::friend_sent(friend_id),
        super::keys::friend_status(user_id, friend_id),
        super::keys::friend_status(friend_id, user_id),
        super::keys::friend_blocked(user_id),
        super::keys::friend_blocked(friend_id),
    ];

    let _: () = cmd("DEL")
        .arg(&keys)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;

    Ok(())
}

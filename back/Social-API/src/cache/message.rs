use deadpool_redis::redis::cmd;
use uuid::Uuid;

use crate::db::message::MessageRecord;

const MESSAGE_CONV_TTL_SECS: usize = 300;
const MAX_CACHED_MESSAGES: usize = 100;

pub async fn get_messages(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<Option<Vec<MessageRecord>>, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::message_conv(user_id, friend_id);
    let cached: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    if let Some(json_str) = cached {
        if let Ok(messages) = serde_json::from_str::<Vec<MessageRecord>>(&json_str) {
            return Ok(Some(messages));
        }
    }
    Ok(None)
}

pub async fn append_message(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    friend_id: &Uuid,
    message: &MessageRecord,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::message_conv(user_id, friend_id);
    let cached: Option<String> = cmd("GET")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis GET failed: {}", e))?;

    let mut messages: Vec<MessageRecord> = cached
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    messages.push(message.clone());
    if messages.len() > MAX_CACHED_MESSAGES {
        messages.remove(0);
    }

    let json_str =
        serde_json::to_string(&messages).map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(MESSAGE_CONV_TTL_SECS)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn set_messages(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    friend_id: &Uuid,
    messages: &[MessageRecord],
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key = super::keys::message_conv(user_id, friend_id);
    let to_cache: Vec<_> = messages
        .iter()
        .rev()
        .take(MAX_CACHED_MESSAGES)
        .cloned()
        .collect();
    let json_str =
        serde_json::to_string(&to_cache).map_err(|e| format!("Serialization failed: {}", e))?;

    let _: () = cmd("SETEX")
        .arg(&key)
        .arg(MESSAGE_CONV_TTL_SECS)
        .arg(json_str)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis SETEX failed: {}", e))?;

    Ok(())
}

pub async fn invalidate_messages(
    pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Redis pool error: {}", e))?;

    let key_user = super::keys::message_conv(user_id, friend_id);
    let key_friend = super::keys::message_conv(friend_id, user_id);

    let _: () = cmd("DEL")
        .arg(&[key_user, key_friend])
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis DEL failed: {}", e))?;
    Ok(())
}

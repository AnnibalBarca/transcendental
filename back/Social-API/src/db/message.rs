use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageRecord {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

pub async fn send_message(
    db_pool: &Pool<Postgres>,
    sender_id: &Uuid,
    receiver_id: &Uuid,
    content: &str,
) -> Result<MessageRecord, String> {
    let row = sqlx::query(
        r#"
        INSERT INTO friend_messages (sender_id, receiver_id, content)
        VALUES ($1, $2, $3)
        RETURNING id, sender_id, receiver_id, content, created_at
        "#,
    )
    .bind(sender_id)
    .bind(receiver_id)
    .bind(content)
    .fetch_one(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(MessageRecord {
        id: row.get("id"),
        sender_id: row.get("sender_id"),
        receiver_id: row.get("receiver_id"),
        content: row.get("content"),
        created_at: row.get("created_at"),
    })
}

pub async fn get_messages_between(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    friend_id: &Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<MessageRecord>, String> {
    let rows = sqlx::query(
        r#"
        SELECT id, sender_id, receiver_id, content, created_at
        FROM friend_messages
        WHERE (sender_id = $1 AND receiver_id = $2)
           OR (sender_id = $2 AND receiver_id = $1)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(user_id)
    .bind(friend_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(MessageRecord {
            id: row.get("id"),
            sender_id: row.get("sender_id"),
            receiver_id: row.get("receiver_id"),
            content: row.get("content"),
            created_at: row.get("created_at"),
        });
    }
    Ok(messages)
}

pub async fn mark_messages_as_read(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<u64, String> {
    let result = sqlx::query(
        r#"
        UPDATE friend_messages
        SET read_at = NOW()
        WHERE receiver_id = $1 AND sender_id = $2 AND read_at IS NULL
        "#,
    )
    .bind(user_id)
    .bind(friend_id)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.rows_affected())
}

pub async fn count_unread_from_friend(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<i64, String> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*) as cnt
        FROM friend_messages
        WHERE receiver_id = $1 AND sender_id = $2 AND read_at IS NULL
        "#,
    )
    .bind(user_id)
    .bind(friend_id)
    .fetch_one(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let count: i64 = row.get("cnt");
    Ok(count)
}

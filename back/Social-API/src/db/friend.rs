use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

use crate::db::message::MessageRecord;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FriendshipRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub friend_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FriendRequestView {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: Option<String>,
    #[serde(default)]
    pub picture_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FriendView {
    pub friend_id: Uuid,
    pub username: Option<String>,
    #[serde(default)]
    pub picture_id: String,
    pub created_at: DateTime<Utc>,
    pub last_message: Option<MessageRecord>,
    #[serde(default)]
    pub unread_count: i64,
}

pub async fn send_request(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO friendships (user_id, friend_id, status)
        VALUES ($1, $2, 'pending')
        ON CONFLICT (user_id, friend_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(friend_id)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn get_request_status(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<Option<String>, String> {
    let row = sqlx::query(
        r#"
        SELECT status FROM friendships
        WHERE user_id = $1 AND friend_id = $2
        "#,
    )
    .bind(user_id)
    .bind(friend_id)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|r| r.get("status")))
}

pub async fn accept_request(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    let mut tx = db_pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        UPDATE friendships
        SET status = 'accepted', updated_at = NOW()
        WHERE user_id = $1 AND friend_id = $2 AND status = 'pending'
        "#,
    )
    .bind(friend_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        INSERT INTO friendships (user_id, friend_id, status)
        VALUES ($1, $2, 'accepted')
        ON CONFLICT (user_id, friend_id) DO UPDATE SET status = 'accepted', updated_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(friend_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn refuse_request(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM friendships
        WHERE user_id = $1 AND friend_id = $2 AND status = 'pending'
        "#,
    )
    .bind(friend_id)
    .bind(user_id)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn cancel_request(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM friendships
        WHERE user_id = $1 AND friend_id = $2 AND status = 'pending'
        "#,
    )
    .bind(user_id)
    .bind(friend_id)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn remove_friend(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM friendships
        WHERE (user_id = $1 AND friend_id = $2)
           OR (user_id = $2 AND friend_id = $1)
        "#,
    )
    .bind(user_id)
    .bind(friend_id)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn block_user(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    blocked_id: &Uuid,
) -> Result<(), String> {
    let mut tx = db_pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        DELETE FROM friendships
        WHERE (user_id = $1 AND friend_id = $2)
           OR (user_id = $2 AND friend_id = $1)
        "#,
    )
    .bind(user_id)
    .bind(blocked_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        INSERT INTO friendships (user_id, friend_id, status)
        VALUES ($1, $2, 'blocked')
        ON CONFLICT (user_id, friend_id) DO UPDATE SET status = 'blocked', updated_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(blocked_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn unblock_user(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    blocked_id: &Uuid,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM friendships
        WHERE user_id = $1 AND friend_id = $2 AND status = 'blocked'
        "#,
    )
    .bind(user_id)
    .bind(blocked_id)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn is_blocked(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    other_id: &Uuid,
) -> Result<bool, String> {
    let row = sqlx::query(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM friendships
            WHERE user_id = $1 AND friend_id = $2 AND status = 'blocked'
        )
        "#,
    )
    .bind(user_id)
    .bind(other_id)
    .fetch_one(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let exists: bool = row.get(0);
    Ok(exists)
}

pub async fn get_friends(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
) -> Result<Vec<FriendView>, String> {
    let rows = sqlx::query(
        r#"
        SELECT u.id as friend_id, u.username, COALESCE(p.picture_id, '') AS picture_id, f.created_at,
               m.id as msg_id, m.sender_id as msg_sender_id, m.receiver_id as msg_receiver_id,
               m.content as msg_content, m.created_at as msg_created_at,
               (
                   SELECT COUNT(*)::BIGINT
                   FROM friend_messages fm
                   WHERE fm.receiver_id = $1 AND fm.sender_id = u.id AND fm.read_at IS NULL
               ) as unread_count
        FROM friendships f
        JOIN users u ON u.id = f.friend_id
        LEFT JOIN user_profile p ON p.user_id = u.id
        LEFT JOIN LATERAL (
            SELECT id, sender_id, receiver_id, content, created_at
            FROM friend_messages
            WHERE (sender_id = $1 AND receiver_id = u.id)
               OR (sender_id = u.id AND receiver_id = $1)
            ORDER BY created_at DESC
            LIMIT 1
        ) m ON true
        WHERE f.user_id = $1 AND f.status = 'accepted'
        ORDER BY GREATEST(f.created_at, COALESCE(m.created_at, f.created_at)) DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut friends = Vec::new();
    for row in rows {
        let msg_id: Option<Uuid> = row.get("msg_id");
        let last_message = msg_id.map(|id| MessageRecord {
            id,
            sender_id: row.get("msg_sender_id"),
            receiver_id: row.get("msg_receiver_id"),
            content: row.get("msg_content"),
            created_at: row.get("msg_created_at"),
        });
        friends.push(FriendView {
            friend_id: row.get("friend_id"),
            username: row.get("username"),
            picture_id: row.get("picture_id"),
            created_at: row.get("created_at"),
            last_message,
            unread_count: row.get("unread_count"),
        });
    }
    Ok(friends)
}

pub async fn get_pending_requests(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
) -> Result<Vec<FriendRequestView>, String> {
    let rows = sqlx::query(
        r#"
        SELECT f.id, u.id as user_id, u.username, COALESCE(p.picture_id, '') AS picture_id, f.created_at
        FROM friendships f
        JOIN users u ON u.id = f.user_id
        LEFT JOIN user_profile p ON p.user_id = u.id
        WHERE f.friend_id = $1 AND f.status = 'pending'
        ORDER BY f.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut requests = Vec::new();
    for row in rows {
        requests.push(FriendRequestView {
            id: row.get("id"),
            user_id: row.get("user_id"),
            username: row.get("username"),
            picture_id: row.get("picture_id"),
            created_at: row.get("created_at"),
        });
    }
    Ok(requests)
}

pub async fn get_blocked_users(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
) -> Result<Vec<FriendView>, String> {
    let rows = sqlx::query(
        r#"
        SELECT u.id as friend_id, u.username, COALESCE(p.picture_id, '') AS picture_id, f.created_at
        FROM friendships f
        JOIN users u ON u.id = f.friend_id
        LEFT JOIN user_profile p ON p.user_id = u.id
        WHERE f.user_id = $1 AND f.status = 'blocked'
        ORDER BY f.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut blocked = Vec::new();
    for row in rows {
        blocked.push(FriendView {
            friend_id: row.get("friend_id"),
            username: row.get("username"),
            picture_id: row.get("picture_id"),
            created_at: row.get("created_at"),
            last_message: None,
            unread_count: 0,
        });
    }
    Ok(blocked)
}

pub async fn get_sent_requests(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
) -> Result<Vec<FriendRequestView>, String> {
    let rows = sqlx::query(
        r#"
        SELECT f.id, u.id as user_id, u.username, COALESCE(p.picture_id, '') AS picture_id, f.created_at
        FROM friendships f
        JOIN users u ON u.id = f.friend_id
        LEFT JOIN user_profile p ON p.user_id = u.id
        WHERE f.user_id = $1 AND f.status = 'pending'
        ORDER BY f.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut requests = Vec::new();
    for row in rows {
        requests.push(FriendRequestView {
            id: row.get("id"),
            user_id: row.get("user_id"),
            username: row.get("username"),
            picture_id: row.get("picture_id"),
            created_at: row.get("created_at"),
        });
    }
    Ok(requests)
}

use crate::db::friend;
use crate::db::message;
use crate::db::user;
use notification::event::{NotificationBus, NotificationEvent};
use sqlx::Pool;
use uuid::Uuid;

pub use friend::FriendRequestView;
pub use friend::FriendView;
pub use message::MessageRecord;

pub async fn send_request(
    db_pool: &Pool<sqlx::Postgres>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    friend::send_request(db_pool, user_id, friend_id).await?;

    if let Ok(Some(u)) = user::get_by_id(db_pool, user_id).await {
        let event = NotificationEvent::FriendRequest {
            from_user_id: *user_id,
            username: u.username,
        };
        notification_bus.send_to_user(*friend_id, &event).await;
    }

    Ok(())
}

pub async fn accept_request(
    db_pool: &Pool<sqlx::Postgres>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    friend::accept_request(db_pool, user_id, friend_id).await?;

    if let Ok(Some(u)) = user::get_by_id(db_pool, user_id).await {
        let username = u.username.clone();
        let event = NotificationEvent::FriendRequestAccepted {
            by_user_id: *user_id,
            username: username.clone(),
        };
        notification_bus.send_to_user(*friend_id, &event).await;
    }

    if let Ok(Some(u)) = user::get_by_id(db_pool, friend_id).await {
        let username = u.username.clone();
        let event = NotificationEvent::FriendRequestAccepted {
            by_user_id: *friend_id,
            username: username.clone(),
        };
        notification_bus.send_to_user(*user_id, &event).await;
    }

    Ok(())
}

pub async fn refuse_request(
    db_pool: &Pool<sqlx::Postgres>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    friend::refuse_request(db_pool, user_id, friend_id).await?;

    if let Ok(Some(u)) = user::get_by_id(db_pool, user_id).await {
        let event = NotificationEvent::FriendRequestRefused {
            by_user_id: *user_id,
            username: u.username,
        };
        notification_bus.send_to_user(*friend_id, &event).await;
    }

    Ok(())
}

pub async fn cancel_request(
    db_pool: &Pool<sqlx::Postgres>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    friend::cancel_request(db_pool, user_id, friend_id).await?;

    if let Ok(Some(u)) = user::get_by_id(db_pool, user_id).await {
        let event = NotificationEvent::FriendRequestCancelled {
            by_user_id: *user_id,
            username: u.username,
        };
        notification_bus.send_to_user(*friend_id, &event).await;
    }

    Ok(())
}

pub async fn remove_friend(
    db_pool: &Pool<sqlx::Postgres>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<(), String> {
    friend::remove_friend(db_pool, user_id, friend_id).await?;

    if let Ok(Some(u)) = user::get_by_id(db_pool, user_id).await {
        let event = NotificationEvent::FriendRemoved {
            by_user_id: *user_id,
            username: u.username,
        };
        notification_bus.send_to_user(*friend_id, &event).await;
    }

    Ok(())
}

pub async fn block_user(
    db_pool: &Pool<sqlx::Postgres>,
    notification_bus: &NotificationBus,
    user_id: &Uuid,
    blocked_id: &Uuid,
) -> Result<(), String> {
    friend::block_user(db_pool, user_id, blocked_id).await?;

    if let Ok(Some(u)) = user::get_by_id(db_pool, user_id).await {
        let event = NotificationEvent::FriendRemoved {
            by_user_id: *user_id,
            username: u.username,
        };
        notification_bus.send_to_user(*blocked_id, &event).await;
    }

    Ok(())
}

pub async fn unblock_user(
    db_pool: &Pool<sqlx::Postgres>,
    _redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    blocked_id: &Uuid,
) -> Result<(), String> {
    friend::unblock_user(db_pool, user_id, blocked_id).await
}

pub async fn is_blocked(
    db_pool: &Pool<sqlx::Postgres>,
    _redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    other_id: &Uuid,
) -> Result<bool, String> {
    friend::is_blocked(db_pool, user_id, other_id).await
}

pub async fn get_friends(
    db_pool: &Pool<sqlx::Postgres>,
    _redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<Vec<FriendView>, String> {
    friend::get_friends(db_pool, user_id).await
}

pub async fn get_pending_requests(
    db_pool: &Pool<sqlx::Postgres>,
    _redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<Vec<FriendRequestView>, String> {
    friend::get_pending_requests(db_pool, user_id).await
}

pub async fn get_blocked_users(
    db_pool: &Pool<sqlx::Postgres>,
    _redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<Vec<FriendView>, String> {
    friend::get_blocked_users(db_pool, user_id).await
}

pub async fn get_sent_requests(
    db_pool: &Pool<sqlx::Postgres>,
    _redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<Vec<FriendRequestView>, String> {
    friend::get_sent_requests(db_pool, user_id).await
}

pub async fn send_message(
    db_pool: &Pool<sqlx::Postgres>,
    notification_bus: &NotificationBus,
    sender_id: &Uuid,
    receiver_id: &Uuid,
    content: &str,
) -> Result<MessageRecord, String> {
    let record = message::send_message(db_pool, sender_id, receiver_id, content).await?;

    if let Ok(Some(u)) = user::get_by_id(db_pool, sender_id).await {
        let event = NotificationEvent::NewMessage {
            from_user_id: *sender_id,
            username: u.username,
            content: content.to_string(),
        };
        notification_bus.send_to_user(*receiver_id, &event).await;
    }

    Ok(record)
}

pub async fn get_messages_between(
    db_pool: &Pool<sqlx::Postgres>,
    _redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    friend_id: &Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<MessageRecord>, String> {
    message::get_messages_between(db_pool, user_id, friend_id, limit, offset).await
}

pub async fn mark_messages_as_read(
    db_pool: &Pool<sqlx::Postgres>,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<u64, String> {
    message::mark_messages_as_read(db_pool, user_id, friend_id).await
}

pub async fn count_unread_from_friend(
    db_pool: &Pool<sqlx::Postgres>,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<i64, String> {
    message::count_unread_from_friend(db_pool, user_id, friend_id).await
}

pub async fn get_request_status(
    db_pool: &Pool<sqlx::Postgres>,
    _redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    friend_id: &Uuid,
) -> Result<Option<String>, String> {
    friend::get_request_status(db_pool, user_id, friend_id).await
}

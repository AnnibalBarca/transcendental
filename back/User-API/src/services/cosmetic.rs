use sqlx::Pool;
use uuid::Uuid;

use api_core::sse::SsePublisher;
use notification::event::{NotificationBus, NotificationEvent, NotificationTarget};

use crate::{
    cache,
    cache::user::invalidate_cached_user,
    db::cosmetic::{self, InventoryItem},
};

pub async fn get_inventory(
    db_pool: &Pool<sqlx::Postgres>,
    redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<Vec<InventoryItem>, String> {
    if let Ok(Some(items)) = cache::cosmetic::get_inventory(redis_pool, user_id).await {
        return Ok(items);
    }

    let items = cosmetic::get_inventory(db_pool, user_id).await?;
    let _ = cache::cosmetic::set_inventory(redis_pool, user_id, &items).await;

    Ok(items)
}

pub async fn add_item(
    db_pool: &Pool<sqlx::Postgres>,
    redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    item_id: &str,
    item_type: &str,
) -> Result<(), String> {
    cosmetic::add_item_to_inventory(db_pool, user_id, item_id, item_type).await?;
    let _ = cache::cosmetic::invalidate_inventory(redis_pool, user_id).await;
    Ok(())
}

pub async fn remove_item(
    db_pool: &Pool<sqlx::Postgres>,
    redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    item_id: &str,
    item_type: &str,
) -> Result<(), String> {
    cosmetic::remove_item_from_inventory(db_pool, user_id, item_id, item_type).await?;
    let _ = cache::cosmetic::invalidate_inventory(redis_pool, user_id).await;
    Ok(())
}

pub async fn has_item(
    db_pool: &Pool<sqlx::Postgres>,
    user_id: &Uuid,
    item_id: &str,
    item_type: &str,
) -> Result<bool, String> {
    cosmetic::has_item_in_inventory(db_pool, user_id, item_id, item_type).await
}

pub async fn check_equiped_item_validity(
    db_pool: &Pool<sqlx::Postgres>,
    items: Vec<InventoryItem>,
) -> Result<bool, String> {
    cosmetic::has_equiped_items_in_inventory(db_pool, items).await
}

pub async fn get_profile_picture(
    db_pool: &Pool<sqlx::Postgres>,
    redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> Result<Option<String>, String> {
    if let Ok(Some(url)) = cache::cosmetic::get_profile_picture(redis_pool, user_id).await {
        return Ok(Some(url));
    }

    let url = cosmetic::get_profile_picture(db_pool, user_id).await?;
    if let Some(ref u) = url {
        let _ = cache::cosmetic::set_profile_picture(redis_pool, user_id, u).await;
    }

    Ok(url)
}

pub async fn set_profile_picture(
    db_pool: &Pool<sqlx::Postgres>,
    redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    picture_id: &str,
) -> Result<(), String> {
    cosmetic::set_profile_picture(db_pool, user_id, picture_id).await?;

    let _ = cache::cosmetic::invalidate_profile_picture(redis_pool, user_id).await;
    let _ = invalidate_cached_user(redis_pool, user_id).await;

    let friend_ids = cosmetic::get_accepted_friend_ids(db_pool, user_id).await?;
    let _ = cache::cosmetic::invalidate_social_caches(redis_pool, &friend_ids).await;

    let publisher = SsePublisher::<NotificationEvent, NotificationTarget>::new(
        redis_pool.clone(),
        "sse:events",
    );
    let bus = NotificationBus::new(publisher);
    let event = NotificationEvent::ProfilePictureUpdated {
        user_id: *user_id,
        picture_id: picture_id.to_string(),
    };
    for friend_id in &friend_ids {
        bus.send_to_user(*friend_id, &event).await;
    }

    Ok(())
}

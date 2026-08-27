use sqlx::{
    types::chrono::{DateTime, Utc},
    Pool, Postgres, Row,
};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InventoryItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub item_id: String,
    pub item_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerProfilePicture {
    pub user_id: Uuid,
    pub picture_id: String,
    pub updated_at: DateTime<Utc>,
}

pub async fn set_profile_picture(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    picture_id: &str,
) -> Result<(), String> {
    sqlx::query(
        r#"
	    UPDATE user_profile
	    SET picture_id = $2,
	        picture_updated_at = NOW()
	    WHERE user_id = $1
	    "#,
    )
    .bind(user_id)
    .bind(picture_id)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn get_profile_picture(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
) -> Result<Option<String>, String> {
    let row = sqlx::query(
        r#"
        SELECT user_id, ranked_elo, picture_id, picture_updated_at
        FROM user_profile
        WHERE user_id = $1
	    "#,
    )
    .bind(user_id)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|r| r.get("picture_id")))
}

pub async fn get_accepted_friend_ids(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
) -> Result<Vec<Uuid>, String> {
    let rows = sqlx::query(
        r#"
        SELECT friend_id AS other_id FROM friendships
        WHERE user_id = $1 AND status = 'accepted'
        UNION
        SELECT user_id AS other_id FROM friendships
        WHERE friend_id = $1 AND status = 'accepted'
        "#,
    )
    .bind(user_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|r| r.get::<Uuid, _>("other_id")).collect())
}

pub async fn add_item_to_inventory(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    item_id: &str,
    item_type: &str,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO player_inventory (user_id, item_id, item_type, item_rarity)
        VALUES ($1, $2, $3, '0')
        ON CONFLICT (user_id, item_id, item_type) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(item_id)
    .bind(item_type)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn remove_item_from_inventory(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    item_id: &str,
    item_type: &str,
) -> Result<(), String> {
    sqlx::query(
        r#"
        DELETE FROM player_inventory
        WHERE user_id = $1 AND item_id = $2 AND item_type = $3
        "#,
    )
    .bind(user_id)
    .bind(item_id)
    .bind(item_type)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn get_inventory(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
) -> Result<Vec<InventoryItem>, String> {
    let rows = sqlx::query(
        r#"
        SELECT id, user_id, item_id, item_type, created_at
        FROM player_inventory
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for row in rows {
        items.push(InventoryItem {
            id: row.get("id"),
            user_id: row.get("user_id"),
            item_id: row.get("item_id"),
            item_type: row.get("item_type"),
            created_at: row.get("created_at"),
        });
    }
    Ok(items)
}

pub async fn has_equiped_items_in_inventory(
    db_pool: &Pool<Postgres>,
    items: Vec<InventoryItem>,
) -> Result<bool, String> {
    if items.is_empty() {
        return Ok(true);
    }

    let expected_count = items.len() as i64;

    let mut user_ids = Vec::with_capacity(items.len());
    let mut item_ids = Vec::with_capacity(items.len());
    let mut item_types = Vec::with_capacity(items.len());

    for item in items {
        user_ids.push(item.user_id);
        item_ids.push(item.item_id);
        item_types.push(item.item_type);
    }

    let row = sqlx::query(
        r#"
        SELECT (
            SELECT COUNT(*)
            FROM UNNEST($1, $2, $3) AS t(u_id, i_id, i_type)
            WHERE EXISTS (
                SELECT 1 FROM player_inventory pi
                WHERE pi.user_id = t.u_id
                  AND pi.item_id = t.i_id
                  AND pi.item_type = t.i_type
            )
        ) = $4
        "#,
    )
    .bind(&user_ids)
    .bind(&item_ids)
    .bind(&item_types)
    .bind(expected_count)
    .fetch_one(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let all_exist: bool = row.get(0);
    Ok(all_exist)
}

pub async fn has_item_in_inventory(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    item_id: &str,
    item_type: &str,
) -> Result<bool, String> {
    let row = sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM player_inventory
            WHERE user_id = $1 AND item_id = $2 AND item_type = $3
        )
        "#,
    )
    .bind(user_id)
    .bind(item_id)
    .bind(item_type)
    .fetch_one(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let exists: bool = row.get(0);
    Ok(exists)
}

use crate::types::{CollectionItemRecord, CollectionRecord};
use sqlx::{Pool, Postgres, Row};
use std::collections::HashMap;
use uuid::Uuid;

pub async fn list_all(db_pool: &Pool<Postgres>) -> Result<Vec<CollectionRecord>, String> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, price, end_date
        FROM collections
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut items_by_collection = list_items_grouped(db_pool).await?;

    Ok(rows
        .iter()
        .map(|r| {
            let id: Uuid = r.get("id");
            CollectionRecord {
                id: id.to_string(),
                title: r.get("title"),
                price: r.get("price"),
                end_date: r
                    .get::<chrono::DateTime<chrono::Utc>, _>("end_date")
                    .to_rfc3339(),
                items: items_by_collection.remove(&id).unwrap_or_default(),
            }
        })
        .collect())
}

async fn list_items_grouped(
    db_pool: &Pool<Postgres>,
) -> Result<HashMap<Uuid, Vec<CollectionItemRecord>>, String> {
    let rows = sqlx::query(
        r#"
        SELECT ci.collection_id,
               ci.item_id,
               ci.item_type,
               ci.position,
               sc.title,
               sc.price,
               sc.asset_key
        FROM collection_items ci
        JOIN shop_catalog sc
          ON sc.item_id = ci.item_id AND sc.item_type = ci.item_type
        ORDER BY ci.collection_id, ci.position
        "#,
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut grouped: HashMap<Uuid, Vec<CollectionItemRecord>> = HashMap::new();
    for r in rows {
        grouped
            .entry(r.get("collection_id"))
            .or_default()
            .push(CollectionItemRecord {
                item_id: r.get("item_id"),
                item_type: r.get("item_type"),
                title: r.get("title"),
                price: r.get("price"),
                asset_key: r.get("asset_key"),
            });
    }
    Ok(grouped)
}

pub async fn items_of(
    db_pool: &Pool<Postgres>,
    collection_id: &Uuid,
) -> Result<Vec<CollectionItemRecord>, String> {
    let rows = sqlx::query(
        r#"
        SELECT ci.item_id, ci.item_type, sc.title, sc.price, sc.asset_key
        FROM collection_items ci
        JOIN shop_catalog sc
          ON sc.item_id = ci.item_id AND sc.item_type = ci.item_type
        WHERE ci.collection_id = $1
        ORDER BY ci.position
        "#,
    )
    .bind(collection_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .iter()
        .map(|r| CollectionItemRecord {
            item_id: r.get("item_id"),
            item_type: r.get("item_type"),
            title: r.get("title"),
            price: r.get("price"),
            asset_key: r.get("asset_key"),
        })
        .collect())
}

pub async fn get_price(
    db_pool: &Pool<Postgres>,
    collection_id: &Uuid,
) -> Result<Option<(String, i32)>, String> {
    let row = sqlx::query(
        r#"
        SELECT title, price FROM collections
        WHERE id = $1 AND end_date > NOW()
        "#,
    )
    .bind(collection_id)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|r| (r.get("title"), r.get("price"))))
}

pub async fn add_item(
    db_pool: &Pool<Postgres>,
    collection_id: &Uuid,
    item_id: &str,
    item_type: &str,
    position: i16,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO collection_items (collection_id, item_id, item_type, position)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (collection_id, item_id, item_type)
        DO UPDATE SET position = EXCLUDED.position
        "#,
    )
    .bind(collection_id)
    .bind(item_id)
    .bind(item_type)
    .bind(position)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

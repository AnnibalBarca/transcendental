// Everything the shop feature does to Postgres: read the catalog, and run
// the three money-moving operations (purchase/purchase_collection/refund)
// as atomic transactions. Schema lives in db::migrations (search "shop" /
// "collection" there) — shop_catalog is the sellable-item list,
// player_inventory is what a user owns, collections/collection_items are
// pre-built bundles of catalog items.
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

// One row of shop_catalog, as returned by list_active() and serialized
// straight into the GET /shop JSON response. NOTE: only `asset_key` is
// exposed here, no ready-made `image_url` — as of the current code the
// front derives the MinIO URL itself from item_type/item_id
// (see front/src/utils/cosmeticImage.ts::cosmeticImageUrl), so `asset_key`
// travels over the wire but isn't actually read by the shop UI today. An
// earlier version of this handler resolved a real URL server-side
// (Storage::public_url); that indirection got dropped along the way in
// favour of the front just hardcoding the "{bucket}/{slot}/{id}.png"
// convention.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CatalogItem {
    pub item_id: String,
    pub item_type: String,
    pub title: String,
    pub price: i64,
    pub asset_key: String,
}

/// Everything currently for sale (is_active = TRUE — soft-delete flag, so
/// retired items stay in the table for purchase history / FK integrity
/// instead of being deleted). Backs GET /shop's `items` field.
pub async fn list_active(db_pool: &Pool<Postgres>) -> Result<Vec<CatalogItem>, String> {
    let rows = sqlx::query(
        r#"
        SELECT item_id, item_type, title, price, asset_key
        FROM shop_catalog
        WHERE is_active = TRUE
        ORDER BY item_type ASC, created_at ASC
        "#,
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|r| CatalogItem {
            item_id: r.get("item_id"),
            item_type: r.get("item_type"),
            title: r.get("title"),
            price: r.get("price"),
            asset_key: r.get("asset_key"),
        })
        .collect())
}

/// (item_id, item_type) -> asset_key lookup table, originally written so
/// get_inventory.rs could attach a resolved image_url to each owned item.
/// DEAD CODE today: get_inventory.rs was reverted back to not calling
/// this (it now returns bare item_id/item_type, letting the front build
/// the image URL itself — see the CatalogItem note above), so nothing
/// calls asset_keys() anymore. Harmless to leave, but there's no live
/// caller left to point to.
pub async fn asset_keys(
    db_pool: &Pool<Postgres>,
) -> Result<std::collections::HashMap<(String, String), String>, String> {
    let rows = sqlx::query(
        r#"
        SELECT item_id, item_type, asset_key
        FROM shop_catalog
        WHERE asset_key <> ''
        "#,
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| {
            (
                (r.get("item_id"), r.get("item_type")),
                r.get::<String, _>("asset_key"),
            )
        })
        .collect())
}

/// Insert-or-update one shop_catalog row (the ON CONFLICT DO UPDATE makes
/// re-uploading the same item_id/item_type replace it in place, including
/// flipping is_active back to TRUE if it had been retired). Called from
/// handle_upload_item once the image is safely stored in MinIO — DB write
/// happens after the object PUT, not before, so a failed upload never
/// leaves a catalog row pointing at a non-existent object.
pub async fn upsert_item(
    db_pool: &Pool<Postgres>,
    item_id: &str,
    item_type: &str,
    title: &str,
    price: i64,
    asset_key: &str,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO shop_catalog (item_id, item_type, title, price, asset_key, is_active)
        VALUES ($1, $2, $3, $4, $5, TRUE)
        ON CONFLICT (item_id, item_type) DO UPDATE
            SET title     = EXCLUDED.title,
                price     = EXCLUDED.price,
                asset_key = EXCLUDED.asset_key,
                is_active = TRUE
        "#,
    )
    .bind(item_id)
    .bind(item_type)
    .bind(title)
    .bind(price)
    .bind(asset_key)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Buys one catalog item for one user, as a single DB transaction so a
/// crash or concurrent request can't leave the wallet debited without the
/// item granted (or vice versa). Order matters: price lookup -> ownership
/// check -> debit -> grant, each step able to abort the whole transaction
/// (an early `return Err` before `tx.commit()` means Postgres rolls
/// everything back, including the earlier statements in this tx).
/// Returns the new wallet balance so the front can update its wallet
/// display without a second round trip.
pub async fn purchase(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    item_id: &str,
    item_type: &str,
) -> Result<i64, String> {
    let mut tx = db_pool.begin().await.map_err(|e| e.to_string())?;

    let price_row = sqlx::query(
        r#"
        SELECT price FROM shop_catalog
        WHERE item_id = $1 AND item_type = $2 AND is_active = TRUE
        "#,
    )
    .bind(item_id)
    .bind(item_type)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    let price: i64 = match price_row {
        Some(r) => r.get("price"),
        None => return Err("item not available".to_string()),
    };

    let owned = sqlx::query(
        r#"
        SELECT 1 AS one FROM player_inventory
        WHERE user_id = $1 AND item_id = $2 AND item_type = $3
        "#,
    )
    .bind(user_id)
    .bind(item_id)
    .bind(item_type)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    if owned.is_some() {
        return Err("already owned".to_string());
    }

    // `wallet >= $1` in the WHERE clause (not a separate SELECT-then-check)
    // is what makes this race-safe: Postgres evaluates the row's current
    // wallet at UPDATE time under the transaction's row lock, so two
    // concurrent purchases racing on the same balance can't both pass —
    // the second one's UPDATE simply matches zero rows.
    let deduct = sqlx::query(
        r#"
        UPDATE users
        SET wallet = wallet - $1, updated_at = NOW()
        WHERE id = $2 AND wallet >= $1
        RETURNING wallet
        "#,
    )
    .bind(price)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    let new_wallet: i64 = match deduct {
        Some(r) => r.get("wallet"),
        None => return Err("insufficient funds".to_string()),
    };

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
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(new_wallet)
}

pub async fn purchase_collection(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    collection_id: &Uuid,
) -> Result<(i64, Vec<(String, String)>), String> {
    let mut tx = db_pool.begin().await.map_err(|e| e.to_string())?;

    let collection = sqlx::query(
        r#"
        SELECT price FROM collections
        WHERE id = $1 AND end_date > NOW()
        "#,
    )
    .bind(collection_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    let price: i64 = match collection {
        Some(r) => r.get::<i32, _>("price") as i64,
        None => return Err("collection not available".to_string()),
    };

    let rows = sqlx::query(
        r#"
        SELECT ci.item_id, ci.item_type
        FROM collection_items ci
        WHERE ci.collection_id = $1
        ORDER BY ci.position
        "#,
    )
    .bind(collection_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    if rows.is_empty() {
        return Err("empty collection".to_string());
    }

    let owned_rows = sqlx::query(
        r#"
        SELECT item_id, item_type FROM player_inventory WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    let owned: std::collections::HashSet<(String, String)> = owned_rows
        .iter()
        .map(|r| (r.get("item_id"), r.get("item_type")))
        .collect();

    let missing: Vec<(String, String)> = rows
        .iter()
        .map(|r| {
            (
                r.get::<String, _>("item_id"),
                r.get::<String, _>("item_type"),
            )
        })
        .filter(|pair| !owned.contains(pair))
        .collect();
    if missing.is_empty() {
        return Err("already owned".to_string());
    }

    // `wallet >= $1` in the WHERE clause (not a separate SELECT-then-check)
    // is what makes this race-safe: Postgres evaluates the row's current
    // wallet at UPDATE time under the transaction's row lock, so two
    // concurrent purchases racing on the same balance can't both pass —
    // the second one's UPDATE simply matches zero rows.
    let deduct = sqlx::query(
        r#"
        UPDATE users
        SET wallet = wallet - $1, updated_at = NOW()
        WHERE id = $2 AND wallet >= $1
        RETURNING wallet
        "#,
    )
    .bind(price)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    let new_wallet: i64 = match deduct {
        Some(r) => r.get("wallet"),
        None => return Err("insufficient funds".to_string()),
    };

    let item_ids: Vec<String> = missing.iter().map(|(i, _)| i.clone()).collect();
    let item_types: Vec<String> = missing.iter().map(|(_, t)| t.clone()).collect();
    sqlx::query(
        r#"
        INSERT INTO player_inventory (user_id, item_id, item_type, item_rarity)
        SELECT $1, i_id, i_type, '0' FROM UNNEST($2, $3) AS t(i_id, i_type)
        ON CONFLICT (user_id, item_id, item_type) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(&item_ids)
    .bind(&item_types)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok((new_wallet, missing))
}

pub async fn refund(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    item_id: &str,
    item_type: &str,
) -> Result<i64, String> {
    let mut tx = db_pool.begin().await.map_err(|e| e.to_string())?;

    let deleted = sqlx::query(
        r#"
        DELETE FROM player_inventory
        WHERE user_id = $1 AND item_id = $2 AND item_type = $3
        RETURNING 1 AS one
        "#,
    )
    .bind(user_id)
    .bind(item_id)
    .bind(item_type)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    if deleted.is_none() {
        return Err("not owned".to_string());
    }

    let price_row = sqlx::query(
        r#"
        SELECT price FROM shop_catalog
        WHERE item_id = $1 AND item_type = $2
        "#,
    )
    .bind(item_id)
    .bind(item_type)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    let refund_amount: i64 = price_row.map(|r| r.get("price")).unwrap_or(0);

    let credited = sqlx::query(
        r#"
        UPDATE users
        SET wallet = wallet + $1, updated_at = NOW()
        WHERE id = $2
        RETURNING wallet
        "#,
    )
    .bind(refund_amount)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    let new_wallet: i64 = credited.get("wallet");

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(new_wallet)
}

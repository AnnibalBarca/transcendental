use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

pub const DEFAULT_COMMON_CARDS: &[&str] = &["1", "2", "3", "5", "6", "7", "8", "9", "10", "11"];

pub async fn card_price(db_pool: &Pool<Postgres>, card_id: &str) -> Result<Option<i64>, String> {
    let row = sqlx::query(
        "SELECT price FROM shop_catalog WHERE item_id = $1 AND item_type = 'card'",
    )
    .bind(card_id)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row.map(|r| r.get("price")))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerCard {
    pub card_id: String,
    pub rarity: i16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeckEntry {
    pub card_id: String,
    pub rarity: i16,
}

pub async fn get_player_cards(db_pool: &Pool<Postgres>, user_id: &Uuid) -> Result<Vec<PlayerCard>, String> {    let rows = sqlx::query(
        r#"
        SELECT card_id, rarity
        FROM player_cards
        WHERE user_id = $1
        ORDER BY card_id, rarity
        "#,
    )
    .bind(user_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| PlayerCard {
            card_id: r.get("card_id"),
            rarity: r.get("rarity"),
        })
        .collect())
}

pub async fn get_player_deck(db_pool: &Pool<Postgres>, user_id: &Uuid) -> Result<Vec<DeckEntry>, String> {    let rows = sqlx::query(
        r#"
        SELECT pc.card_id,
               COALESCE(pd.rarity, pc.min_rarity) AS rarity
        FROM (
            SELECT card_id, MIN(rarity) AS min_rarity
            FROM player_cards
            WHERE user_id = $1
            GROUP BY card_id
        ) pc
        LEFT JOIN player_deck pd
            ON pd.user_id = $1 AND pd.card_id = pc.card_id
        ORDER BY pc.card_id
        "#,
    )
    .bind(user_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| DeckEntry {
            card_id: r.get("card_id"),
            rarity: r.get("rarity"),
        })
        .collect())
}

pub async fn ensure_default_cards(db_pool: &Pool<Postgres>, user_id: &Uuid) -> Result<(), String> {
    let mut tx = db_pool.begin().await.map_err(|e| e.to_string())?;

    for card_id in DEFAULT_COMMON_CARDS {
        sqlx::query(
            r#"
            INSERT INTO player_cards (user_id, card_id, rarity)
            VALUES ($1, $2, 0)
            ON CONFLICT (user_id, card_id, rarity) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(card_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            INSERT INTO player_deck (user_id, card_id, rarity)
            VALUES ($1, $2, 0)
            ON CONFLICT (user_id, card_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(card_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn set_deck_rarity(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    card_id: &str,
    rarity: i16,
) -> Result<(), String> {
    let owned = sqlx::query(
        r#"
        SELECT 1 AS one FROM player_cards
        WHERE user_id = $1 AND card_id = $2 AND rarity = $3
        "#,
    )
    .bind(user_id)
    .bind(card_id)
    .bind(rarity)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    if owned.is_none() {
        return Err("card rarity not owned".to_string());
    }

    sqlx::query(
        r#"
        INSERT INTO player_deck (user_id, card_id, rarity)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, card_id) DO UPDATE SET rarity = EXCLUDED.rarity
        "#,
    )
    .bind(user_id)
    .bind(card_id)
    .bind(rarity)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn grant_card(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    card_id: &str,
    rarity: i16,
) -> Result<(), String> {
    let mut tx = db_pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        INSERT INTO player_cards (user_id, card_id, rarity)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, card_id, rarity) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(card_id)
    .bind(rarity)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        INSERT INTO player_deck (user_id, card_id, rarity)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, card_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(card_id)
    .bind(rarity)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn remove_card_rarity(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
    card_id: &str,
    rarity: i16,
) -> Result<(), String> {
    let mut tx = db_pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        DELETE FROM player_cards
        WHERE user_id = $1 AND card_id = $2 AND rarity = $3
        "#,
    )
    .bind(user_id)
    .bind(card_id)
    .bind(rarity)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        UPDATE player_deck pd
        SET rarity = COALESCE(
            (SELECT MIN(rarity) FROM player_cards pc
             WHERE pc.user_id = pd.user_id AND pc.card_id = pd.card_id),
            0
        )
        WHERE pd.user_id = $1 AND pd.card_id = $2 AND pd.rarity = $3
        "#,
    )
    .bind(user_id)
    .bind(card_id)
    .bind(rarity)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        DELETE FROM player_deck pd
        WHERE pd.user_id = $1
          AND pd.card_id = $2
          AND NOT EXISTS (
            SELECT 1 FROM player_cards pc
            WHERE pc.user_id = pd.user_id AND pc.card_id = pd.card_id
          )
        "#,
    )
    .bind(user_id)
    .bind(card_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

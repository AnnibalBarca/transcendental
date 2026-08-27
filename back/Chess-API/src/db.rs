use sqlx::{Pool, Postgres, Row};

pub async fn get_player_deck(db_pool: &Pool<Postgres>, user_id: &str) -> Result<Vec<(String, i16)>, String> {
    let rows = sqlx::query(
        r#"
        SELECT pc.card_id,
               COALESCE(pd.rarity, pc.min_rarity) AS rarity
        FROM (
            SELECT card_id, MIN(rarity) AS min_rarity
            FROM player_cards
            WHERE user_id = $1::uuid
            GROUP BY card_id
        ) pc
        LEFT JOIN player_deck pd
            ON pd.user_id = $1::uuid AND pd.card_id = pc.card_id
        "#,
    )
    .bind(user_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| (r.get("card_id"), r.get("rarity")))
        .collect())
}

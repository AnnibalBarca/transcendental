use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PackReward {
    pub item_id: String,
    pub item_type: String,
    pub title: String,
    pub price: i64,
    pub is_duplicate: bool,
    pub refunded: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<i16>,
}

#[derive(Clone)]
struct Candidate {
    item_id: String,
    item_type: String,
    title: String,
    price: i64,
}

fn weight_for_price(price: i64) -> i64 {
    if price >= 1000 {
        5
    } else if price >= 800 {
        12
    } else if price >= 600 {
        25
    } else if price >= 250 {
        40
    } else {
        50
    }
}

fn weighted_pick(pool: &[Candidate]) -> &Candidate {
    let weights: Vec<i64> = pool.iter().map(|c| weight_for_price(c.price)).collect();
    let total: i64 = weights.iter().sum();
    let mut r = rand::random::<i64>().rem_euclid(total.max(1));
    for (i, w) in weights.iter().enumerate() {
        if r < *w {
            return &pool[i];
        }
        r -= *w;
    }
    &pool[0]
}

fn cosmetic_rarity_char(price: i64) -> &'static str {
    if price >= 250 {
        "3"
    } else if price >= 150 {
        "2"
    } else if price >= 80 {
        "1"
    } else {
        "0"
    }
}

pub const RARITY_ELIGIBLE_CARDS: &[&str] = &["0", "4", "13", "14", "17", "20", "21", "22", "27"];

pub fn max_rarity_for_card(card_id: &str) -> i16 {
    if RARITY_ELIGIBLE_CARDS.contains(&card_id) {
        2
    } else {
        0
    }
}

fn random_rarity(max_rarity: i16) -> i16 {
    let r = rand::random::<u32>().rem_euclid(100);
    match max_rarity {
        2 => {
            if r < 10 {
                2
            } else if r < 30 {
                1
            } else {
                0
            }
        }
        1 => {
            if r < 25 {
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

async fn load_cosmetics(db: &Pool<Postgres>) -> Result<Vec<Candidate>, String> {
    let rows = sqlx::query(
        r#"
        SELECT item_id, item_type, title, price
        FROM shop_catalog
        WHERE is_active = TRUE
          AND price > 0
          AND item_type IN ('base','hat','mask','clothes','accessory')
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|r| Candidate {
            item_id: r.get("item_id"),
            item_type: r.get("item_type"),
            title: r.get("title"),
            price: r.get("price"),
        })
        .collect())
}

async fn load_cards(db: &Pool<Postgres>) -> Result<Vec<Candidate>, String> {
    let rows = sqlx::query(
        r#"
        SELECT item_id, item_type, title, price
        FROM shop_catalog
        WHERE is_active = TRUE
          AND price > 0
          AND item_type = 'card'
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|r| Candidate {
            item_id: r.get("item_id"),
            item_type: r.get("item_type"),
            title: r.get("title"),
            price: r.get("price"),
        })
        .collect())
}

pub async fn open_skins(
    db: &Pool<Postgres>,
    user_id: &Uuid,
    count: usize,
    pack_price: i64,
    dup_refund: i64,
) -> Result<(i64, Vec<PackReward>), String> {
    let pool = load_cosmetics(db).await?;
    if pool.is_empty() {
        return Err("no cosmetics available".to_string());
    }

    let mut tx = db.begin().await.map_err(|e| e.to_string())?;

    let wallet_after_buy = {
        let row = sqlx::query(
            r#"
            UPDATE users SET wallet = wallet - $1, updated_at = NOW()
            WHERE id = $2 AND wallet >= $1
            RETURNING wallet
            "#,
        )
        .bind(pack_price)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        match row {
            Some(r) => r.get::<i64, _>("wallet"),
            None => return Err("insufficient funds".to_string()),
        }
    };

    let mut rewards = Vec::with_capacity(count);
    let mut refund_total = 0i64;

    for _ in 0..count {
        let pick = weighted_pick(&pool).clone();
        let owned = sqlx::query(
            "SELECT 1 FROM player_inventory WHERE user_id = $1 AND item_id = $2 AND item_type = $3",
        )
        .bind(user_id)
        .bind(&pick.item_id)
        .bind(&pick.item_type)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .is_some();

        if owned {
            refund_total += dup_refund;
            rewards.push(PackReward {
                is_duplicate: true,
                refunded: dup_refund,
                item_id: pick.item_id,
                item_type: pick.item_type,
                title: pick.title,
                price: pick.price,
                rarity: None,
            });
        } else {
            sqlx::query(
                r#"
                INSERT INTO player_inventory (user_id, item_id, item_type, item_rarity)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (user_id, item_id, item_type) DO NOTHING
                "#,
            )
            .bind(user_id)
            .bind(&pick.item_id)
            .bind(&pick.item_type)
            .bind(cosmetic_rarity_char(pick.price))
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            rewards.push(PackReward {
                is_duplicate: false,
                refunded: 0,
                item_id: pick.item_id,
                item_type: pick.item_type,
                title: pick.title,
                price: pick.price,
                rarity: None,
            });
        }
    }

    let final_wallet = if refund_total > 0 {
        let row = sqlx::query(
            "UPDATE users SET wallet = wallet + $1, updated_at = NOW() WHERE id = $2 RETURNING wallet",
        )
        .bind(refund_total)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        row.get("wallet")
    } else {
        wallet_after_buy
    };

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok((final_wallet, rewards))
}

pub async fn open_cards(
    db: &Pool<Postgres>,
    user_id: &Uuid,
    count: usize,
    pack_price: i64,
    dup_refund: i64,
) -> Result<(i64, Vec<PackReward>), String> {
    let pool = load_cards(db).await?;
    if pool.is_empty() {
        return Err("no cards available".to_string());
    }

    let mut tx = db.begin().await.map_err(|e| e.to_string())?;

    let wallet_after_buy = {
        let row = sqlx::query(
            r#"
            UPDATE users SET wallet = wallet - $1, updated_at = NOW()
            WHERE id = $2 AND wallet >= $1
            RETURNING wallet
            "#,
        )
        .bind(pack_price)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        match row {
            Some(r) => r.get::<i64, _>("wallet"),
            None => return Err("insufficient funds".to_string()),
        }
    };

    let mut rewards = Vec::with_capacity(count);
    let mut refund_total = 0i64;

    for _ in 0..count {

        let pick = pool[rand::random::<usize>().rem_euclid(pool.len())].clone();
        let max_rarity = max_rarity_for_card(&pick.item_id);
        let rarity = random_rarity(max_rarity);

        let owned = sqlx::query(
            "SELECT 1 FROM player_cards WHERE user_id = $1 AND card_id = $2 AND rarity = $3",
        )
        .bind(user_id)
        .bind(&pick.item_id)
        .bind(rarity)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .is_some();

        if owned {
            refund_total += dup_refund;
            rewards.push(PackReward {
                is_duplicate: true,
                refunded: dup_refund,
                item_id: pick.item_id,
                item_type: pick.item_type,
                title: pick.title,
                price: pick.price,
                rarity: Some(rarity),
            });
        } else {
            sqlx::query(
                r#"
                INSERT INTO player_cards (user_id, card_id, rarity)
                VALUES ($1, $2, $3)
                ON CONFLICT (user_id, card_id, rarity) DO NOTHING
                "#,
            )
            .bind(user_id)
            .bind(&pick.item_id)
            .bind(rarity)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            rewards.push(PackReward {
                is_duplicate: false,
                refunded: 0,
                item_id: pick.item_id,
                item_type: pick.item_type,
                title: pick.title,
                price: pick.price,
                rarity: Some(rarity),
            });
        }
    }

    let final_wallet = if refund_total > 0 {
        let row = sqlx::query(
            "UPDATE users SET wallet = wallet + $1, updated_at = NOW() WHERE id = $2 RETURNING wallet",
        )
        .bind(refund_total)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        row.get("wallet")
    } else {
        wallet_after_buy
    };

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok((final_wallet, rewards))
}

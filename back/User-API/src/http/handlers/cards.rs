use crate::AppContext;
use crate::db::cards;
use api_core::auth::validate_and_get_claims;
use api_core::http::response::json_error;
use api_core::types::ServiceRequest;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

async fn authed_user_id(ctx: &AppContext, request: &ServiceRequest) -> Option<Uuid> {
    let token = request.cookies.get("access_token")?;
    let mut conn = ctx.redis_pool.get().await.ok()?;
    let claims = validate_and_get_claims(&mut conn, token).await.ok()?;
    Uuid::parse_str(&claims.sub).ok()
}

pub async fn handle_get_cards(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
    let user_id = match authed_user_id(ctx, request).await {
        Some(id) => id,
        None => return json_error(401, "Missing or invalid access token"),
    };    let cards = match cards::get_player_cards(ctx.db.get_pool(), &user_id).await {
        Ok(cards) => cards,
        Err(e) => {
            log::error!("[User] Failed to get player cards: {}", e);
            return json_error(500, "Failed to get cards");
        }
    };

    json!({
        "status": 200,
        "cards": cards,
    })
}

pub async fn handle_get_deck(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
    let user_id = match authed_user_id(ctx, request).await {
        Some(id) => id,
        None => return json_error(401, "Missing or invalid access token"),
    };

    let deck = match cards::get_player_deck(ctx.db.get_pool(), &user_id).await {        Ok(deck) => deck,
        Err(e) => {
            log::error!("[User] Failed to get player deck: {}", e);
            return json_error(500, "Failed to get deck");
        }
    };

    json!({
        "status": 200,
        "deck": deck,
    })
}

#[derive(Deserialize)]
struct SetDeckRarityRequest {
    card_id: String,
    rarity: i16,
}

pub async fn handle_set_deck_rarity(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
    let user_id = match authed_user_id(ctx, request).await {
        Some(id) => id,
        None => return json_error(401, "Missing or invalid access token"),
    };

    let body: SetDeckRarityRequest = match serde_json::from_str(&request.body) {        Ok(b) => b,
        Err(_) => return json_error(400, "Invalid request body"),
    };

    match cards::set_deck_rarity(ctx.db.get_pool(), &user_id, &body.card_id, body.rarity).await {
        Ok(()) => json!({
            "status": 200,
            "message": "Deck updated",
        }),
        Err(e) => match e.as_str() {
            "card rarity not owned" => json_error(409, "You do not own this card rarity"),
            _ => json_error(500, "Failed to update deck"),
        },
    }
}

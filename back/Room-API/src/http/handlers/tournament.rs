use crate::http::response::json_error;
use crate::services::tournament as tournament_service;
use crate::types::ServiceRequest;
use crate::utils::extract_user_id_from_access_token;
use serde_json::json;

pub async fn handle_status(
    redis_pool: &deadpool_redis::Pool,
    tournament_id: &str,
) -> serde_json::Value {
    match tournament_service::get_tournament(redis_pool, tournament_id).await {
        Ok(record) => json!({
            "status": 200,
            "tournament": record,
        }),
        Err(e) => json_error(404, &e),
    }
}

pub async fn handle_list(redis_pool: &deadpool_redis::Pool) -> serde_json::Value {
    match tournament_service::list_tournaments(redis_pool).await {
        Ok(records) => json!({
            "status": 200,
            "tournaments": records,
        }),
        Err(e) => json_error(500, &e),
    }
}

pub async fn handle_my(
    redis_pool: &deadpool_redis::Pool,
    request: &ServiceRequest,
) -> serde_json::Value {
    let token = match request.cookies.get("access_token") {
        Some(t) => t,
        None => return json_error(401, "Missing access token"),
    };
    let user_id = match extract_user_id_from_access_token(token) {
        Ok(id) => id,
        Err(e) => return e,
    };

    match tournament_service::get_user_tournament(redis_pool, &user_id).await {
        Ok(Some(record)) => json!({
            "status": 200,
            "tournament": record,
        }),
        Ok(None) => json!({
            "status": 200,
            "tournament": null,
        }),
        Err(e) => json_error(500, &e),
    }
}

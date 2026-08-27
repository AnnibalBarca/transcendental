use api_core::http::response::json_error;
use log::{error, info};
use serde::Deserialize;
use serde_json::json;

use crate::AppContext;
use crate::db::user::delete_user;
use api_core::types::ServiceRequest;

#[derive(Deserialize)]
struct CleanupPayload {
    user_id: String,
}

pub async fn handle_delete_user(ctx: &AppContext, request: &ServiceRequest) -> serde_json::Value {
    info!("receive delete message");

    let payload: CleanupPayload = match serde_json::from_str(&request.body) {
        Ok(p) => p,
        Err(e) => {
            error!("[User] Failed to parse cleanup payload: {}", e);
            return json_error(400, "Invalid request body");
        }
    };

    match delete_user(&ctx.db.pool, &payload.user_id).await {
        Ok(_) => {}
        Err(e) => {
            error!("[User] Failed to delete user {}: {}", payload.user_id, e);
            return json_error(500, "Internal server error");
        }
    }

    let extracted_user_id = payload.user_id;

    info!("deleted_user success :{}", extracted_user_id);
    json!({
        "status": "success",
        "deleted_user": extracted_user_id
    })
}

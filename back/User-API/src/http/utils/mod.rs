use api_core::http::response::json_error;
use uuid::Uuid;

pub fn parse_user_id(id_str: &str) -> Result<Uuid, serde_json::Value> {
    Uuid::parse_str(id_str).map_err(|_| json_error(400, "Invalid user id"))
}

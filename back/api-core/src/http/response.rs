use serde::Serialize;
use serde_json::json;

pub fn json_error(status: u16, message: &str) -> serde_json::Value {
    json!({
        "status": status,
        "error": message
    })
}

pub fn json_success<T: Serialize>(data: T) -> serde_json::Value {
    json!({
        "status": 200,
        "data": data
    })
}

pub fn json_user<T: Serialize>(payload: T) -> serde_json::Value {
    json!({
        "status": 200,
        "user": payload
    })
}

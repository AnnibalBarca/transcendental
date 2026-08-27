use crate::types::RoomPayload;
use serde_json::json;

pub fn json_error(status: u16, message: &str) -> serde_json::Value {
    json!({
        "status": status,
        "error": message
    })
}

pub fn json_room(payload: RoomPayload) -> serde_json::Value {
    json!({
        "status": 200,
        "user": payload
    })
}

pub fn json_ok(message: &str) -> serde_json::Value {
    json!({
        "status": 200,
        "message": message
    })
}

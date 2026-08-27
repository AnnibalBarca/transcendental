use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use api_core::types::ServiceRequest;

#[derive(Debug, Serialize)]
pub struct RoomPayload {
    pub room_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub title: Option<String>,
    pub private: bool,
    pub player_count: u32,
    pub bot_difficulty: Option<String>,
}

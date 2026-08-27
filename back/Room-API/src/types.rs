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
    pub max_players: u32,
    pub bot_difficulty: Option<String>,
    #[serde(default)]
    pub time_control: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct JoinRoomRequest {
    pub room_id: Option<String>,
    pub join_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StartRoomRequest {
    pub room_id: String,
}

#[derive(Debug, Deserialize)]
pub struct KickRoomRequest {
    pub room_id: String,
    pub user_id: String,
    #[serde(default)]
    pub ban: bool,
}

#[derive(Debug, Serialize)]
pub struct RoomListItem {
    pub id: String,
    pub title: Option<String>,
    pub host_username: String,
    pub player_count: u32,
    pub max_players: u32,
    pub created_at: i64,
    pub private: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_code: Option<String>,
    pub mode: String,
}

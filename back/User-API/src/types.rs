use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: String,
    pub username: Option<String>,
    pub email: String,
    #[serde(default)]
    pub account_validated: bool,
    #[serde(default)]
    pub email_validated: bool,
    pub auth_provider: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub is_banned: bool,
    pub wallet: i64,
    pub ranked_elo: i32,
    pub level: i32,
    pub xp: i64,
    #[serde(default)]
    pub picture_id: String,
    #[serde(default)]
    pub bio: String,
    #[serde(default)]
    pub github: String,
    #[serde(default)]
    pub discord: String,
    #[serde(default)]
    pub twitter: String,
    #[serde(default)]
    pub is_private: bool,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleRecord {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRecord {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub routes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRecord {
    pub id: i32,
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub requests_per_minute: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct UserPayload {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chess_game_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chess_ws_url: Option<String>,
    pub account_validated: bool,
    pub email_validated: bool,
    pub access_token_expires_in: Option<i64>,
    pub auth_provider: String,
    pub wallet: i64,
    pub ranked_elo: i32,
    pub level: i32,
    pub xp: i64,
    pub xp_progress: f64,
    #[serde(default)]
    pub picture_id: String,
    #[serde(default)]
    pub has_panel_access: bool,
    #[serde(default)]
    pub bio: String,
    #[serde(default)]
    pub github: String,
    #[serde(default)]
    pub discord: String,
    #[serde(default)]
    pub twitter: String,
    #[serde(default)]
    pub is_private: bool,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub lang: String,
}

#[derive(Debug, Serialize)]
pub struct UserStatePayload {
    pub state: String,
    pub room_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRecord {
    pub id: String,
    pub title: String,
    pub price: i32,
    pub end_date: String,
    #[serde(default)]
    pub items: Vec<CollectionItemRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionItemRecord {
    pub item_id: String,
    pub item_type: String,
    pub title: String,
    pub price: i64,
    pub asset_key: String,
}


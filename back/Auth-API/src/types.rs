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
}

impl From<crate::db::models::User> for UserRecord {
    fn from(user: crate::db::models::User) -> Self {
        UserRecord {
            id: user.id,
            username: user.username,
            email: user.email,
            account_validated: user.account_validated,
            email_validated: user.email_validated,
            auth_provider: user.auth_provider,
        }
    }
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
    pub account_validated: bool,
    pub email_validated: bool,
}

#[derive(Debug, Serialize)]
pub struct UserStatePayload {
    pub state: String,
    pub room_id: Uuid,
}

use api_core::sse::{ConnectionMetadata, SseEventWithMetadata, SsePublisher, SseTarget};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMetadata {
    pub user_id: Uuid,
}

impl ConnectionMetadata for NotificationMetadata {
    fn routing_keys(&self) -> Vec<String> {
        vec![format!("user:{}", self.user_id)]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTarget {
    pub user_id: Uuid,
}

impl SseTarget<NotificationMetadata> for NotificationTarget {
    fn routing_keys(&self) -> Vec<String> {
// user_id nil = broadcast à toutes les connexions SSE
        if self.user_id.is_nil() {
            Vec::new()
        } else {
            vec![format!("user:{}", self.user_id)]
        }
    }

    fn matches(&self, metadata: &NotificationMetadata) -> bool {
        self.user_id.is_nil() || self.user_id == metadata.user_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveGame {
    pub game_id: String,
    pub kind: String,
    pub player1: Option<String>,
    pub player2: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicRoom {
    pub id: String,
    pub title: Option<String>,
    pub host_username: String,
    pub player_count: u32,
    pub max_players: u32,
    pub created_at: i64,
    pub private: bool,
    pub join_code: Option<String>,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomPlayerInfo {
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomStatePayload {
    pub room_id: Uuid,
    pub title: Option<String>,
    pub private: bool,
    pub join_code: Option<String>,
    pub host_id: Uuid,
    pub players: Vec<RoomPlayerInfo>,
    pub max_players: u32,
    pub status: String,
    pub time_control: Option<u32>,
    pub game_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum NotificationEvent {
    SetState {
        user_id: Uuid,
        state: String,
        room_id: Option<Uuid>,
        chess_ws_url: Option<String>,
        chess_game_id: Option<String>,
    },
    RoomUpdate {
        room: RoomStatePayload,
    },
    FriendRequest {
        from_user_id: Uuid,
        username: String,
    },
    FriendRequestAccepted {
        by_user_id: Uuid,
        username: String,
    },
    FriendRequestRefused {
        by_user_id: Uuid,
        username: String,
    },
    FriendRequestCancelled {
        by_user_id: Uuid,
        username: String,
    },
    FriendRemoved {
        by_user_id: Uuid,
        username: String,
    },
    NewMessage {
        from_user_id: Uuid,
        username: String,
        content: String,
    },
    ProfilePictureUpdated {
        user_id: Uuid,
        picture_id: String,
    },
    LiveGames {
        games: Vec<LiveGame>,
    },
    PublicRooms {
        rooms: Vec<PublicRoom>,
    },
}

impl SseEventWithMetadata<NotificationMetadata> for NotificationEvent {}

#[derive(Clone)]
pub struct NotificationBus {
    publisher: SsePublisher<NotificationEvent, NotificationTarget>,
}

impl NotificationBus {
    pub fn new(publisher: SsePublisher<NotificationEvent, NotificationTarget>) -> Self {
        Self { publisher }
    }

    pub async fn send_to_user(&self, user_id: Uuid, event: &NotificationEvent) {
        let target = NotificationTarget { user_id };
        self.publisher.publish(target, event).await;
    }

    pub async fn broadcast(&self, event: &NotificationEvent) {
        let target = NotificationTarget {
            user_id: Uuid::nil(),
        };
        self.publisher.publish(target, event).await;
    }
}

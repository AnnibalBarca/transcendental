use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerId {
    #[serde(rename = "player1")]
    Player1,
    #[serde(rename = "player2")]
    Player2,
}

impl PlayerId {
    pub fn idx(self) -> usize {
        match self {
            PlayerId::Player1 => 0,
            PlayerId::Player2 => 1,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PlayerId::Player1 => "player1",
            PlayerId::Player2 => "player2",
        }
    }

    pub fn other(self) -> Self {
        match self {
            PlayerId::Player1 => PlayerId::Player2,
            PlayerId::Player2 => PlayerId::Player1,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OutgoingMessage {
    pub action: String,
    pub from: Option<PlayerId>,
    pub message: Value,
}

#[derive(Debug)]
pub struct PlayerSlot {
    pub id: PlayerId,
    pub sender: mpsc::UnboundedSender<OutgoingMessage>,
    pub username: String,
    pub user_id: String,
    pub picture_id: String,
}

#[derive(Debug)]
pub struct LobbyState {
    pub players: [Option<PlayerSlot>; 2],

    pub user_slot_map: HashMap<String, PlayerId>,
}

impl Default for LobbyState {
    fn default() -> Self {
        Self::new()
    }
}

impl LobbyState {
    pub fn new() -> Self {
        Self {
            players: [None, None],
            user_slot_map: HashMap::new(),
        }
    }

    pub fn connect(
        &mut self,
        sender: mpsc::UnboundedSender<OutgoingMessage>,
        username: String,
        user_id: String,
        picture_id: String,
    ) -> Option<PlayerId> {

        if let Some(&existing_id) = self.user_slot_map.get(&user_id) {
            let idx = existing_id.idx();
            if self.players[idx].is_none() {
                info!("[Lobby] {} reconnected to slot {}", username, existing_id.label());
                self.players[idx] = Some(PlayerSlot {
                    id: existing_id,
                    sender,
                    username,
                    user_id: user_id.clone(),
                    picture_id,
                });
                return Some(existing_id);
            }

            if let Some(slot) = &self.players[idx] {
                if slot.user_id == user_id {
                    info!("[Lobby] {} replacing stale session on slot {}", username, existing_id.label());
                    self.players[idx] = Some(PlayerSlot {
                        id: existing_id,
                        sender,
                        username,
                        user_id: user_id.clone(),
                        picture_id,
                    });
                    return Some(existing_id);
                }
            }
        }

        if self.players[0].is_none() {
            let id = PlayerId::Player1;
            self.players[0] = Some(PlayerSlot {
                id,
                sender,
                username: username.clone(),
                user_id: user_id.clone(),
                picture_id,
            });
            self.user_slot_map.insert(user_id, id);
            Some(id)
        } else if self.players[1].is_none() {
            let id = PlayerId::Player2;
            self.players[1] = Some(PlayerSlot {
                id,
                sender,
                username: username.clone(),
                user_id: user_id.clone(),
                picture_id,
            });
            self.user_slot_map.insert(user_id, id);
            Some(id)
        } else {
            None
        }
    }

    pub fn disconnect(&mut self, id: PlayerId, left_for_good: bool) {
        let idx = id.idx();
        if self.players[idx].is_some() {
            info!("[Lobby] {} disconnected, slot kept for reconnection", id.label());
            self.players[idx] = None;

            if left_for_good {
                self.broadcast(OutgoingMessage {
                    action: "opponent_left".to_string(),
                    from: None,
                    message: json!({"message": "Your opponent left the game"}),
                });
            } else {
                self.broadcast(OutgoingMessage {
                    action: "opponent_disconnected".to_string(),
                    from: None,
                    message: json!({"message": "Your opponent disconnected, waiting for return..."}),
                });
            }
        }
    }

    pub fn both_connected(&self) -> bool {
        self.players[0].is_some() && self.players[1].is_some()
    }

    pub fn color_user_ids(&self) -> (Option<String>, Option<String>) {
        let mut white = None;
        let mut black = None;
        for (uid, pid) in &self.user_slot_map {
            match pid {
                PlayerId::Player1 => white = Some(uid.clone()),
                PlayerId::Player2 => black = Some(uid.clone()),
            }
        }
        (white, black)
    }

    pub fn send_to(&self, id: PlayerId, msg: OutgoingMessage) {
        if let Some(slot) = &self.players[id.idx()] {
            if let Err(e) = slot.sender.send(msg) {
                warn!("[Lobby] Failed to send to {}: {}", id.label(), e);
            }
        }
    }

    pub fn send_to_player1(&self, msg: OutgoingMessage) {
        self.send_to(PlayerId::Player1, msg);
    }

    pub fn send_to_player2(&self, msg: OutgoingMessage) {
        self.send_to(PlayerId::Player2, msg);
    }

    pub fn broadcast(&self, msg: OutgoingMessage) {
        for slot in self.players.iter().flatten() {
            if let Err(e) = slot.sender.send(msg.clone()) {
                warn!("[Lobby] Failed to broadcast to {}: {}", slot.id.label(), e);
            }
        }
    }
}

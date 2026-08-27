use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Mutex;
use log::info;
use serde_json::json;

use crate::game::manager::core::GameInstance;
use crate::websocket::lobby::OutgoingMessage;

#[derive(Debug)]
pub struct GameManager {
    games: Mutex<HashMap<String, Arc<GameInstance>>>,
    db_pool: Option<PgPool>,
}

impl GameManager {
    pub fn new(db_pool: Option<PgPool>) -> Self {
        Self {
            games: Mutex::new(HashMap::new()),
            db_pool,
        }
    }

    pub async fn create_game(
        self: &Arc<Self>,
        game_id: String,
        redis_pool: deadpool_redis::Pool,
        initial_time_ms: u64,
    ) -> Arc<GameInstance> {
        let mut games = self.games.lock().await;

        if let Some(existing) = games.get(&game_id) {
            info!("[GameManager] Game {} already exists, returning it", game_id);
            return Arc::clone(existing);
        }

        info!("[GameManager] Creating new game {}", game_id);
        let manager = Arc::clone(self);
        let instance = Arc::new(GameInstance::new(
            game_id.clone(),
            redis_pool,
            initial_time_ms,
            self.db_pool.clone(),
            manager,
        ));
        games.insert(game_id, Arc::clone(&instance));

        instance
    }

    pub async fn get_game(&self, game_id: &str) -> Option<Arc<GameInstance>> {
        let games = self.games.lock().await;
        games.get(game_id).map(Arc::clone)
    }

    pub async fn remove_game(&self, game_id: &str) {
        let mut games = self.games.lock().await;
        if games.remove(game_id).is_some() {
            info!("[GameManager] Game {} removed", game_id);
        }
    }

    pub async fn game_count(&self) -> usize {
        let games = self.games.lock().await;
        games.len()
    }

    pub async fn broadcast_picture_update(&self, user_id: &str, picture_id: &str) {
        let games = self.games.lock().await;

        for instance in games.values() {
            let lobby = instance.lobby.lock().await;

            let color = lobby.players[0]
                .as_ref()
                .filter(|p| p.user_id == user_id)
                .map(|_| "white")
                .or_else(|| {
                    lobby.players[1]
                        .as_ref()
                        .filter(|p| p.user_id == user_id)
                        .map(|_| "black")
                });

            if let Some(color) = color {
                lobby.broadcast(OutgoingMessage {
                    action: "players_picture".to_string(),
                    from: None,
                    message: json!({
                        "color": color,
                        "picture_id": picture_id,
                    }),
                });
            }
        }
    }
}

impl Default for GameManager {
    fn default() -> Self {
        Self::new(None)
    }
}

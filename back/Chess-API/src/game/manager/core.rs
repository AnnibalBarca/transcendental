use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::game::game_loop::{run_game_loop, GameLoop};
use crate::game::manager::GameManager;
use crate::game::redis_helpers::cleanup_user_session;
use crate::websocket::lobby::LobbyState;

#[derive(Debug)]
pub struct GameInstance {
    pub game_id: String,
    pub game_loop: Arc<GameLoop>,
    pub lobby: Arc<Mutex<LobbyState>>,
    pub redis_pool: deadpool_redis::Pool,
    pub db_pool: Option<PgPool>,

    pub player_user_ids: Arc<Mutex<Vec<String>>>,
}

impl GameInstance {
    pub fn new(
        game_id: String,
        redis_pool: deadpool_redis::Pool,
        initial_time_ms: u64,
        db_pool: Option<PgPool>,
        manager: Arc<GameManager>,
    ) -> Self {
        let game_loop = Arc::new(GameLoop::new());
        game_loop.set_initial_time(initial_time_ms);
        let lobby = Arc::new(Mutex::new(LobbyState::new()));
        let player_user_ids = Arc::new(Mutex::new(Vec::new()));

        let game_loop_clone = Arc::clone(&game_loop);
        let lobby_clone = Arc::clone(&lobby);
        let pool_clone = redis_pool.clone();
        let ids_clone = Arc::clone(&player_user_ids);
        let gid_clone = game_id.clone();

        tokio::spawn(async move {
            let _ = run_game_loop(
                game_loop_clone,
                lobby_clone,
                pool_clone,
                ids_clone,
                gid_clone,
                manager,
            )
            .await;
        });

        Self {
            game_id,
            game_loop,
            lobby,
            redis_pool,
            db_pool,
            player_user_ids,
        }
    }

    pub async fn end_game_cleanup(&self) {
        let mut ids = self.player_user_ids.lock().await.clone();

        use deadpool_redis::redis::cmd;

        if let Ok(mut conn) = self.redis_pool.get().await {
            let players_key = format!("chess:game_players:{}", self.game_id);
            let data: Option<String> = cmd("GET")
                .arg(&players_key)
                .query_async(&mut *conn)
                .await
                .ok()
                .flatten();
            if let Some(data) = data {
                let players: Vec<String> = serde_json::from_str(&data).unwrap_or_default();
                for p in players {
                    if !p.is_empty() && !ids.contains(&p) {
                        ids.push(p);
                    }
                }
            }
        }

        for uid in ids {
            if !uid.is_empty() {
                cleanup_user_session(&self.redis_pool, &uid).await;
            }
        }

        let mapping_key = format!("chess:game_room:{}", self.game_id);
        if let Ok(mut conn) = self.redis_pool.get().await {
            let room_id: Option<String> = cmd("GET")
                .arg(&mapping_key)
                .query_async(&mut *conn)
                .await
                .ok()
                .flatten();

            if let Some(rid) = room_id {
                let room_key = format!("room:ranked:{}", rid);
                let _: deadpool_redis::redis::RedisResult<()> = cmd("DEL")
                    .arg(&room_key)
                    .query_async(&mut *conn)
                    .await;
                let _: deadpool_redis::redis::RedisResult<()> = cmd("SREM")
                    .arg("room:ranked:index")
                    .arg(&rid)
                    .query_async(&mut *conn)
                    .await;
            }

            let _: deadpool_redis::redis::RedisResult<()> = cmd("DEL")
                .arg(&mapping_key)
                .query_async(&mut *conn)
                .await;
        }
        log::info!("[GameInstance] End-game cleanup done for {}", self.game_id);
    }
}

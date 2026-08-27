use deadpool_redis::Pool;
use redis::AsyncCommands;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PlayerSession {
    pub room_id: String,
    pub status: String,
    pub chess_ws_url: String,
}

pub struct RedisSessionManager {
    pool: Pool,
}

impl RedisSessionManager {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn save_session(
        &self,
        user_id: &Uuid,
        session: &PlayerSession,
    ) -> Result<(), String> {
        let mut con = self
            .pool
            .get()
            .await
            .map_err(|e| format!("Erreur de pool Redis : {}", e))?;

        let redis_key = format!("user:session:{}", user_id);

        let data = [
            ("room_id", session.room_id.as_str()),
            ("status", session.status.as_str()),
            ("chess_ws_url", session.chess_ws_url.as_str()),
        ];

        let (): () = con
            .hset_multiple(&redis_key, &data)
            .await
            .map_err(|e| e.to_string())?;

        let (): () = con
            .expire(&redis_key, 7200)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn get_session(&self, user_id: &Uuid) -> Result<Option<PlayerSession>, String> {
        let mut con = self
            .pool
            .get()
            .await
            .map_err(|e| format!("Erreur de pool Redis : {}", e))?;

        let redis_key = format!("user:session:{}", user_id);

        let fields: std::collections::HashMap<String, String> =
            con.hgetall(&redis_key).await.map_err(|e| e.to_string())?;

        if fields.is_empty() {
            return Ok(None);
        }

        let session = PlayerSession {
            room_id: fields.get("room_id").cloned().unwrap_or_default(),
            status: fields.get("status").cloned().unwrap_or_default(),
            chess_ws_url: fields.get("chess_ws_url").cloned().unwrap_or_default(),
        };

        Ok(Some(session))
    }

    pub async fn delete_session(&self, user_id: &Uuid) -> Result<(), String> {
        let mut con = self
            .pool
            .get()
            .await
            .map_err(|e| format!("Erreur de pool Redis : {}", e))?;

        let redis_key = format!("user:session:{}", user_id);

        let (): () = con.del(&redis_key).await.map_err(|e| e.to_string())?;

        Ok(())
    }
}

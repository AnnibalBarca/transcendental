use deadpool_redis::Connection;
use redis::AsyncCommands;
use uuid::Uuid;

pub async fn generate_unique_user_id(pool: &deadpool_redis::Pool) -> Result<Uuid, String> {
    let mut con: Connection = pool
        .get()
        .await
        .map_err(|e| format!("Erreur de pool Redis : {}", e))?;

    loop {
        let candidate_id = Uuid::new_v4();
        let redis_key = format!("user:session:{}", candidate_id);

        let exists: i32 = con.exists(&redis_key).await.map_err(|e| e.to_string())?;

        if exists == 0 {
            return Ok(candidate_id);
        }

        println!(
            "Collision incroyable détectée pour l'ID {}, on régénère...",
            candidate_id
        );
    }
}

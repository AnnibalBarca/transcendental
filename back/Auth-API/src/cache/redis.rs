use deadpool_redis::Pool;

pub struct RedisCache {
    pool: Pool,
}

impl RedisCache {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    fn blacklist_key(token: &str) -> String {
        format!("blacklist:{}", token)
    }

    fn refresh_token_key(token_hash: &str) -> String {
        format!("refresh_token:{}", token_hash)
    }

    pub async fn blacklist_token(&self, token: &str, ttl: usize) -> Result<(), String> {
        let key = Self::blacklist_key(token);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl)
            .arg("revoked")
            .query_async::<_, ()>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn is_token_blacklisted(&self, token: &str) -> Result<bool, String> {
        let key = Self::blacklist_key(token);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn is_refresh_token_cached_valid(
        &self,
        token_hash: &str,
    ) -> Result<Option<bool>, String> {
        let key = Self::refresh_token_key(token_hash);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(if exists { Some(true) } else { None })
    }

    pub async fn invalidate_refresh_token(&self, token_hash: &str) -> Result<(), String> {
        let key = Self::refresh_token_key(token_hash);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        redis::cmd("DEL")
            .arg(&key)
            .query_async::<_, ()>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn invalidate_user_profile_cache(&self, user_id: &uuid::Uuid) -> Result<(), String> {
        let key = format!("user:profile:{}", user_id);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        redis::cmd("DEL")
            .arg(&key)
            .query_async::<_, ()>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn email_validation_key(user_id: &str) -> String {
        format!("email_validation:{}", user_id)
    }

    pub async fn get_email_validation_code(
        &self,
        user_id: &str,
    ) -> Result<Option<(String, i64)>, String> {
        let key = Self::email_validation_key(user_id);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        let value: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        match value {
            Some(v) => {
                let parts: Vec<&str> = v.split(':').collect();
                if parts.len() == 2 {
                    let code = parts[0].to_string();
                    let timestamp = parts[1].parse::<i64>().map_err(|e| e.to_string())?;
                    Ok(Some((code, timestamp)))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub async fn set_email_validation_code(
        &self,
        user_id: &str,
        code: &str,
        ttl_seconds: usize,
    ) -> Result<(), String> {
        let key = Self::email_validation_key(user_id);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;

        let value = format!("{}:{}", code, timestamp);

        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl_seconds)
            .arg(&value)
            .query_async::<_, ()>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn delete_email_validation_code(&self, user_id: &str) -> Result<(), String> {
        let key = Self::email_validation_key(user_id);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        redis::cmd("DEL")
            .arg(&key)
            .query_async::<_, ()>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn email_validated_key(user_id: &str) -> String {
        format!("email_validated:{}", user_id)
    }

    pub async fn set_email_validated(
        &self,
        user_id: &str,
        ttl_seconds: usize,
    ) -> Result<(), String> {
        let key = Self::email_validated_key(user_id);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl_seconds)
            .arg("true")
            .query_async::<_, ()>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn password_reset_key(token: &str) -> String {
        format!("password_reset:{}", token)
    }

    fn user_reset_token_key(user_id: &str) -> String {
        format!("user_reset_token:{}", user_id)
    }

    pub async fn set_password_reset_token(
        &self,
        token: &str,
        user_id: &str,
        ttl_seconds: usize,
    ) -> Result<(), String> {
        let key = Self::password_reset_key(token);
        let user_token_key = Self::user_reset_token_key(user_id);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        let old_token: Option<String> = redis::cmd("GET")
            .arg(&user_token_key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(old_token) = old_token {
            let old_key = Self::password_reset_key(&old_token);
            let _ = redis::cmd("DEL")
                .arg(&old_key)
                .query_async::<_, ()>(&mut *conn)
                .await;
        }

        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl_seconds)
            .arg(user_id)
            .query_async::<_, ()>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        redis::cmd("SETEX")
            .arg(&user_token_key)
            .arg(ttl_seconds)
            .arg(token)
            .query_async::<_, ()>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn get_current_reset_token(&self, user_id: &str) -> Result<Option<String>, String> {
        let key = Self::user_reset_token_key(user_id);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        redis::cmd("GET")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_password_reset_user_id(&self, token: &str) -> Result<Option<String>, String> {
        let key = Self::password_reset_key(token);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        redis::cmd("GET")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete_password_reset_token(&self, token: &str) -> Result<(), String> {
        let key = Self::password_reset_key(token);
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;

        redis::cmd("DEL")
            .arg(&key)
            .query_async::<_, ()>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_keys() {
        assert_eq!(RedisCache::blacklist_key("token123"), "blacklist:token123");
        assert_eq!(
            RedisCache::refresh_token_key("hash456"),
            "refresh_token:hash456"
        );
    }
}

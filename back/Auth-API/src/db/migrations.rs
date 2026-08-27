use api_core::db::migration::Migration;

pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration::new(
            "001_create_users_table",
            r#"
			CREATE TABLE IF NOT EXISTS users (
				id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
				username VARCHAR(255) UNIQUE,
				email VARCHAR(255) NOT NULL UNIQUE,
				password_hash VARCHAR(255) NOT NULL,
				account_validated BOOLEAN NOT NULL DEFAULT FALSE,
				email_validated BOOLEAN NOT NULL DEFAULT FALSE,
				wallet BIGINT NOT NULL DEFAULT 0 CHECK (wallet >= 0),
				role VARCHAR(50) DEFAULT 'player',
				auth_provider VARCHAR(255) NOT NULL,
				created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
				updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
			)
			"#,
        ),
        Migration::new(
            "002_create_users_indexes",
            r#"
			CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
			CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
			"#,
        ),
        Migration::new(
            "003_create_refresh_tokens_table",
            r#"
			CREATE TABLE IF NOT EXISTS refresh_tokens (
				id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
				user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
				token_hash VARCHAR(255) NOT NULL,
				expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
				created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
				revoked BOOLEAN NOT NULL DEFAULT FALSE
			)
			"#,
        ),
        Migration::new(
            "004_create_refresh_tokens_indexes",
            r#"
			CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id ON refresh_tokens(user_id);
			CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
			CREATE INDEX IF NOT EXISTS idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
			"#,
        ),
        Migration::new(
            "005_create_user_profile_table",
            r#"
			CREATE TABLE IF NOT EXISTS user_profile (
				user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
				ranked_elo INT NOT NULL DEFAULT 1500,
				picture_id TEXT NOT NULL DEFAULT '0',
				picture_updated_at TIMESTAMPTZ DEFAULT NOW()
			)
			"#,
        ),
        Migration::new(
            "006_create_games_table",
            r#"
			CREATE TABLE IF NOT EXISTS games (
				id BIGSERIAL PRIMARY KEY,
				game_id VARCHAR(255) NOT NULL UNIQUE,
				result VARCHAR(50) NOT NULL,
				winner VARCHAR(10),
				white_user_id UUID,
				black_user_id UUID,
				created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
			)
			"#,
        ),
        Migration::new(
            "007_create_games_indexes",
            r#"
			CREATE INDEX IF NOT EXISTS idx_games_created_at ON games(created_at);
			"#,
        ),
        Migration::new(
            "008_add_is_banned_to_users",
            r#"
			ALTER TABLE users ADD COLUMN IF NOT EXISTS is_banned BOOLEAN NOT NULL DEFAULT FALSE;
			"#,
        ),
        Migration::new(
            "009_allow_multiple_refresh_tokens",
            r#"
			ALTER TABLE refresh_tokens DROP CONSTRAINT IF EXISTS refresh_tokens_user_id_key;
			"#,
        ),
    ]
}

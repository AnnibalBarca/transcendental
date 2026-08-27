use crate::types::UserRecord;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

pub async fn get_by_id(db_pool: &Pool<Postgres>, id: &Uuid) -> Result<Option<UserRecord>, String> {
    let row = sqlx::query(
        r#"
        SELECT
            u.id, u.username, u.email, u.account_validated, u.email_validated,
            u.auth_provider, u.is_banned, u.wallet,
            COALESCE(p.ranked_elo, 0) AS ranked_elo,
            COALESCE(p.level, 1) AS level,
            COALESCE(p.xp, 0) AS xp,
            COALESCE(p.picture_id, '') AS picture_id,
            COALESCE(p.bio, '') AS bio,
            COALESCE(p.github, '') AS github,
            COALESCE(p.discord, '') AS discord,
            COALESCE(p.twitter, '') AS twitter,
            COALESCE(p.is_private, FALSE) AS is_private,
            COALESCE(p.theme, 'dark') AS theme,
            COALESCE(p.lang, 'fr') AS lang,
            COALESCE(array_agg(r.name) FILTER (WHERE r.name IS NOT NULL), ARRAY[]::text[]) AS roles
        FROM users u
        LEFT JOIN user_profile p ON p.user_id = u.id
        LEFT JOIN user_roles ur ON ur.user_id = u.id
        LEFT JOIN roles r ON r.id = ur.role_id
        WHERE u.id = $1
        GROUP BY u.id, u.username, u.email, u.account_validated, u.email_validated,
                 u.auth_provider, u.is_banned, u.wallet, p.ranked_elo,
                 p.level, p.xp, p.picture_id, p.bio, p.github, p.discord,
                 p.twitter, p.is_private, p.theme, p.lang
        "#,
    )
    .bind(id)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(row_to_record))
}

pub async fn get_by_username(
    db_pool: &Pool<Postgres>,
    username: &str,
) -> Result<Option<UserRecord>, String> {
    let row = sqlx::query(
        r#"
        SELECT
            u.id, u.username, u.email, u.account_validated, u.email_validated,
            u.auth_provider, u.is_banned, u.wallet,
            COALESCE(p.ranked_elo, 0) AS ranked_elo,
            COALESCE(p.level, 1) AS level,
            COALESCE(p.xp, 0) AS xp,
            COALESCE(p.picture_id, '') AS picture_id,
            COALESCE(p.bio, '') AS bio,
            COALESCE(p.github, '') AS github,
            COALESCE(p.discord, '') AS discord,
            COALESCE(p.twitter, '') AS twitter,
            COALESCE(p.is_private, FALSE) AS is_private,
            COALESCE(p.theme, 'dark') AS theme,
            COALESCE(p.lang, 'fr') AS lang,
            COALESCE(array_agg(r.name) FILTER (WHERE r.name IS NOT NULL), ARRAY[]::text[]) AS roles
        FROM users u
        LEFT JOIN user_profile p ON p.user_id = u.id
        LEFT JOIN user_roles ur ON ur.user_id = u.id
        LEFT JOIN roles r ON r.id = ur.role_id
        WHERE u.username = $1
        GROUP BY u.id, u.username, u.email, u.account_validated, u.email_validated,
                 u.auth_provider, u.is_banned, u.wallet, p.ranked_elo,
                 p.level, p.xp, p.picture_id, p.bio, p.github, p.discord,
                 p.twitter, p.is_private, p.theme, p.lang
        "#,
    )
    .bind(username)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(row_to_record))
}

fn row_to_record(r: sqlx::postgres::PgRow) -> UserRecord {
    UserRecord {
        id: r.get::<Uuid, _>("id").to_string(),
        username: r.get("username"),
        email: r.get("email"),
        account_validated: r.get("account_validated"),
        email_validated: r.get("email_validated"),
        auth_provider: r.get("auth_provider"),
        roles: r.get("roles"),
        is_banned: r.get("is_banned"),
        wallet: r.get("wallet"),
        ranked_elo: r.get("ranked_elo"),
        level: r.get("level"),
        xp: r.get("xp"),
        picture_id: r.get("picture_id"),
        bio: r.get("bio"),
        github: r.get("github"),
        discord: r.get("discord"),
        twitter: r.get("twitter"),
        is_private: r.get("is_private"),
        theme: r.get("theme"),
        lang: r.get("lang"),
    }
}

pub async fn list_users(
    db_pool: &Pool<Postgres>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<UserRecord>, i64), String> {
    let limit = limit.clamp(1, 100);
    let offset = offset.max(0);

    let total: i64 = sqlx::query("SELECT COUNT(*) FROM users")
        .fetch_one(db_pool)
        .await
        .map_err(|e| e.to_string())?
        .get(0);

    let rows = sqlx::query(
        r#"
        SELECT
            u.id, u.username, u.email, u.account_validated, u.email_validated,
            u.auth_provider, u.is_banned, u.wallet,
            COALESCE(p.ranked_elo, 0) AS ranked_elo,
            COALESCE(p.level, 1) AS level,
            COALESCE(p.xp, 0) AS xp,
            COALESCE(p.picture_id, '') AS picture_id,
            COALESCE(p.bio, '') AS bio,
            COALESCE(p.github, '') AS github,
            COALESCE(p.discord, '') AS discord,
            COALESCE(p.twitter, '') AS twitter,
            COALESCE(p.is_private, FALSE) AS is_private,
            COALESCE(p.theme, 'dark') AS theme,
            COALESCE(p.lang, 'fr') AS lang,
            COALESCE(array_agg(r.name) FILTER (WHERE r.name IS NOT NULL), ARRAY[]::text[]) AS roles
        FROM users u
        LEFT JOIN user_profile p ON p.user_id = u.id
        LEFT JOIN user_roles ur ON ur.user_id = u.id
        LEFT JOIN roles r ON r.id = ur.role_id
        GROUP BY u.id, u.username, u.email, u.account_validated, u.email_validated,
                 u.auth_provider, u.is_banned, u.wallet, p.ranked_elo,
                 p.level, p.xp, p.picture_id, p.bio, p.github, p.discord,
                 p.twitter, p.is_private, p.theme, p.lang
        ORDER BY u.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let users = rows
        .into_iter()
        .map(|r| UserRecord {
            id: r.get::<Uuid, _>("id").to_string(),
            username: r.get("username"),
            email: r.get("email"),
            account_validated: r.get("account_validated"),
            email_validated: r.get("email_validated"),
            auth_provider: r.get("auth_provider"),
            roles: r.get("roles"),
            is_banned: r.get("is_banned"),
            wallet: r.get("wallet"),
            ranked_elo: r.get("ranked_elo"),
            level: r.get("level"),
            xp: r.get("xp"),
            picture_id: r.get("picture_id"),
            bio: r.get("bio"),
            github: r.get("github"),
            discord: r.get("discord"),
            twitter: r.get("twitter"),
            is_private: r.get("is_private"),
            theme: r.get("theme"),
        lang: r.get("lang"),
        })
        .collect();

    Ok((users, total))
}

#[allow(clippy::too_many_arguments)]
pub async fn update_user(
    db_pool: &Pool<Postgres>,
    id: &Uuid,
    username: Option<&str>,
    email: Option<&str>,
    account_validated: Option<bool>,
    email_validated: Option<bool>,
    is_banned: Option<bool>,
    wallet: Option<i64>,
    ranked_elo: Option<i32>,
    xp: Option<i64>,
) -> Result<(), String> {
    if let Some(username) = username {
        sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
            .bind(username)
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(email) = email {
        sqlx::query("UPDATE users SET email = $1 WHERE id = $2")
            .bind(email)
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if account_validated.is_some() || email_validated.is_some() {
        let validated = account_validated.unwrap_or(false);
        let email_ok = email_validated.unwrap_or(false);
        sqlx::query(
            "UPDATE users SET account_validated = $1, email_validated = $2 WHERE id = $3",
        )
        .bind(validated)
        .bind(email_ok)
        .bind(id)
        .execute(db_pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    if let Some(wallet) = wallet {
        sqlx::query("UPDATE users SET wallet = $1 WHERE id = $2")
            .bind(wallet)
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(is_banned) = is_banned {
        sqlx::query("UPDATE users SET is_banned = $1 WHERE id = $2")
            .bind(is_banned)
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if ranked_elo.is_some() {
        sqlx::query("INSERT INTO user_profile (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING")
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;

        let ranked = ranked_elo.unwrap_or(0);
        sqlx::query("UPDATE user_profile SET ranked_elo = $1 WHERE user_id = $2")
            .bind(ranked)
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(xp) = xp {
        let xp = xp.clamp(0, crate::xp::MAX_XP);
        let level = crate::xp::level_from_xp(xp) as i32;
        sqlx::query(
            "INSERT INTO user_profile (user_id, xp, level) VALUES ($1, $2, $3) \
             ON CONFLICT (user_id) DO UPDATE SET xp = EXCLUDED.xp, level = EXCLUDED.level",
        )
        .bind(id)
        .bind(xp)
        .bind(level)
        .execute(db_pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub async fn admin_delete_user(db_pool: &Pool<Postgres>, id: &Uuid) -> Result<bool, String> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(db_pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.rows_affected() > 0)
}

pub async fn update_name(
    db_pool: &Pool<Postgres>,
    id: &Uuid,
    new_name: &str,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE users
        SET username = $1
        WHERE id = $2
        "#,
    )
    .bind(new_name)
    .bind(id)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn name_exists(db_pool: &Pool<Postgres>, name: &str) -> Result<bool, String> {
    let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
        .bind(name)
        .fetch_one(db_pool)
        .await
        .map_err(|e| e.to_string())?;

    let exists: bool = row.get(0);
    Ok(exists)
}

pub async fn update_profile_settings(
    db_pool: &Pool<Postgres>,
    id: &Uuid,
    bio: Option<&str>,
    github: Option<&str>,
    discord: Option<&str>,
    twitter: Option<&str>,
    is_private: Option<bool>,
    theme: Option<&str>,
    lang: Option<&str>,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO user_profile (user_id)
        VALUES ($1)
        ON CONFLICT (user_id) DO NOTHING
        "#,
    )
    .bind(id)
    .execute(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(bio) = bio {
        sqlx::query("UPDATE user_profile SET bio = $1 WHERE user_id = $2")
            .bind(bio)
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(github) = github {
        sqlx::query("UPDATE user_profile SET github = $1 WHERE user_id = $2")
            .bind(github)
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(discord) = discord {
        sqlx::query("UPDATE user_profile SET discord = $1 WHERE user_id = $2")
            .bind(discord)
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(twitter) = twitter {
        sqlx::query("UPDATE user_profile SET twitter = $1 WHERE user_id = $2")
            .bind(twitter)
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(is_private) = is_private {
        sqlx::query("UPDATE user_profile SET is_private = $1 WHERE user_id = $2")
            .bind(is_private)
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(theme) = theme {
        sqlx::query("UPDATE user_profile SET theme = $1 WHERE user_id = $2")
            .bind(theme)
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(lang) = lang {
        sqlx::query("UPDATE user_profile SET lang = $1 WHERE user_id = $2")
            .bind(lang)
            .bind(id)
            .execute(db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub async fn delete_user(db_pool: &Pool<Postgres>, id: &str) -> Result<bool, String> {
    let user_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("Invalid UUID: {}", e))?;

    let result_1 = sqlx::query("DELETE FROM friendships WHERE user_id = $1")
        .bind(user_uuid)
        .execute(db_pool)
        .await
        .map_err(|e| e.to_string())?;

    let result_2 = sqlx::query("DELETE FROM player_inventory WHERE user_id = $1")
        .bind(user_uuid)
        .execute(db_pool)
        .await
        .map_err(|e| e.to_string())?;

    let result_3 = sqlx::query("DELETE FROM friend_messages WHERE user_id = $1")
        .bind(user_uuid)
        .execute(db_pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result_1.rows_affected() + result_2.rows_affected() + result_3.rows_affected() > 0)
}

pub async fn are_friends(
    db_pool: &Pool<Postgres>,
    user_a: &Uuid,
    user_b: &Uuid,
) -> Result<bool, String> {
    let row = sqlx::query(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM friendships
            WHERE status = 'accepted'
              AND ((user_id = $1 AND friend_id = $2) OR (user_id = $2 AND friend_id = $1))
        )
        "#,
    )
    .bind(user_a)
    .bind(user_b)
    .fetch_one(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let is_friend: bool = row.get(0);
    Ok(is_friend)
}

pub async fn permissions_of_user(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT p.name
        FROM permissions p
        JOIN role_permissions rp ON rp.permission_id = p.id
        JOIN user_roles ur ON ur.role_id = rp.role_id
        WHERE ur.user_id = $1
        ORDER BY p.name
        "#,
    )
    .bind(user_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| r.get::<String, _>("name"))
        .collect())
}

pub async fn all_user_ids(db_pool: &Pool<Postgres>) -> Result<Vec<Uuid>, String> {
    let rows = sqlx::query("SELECT id FROM users ORDER BY id")
        .fetch_all(db_pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| r.get::<Uuid, _>("id")).collect())
}

pub async fn user_has_route_permission(
    db_pool: &Pool<Postgres>,
    user_id: &Uuid,
) -> Result<bool, String> {
    let row = sqlx::query(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM user_roles ur
            JOIN role_permissions rp ON rp.role_id = ur.role_id
            WHERE ur.user_id = $1
              AND EXISTS (
                SELECT 1 FROM permission_routes pr WHERE pr.permission_id = rp.permission_id
              )
        )
        "#,
    )
    .bind(user_id)
    .fetch_one(db_pool)
    .await
    .map_err(|e| e.to_string())?;
    let exists: bool = row.get(0);
    Ok(exists)
}

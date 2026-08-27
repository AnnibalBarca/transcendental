use api_core::cache::set_json;
use api_core::permission::{route_permission_key, user_permission_key};
use log::error;
use uuid::Uuid;

use crate::AppContext;
use crate::db::{permissions as perm_repo, user as user_repo};

const PERM_CACHE_TTL: usize = 86400;

pub async fn sync_permissions_to_redis(ctx: &AppContext) -> Result<(), String> {
    let routes = perm_repo::routes_with_permissions(ctx.db.get_pool()).await?;
    for (method, path, perms) in routes {
        let key = route_permission_key(&method, &path);
        set_json(&ctx.redis_pool, &key, &perms, PERM_CACHE_TTL).await?;
    }

    let user_ids = user_repo::all_user_ids(ctx.db.get_pool()).await?;
    for uid in user_ids {
        let key = user_permission_key(&uid.to_string());
        match user_repo::permissions_of_user(ctx.db.get_pool(), &uid).await {
            Ok(perms) => {
                let _ = set_json(&ctx.redis_pool, &key, &perms, PERM_CACHE_TTL).await;
            }
            Err(e) => error!("[Perm] Failed to resolve permissions for user {}: {}", uid, e),
        }
    }

    Ok(())
}

pub async fn sync_user_permissions(ctx: &AppContext, user_id: &Uuid) -> Result<(), String> {
    let perms = user_repo::permissions_of_user(ctx.db.get_pool(), user_id).await?;
    let key = user_permission_key(&user_id.to_string());
    set_json(&ctx.redis_pool, &key, &perms, PERM_CACHE_TTL).await?;
    Ok(())
}

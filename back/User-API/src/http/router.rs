use api_core::http::router::Router;
use api_core::types::ServiceRequest;

use crate::AppContext;
use crate::http::handlers::{
    add_item, admin, cards, change_username, get_inventory, get_profile_picture, health, me, pack,
    remove_item, set_profile_picture, settings, shop, state, user,
};

pub fn build_router() -> Router<AppContext> {
    let mut router = Router::new();

    router.register_public("GET", "health", |ctx: AppContext, _req: ServiceRequest| {
        Box::pin(async move { health::handle_health(&ctx).await })
    });

    router.register_public("GET", "shop", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { shop::handle_get_shop(&ctx, &req).await })
    });

    router.register_public(
        "POST",
        "shop/items",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { shop::handle_upload_item(&ctx, &req).await })
        },
    );

    router.register_public(
        "GET",
        "collections",
        |ctx: AppContext, _req: ServiceRequest| {
            Box::pin(async move { shop::handle_get_collections(&ctx).await })
        },
    );

    router.register_public(
        "POST",
        "collections/purchase",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { shop::handle_purchase_collection(&ctx, &req).await })
        },
    );

    router.register_public("POST", "packs/open", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { pack::handle_open_pack(&ctx, &req).await })
    });

    router.register_public("GET", "me", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { me::handle_me(&ctx, &req).await })
    });

    router.register_public("GET", "users/*", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { user::handle_user(&ctx, &req).await })
    });

    router.register_public("POST", "admin/users", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { admin::handle_admin_list(&ctx, &req).await })
    });

    router.register_public(
        "PATCH",
        "admin/users/*",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_update(&ctx, &req).await })
        },
    );

    router.register_public(
        "DELETE",
        "admin/users/*",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_delete(&ctx, &req).await })
        },
    );

    router.register_public("GET", "admin/roles", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { admin::handle_admin_roles_list(&ctx, &req).await })
    });

    router.register_public("POST", "admin/roles", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { admin::handle_admin_roles_create(&ctx, &req).await })
    });

    router.register_public("PATCH", "admin/roles/*", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { admin::handle_admin_roles_update(&ctx, &req).await })
    });

    router.register_public("DELETE", "admin/roles/*", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { admin::handle_admin_roles_delete(&ctx, &req).await })
    });

    router.register_public(
        "POST",
        "admin/roles/*/permissions",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_roles_add_permission(&ctx, &req).await })
        },
    );

    router.register_public(
        "DELETE",
        "admin/roles/*/permissions/*",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_roles_remove_permission(&ctx, &req).await })
        },
    );

    router.register_public(
        "GET",
        "admin/permissions",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_permissions_list(&ctx, &req).await })
        },
    );

    router.register_public(
        "POST",
        "admin/permissions",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_permissions_create(&ctx, &req).await })
        },
    );

    router.register_public(
        "PATCH",
        "admin/permissions/*",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_permissions_update(&ctx, &req).await })
        },
    );

    router.register_public(
        "DELETE",
        "admin/permissions/*",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_permissions_delete(&ctx, &req).await })
        },
    );

    router.register_public(
        "POST",
        "admin/permissions/*/routes",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_permissions_add_route(&ctx, &req).await })
        },
    );

    router.register_public(
        "DELETE",
        "admin/permissions/*/routes/*",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_permissions_remove_route(&ctx, &req).await })
        },
    );

    router.register_public("GET", "admin/routes", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { admin::handle_admin_routes_list(&ctx, &req).await })
    });

    router.register_public(
        "PUT",
        "admin/routes/*/rate-limit",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_routes_set_rate_limit(&ctx, &req).await })
        },
    );

    router.register_public(
        "POST",
        "admin/users/*/roles",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_users_add_role(&ctx, &req).await })
        },
    );

    router.register_public(
        "DELETE",
        "admin/users/*/roles/*",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_users_remove_role(&ctx, &req).await })
        },
    );

    router.register_public(
        "GET",
        "admin/users/*/cards",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_player_cards_list(&ctx, &req).await })
        },
    );

    router.register_public(
        "POST",
        "admin/users/*/cards",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_grant_card(&ctx, &req).await })
        },
    );

    router.register_public(
        "DELETE",
        "admin/users/*/cards/*/*",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { admin::handle_admin_remove_card_rarity(&ctx, &req).await })
        },
    );

    router.register_public("GET", "cards", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { cards::handle_get_cards(&ctx, &req).await })
    });

    router.register_public("GET", "deck", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { cards::handle_get_deck(&ctx, &req).await })
    });

    router.register_public("PUT", "deck/rarity", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { cards::handle_set_deck_rarity(&ctx, &req).await })
    });

    router.register_public("GET", "state", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { state::handle_state(&ctx, &req).await })
    });

    router.register_public(
        "GET",
        "inventory",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { get_inventory::handle_get_inventory(&ctx, &req).await })
        },
    );

    router.register_public(
        "GET",
        "profile-picture",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(
                async move { get_profile_picture::handle_get_profile_picture(&ctx, &req).await },
            )
        },
    );

    router.register_public(
        "PATCH",
        "change-username",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { change_username::handle_change_username(&ctx, &req).await })
        },
    );

    router.register_public(
        "POST",
        "inventory",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { add_item::handle_add_item(&ctx, &req).await })
        },
    );

    router.register_public(
        "DELETE",
        "inventory",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(async move { remove_item::handle_remove_item(&ctx, &req).await })
        },
    );

    router.register_public(
        "POST",
        "profile-picture",
        |ctx: AppContext, req: ServiceRequest| {
            Box::pin(
                async move { set_profile_picture::handle_set_profile_picture(&ctx, &req).await },
            )
        },
    );

    router.register_public("GET", "settings", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { settings::handle_get_settings(&ctx, &req).await })
    });

    router.register_public("PATCH", "settings", |ctx: AppContext, req: ServiceRequest| {
        Box::pin(async move { settings::handle_update_settings(&ctx, &req).await })
    });

    router
}

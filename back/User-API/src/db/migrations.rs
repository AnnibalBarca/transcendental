use api_core::db::migration::Migration;

pub fn all() -> Vec<Migration> {
    vec![
        Migration::new(
            "006_create_friendships",
            r#"
            CREATE TABLE IF NOT EXISTS friendships (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                friend_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'accepted', 'blocked')),
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                UNIQUE(user_id, friend_id)
            )
            "#,
        ),
        Migration::new(
            "007_create_friendships_indexes",
            r#"
            CREATE INDEX IF NOT EXISTS idx_friendships_user_id ON friendships(user_id);
            CREATE INDEX IF NOT EXISTS idx_friendships_friend_id ON friendships(friend_id);
            CREATE INDEX IF NOT EXISTS idx_friendships_status ON friendships(status);
            "#,
        ),
        Migration::new(
            "008_create_friend_messages",
            r#"
            CREATE TABLE IF NOT EXISTS friend_messages (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                sender_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                receiver_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                content TEXT NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
        ),
        Migration::new(
            "009_create_friend_messages_indexes",
            r#"
            CREATE INDEX IF NOT EXISTS idx_friend_messages_sender ON friend_messages(sender_id);
            CREATE INDEX IF NOT EXISTS idx_friend_messages_receiver ON friend_messages(receiver_id);
            CREATE INDEX IF NOT EXISTS idx_friend_messages_created_at ON friend_messages(created_at);
            "#,
        ),
        Migration::new(
            "010_create_user_profile",
            r#"
            CREATE TABLE IF NOT EXISTS user_profile (
                user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
                ranked_elo INTEGER NOT NULL DEFAULT 0,
                picture_id VARCHAR(255) NOT NULL DEFAULT '',
                picture_updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
        ),
        Migration::new(
            "011_create_player_inventory",
            r#"
            CREATE TABLE IF NOT EXISTS player_inventory (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                item_id VARCHAR(255) NOT NULL,
                item_type VARCHAR(50) NOT NULL,
                item_rarity VARCHAR(1) NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                UNIQUE(user_id, item_id, item_type)
            )
            "#,
        ),
        Migration::new(
            "012_user_setting",
            r#"
            CREATE TABLE IF NOT EXISTS user_setting (
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                lang VARCHAR(255) NOT NULL
            )
            "#,
        ),
        Migration::new(
            "013_create_collections",
            r#"
            CREATE TABLE IF NOT EXISTS collections (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                title VARCHAR(255) NOT NULL,
                price INTEGER NOT NULL DEFAULT 0,
                end_date TIMESTAMP WITH TIME ZONE NOT NULL,
                image_url TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
        ),
        Migration::new(
            "014_add_level_to_profile",
            r#"
            ALTER TABLE user_profile ADD COLUMN IF NOT EXISTS level INTEGER NOT NULL DEFAULT 1
            "#,
        ),
        Migration::new(
            "015_add_tournament_elo",
            r#"
            ALTER TABLE user_profile ADD COLUMN IF NOT EXISTS tournament_elo INTEGER NOT NULL DEFAULT 0
            "#,
        ),
        Migration::new(
            "016_add_xp",
            r#"
            ALTER TABLE user_profile ADD COLUMN IF NOT EXISTS xp BIGINT NOT NULL DEFAULT 0
            "#,
        ),
        Migration::new(
            "017_add_read_at_to_friend_messages",
            r#"
            ALTER TABLE friend_messages ADD COLUMN IF NOT EXISTS read_at TIMESTAMP WITH TIME ZONE;
            CREATE INDEX IF NOT EXISTS idx_friend_messages_read_at ON friend_messages(receiver_id, read_at);
            "#,
        ),
        Migration::new(
            "018_create_shop_catalog",
            r#"
            CREATE TABLE IF NOT EXISTS shop_catalog (
                item_id   VARCHAR(255) NOT NULL,
                item_type VARCHAR(50)  NOT NULL,
                title     VARCHAR(255) NOT NULL,
                price     BIGINT       NOT NULL CHECK (price >= 0),
                image_url TEXT         NOT NULL DEFAULT '',
                is_active BOOLEAN      NOT NULL DEFAULT TRUE,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                PRIMARY KEY (item_id, item_type)
            );
            "#,
        ),
        Migration::new(
            "020_add_asset_key_to_shop_catalog",
            r#"
            ALTER TABLE shop_catalog ADD COLUMN IF NOT EXISTS asset_key TEXT NOT NULL DEFAULT '';
            CREATE INDEX IF NOT EXISTS idx_shop_catalog_type ON shop_catalog(item_type);
            "#,
        ),
        Migration::new(
            "021_create_collection_items",
            r#"
            DELETE FROM collections a
            USING collections b
            WHERE a.title = b.title AND a.ctid > b.ctid;

            CREATE UNIQUE INDEX IF NOT EXISTS idx_collections_title ON collections(title);

            CREATE TABLE IF NOT EXISTS collection_items (
                collection_id UUID         NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
                item_id       VARCHAR(255) NOT NULL,
                item_type     VARCHAR(50)  NOT NULL,
                position      SMALLINT     NOT NULL DEFAULT 0,
                PRIMARY KEY (collection_id, item_id, item_type),
                FOREIGN KEY (item_id, item_type)
                    REFERENCES shop_catalog(item_id, item_type) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_collection_items_collection
                ON collection_items(collection_id);
            "#,
        ),
        Migration::new(
            "022_seed_cosmetic_catalog",
            r#"
            INSERT INTO shop_catalog (item_id, item_type, title, price, asset_key, is_active)
            VALUES
                ('1', 'base',      'Corps Classique',   0,   'base/1.png',      TRUE),
                ('2', 'base',      'Corps Neon',        250, 'base/2.png',      TRUE),
                ('1', 'hat',       'Casque Royal',      300, 'hat/1.png',       TRUE),
                ('2', 'hat',       'Casque Neon',       300, 'hat/2.png',       TRUE),
                ('1', 'mask',      'Masque Royal',      350, 'mask/1.png',      TRUE),
                ('2', 'mask',      'Masque Neon',       350, 'mask/2.png',      TRUE),
                ('1', 'clothes',   'Tenue Royale',      400, 'clothes/1.png',   TRUE),
                ('2', 'clothes',   'Tenue Neon',        400, 'clothes/2.png',   TRUE),
                ('1', 'accessory', 'Accessoire Royal',  200, 'accessory/1.png', TRUE),
                ('2', 'accessory', 'Accessoire Neon',   200, 'accessory/2.png', TRUE)
            ON CONFLICT (item_id, item_type) DO UPDATE
                SET title     = EXCLUDED.title,
                    price     = EXCLUDED.price,
                    asset_key = EXCLUDED.asset_key;
            "#,
        ),
        Migration::new(
            "023_seed_collection_bundles",
            r#"
            INSERT INTO collections (title, price, end_date, image_url)
            VALUES
                ('Pack Royal', 1000, NOW() + INTERVAL '30 days', ''),
                ('Pack Neon',  1100, NOW() + INTERVAL '30 days', '')
            ON CONFLICT (title) DO UPDATE
                SET price = EXCLUDED.price, end_date = EXCLUDED.end_date;

            INSERT INTO collection_items (collection_id, item_id, item_type, position)
            SELECT c.id, v.item_id, v.item_type, v.position
            FROM collections c
            JOIN (VALUES
                ('Pack Royal', '1', 'base',      0),
                ('Pack Royal', '1', 'hat',       1),
                ('Pack Royal', '1', 'mask',      2),
                ('Pack Royal', '1', 'clothes',   3),
                ('Pack Royal', '1', 'accessory', 4),
                ('Pack Neon',  '2', 'base',      0),
                ('Pack Neon',  '2', 'hat',       1),
                ('Pack Neon',  '2', 'mask',      2),
                ('Pack Neon',  '2', 'clothes',   3),
                ('Pack Neon',  '2', 'accessory', 4)
            ) AS v(title, item_id, item_type, position) ON v.title = c.title
            ON CONFLICT (collection_id, item_id, item_type) DO NOTHING;
            "#,
        ),
        Migration::new(
            "024_retire_legacy_shop_seed",
            r#"
            UPDATE shop_catalog
            SET is_active = FALSE
            WHERE asset_key = ''
              AND (item_id, item_type) IN (
                  ('neon-paddle', 'skin'),
                  ('gold-border', 'border'),
                  ('rocket-emote', 'emote')
              );

            DELETE FROM collections c
            WHERE c.title = 'Collection 1'
              AND NOT EXISTS (
                  SELECT 1 FROM collection_items ci WHERE ci.collection_id = c.id
              );
            "#,
        ),
        Migration::new(
            "025_seed_crew_collections",
            r#"
            INSERT INTO shop_catalog (item_id, item_type, title, price, asset_key, is_active)
            SELECT m.id, s.slot, s.label || ' ' || m.login, s.price,
                   s.slot || '/' || m.id || '.png', TRUE
            FROM (VALUES
                ('1'::varchar, 'almeekel'::varchar),
                ('2', 'tarini'),
                ('3', 'qutruche'),
                ('4', 'madelvin'),
                ('5', 'agantaum')
            ) AS m(id, login)
            CROSS JOIN (VALUES
                ('base'::varchar,  'Corps'::varchar,      120::bigint),
                ('hat',            'Casque',              130),
                ('mask',           'Masque',              140),
                ('clothes',        'Tenue',               150),
                ('accessory',      'Accessoire',          110)
            ) AS s(slot, label, price)
            ON CONFLICT (item_id, item_type) DO UPDATE
                SET title     = EXCLUDED.title,
                    price     = EXCLUDED.price,
                    asset_key = EXCLUDED.asset_key,
                    is_active = TRUE;

            INSERT INTO collections (title, price, end_date, image_url)
            SELECT 'Collection ' || m.login, 499, NOW() + INTERVAL '30 days', ''
            FROM (VALUES
                ('almeekel'::varchar), ('tarini'), ('qutruche'),
                ('madelvin'), ('agantaum')
            ) AS m(login)
            ON CONFLICT (title) DO UPDATE
                SET price = EXCLUDED.price, end_date = EXCLUDED.end_date;

            INSERT INTO collection_items (collection_id, item_id, item_type, position)
            SELECT c.id, m.id, s.slot, s.pos
            FROM (VALUES
                ('1'::varchar, 'almeekel'::varchar),
                ('2', 'tarini'),
                ('3', 'qutruche'),
                ('4', 'madelvin'),
                ('5', 'agantaum')
            ) AS m(id, login)
            CROSS JOIN (VALUES
                ('base'::varchar, 0::smallint),
                ('hat',           1),
                ('mask',          2),
                ('clothes',       3),
                ('accessory',     4)
            ) AS s(slot, pos)
            JOIN collections c ON c.title = 'Collection ' || m.login
            ON CONFLICT (collection_id, item_id, item_type) DO NOTHING;

            DELETE FROM collections WHERE title IN ('Pack Royal', 'Pack Neon');
            "#,
        ),
        Migration::new(
            "026_create_roles",
            r#"
            CREATE TABLE IF NOT EXISTS roles (
                id SERIAL PRIMARY KEY,
                name VARCHAR(50) NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        ),
        Migration::new(
            "027_create_permissions",
            r#"
            CREATE TABLE IF NOT EXISTS permissions (
                id SERIAL PRIMARY KEY,
                name VARCHAR(100) NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        ),
        Migration::new(
            "028_create_api_routes",
            r#"
            CREATE TABLE IF NOT EXISTS api_routes (
                id SERIAL PRIMARY KEY,
                method VARCHAR(10) NOT NULL,
                path TEXT NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE (method, path)
            );
            CREATE INDEX IF NOT EXISTS idx_api_routes_path ON api_routes(path);
            "#,
        ),
        Migration::new(
            "029_create_role_permissions",
            r#"
            CREATE TABLE IF NOT EXISTS role_permissions (
                role_id INT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
                permission_id INT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
                PRIMARY KEY (role_id, permission_id)
            );
            CREATE INDEX IF NOT EXISTS idx_role_permissions_role ON role_permissions(role_id);
            "#,
        ),
        Migration::new(
            "030_create_permission_routes",
            r#"
            CREATE TABLE IF NOT EXISTS permission_routes (
                permission_id INT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
                route_id INT NOT NULL REFERENCES api_routes(id) ON DELETE CASCADE,
                PRIMARY KEY (permission_id, route_id)
            );
            CREATE INDEX IF NOT EXISTS idx_permission_routes_permission ON permission_routes(permission_id);
            "#,
        ),
        Migration::new(
            "031_create_rate_limits",
            r#"
            CREATE TABLE IF NOT EXISTS rate_limits (
                route_id INT PRIMARY KEY REFERENCES api_routes(id) ON DELETE CASCADE,
                requests_per_minute INT NOT NULL DEFAULT 60
            )
            "#,
        ),
        Migration::new(
            "032_create_user_roles",
            r#"
            CREATE TABLE IF NOT EXISTS user_roles (
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                role_id INT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
                PRIMARY KEY (user_id, role_id)
            );
            CREATE INDEX IF NOT EXISTS idx_user_roles_user ON user_roles(user_id);
            "#,
        ),
        Migration::new(
            "033_seed_default_role",
            r#"
            INSERT INTO roles (name, description)
            VALUES ('player', 'Rôle par défaut des joueurs')
            ON CONFLICT (name) DO NOTHING;
            "#,
        ),
        Migration::new(
            "034_seed_base_permissions",
            r#"
            INSERT INTO permissions (name, description)
            VALUES
                ('panel.access', 'Accéder au panneau d administration'),
                ('users.view',   'Voir la liste des utilisateurs'),
                ('users.edit',   'Modifier un utilisateur'),
                ('users.delete', 'Supprimer un utilisateur'),
                ('roles.manage',      'Créer / modifier / supprimer des rôles'),
                ('permissions.manage','Créer / modifier / supprimer des permissions'),
                ('routes.manage',     'Gérer les routes API'),
                ('rate-limits.manage','Définir les limites de requêtes par minute')
            ON CONFLICT (name) DO NOTHING;
            "#,
        ),
        Migration::new(
            "035_seed_api_routes",
            r#"
            INSERT INTO api_routes (method, path, name)
            VALUES
                ('GET',    '/api/auth/health', 'Auth health'),
                ('POST',   '/api/auth/login/email', 'Connexion email'),
                ('POST',   '/api/auth/google/code', 'Connexion Google'),
                ('GET',    '/api/auth/42/login', 'Connexion 42 (login)'),
                ('GET',    '/api/auth/42/callback', 'Connexion 42 (callback)'),
                ('POST',   '/api/auth/finish_account', 'Finaliser le profil'),
                ('POST',   '/api/auth/register', 'Inscription'),
                ('POST',   '/api/auth/send_validation_email_code', 'Envoyer code validation email'),
                ('DELETE', '/api/auth/delete_user', 'Suppression du compte'),
                ('POST',   '/api/auth/validate_email', 'Valider l email'),
                ('POST',   '/api/auth/forgot_password', 'Mot de passe oublié'),
                ('POST',   '/api/auth/reset_password', 'Réinitialiser le mot de passe'),
                ('GET',    '/api/auth/refresh', 'Rafraîchir le token'),
                ('GET',    '/api/auth/logout', 'Déconnexion'),
                ('POST',   '/api/auth/validate', 'Valider un token'),
                ('POST',   '/api/auth/change_password', 'Changer le mot de passe'),
                ('POST',   '/api/auth/change_provider/email/send_validation_code', 'Switch provider email (code)'),
                ('POST',   '/api/auth/change_provider/email/switch', 'Switch provider email'),
                ('POST',   '/api/auth/change_provider/google/switch', 'Switch provider google'),
                ('GET',    '/api/auth/stats', 'Stats auth'),

                ('GET',    '/api/user/health', 'User health'),
                ('GET',    '/api/user/shop', 'Boutique'),
                ('POST',   '/api/user/shop/items', 'Upload item boutique'),
                ('GET',    '/api/user/collections', 'Collections'),
                ('POST',   '/api/user/collections/purchase', 'Acheter une collection'),
                ('GET',    '/api/user/me', 'Profil utilisateur'),
                ('GET',    '/api/user/users/{id}', 'Profil public'),
                ('GET',    '/api/user/state', 'État de session'),
                ('GET',    '/api/user/inventory', 'Inventaire'),
                ('GET',    '/api/user/profile-picture', 'Photo de profil'),
                ('PATCH',  '/api/user/change-username', 'Changer le pseudo'),
                ('POST',   '/api/user/inventory', 'Ajouter un item'),
                ('DELETE', '/api/user/inventory', 'Retirer un item'),
                ('POST',   '/api/user/profile-picture', 'Définir la photo de profil'),
                ('POST',   '/api/user/admin/users', 'Admin : lister les utilisateurs'),
                ('PATCH',  '/api/user/admin/users/{id}', 'Admin : modifier un utilisateur'),
                ('DELETE', '/api/user/admin/users/{id}', 'Admin : supprimer un utilisateur'),
                ('GET',    '/api/user/admin/roles', 'Admin : lister les rôles'),
                ('POST',   '/api/user/admin/roles', 'Admin : créer un rôle'),
                ('PATCH',  '/api/user/admin/roles/{id}', 'Admin : modifier un rôle'),
                ('DELETE', '/api/user/admin/roles/{id}', 'Admin : supprimer un rôle'),
                ('POST',   '/api/user/admin/roles/{id}/permissions', 'Admin : attribuer une permission à un rôle'),
                ('DELETE', '/api/user/admin/roles/{id}/permissions/{perm_id}', 'Admin : retirer une permission d un rôle'),
                ('GET',    '/api/user/admin/permissions', 'Admin : lister les permissions'),
                ('POST',   '/api/user/admin/permissions', 'Admin : créer une permission'),
                ('PATCH',  '/api/user/admin/permissions/{id}', 'Admin : modifier une permission'),
                ('DELETE', '/api/user/admin/permissions/{id}', 'Admin : supprimer une permission'),
                ('POST',   '/api/user/admin/permissions/{id}/routes', 'Admin : lier une route à une permission'),
                ('DELETE', '/api/user/admin/permissions/{id}/routes/{route_id}', 'Admin : détacher une route d une permission'),
                ('GET',    '/api/user/admin/routes', 'Admin : lister les routes API'),
                ('PUT',    '/api/user/admin/routes/{id}/rate-limit', 'Admin : définir le rate limit d une route'),
                ('POST',   '/api/user/admin/users/{id}/roles', 'Admin : attribuer un rôle à un utilisateur'),
                ('DELETE', '/api/user/admin/users/{id}/roles/{role_id}', 'Admin : retirer un rôle d un utilisateur'),

                ('GET',    '/api/room/health', 'Room health'),
                ('POST',   '/api/room/create_room', 'Créer une room'),
                ('POST',   '/api/room/play_ranked', 'Matchmaking classé'),
                ('GET',    '/api/room/queue_size', 'Taille de la file'),
                ('POST',   '/api/room/cancel_ranked', 'Annuler le matchmaking'),
                ('GET',    '/api/room/status', 'Statut de la room'),
                ('GET',    '/api/room/list_public', 'Rooms publiques'),
                ('POST',   '/api/room/join_room', 'Rejoindre une room'),
                ('POST',   '/api/room/leave_room', 'Quitter une room'),
                ('POST',   '/api/room/start_room', 'Démarrer une room'),
                ('POST',   '/api/room/kick_room', 'Expulser un joueur'),
                ('POST',   '/api/room/room_info', 'Infos de la room'),
                ('GET',    '/api/room/tournament/{id}', 'Tournoi'),

                ('POST',   '/api/social/friend-requests', 'Envoyer une demande d ami'),
                ('GET',    '/api/social/friend-requests', 'Demandes reçues'),
                ('GET',    '/api/social/friend-requests/sent', 'Demandes envoyées'),
                ('PATCH',  '/api/social/friend-requests/{id}/accept', 'Accepter une demande'),
                ('PATCH',  '/api/social/friend-requests/{id}/refuse', 'Refuser une demande'),
                ('DELETE', '/api/social/friend-requests/{id}', 'Annuler une demande'),
                ('GET',    '/api/social/friends', 'Liste des amis'),
                ('GET',    '/api/social/friends/blocked', 'Liste des bloqués'),
                ('DELETE', '/api/social/friends/{id}', 'Retirer un ami'),
                ('POST',   '/api/social/friends/{id}/block', 'Bloquer un utilisateur'),
                ('DELETE', '/api/social/friends/{id}/block', 'Débloquer un utilisateur'),
                ('POST',   '/api/social/friends/{id}/messages', 'Envoyer un message'),
                ('GET',    '/api/social/friends/{id}/messages', 'Historique des messages'),
                ('POST',   '/api/social/friends/{id}/messages/read', 'Marquer les messages comme lus'),

                ('POST',   '/api/chess/game/create', 'Créer une partie'),
                ('POST',   '/api/chess/game/abandon', 'Abandonner une partie'),

                ('GET',    '/api/notifications/health', 'Notifications health'),
                ('GET',    '/api/notifications/sse/rooms', 'SSE des rooms'),
                ('GET',    '/api/notifications/sse/{user_id}', 'SSE notifications')
            ON CONFLICT (method, path) DO NOTHING;
            "#,
        ),
        Migration::new(
            "036_drop_tournament_elo",
            r#"
            ALTER TABLE user_profile DROP COLUMN IF EXISTS tournament_elo;
            "#,
        ),
        Migration::new(
            "037_create_player_cards",
            r#"
            CREATE TABLE IF NOT EXISTS player_cards (
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                card_id VARCHAR(10) NOT NULL,
                rarity SMALLINT NOT NULL DEFAULT 0,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                PRIMARY KEY (user_id, card_id, rarity)
            );

            CREATE INDEX IF NOT EXISTS idx_player_cards_user ON player_cards(user_id);
            "#,
        ),
        Migration::new(
            "038_create_player_deck",
            r#"
            CREATE TABLE IF NOT EXISTS player_deck (
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                card_id VARCHAR(10) NOT NULL,
                rarity SMALLINT NOT NULL DEFAULT 0,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                PRIMARY KEY (user_id, card_id)
            );

            CREATE INDEX IF NOT EXISTS idx_player_deck_user ON player_deck(user_id);
            "#,
        ),
        Migration::new(
            "039_seed_default_player_cards",
            r#"
            INSERT INTO player_cards (user_id, card_id, rarity)
            SELECT u.id, c.card_id, 0
            FROM users u
            CROSS JOIN (VALUES ('1'),('2'),('3'),('5'),('6'),('7'),('8'),('9'),('10'),('11')) AS c(card_id)
            ON CONFLICT (user_id, card_id, rarity) DO NOTHING;

            INSERT INTO player_deck (user_id, card_id, rarity)
            SELECT u.id, c.card_id, 0
            FROM users u
            CROSS JOIN (VALUES ('1'),('2'),('3'),('5'),('6'),('7'),('8'),('9'),('10'),('11')) AS c(card_id)
            ON CONFLICT (user_id, card_id) DO NOTHING;
            "#,
        ),
        Migration::new(
            "040_seed_default_cards_trigger",
            r#"
            CREATE OR REPLACE FUNCTION seed_default_player_cards()
            RETURNS TRIGGER AS $$
            DECLARE
                c TEXT;
            BEGIN
                FOREACH c IN ARRAY ARRAY['1','2','3','5','6','7','8','9','10','11'] LOOP
                    INSERT INTO player_cards (user_id, card_id, rarity)
                    VALUES (NEW.id, c, 0)
                    ON CONFLICT (user_id, card_id, rarity) DO NOTHING;
                    INSERT INTO player_deck (user_id, card_id, rarity)
                    VALUES (NEW.id, c, 0)
                    ON CONFLICT (user_id, card_id) DO NOTHING;
                END LOOP;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;

            DROP TRIGGER IF EXISTS trg_seed_default_player_cards ON users;
            CREATE TRIGGER trg_seed_default_player_cards
            AFTER INSERT ON users
            FOR EACH ROW EXECUTE FUNCTION seed_default_player_cards();
            "#,
        ),
        Migration::new(
            "041_seed_card_catalog",
            r#"
            INSERT INTO shop_catalog (item_id, item_type, title, price, asset_key, is_active)
            VALUES
                ('0',  'card', 'Zone mortelle',              800,  '', TRUE),
                ('4',  'card', 'Fog',                         600,  '', TRUE),
                ('12', 'card', 'Pyromane',                    500,  '', TRUE),
                ('13', 'card', 'Canon',                       900,  '', TRUE),
                ('14', 'card', 'Sniper',                      900,  '', TRUE),
                ('15', 'card', 'Poubelle',                    700,  '', TRUE),
                ('16', 'card', 'Repousser',                   900,  '', TRUE),
                ('17', 'card', 'Champ de bataille',          1100,  '', TRUE),
                ('19', 'card', 'Anihilation',                1000,  '', TRUE),
                ('20', 'card', 'Veteran cheval',              900,  '', TRUE),
                ('21', 'card', 'Veteran tour',               1000,  '', TRUE),
                ('22', 'card', 'Veteran fou',                1000,  '', TRUE),
                ('23', 'card', 'Frog',                        500,  '', TRUE),
                ('24', 'card', 'Roue de la fortune',         1200,  '', TRUE),
                ('25', 'card', 'Magnetisme',                 1000,  '', TRUE),
                ('26', 'card', 'Bastion',                    1100,  '', TRUE),
                ('27', 'card', 'Ninjaaaa',                   1000,  '', TRUE),
                ('28', 'card', 'Traitre',                    1200,  '', TRUE),
                ('29', 'card', 'Percee',                      700,  '', TRUE),
                ('30', 'card', 'Sauvetage inespere',          900,  '', TRUE)
            ON CONFLICT (item_id, item_type) DO UPDATE
                SET title = EXCLUDED.title,
                    price = EXCLUDED.price,
                    is_active = TRUE;
            "#,
        ),
        Migration::new(
            "042_hide_garbage_card",
            r#"
            UPDATE shop_catalog SET is_active = FALSE
            WHERE item_id = '18' AND item_type = 'card';

            DELETE FROM player_cards WHERE card_id = '18';
            DELETE FROM player_deck WHERE card_id = '18';
            "#,
        ),
        Migration::new(
            "043_seed_missing_routes",
            r#"
            INSERT INTO api_routes (method, path, name)
            VALUES
                ('GET',    '/api/user/cards',              'Cartes du joueur'),
                ('GET',    '/api/user/deck',               'Deck du joueur'),
                ('PUT',    '/api/user/deck/rarity',        'Changer la rareté d une carte'),
                ('POST',   '/api/user/cards/buy',          'Acheter une carte'),
                ('GET',    '/api/user/admin/users/{id}/cards',           'Admin : cartes d un utilisateur'),
                ('POST',   '/api/user/admin/users/{id}/cards',           'Admin : débloquer une carte'),
                ('DELETE', '/api/user/admin/users/{id}/cards/{card_id}/{rarity}', 'Admin : retirer une rareté')
            ON CONFLICT (method, path) DO NOTHING;
            "#,
        ),
        Migration::new(
            "044_link_default_permission_routes",
            r#"
            -- Nettoie les liens existants pour pouvoir re-semer proprement.
            DELETE FROM permission_routes;

            -- panel.access => toutes les routes admin
            INSERT INTO permission_routes (permission_id, route_id)
            SELECT p.id, ar.id
            FROM permissions p
            CROSS JOIN api_routes ar
            WHERE p.name = 'panel.access'
              AND ar.path LIKE '/api/user/admin/%';

            -- users.view => lister les utilisateurs
            INSERT INTO permission_routes (permission_id, route_id)
            SELECT p.id, ar.id
            FROM permissions p
            JOIN api_routes ar ON ar.method = 'POST' AND ar.path = '/api/user/admin/users'
            WHERE p.name = 'users.view';

            -- users.edit => modifier un utilisateur + gérer ses cartes
            INSERT INTO permission_routes (permission_id, route_id)
            SELECT p.id, ar.id
            FROM permissions p
            CROSS JOIN api_routes ar
            WHERE p.name = 'users.edit'
              AND (
                    ar.method = 'PATCH' AND ar.path LIKE '/api/user/admin/users/%'
                    OR ar.path LIKE '/api/user/admin/users/%/cards%'
                  );

            -- users.delete => supprimer un utilisateur
            INSERT INTO permission_routes (permission_id, route_id)
            SELECT p.id, ar.id
            FROM permissions p
            CROSS JOIN api_routes ar
            WHERE p.name = 'users.delete'
              AND ar.method = 'DELETE'
              AND ar.path = '/api/user/admin/users/{id}';

            -- roles.manage => toutes les routes de rôles
            INSERT INTO permission_routes (permission_id, route_id)
            SELECT p.id, ar.id
            FROM permissions p
            CROSS JOIN api_routes ar
            WHERE p.name = 'roles.manage'
              AND ar.path LIKE '/api/user/admin/roles%';

            -- permissions.manage => toutes les routes de permissions
            INSERT INTO permission_routes (permission_id, route_id)
            SELECT p.id, ar.id
            FROM permissions p
            CROSS JOIN api_routes ar
            WHERE p.name = 'permissions.manage'
              AND ar.path LIKE '/api/user/admin/permissions%';

            -- routes.manage => lister les routes API
            INSERT INTO permission_routes (permission_id, route_id)
            SELECT p.id, ar.id
            FROM permissions p
            CROSS JOIN api_routes ar
            WHERE p.name = 'routes.manage'
              AND ar.method = 'GET'
              AND ar.path = '/api/user/admin/routes';

            -- rate-limits.manage => routes rate-limit
            INSERT INTO permission_routes (permission_id, route_id)
            SELECT p.id, ar.id
            FROM permissions p
            CROSS JOIN api_routes ar
            WHERE p.name = 'rate-limits.manage'
              AND ar.path LIKE '/api/user/admin/routes/%/rate-limit';
            "#,
        ),
        Migration::new(
            "045_seed_missing_card_catalog",
            r#"
            INSERT INTO shop_catalog (item_id, item_type, title, price, asset_key, is_active)
            VALUES
                ('1',  'card', 'Top chrono',                   800, '', TRUE),
                ('2',  'card', 'Roulette russe',               900, '', TRUE),
                ('3',  'card', 'Voyage',                       700, '', TRUE),
                ('5',  'card', 'Cheval fou',                   500, '', TRUE),
                ('6',  'card', 'Travail de bete',              500, '', TRUE),
                ('7',  'card', 'Macon en furie',               500, '', TRUE),
                ('8',  'card', 'Attraper le voleur de cheval', 600, '', TRUE),
                ('9',  'card', 'Bete de travail',              600, '', TRUE),
                ('10', 'card', 'Bestification',                600, '', TRUE),
                ('11', 'card', 'Architecte ermite',            600, '', TRUE)
            ON CONFLICT (item_id, item_type) DO UPDATE
                SET title = EXCLUDED.title,
                    price = EXCLUDED.price,
                    is_active = TRUE;
            "#,
        ),
        Migration::new(
            "046_link_user_role_permissions",
            r#"
            -- users.edit permet aussi d'assigner/retirer des rôles à un utilisateur.
            INSERT INTO permission_routes (permission_id, route_id)
            SELECT p.id, ar.id
            FROM permissions p
            CROSS JOIN api_routes ar
            WHERE p.name = 'users.edit'
              AND (ar.method = 'POST' OR ar.method = 'DELETE')
              AND ar.path LIKE '/api/user/admin/users/%/roles';
            "#,
        ),
        Migration::new(
            "047_seed_default_rate_limits",
            r#"
            -- Récupère l'id d'une route (ou laisse la ligne sans rien si absente).
            INSERT INTO rate_limits (route_id, requests_per_minute)
            SELECT ar.id, v.rpm
            FROM api_routes ar
            JOIN (VALUES
                ('POST', '/api/auth/login/email', 10),
                ('POST', '/api/auth/register', 5),
                ('POST', '/api/auth/forgot_password', 5),
                ('POST', '/api/auth/reset_password', 5),
                ('POST', '/api/auth/send_validation_email_code', 10),
                ('POST', '/api/auth/change_password', 10),
                ('GET',  '/api/auth/refresh', 30),
                ('GET',  '/api/user/me', 60),
                ('GET',  '/api/user/shop', 60),
                ('GET',  '/api/user/collections', 60),
                ('POST', '/api/user/collections/purchase', 30),
                ('GET',  '/api/user/inventory', 60),
                ('POST', '/api/user/inventory', 30),
                ('DELETE', '/api/user/inventory', 30),
                ('GET',  '/api/user/profile-picture', 60),
                ('POST', '/api/user/profile-picture', 10),
                ('PATCH', '/api/user/change-username', 10),
                ('GET',  '/api/user/state', 60),
                ('POST', '/api/user/packs/open', 20),
                ('GET',  '/api/user/cards', 60),
                ('GET',  '/api/user/deck', 60),
                ('POST', '/api/room/create_room', 20),
                ('POST', '/api/room/play_ranked', 20),
                ('POST', '/api/room/join_room', 30),
                ('POST', '/api/room/leave_room', 30),
                ('POST', '/api/room/start_room', 20),
                ('POST', '/api/room/kick_room', 20),
                ('GET',  '/api/room/list_public', 60),
                ('GET',  '/api/room/status', 60),
                ('POST', '/api/social/friend-requests', 30),
                ('GET',  '/api/social/friend-requests', 60),
                ('GET',  '/api/social/friends', 60),
                ('POST', '/api/social/friends/{id}/messages', 60),
                ('GET',  '/api/social/friends/{id}/messages', 120),
                ('POST', '/api/chess/game/create', 10),
                ('POST', '/api/chess/game/abandon', 20)
            ) AS v(method, path, rpm)
              ON v.method = ar.method AND v.path = ar.path
            ON CONFLICT (route_id) DO UPDATE
                SET requests_per_minute = EXCLUDED.requests_per_minute;

            -- Routes admin : GET/POST à 60/min, mutations (PATCH/PUT/DELETE) à 30/min.
            INSERT INTO rate_limits (route_id, requests_per_minute)
            SELECT ar.id, 60 FROM api_routes ar
            WHERE ar.path LIKE '/api/user/admin/%' AND ar.method IN ('GET', 'POST')
            ON CONFLICT (route_id) DO UPDATE
                SET requests_per_minute = EXCLUDED.requests_per_minute;

            INSERT INTO rate_limits (route_id, requests_per_minute)
            SELECT ar.id, 30 FROM api_routes ar
            WHERE ar.path LIKE '/api/user/admin/%' AND ar.method IN ('PATCH', 'PUT', 'DELETE')
            ON CONFLICT (route_id) DO UPDATE
                SET requests_per_minute = EXCLUDED.requests_per_minute;
            "#,
        ),
        Migration::new(
            "048_drop_users_role",
            r#"
            ALTER TABLE users DROP COLUMN IF EXISTS role;
            "#,
        ),
        Migration::new(
            "049_seed_default_player_role",
            r#"
            CREATE OR REPLACE FUNCTION seed_default_player_role()
            RETURNS TRIGGER AS $$
            BEGIN
                INSERT INTO user_roles (user_id, role_id)
                SELECT NEW.id, r.id FROM roles r WHERE r.name = 'player'
                ON CONFLICT (user_id, role_id) DO NOTHING;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;

            DROP TRIGGER IF EXISTS trg_seed_default_player_role ON users;
            CREATE TRIGGER trg_seed_default_player_role
            AFTER INSERT ON users
            FOR EACH ROW EXECUTE FUNCTION seed_default_player_role();

            -- Backfill : les comptes existants sans aucun rôle reçoivent player.
            INSERT INTO user_roles (user_id, role_id)
            SELECT u.id, r.id
            FROM users u
            CROSS JOIN roles r
            WHERE r.name = 'player'
              AND NOT EXISTS (SELECT 1 FROM user_roles ur WHERE ur.user_id = u.id)
            ON CONFLICT (user_id, role_id) DO NOTHING;
            "#,
        ),
        Migration::new(
            "050_seed_admin_role",
            r#"
            INSERT INTO roles (name, description)
            VALUES ('admin', 'Administrateur avec tous les accès')
            ON CONFLICT (name) DO NOTHING;

            -- Le rôle admin reçoit toutes les permissions existantes.
            INSERT INTO role_permissions (role_id, permission_id)
            SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
            WHERE r.name = 'admin'
            ON CONFLICT (role_id, permission_id) DO NOTHING;
            "#,
        ),
        Migration::new(
            "051_add_profile_settings",
            r#"
            ALTER TABLE user_profile
                ADD COLUMN IF NOT EXISTS bio TEXT NOT NULL DEFAULT '',
                ADD COLUMN IF NOT EXISTS github TEXT NOT NULL DEFAULT '',
                ADD COLUMN IF NOT EXISTS discord TEXT NOT NULL DEFAULT '',
                ADD COLUMN IF NOT EXISTS twitter TEXT NOT NULL DEFAULT '',
                ADD COLUMN IF NOT EXISTS is_private BOOLEAN NOT NULL DEFAULT FALSE,
                ADD COLUMN IF NOT EXISTS theme VARCHAR(20) NOT NULL DEFAULT 'dark',
                ADD COLUMN IF NOT EXISTS lang VARCHAR(20) NOT NULL DEFAULT 'fr';
            "#,
        ),
        Migration::new(
            "052_default_base_to_99",
            r#"
            WITH to_update AS (
                SELECT pi.id
                FROM player_inventory pi
                WHERE pi.item_type = 'base'
                  AND pi.item_id = '0'
                  AND NOT EXISTS (
                      SELECT 1 FROM player_inventory pi2
                      WHERE pi2.user_id = pi.user_id
                        AND pi2.item_type = 'base'
                        AND pi2.item_id = '99'
                  )
            )
            UPDATE player_inventory
            SET item_id = '99'
            WHERE id IN (SELECT id FROM to_update);

            DELETE FROM player_inventory
            WHERE item_type = 'base' AND item_id = '0';
            "#,
        ),
        Migration::new(
            "053_player_inventory_rarity_default",
            r#"
            ALTER TABLE player_inventory
            ALTER COLUMN item_rarity SET DEFAULT '0';
            "#,
        ),
        Migration::new(
            "054_disable_magnetisme_bastion",
            r#"
            UPDATE shop_catalog SET is_active = FALSE
            WHERE item_id IN ('25', '26') AND item_type = 'card';

            DELETE FROM player_cards WHERE card_id IN ('25', '26');
            DELETE FROM player_deck WHERE card_id IN ('25', '26');
            "#,
        ),
        Migration::new(
            "055_translate_card_titles",
            r#"
            UPDATE shop_catalog SET title = v.title
            FROM (VALUES
                ('0',  'Deadly zone'),
                ('1',  'Time boost'),
                ('2',  'Russian roulette'),
                ('3',  'Journey'),
                ('4',  'Fog'),
                ('5',  'Crazy knight'),
                ('6',  'Beast work'),
                ('7',  'Furious mason'),
                ('8',  'Catch the knight thief'),
                ('9',  'Beast of burden'),
                ('10', 'Bestification'),
                ('11', 'Hermit architect'),
                ('12', 'Pyromaniac'),
                ('13', 'Cannon'),
                ('14', 'Sniper'),
                ('15', 'Trash'),
                ('16', 'Push back'),
                ('17', 'Battlefield'),
                ('18', 'Garbage'),
                ('19', 'Annihilation'),
                ('20', 'Veteran knight'),
                ('21', 'Veteran rook'),
                ('22', 'Veteran bishop'),
                ('23', 'Frog'),
                ('24', 'Wheel of fortune'),
                ('25', 'Magnetism'),
                ('26', 'Bastion'),
                ('27', 'Ninja'),
                ('28', 'Traitor'),
                ('29', 'Breakthrough'),
                ('30', 'Desperate rescue')
            ) AS v(item_id, title)
            WHERE shop_catalog.item_id = v.item_id
              AND shop_catalog.item_type = 'card';
            "#,
        ),
    ]
}

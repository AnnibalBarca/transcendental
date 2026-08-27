# Architecture réelle — ft_transcendence

> Audit en mode lecture seule. Tout ce qui suit a été vérifié dans le code des
> sous-modules (pointeurs git du superprojet, submodules initialisés).

## 1. Vue d'ensemble

```
 navigateur ──HTTP/WS──> nginx (:8000)
                          ├── /api/*     ──> Gateway-API (:4000)
                          ├── /img/*     ──> MinIO (:9000)
                          ├── /docs,/openapi ──> Scalar (:8080)
                          └── /*         ──> Vite dev server (:5173)  [SPA fallback]

 Gateway-API ──HTTP direct──> Auth-API, Notification-API
 Gateway-API ──Redis Stream `user:requests`, `room:requests`, …──> services
 Gateway-API ──WS proxy (découverte Redis)──> Chess-API chess-1 / chess-2 (:8082)

 services back ──Postgres (sqlx)──> postgres_db
 services back ──Redis (pub/sub, streams, hashes)──> redis_db
 User-API ──S3 API──> MinIO (cosmétiques, images)
 Auth-API ──SMTP/API Resend──> e-mails de validation
 Room-API ──Stream `chess:game:events`──> Chess-API (résultats de partie)
 Notification-API ──SSE──> front (notifications)
 ELK + Prometheus/Grafana──> métriques (ports 9101-9106 exposés par service)
```

## 2. Canaux de communication réels

### 2.1 HTTP synchrone via Redis Streams (le cœur du système)

- La gateway reçoit `/api/<service>/…` et publie la requête sur le stream
  Redis `<service>:requests` avec un correlation-id, puis attend la réponse sur
  `gateway:responses` (`back/Gateway-API/src/http/handlers/proxy_redis.rs`,
  `back/Gateway-API/src/http/response_listener.rs:40-63`).
- Chaque service back (User, Room, Social, Chess) est un worker sans serveur
  HTTP exposé : il consomme son stream et répond en JSON ; la gateway prend le
  champ `status` du JSON comme code HTTP.
- **Exceptions** : `auth` et `notifications` passent en HTTP direct
  (`back/Gateway-API/src/http/handlers/router.rs:116-151`).
- Conséquence opérationnelle : une panne peut venir du service, de Redis ou de
  la gateway ; les traces sont réparties sur trois conteneurs (confirmé).

### 2.2 Authentification

- Auth-API émet des JWT RSA (RS256), la clé publique est publiée dans Redis
  (`auth:jwt:public_pem`). La gateway valide les JWT pour les services
  protégés et gère une blacklist Redis (`back/Gateway-API/src/http/token_validator.rs`).
- Les cookies `access_token` / `refresh_token` sont la seule identité du front.
- **Le websocket d'échecs échappe à ce contrôle** (voir §5, contradiction 3).

### 2.3 Websocket du jeu d'échecs

> **Mise à jour (re-vérification du 20/08)** : `back/Chess-API/src/game.rs`
> n'existe plus comme fichier unique. Il a été éclaté en
> `back/Chess-API/src/game/commands.rs` (dispatch des commandes WS,
> `turn_changed`/`card_result`/…), `.../session.rs` (boucle de session par
> connexion, heartbeat, reconnexion) et `.../redis_helpers.rs` (persistance
> Redis liée à la session). Toutes les citations `game.rs:NNNN` de versions
> antérieures de ce document sont donc caduques ; les sections ci-dessous
> ont été remises à jour avec les nouveaux chemins.

- Le front construit l'URL depuis l'adresse de la page :
  `front/src/features/games/ChessGame/hooks/useChessGame.ts:174-181`
  (`${protocol}//${host}/api/chess/chess?game_id=…`). `CHESS_PUBLIC_WS_URL`
  (docker-compose.yml:194,227) n'est utilisé par aucun code du front —
  variable morte pour le client (les ports 8082/8083 restent cependant
  publiés, voir contradiction 6).
- La gateway résout l'instance Chess-API via Redis (`chess:game:{id}` →
  instance, registre `chess:instances`) :
  `back/Gateway-API/src/chess_discovery.rs:8-73`,
  `back/Gateway-API/src/http/handlers/router.rs:64-116`.
- Chess-API maintient par partie : un `GameLoop` (plateau + mains + timer),
  un `LobbyState` (**2 slots joueur uniquement, plus de notion de
  spectateur dans la structure de données** —
  `back/Chess-API/src/websocket/lobby.rs:38-54`, voir contradiction 3 mise à
  jour) et une tâche `run_game_loop` qui tick chaque seconde
  (`back/Chess-API/src/game/manager/core.rs`,
  `back/Chess-API/src/game/game_loop/runner.rs:60`).
- Le client n'appelle jamais Chess-API en HTTP pour jouer : tout passe par le
  WS (`play_card`, `move_piece`, `get_hand`, …, dispatché depuis
  `back/Chess-API/src/game/commands.rs:234` `handle_message`).
- **Authentification WS désormais forte** (changement majeur depuis la
  dernière version de cet audit) : `back/Chess-API/src/websocket/handler.rs`
  exige un jeton d'accès valide (401 sinon) et vérifie que
  `claims.sub` figure dans `chess:game_players:{game_id}` — une clé Redis
  écrite par Room-API à la création de la partie
  (`back/Room-API/src/services/chess_client.rs::set_game_players`) — sinon
  403. Un utilisateur qui n'est pas l'un des deux joueurs enregistrés de la
  partie ne peut donc plus ouvrir cette WS du tout, y compris pour
  spectater (voir bug #18 : le mode spectateur en a été supprimé comme
  effet de bord).
- Heartbeat applicatif : le serveur envoie `{"action":"ping"}` toutes les 15 s
  et coupe à 30 s sans `pong`
  (`back/Chess-API/src/game/session.rs:199-221`) ; le pong est répondu par la
  page montée (`useChessGame.ts:438-439`) via un SharedWorker
  (`front/public/shared-ws-worker.js`) qui multiplexe plusieurs onglets sur un
  seul socket et met en cache les derniers messages d'état (replay au
  reconnect, désormais dans un ordre explicite et correct :
  `shared-ws-worker.js:58-81`, `replayState`).

### 2.4 Cycle de vie d'une partie

1. Room-API (join/create/matchmaking) écrit `user:session:{id}` (statut
   `waiting`/`playing`, TTL 2 h) et crée la room.
2. Au démarrage de partie, Room-API publie la création côté Chess-API (stream
   `chess:requests`), Chess-API enregistre `chess:game:{id}` dans Redis.
3. Les joueurs ouvrent le WS ; le premier slot libre devient player1 (blancs).
4. Fin de partie : Chess-API publie sur `chess:game:events`
   (`game.rs:91-129`), Room-API `game_result` réinitialise les sessions
   (`back/Room-API/src/services/game_result.rs:366-387`), ELO, etc.

### 2.5 Images / cosmétiques

- MinIO derrière nginx sur `/img/` (`infra/Nginx/nginx.conf:99-106`).
- Le catalogue ne stocke que des `asset_key` ; User-API reconstruit l'URL
  (`back/User-API/src/services/storage.rs`). Les buckets sont rendus publics
  par la boucle `mc-public` (docker-compose.yml:389-412).
- **L'écriture dans ce catalogue n'est protégée par rien** :
  `POST /api/user/shop/items` (upload d'un item + son image) n'exige ni
  cookie ni permission — voir bug #36. C'est le point d'entrée qui alimente
  ce même mécanisme de stockage public.

## 3. Responsabilités par sous-module (constaté)

| Sous-module | Responsabilité réelle | Points remarquables |
|---|---|---|
| Gateway-API | routage, proxy WS/HTTP, RBAC partiel, rate-limit par service, métriques | **mise à jour** : `enforce_access` couvre désormais aussi `auth` (commit `206b727`), mais reste un no-op silencieux pour tout appel sans cookie `access_token` (donc inefficace sur `register`/`login`, voir bug #30) ; WS chess délègue désormais son auth à Chess-API lui-même (voir §2.3), plus « sans auth » mais toujours sans validation côté gateway |
| Auth-API | register/login/OAuth Google+42/JWT/e-mails Resend | middleware rate-limit écrite mais **toujours jamais branchée** (`src/http/rate_limit.rs`, `router.rs` — aucun `.layer`) ; logout désormais uniformément en POST (bug #29 résolu) |
| User-API | profil, inventaire, boutique, packs, admin, migrations SQL | deck par défaut semé en migration (`migrations.rs:527-537`) ; **`POST shop/items` (upload catalogue) sans aucune authentification, cf. bug #36** |
| Chess-API | moteur d'échecs, cartes, 2 instances | tout l'état de jeu est en mémoire par instance |
| Room-API | rooms publiques/privées, matchmaking, ELO, tournois | `src/ws/ws_handler.rs` et `ws/socket_manager.rs` **vides** : aucune gestion de connexion |
| Social-API | amis, messages | worker Redis uniquement |
| Notification-API | SSE | ne remonte aucune présence aux autres services |
| api-core | routeur Redis partagé, cache, JWT, métriques | utilisé par tous les services back |
| front | React 19 + TS + Vite + Tailwind + i18next | un SharedWorker par type de socket (ws jeu, refresh, sse) |
| infra/Nginx | reverse proxy unique sur :8000 | aucun header de sécurité, aucun TLS |
| infra/metrics | ELK + Prometheus/Grafana | ports 5601/3000 publiés sur 0.0.0.0 |

## 4. Cartographie des états (source de vérité)

| État | Où il vit | TTL / cleanup |
|---|---|---|
| Session joueur (statut, room, game) | Redis `user:session:{id}` | TTL 7200 s réarmé à chaque écriture ; cleanup uniquement leave/cancel/fin de partie |
| Partie d'échecs | mémoire Chess-API (`GameLoop`) | instance jamais retirée du `GameManager` |
| Mapping partie→instance | Redis `chess:game:{id}` | DEL à la fin de partie |
| Rooms | Redis `room:{id}` (TTL 7200) + ZSET de listing | `clean_stale_public` ne purge que le listing |
| File de matchmaking | Redis ZSET **sans TTL** | pairing forcé après 30 s |
| Refresh tokens | Postgres | purge des expirés uniquement |
| Blacklist access tokens | Redis `blacklist:{token}` | — |

## 5. Contradictions entre le code et l'architecture annoncée

1. **Room-API « gère les connexions » : faux.** Les fichiers
   `back/Room-API/src/ws/ws_handler.rs` et `ws/socket_manager.rs` sont vides.
   Rien n'observe les déconnexions ; l'état `user:session` ne peut être libéré
   que par appels API explicites.
2. **[RÉSOLU, re-vérifié le 20/08]** « Le healthcheck de tous les services
   Rust appelle `curl` alors que les images ne l'embarquent pas » — ce
   n'est plus vrai. Les 7 `Dockerfile` de `back/*` (Gateway, Auth, User,
   Chess, Room, Social, Notification) installent désormais explicitement
   `curl` dans leur étage final (`apt-get install -y --no-install-recommends
   ca-certificates curl`), confirmé par lecture directe de chaque
   Dockerfile. Ce piège d'exploitation, documenté à la fois dans
   `AUDIT_PROMPT.md` et `AGENTS.md`, ne devrait donc plus se produire sur le
   code actuel — mais comme il concerne le comportement d'exécution du
   stack (jamais démarré pendant cet audit), une confirmation en conditions
   réelles (`docker compose ps`, tous les services `healthy`) reste
   recommandée avant de considérer le point définitivement clos.
3. **La gateway « fait du RBAC » sauf sur le WS d'échecs — MISE À JOUR : plus
   vrai depuis le 19-20/08.** La gateway elle-même ne valide toujours rien sur
   la branche WS (`Gateway-API/src/http/handlers/router.rs:64-116`, simple
   transfert du cookie), mais **Chess-API valide désormais lui-même** :
   `back/Chess-API/src/websocket/handler.rs` exige un jeton (401 sinon) et
   vérifie que l'utilisateur fait partie de `chess:game_players:{game_id}`
   (403 sinon), une liste écrite par Room-API à la création de la partie. Les
   slots ne sont plus attribués « premier arrivé premier servi » à
   n'importe qui : seuls les deux `user_id` enregistrés peuvent se voir
   attribuer `player1`/`player2` (`websocket/lobby.rs::connect`). Contrepartie
   non désirée : le mode spectateur, qui reposait sur ce même chemin, a été
   supprimé en même temps (voir bug #18) — la contradiction RBAC est résolue
   au prix d'une régression fonctionnelle.
4. **Room-API et Social-API ont le même port par défaut (8003)**
   (docker-compose.yml:244, 275). Ça ne collisionne que parce que `.env`
   surcharge SOCIAL_API_PORT ; une installation neuve avec `.env.example`
   (SOCIAL_API_PORT=4004 mais ROOM absent → 8003) passe, mais deux services
   « différents » revendiquent le même port par défaut — fragile.
5. **[RÉSOLU, re-vérifié le 20/08]** « `run_game_loop` ne se termine jamais
   et les parties ne sont jamais retirées du `GameManager` » — ce n'est
   plus vrai. `back/Chess-API/src/game/manager/manager.rs:58-63` a
   désormais une méthode `remove_game`, appelée par
   `back/Chess-API/src/game/game_loop/runner.rs:110` (fin de partie par
   grace period sans joueurs) **et surtout ligne 183**, en fin de fonction,
   **inconditionnellement après toute sortie de la boucle** (`break` sur
   `game.ended`, timeout premier coup, ou timeout d'horloge). Le passage à
   `ended=true` sur échec et mat est fait depuis
   `back/Chess-API/src/game/commands.rs:420`
   (`instance.game_loop.end()`), donc ce chemin est également couvert. La
   fuite de tâche/mémoire décrite initialement semble corrigée par ce
   refactor (commit `dfe8abf "feat: game loop refactor + session/redis
   helpers"`).
6. **Les WS Chess-API (8082/8083) sont publiés directement sur l'hôte**
   (docker-compose.yml:177-178, 210-211), contournant nginx et la gateway :
   le discovery Redis de la gateway devient optionnel pour un attaquant, et
   `CHESS_PUBLIC_WS_URL` (variable morte côté front) témoigne d'une intention
   abandonnée à moitié nettoyée. **Nuance ajoutée le 20/08** : contourner la
   gateway ne contourne plus le contrôle d'accès (point 3 mis à jour) —
   Chess-API applique son propre gate jeton+appartenance quel que soit le
   chemin réseau emprunté. L'exposition directe des ports reste néanmoins à
   corriger (surface d'attaque, bypass du rate-limit/logs centralisés de la
   gateway — voir bug #31).
7. **La description officielle dit « le front construit son URL depuis
   l'adresse de la page » — exact** (useChessGame.ts:174-181), mais la
   résolution gateway échoue silencieusement en fallback statique
   (`Gateway-API/src/http/handlers/router.rs:77-79`) : si le mapping Redis a
   expiré, le WS est routé vers la mauvaise instance et la connexion échoue
   avec « Game not found » sans que la cause soit visible côté front.
8. **Un token de tunnel Cloudflare est commité (commenté) dans
   docker-compose.yml:414-421** — resté là après suppression du service.

## 6. Points d'attention pour la suite des corrections

- Toute la synchronisation d'état du jeu repose sur des messages complets
  (`game_state`, `hand`, `turn_changed`) poussés à chaque événement, sans
  séquenceur ni horloge serveur périodique : les timers côté client sont
  extrapolés localement. Voir ADR-002. **Mise à jour** : l'ordre de replay du
  cache du SharedWorker au reconnect (`shared-ws-worker.js::replayState`) a
  été corrigé depuis la version précédente de cet audit (`started` est
  désormais rejoué avant `turn_changed`) — un des contributeurs identifiés
  pour les bugs #15-#17 est donc probablement résolu, l'absence d'horloge
  serveur périodique reste vraie.
- La jouabilité des cartes est calculée serveur (`is_card_playable`,
  envoyée dans chaque `hand`) — **mise à jour : n'est plus ignorée par le
  front**. `CardRegister.playable` existe désormais côté client
  (`cardTypes.ts:9`) et conditionne l'affichage du bouton « Jouer »
  (`CardsModal.tsx:110,182-206`). Le champ `playable_if` du serveur ne teste
  cependant que l'existence d'une pièce du bon type, jamais son `has_moved` :
  les cartes Vétéran restent donc mal couvertes par ce garde-fou (voir
  « Cause commune » dans `BUGS.md`). Voir ADR-001, dont la recommandation
  de rejet explicite serveur reste valable pour fermer complètement le
  protocole (y compris contre un client qui n'implémente pas la vérification
  front, ex. un client WS écrit à la main).

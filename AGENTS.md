# AGENTS.md

## Contexte projet

Projet 42 ft_transcendence (« Cardmate ») : échecs en ligne avec cartes, avant la soutenance.
**Audit/recherche uniquement demandé par l'utilisateur : ne pas modifier les fichiers ni pusher ; git uniquement pour `pull` et lecture.**

## Structure — superprojet à 13 sous-modules git

`git pull` ne met à jour que les pointeurs : **toujours suivre de `git submodule update --init --recursive`**, sinon on lit du code périmé.

- `back/Gateway-API` — seule entrée back (axum). Route `/api/<service>/…` : HTTP direct pour `auth`/`notifications`, **streams Redis** (`<service>:requests` → `gateway:responses`) pour user/room/social/chess. Proxy WS avec découverte Redis de l'instance Chess (`chess:game:{id}` → `chess:instances`).
- `back/Auth-API` — register/login, OAuth Google+42, JWT RSA (clé pub dans Redis `auth:jwt:public_pem`), e-mails Resend.
- `back/User-API` — profil, inventaire, **shop**, packs, admin ; migrations SQL dans `src/db/migrations.rs` (run noms `NNN_…`, **une migration appliquée ne se rejoue jamais** — modifier une existante n'a aucun effet sur une base en place).
- `back/Chess-API` — moteur + cartes, 2 instances (chess-1/chess-2), état en mémoire par partie. Refactoré : `game.rs` découpé en `game/commands.rs` (handle des commandes WS), `game/session.rs` (session joueur), `game/redis_helpers.rs`. `EXPLICATION.md` à la racule du sous-module documente l'architecture — le lire en premier.
- `back/Room-API` — rooms, matchmaking, ELO, tournois ; worker Redis uniquement, **aucun websocket** (les fichiers ws sont vides).
- `back/Social-API` — amis, messages. `back/Notification-API` — SSE.
- `back/api-core` — bibliothèque partagée (routeur Redis, JWT, SSE, rate-limit, métriques) utilisée par tous les services Rust.
- `front/` — React 19 + TS + Vite + Tailwind + react-i18next (6 locales dans `src/i18n/locales/`). Sockets via SharedWorker dans `front/public/shared-ws-worker.js` (cache/replay des messages d'état).
- `infra/Nginx` — reverse proxy unique (:8000) ; `infra/metrics` (DevOps) — ELK + Prometheus/Grafana.

## Commandes clés

- Démarrage complet : `make up` (build + up + `publish-assets` + import dashboard Kibana). Prod : `make prod`. Dev attaché : `make dev`.
- **Assets → MinIO uniquement sur `make publish-assets`** : tout fichier ajouté/modifié dans `assets/` reste invisible du navigateur sinon (le bucket garde les anciennes clés après renommage).
- Front seul : `cd front && npm run dev|build|lint` (`build` = `tsc -b && vite build`). Pas de test front.
- Back : `cargo build`/`cargo test` dans chaque sous-module Rust (ex. `cd back/Chess-API && cargo test` — tests unitaires dans `src/game/board/mod.rs`).
- ELK : `make kibana-password` (régénère `kibana.env` au démarrage), `make kibana-import`.

## Pièges d'exploitation (ne pas confondre avec des bugs de code)

- **nginx résout les noms de services au démarrage seulement** : après redémarrage d'un service back → 502 partout ; remède `docker compose restart nginx`.
- **`front` monte `/app/node_modules` en volume anonyme** : une dépendance ajoutée n'est jamais installée par un pull ; `docker exec frontend_dev npm install`.
- **Toute route hors `/img/…` renvoie `index.html` en 200** (SPA fallback) : une image à URL fausse produit un 200 HTML, pas de 404 — le `onError` ne se déclenche pas.
- **Healthchecks des services Rust appellent `curl` absent des images** → conteneurs `unhealthy` et nginx (qui dépend de `service_healthy`) peut ne jamais démarrer à froid. Vérifier `docker compose ps` avant toute démo.
- Secrets réels (OAuth Google/42, token Cloudflare) encore dans l'historique git ; `kibana.env` (mot de passe) est généré et commité par le Makefile.

## Conventions

- Issues GitHub : `[BUG]`/`[FEATURE]`/`[TECH]`/`[DOC]`/`[SECURITY]` + labels `area: backend|frontend|infra-cicd`, `p0`–`p2`. L'audit complet est dans `audit/` (BUGS.md, ADR/, INCONNUES.md) — issues #38-#79 en découlent.
- Cartes retirées volontairement du jeu (Bastion, etc.) : commentées dans `back/Chess-API/src/game/cards/{registry,types,effects}.rs` mais parfois encore référencées front — c'est un choix, pas un bug à « corriger » sans demande.
- Noms d'assets cartes : base `canon_0` (un n), variantes `_<rarity>` ; sérialisation serde `snake_case` côté Rust, URLs composées dans `front/src/data/cards.ts` (`cardImageDeckVariant` etc.).
- Front parle au WS via `/api/chess/chess?game_id=…` (jamais les ports 8082/8083 directs).

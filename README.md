*This project has been created as part of the 42 curriculum by madelvin, qutruche, tarini, agantaum, and almeekel.*

# Cardmate

Real-time online chess with a twist: players draw **cards** that bend the rules of
the game : fog of war, extra moves, piece transformations, deadly zones, etc. Around the board sit ranked 1v1 matchmaking,
private rooms with a join code, a public room list, a friends system with
direct messaging, and a cosmetics shop where players spend an in-game currency
on avatar parts. They can review their skins in a dressing room.

## Description

### Goal

Build a production-shaped web application as a team: a multiplayer browser
game, backed by a microservice architecture, deployed with a single command,
observable in production (metrics, logs, alerting).

### Key features

- **Chess with cards** : a full chess engine (`back/Chess-API`) plus a
  30-card system with three rarity tiers (Common / Epic / Legendary, currently only available for some cards) that
  alter play: fog of war, Russian roulette, piece transformations (cannon,
  sniper, veteran knight/rook/bishop, ninja), deadly zones, a battlefield
  zone, a fortune wheel, a traitor, etc.
- **Matchmaking and rooms** : a ranked 1v1 queue, private rooms opened by a
  join code, a public room list.
- **Social** : friend requests, block/unblock, 1:1 direct messages, room invites sent directly in chat.
- **Avatar system and shop** : cosmetic items (base, hat, mask, clothes,
  accessory) equipped in a dressing room, cosmetics and bundle collections
  sold for an in-game currency, gacha-style pack opening.
- **XP and levels** : a persisted level/XP system shown as a progress bar
  on the player's profile.
- **Admin panel** : user CRUD, a full roles/permissions (RBAC) editor, and
  per-route rate-limit configuration, all from the browser.
- **Six languages** : German, English, Spanish, French, Italian, Serbian.
- **Full observability stack** : Prometheus + Grafana dashboards and an
  ELK (Elasticsearch/Logstash/Kibana) log pipeline, both actually wired into
  the deployment, not just present in the repo.

## Instructions

### Prerequisites

| Tool | Version |
| --- | --- |
| Docker | 28+ with Compose v2 (the root `docker-compose.yml` uses `include:`, which needs a recent Compose) |
| Git | any recent version, with submodule support |

### Setup

```bash
git clone --recurse-submodules git@github.com:ft-transcendence-tkt-on-vera/ft_transcendence.git
cd ft_transcendence
```

If you cloned without `--recurse-submodules`, or a submodule was added since
your last pull:

```bash
git submodule update --init --recursive
```

The project is split across **13 submodules** declared in `.gitmodules`.
Use `git submodule update` to fetch their content.

Then create `.env` at the repo root. 
The variables it expects, grouped by what they're for:

| Group | Variables | Notes |
| --- | --- | --- |
| Database | `POSTGRES_DB`, `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_PORT`, `DATABASE_URL` | |
| Redis | `REDIS_PORT`, `REDIS_URL` | sessions, inter-service streams, JWT key distribution |
| Object storage | `MINIO_ROOT_USER`, `MINIO_ROOT_PASSWORD`, `MINIO_PUBLIC_ENDPOINT` | cosmetic and card assets |
| Service ports/URLs | `GATEWAY_API_PORT`, `AUTH_API_PORT`, `AUTH_API_URL`, `USER_API_PORT`, `USER_API_URL`, `CHESS_PORT`, `SOCIAL_API_PORT`, `NOTIFICATION_API_PORT`, `NOTIFICATION_API_URL`, `ADMINER_PORT` | |
| OAuth | `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_REDIRECT_URI`, `FT_CLIENT_ID`, `FT_CLIENT_SECRET`, `FT_REDIRECT_URI` | Google and 42 login |
| Email | `RESEND_API_KEY`, `DOMAIN_EMAIL` | transactional email; **without a valid key no account can be validated by email** |
| HTTPS delivery | `CLOUDFLARE_TUNNEL_TOKEN`, `DOMAIN_NAME`, `ALLOWED_HOSTS` | see [HTTPS delivery](#https-delivery) below |
| Monitoring / ELK | `GRAFANA_PASSWORD`, `ELASTIC_PASSWORD`, `LOGSTASH_PASSWORD`, `KIBANA_PASSWORD`, `VIEWER_PASSWORD`, `METRICS_MANAGER_URL` | |
| Admin bootstrap | `ADMIN_EMAIL`, `ADMIN_PASSWORD`, `ADMIN_BIO` | seeds the first admin account |
| Misc | `DISCORD_WEBHOOK_URL` | Grafana alert contact point (confirm with the team whether it doubles as the project's chat channel) |

### Run

```bash
make
```

`make` (alias for `make up`) checks the `.env` file exists, builds every
service, brings up Postgres/Redis/MinIO/Nginx, the Cloudflare tunnel, the
`scalar_doc` API-reference container, and — via `docker-compose.yml`'s
`include:` directive — the ELK stack (`infra/metrics/elk/docker-compose.yml`)
and the Prometheus/Grafana stack
(`infra/metrics/monitoring/docker-compose.yml`), then publishes assets to
MinIO and imports the prebuilt Kibana dashboard. `docker compose up -d --build`
also works but skips those last two steps.

| Command | Effect |
| --- | --- |
| `make` / `make up` | build, start everything, publish assets, import Kibana dashboard |
| `make dev` | same, but attached (streams logs) |
| `make publish-assets` | re-upload `assets/` to MinIO after changing an image |
| `make down` | stop containers, keep volumes |
| `make re` | full restart |
| `make ps` / `make logs` | status / follow logs |
| `make ultraclean` | **destroys volumes**: database, MinIO objects, ELK indices |

### Operational notes

- Nginx resolves service names once at startup. After rebuilding a backend
  service, `docker compose restart nginx` — otherwise routes answer 502.
- Every backend service embeds `api-core` at compile time, so a change to the
  shared library needs a rebuild (`--build`), not just a restart.
- After `make ultraclean`, assets must be republished; `make` does this
  automatically.

## Team Information

*Roles below were drafted from git commit distribution across all 13
submodules (see [Individual Contributions](#individual-contributions) for the
underlying data), then confirmed and refined by the team.*

| Login | Role(s) | Responsibilities |
| --- | --- | --- |
| madelvin | Technical Lead / Architect, Developer | Owns the large majority of every backend service, plus a share of the infrastructure — the project's backend architecture, chess engine, card system, and shared library are primarily this person's work. Also the top single contributor to the frontend (43% of `front` commits). |
| almeekel | Lead Design, Developer | Drove the project's visual and artistic identity (card and piece art, UI/UX and design-system choices). Also built frontend features — notably the shop. |
| qutruche | Frontend Developer, Lead Front, Scrum Master | Near-exclusively frontend, he was the frontend specialist and lead on the client codebase. |
| tarini | DevOps, Developer | Owns `infra/metrics` (96% of its commits) and is the #2 contributor to `infra/Nginx` (28%) — owner of the Prometheus/Grafana/ELK monitoring stack. |
| agantaum | Product Owner, Developer | Originated the game's core concept — chess twisted by a card system — and worked mainly on the dressing room / skin room feature. Plus a handful in Auth-API, User-API, Chess-API. |

## Project Management

The team's first key decision — and the real foundation the rest of the
project was built on — was to commit to a genuine engineering organization
from day one: a proper containerized development infrastructure (every
service in Docker Compose) paired with real DevOps practice (monitoring,
logging, alerting), rather than treating those as an afterthought.

- **Structure**: the codebase is a superproject with 13 git submodules, one
  per service/concern, each with its own history, pull requests, and
  branches.
- **Communication**: Discord is the team's main channel. Dedicated channels
  filter and route information — proposed features, bugs, and optimizations
  each have their own space — with processes adapted to keep discussion
  organized rather than scattered. The `DISCORD_WEBHOOK_URL` configured for
  Grafana alerting posts into the same server.
- **Tooling**: GitHub is used for everything — pull requests, issues, the
  submodule hierarchy, and branching — with a constant, real back-and-forth
  review between members on every feature. Notion was tried as an external
  planning tool but was never really exploited to its full potential.
- **Creative decisions**: anything touching the artistic direction and
  gameplay feel — the soul of the project — was consistently discussed and
  debated as a team, and only moved forward once everyone had agreed.
- **Cadence**: weekly meetings, increasing in frequency toward the end of the
  project. Most day-to-day communication happened in person / directly
  rather than asynchronously.
- **Feedback loop**: some members recruited outside testers during
  development, building a live, dynamic feedback loop rather than relying
  only on internal testing.
- **Code review**: qutruche and madelvin most consistently tested and
  re-read incoming code, leaving comments on practice and usage directly in
  the PR and sending changes back for rework when needed — a real,
  substantive review process, not a rubber stamp. The frontend repo (`front`)
  reflects this: by far the most merge-PR commits of any submodule (121, vs.
  single digits to low-teens elsewhere).

## Technical Stack

### Frontend

React 19 with TypeScript, built by Vite 8 (`front/package.json`). Routing
with react-router 6, styling with Tailwind CSS 4, internationalisation with
i18next (6 languages, `front/src/i18n/locales/{de,en,es,fr,it,sr}`),
drag-and-drop with `@dnd-kit`, animation with Framer Motion, carousels with
Embla, motion graphics with `lottie-react`. UI primitives follow a shadcn-style
pattern (14 components in `front/src/components/ui/`) on one dedicated
display font (`@fontsource/libre-caslon-text`)
plus Geist for UI text.

**Why React** — the team wanted to learn it, and its ecosystem covers routing,
i18n, and state without pulling in a heavier framework. The subject counts it
as a framework.

### Backend

Rust across all seven services, with `axum` for HTTP/WebSocket, `tokio` for
the async runtime, `sqlx` for compile-time-checked Postgres access,
`jsonwebtoken` + in-house RSA key management for auth, `bcrypt` for password
hashing.

**Why Rust** — The team was interested into the highly performant and adaptative nature of the back.

Eight services (seven APIs + the `api-core` shared library) communicate over
**Redis Streams**, not direct HTTP, except for the game WebSocket which the
Gateway proxies directly to whichever Chess-API instance owns the game:

| Service | Responsibility |
| --- | --- |
| `Gateway-API` | Single public entry point behind Nginx. Proxies HTTP to services over Redis Streams, proxies the chess WebSocket to the right `chess-1`/`chess-2` instance (Redis-based discovery), validates the JWT session cookie, enforces per-user/per-route rate limiting and RBAC permission checks, exposes Prometheus metrics. |
| `Auth-API` | Registration, email+password login (bcrypt), 42 and Google OAuth, JWT access/refresh issuance and rotation, email verification (6-digit code + link) and password reset via Resend, account ban/deletion, provider switching. |
| `User-API` | Player profile, settings/language, profile picture, the cosmetics shop and pack system, card inventory/deck, XP/leveling, and the full admin RBAC panel (roles, permissions, per-route rate limits). Owns the largest slice of the schema (53 migrations). |
| `Room-API` | Room lifecycle (create/join/leave/kick/start), ranked 1v1 matchmaking, tournament logic (backend only — no frontend UI exists for it), live-game and public-room list broadcasting. Talks to `Chess-API` over Redis Streams to start games; keeps most of its own state in Redis rather than Postgres. |
| `Chess-API` | The chess engine and per-game WebSocket loop: board state, legal moves, check/mate detection, and the 30-card modifier/effect system (fog, deadly zones, piece transformations, etc.). Deployed as two instances (`chess-1`, `chess-2`) registered in Redis for discovery/load-balancing. |
| `Social-API` | Friend requests (send/accept/refuse/cancel), block/unblock, friend list, direct messages with read tracking. Operates on tables owned by `Auth-API`/`User-API`. |
| `Notification-API` | Real-time event fan-out over Server-Sent Events — no database of its own. Relays presence, room updates, the friend-request lifecycle, new chat messages, profile-picture updates, and live-game/room lists to connected clients. |
| `api-core` | Shared library: JWT manager (RS256, keys generated on first boot and shared via Redis), a lightweight SQL migration runner, Redis pool/pub-sub helpers, the generic SSE primitives used by `Notification-API`, RBAC permission-key helpers, and the rate-limit counter logic used by the Gateway. |

### Database

PostgreSQL 16 via `sqlx`. Redis is used for sessions, JWT key distribution,
inter-service Streams messaging, and most of `Room-API`'s transient state
(queues, rooms, tournaments). MinIO (S3-compatible) stores cosmetic and card
images.

**Why PostgreSQL** — The technical manager (madelvin) and front expert dev (qutruche) were most familiar with PostgreSQL and the rest of the team was planning to learn it.
There are no `.sql` migration files anywhere in the repo — the schema is
defined as Rust string-literal migrations run at boot
(`back/api-core/src/db/migration.rs`, per-service `src/db/migrations.rs`),
tracked in the `_migrations` table.

## Database Schema

```
users ──1:1── user_profile
  │
  ├──1:N── player_inventory
  ├──1:N── player_cards / player_deck
  ├──1:N── friendships          (user_id, friend_id)
  ├──1:N── friend_messages      (sender_id, receiver_id)
  ├──1:N── refresh_tokens
  ├──1:N── user_setting
  ├──1:N── user_roles ──N:1── roles ──N:N── permissions
  └──1:N── games                (white_user_id, black_user_id)

collections ──1:N── collection_items ──N:1── shop_catalog
roles ──N:N── permissions (role_permissions) ──N:N── api_routes (permission_routes)
api_routes ──1:1── rate_limits
```

| Table | Owner | Key fields |
| --- | --- | --- |
| `users` | Auth-API | `id`, `username`, `email`, `password_hash`, `account_validated`, `email_validated`, `wallet`, `role`, `auth_provider`, `is_banned` |
| `refresh_tokens` | Auth-API | `id`, `user_id → users`, `token_hash` (SHA-256), `expires_at`, `revoked` |
| `user_profile` | Auth-API + User-API | `user_id → users`, `ranked_elo`, `picture_id`, `level`, `xp` |
| `games` | Auth-API | `id`, `game_id`, `result`, `winner`, `white_user_id`, `black_user_id` |
| `friendships` | User-API | `(user_id, friend_id)` unique, `status` in pending/accepted/blocked |
| `friend_messages` | User-API | `sender_id`, `receiver_id`, `content`, `read_at` |
| `player_inventory` | User-API | `user_id`, `item_id`, `item_type`, `item_rarity` |
| `player_cards` / `player_deck` | User-API | card ownership + active deck; a Postgres trigger seeds default cards on account creation |
| `shop_catalog` | User-API | `(item_id, item_type)` PK, `title`, `price`, `asset_key`, `is_active` |
| `collections` / `collection_items` | User-API | bundled cosmetics with a price and optional end date |
| `user_setting` | User-API | `user_id`, `lang` |
| `roles` / `permissions` / `role_permissions` / `permission_routes` / `api_routes` / `rate_limits` / `user_roles` | User-API | the admin-panel RBAC system: custom roles, permission definitions, which API routes each permission covers, and per-route rate limits |

`Room-API`, `Chess-API`, and `Social-API` own no tables of their own — they
query the tables above directly and keep their own transient state (rooms,
matchmaking queues, tournament brackets, live game sessions) in Redis.

### The avatar identifier

`user_profile.picture_id` encodes a whole outfit in one value: five 12-bit
fields packed into a `u64`, then base62-encoded.

```
picture_id = base | hat<<12 | mask<<24 | clothes<<36 | accessory<<48
```

An item's `item_id` **is** its slot value, which is why shop items are
numbered 1..4095 per slot and why an asset lives at
`<item_type>/<item_id>.png` in the `cosmetics` MinIO bucket. The server
refuses to equip an item the player does not own.

## Features List

| Feature | Author(s) | Description |
| --- | --- | --- |
| Chess engine | madelvin | Move generation, legality checking, check/checkmate detection (`back/Chess-API/src/game/board`) |
| Card system | agantaum | 30 cards, 3 rarity tiers, board- and piece-level effects (fog, deadly zones, piece transformations, time manipulation, etc.) (`back/Chess-API/src/game/cards`) |
| Artistic identity and design | almeekel | Visual language and design direction for the project — card and piece sprite art, and the site's overall design choices (UI/UX, typography, palette) |
| Live game WebSocket | madelvin | Per-game session/loop, two horizontally-scaled instances discovered via Redis (`back/Chess-API/src/websocket`) |
| Ranked matchmaking | qutruche, madelvin | 1v1 queue pairing players by ELO (`back/Room-API`, `useMatchmaking.ts`) |
| Rooms | qutruche, madelvin | Private rooms by join code, public room list, live-games list |
| Authentication | madelvin | Email/password, Google and 42 OAuth, email verification, password reset, session refresh via a SharedWorker |
| Friends and chat | madelvin, qutruche | Requests, block/unblock, 1:1 direct messages, room invites sent in chat |
| Avatar / dressing room | agantaum | Five cosmetic slots composited into one `picture_id`, equipped (not uploaded) from owned items |
| Shop and packs | almeekel, madelvin | Cosmetics and collections bought with in-game currency, gacha-style pack opening |
| XP / levels | madelvin, qutruche | Persisted level/XP shown as a progress bar on the profile |
| Admin panel | madelvin, tarini | User CRUD, roles/permissions editor (RBAC), per-route rate-limit configuration, card grants |
| Internationalisation | almeekel, madelvin | Six languages with a switcher, all UI strings translatable |
| Monitoring | tarini | Prometheus + Grafana, custom dashboards, Discord alert contact point |
| Log management | tarini, madelvin | Filebeat → Logstash → Elasticsearch → Kibana, dashboard auto-imported by `make` |

## Modules

| Module | Type | Points | Author(s) | Evidence |
| --- | --- | --- | --- | --- |
| Web-based game | Major | 2 | madelvin | Full chess engine + rules, live matches over WebSocket |
| Remote players | Major | 2 | madelvin | WebSocket game session reachable from any two browsers/computers |
| Frameworks (frontend) | Minor | 1 | qutruche | React (frontend) |
| Real-time features (WebSocket) | Major | 2 | madelvin | Chess moves over WebSocket |
| User interaction (chat, profile, friends) | Major | 2 | madelvin, qutruche | 1:1 chat, profile page, friend requests/list all present and working |
| Backend as microservices | Major | 2 | madelvin | 7 single-responsibility Rust services + shared lib, communicating over Redis Streams |
| Monitoring (Prometheus / Grafana) | Major | 2 | tarini | Real scrape config, provisioned dashboards, alerting rules with a Discord contact point (`infra/metrics/monitoring/`) — actually included in `docker-compose.yml`, not vestigial |
| Log management (ELK) | Major | 2 | tarini, madelvin | Filebeat → Logstash → Elasticsearch → Kibana, real pipeline config, dashboard auto-imported by `make` (`infra/metrics/elk/`) |
| OAuth 2.0 | Minor | 1 | madelvin | Google and 42 login (`back/Auth-API/src/http/handlers/{google_code,ft_oauth_42}.rs`) |
| Multiple languages | Minor | 1 | almeekel, madelvin | 6 complete translations, language switcher |
| Game customization (cards) | Minor | 1 | agantaum | 30 cards, 3 rarity tiers, admin card-grant tool |
| Advanced permissions system | Major | 2 | madelvin, tarini | Strongly implemented (`UsersPanel`, `RolesPanel`, `PermissionsPanel` CRUD, RBAC tables) |
| Health check / status page with automated backups and disaster recovery | Minor | 1 | tarini | Every service exposes `/health` and Docker healthchecks are configured — but no automated backup or disaster-recovery procedure was found. |

## Individual Contributions

Project ran roughly mid-April to late-August 2026 (first commits across
`root`, `Gateway-API`, `Auth-API`, `infra/Nginx` on the same day, consistent
with a coordinated bootstrap), building out in phases: auth/gateway/infra
first, then user/frontend, then the chess engine (mid-June), then
social/notification features last (July). The design flavor came last in August.

| Login | Contribution |
| --- | --- |
| madelvin | Most of every backend service (Gateway, Auth, User, Room, Chess, Social, Notification, api-core), plus a sizeable contribution to `front` and all of `documentation`. |
| qutruche | The core of `front`. |
| almeekel | A contribution to `front`, some root-level integration commits (submodule bumps, Nginx), asset creation and design language choices (assets, cards, colors, UX/UI choices). |
| tarini | All of `infra/metrics` (Prometheus/Grafana/ELK config), a sizeable contribution to `infra/Nginx`. |
| agantaum | A key contribution in `front` and in the back, Auth-API/User-API/Chess-API. |

### Challenges faced

- **almeekel** — For most of the developmental stage of the project, a
  placeholder design disgruntled testers and put us at the greatest risk: a
  solid spine (the code) but the lack of a soul that one could feel in the
  website and the game. The key improvement I brought to the project lies in
  the UI/UX and graphic design of the website and game, which gives it a
  distinct, gritty low-poly 2010s bloody and dark vibe, combined with a
  clean, boxy aesthetic for the icons.
- **agantaum** — The struggle during the project was to learn to use
  efficient CSS, then Tailwind, which isn't a framework I'm proficient with.
  Moreover, the challenge was to create something not ugly, and it was far
  harder than expected.
- **madelvin** — I ended up touching almost everything and building most of
  the backend, but that's mostly because I started earlier than
  the rest of the team, so I was the one laying the foundations while
  everyone else was still on the previous milestone. Writing the base was a challenge, but I think the most challenging ended up leading the project
  when were up and running. Reviewing what everyone built, and taking on the less
  glamorous work so the team could keep moving instead of getting
  stuck on it. Less about carrying the project alone, more about making sure
  everyone else could keep building on solid ground.
- **qutruche** — Getting the team onto a real GitHub workflow, with an actual
  branches, pull requests, and reviews instead of pushing straight to main, 
  was a fight at first. it slowed everyone down for a week and I had to keep
  explaining why it was worth it. It paid off: `front` ended up with far more
  reviewed PRs than any other repo, and it's the reason we could move fast
  later without stepping on each other's changes. On top of that I laid out
  the frontend architecture, with a feature-based structure (`src/features/*`)
  instead of one flat pile of components, so chess, shop, social, and auth
  each own their components/hooks/services and don't leak into each other.
- **tarini** — I set up the entire stack from scratch: ELK to centralize logs, Prometheus/Grafana for monitoring and alerting, healthchecks across all microservices, end-to-end HTTPS, and Cloudflare Tunnel with Access to secure public exposure.
The real challenge was making it all hold together while learning new tech along the way, without breaking everything else with each addition.

## Resources

### Documentation

- [axum](https://docs.rs/axum/) — backend HTTP/WebSocket framework
- [sqlx](https://docs.rs/sqlx/) — compile-time checked SQL
- [React](https://react.dev/) and [react-router](https://reactrouter.com/)
- [Tailwind CSS](https://tailwindcss.com/docs)
- [Redis Streams](https://redis.io/docs/latest/develop/data-types/streams/) — inter-service messaging
- [MinIO](https://min.io/docs/minio/linux/index.html) — S3-compatible object storage
- [Elastic Stack](https://www.elastic.co/guide/index.html)
- [Prometheus](https://prometheus.io/docs/introduction/overview/) / [Grafana](https://grafana.com/docs/)
- [Scalar](https://guides.scalar.com/scalar/scalar-api-references/introduction) — OpenAPI reference UI (`scalar_doc` service)

### Use of AI

AI assistance was used for the following, specific
tasks :

- **Architecture and bug audit**: a structured audit of the whole
  microservice architecture was requested — analysis documents only, no code was written or modified by that session.
- **Card and chess-piece artwork**: in a human/AI working organization using Midjourney, ComfyUI, Claude, Gemini, human fine-tuning and design decisions, the help of artists and 100% human-made art, a visual language was crafted and the cards, UI/UX and background images were brought to the game.
- **This README**: the base was drafted with AI assistance from git history and a static read of the codebase across all 13 submodules, then thoroughly reviewed and corrected by the team.
- **Documentation and understanding**: In some cases, documentation and code patterns were obtained through usage of AI or AI-powered web research. The same goes for learning new concepts.
- **Coding repetitive patterns and completion**: For documentation-copying patterns as well as for reproduced patterns, AI was used. This intertwines with the next point. We believe this is an important skill to cultivate as it appears as a likely prominent productivity factor in the upcoming years in the coding industry.
- **POCs**: Another big use of AI was creating code and testing it into the project to achieve a temporary feature or goal and then modify, edit, understand it to finally get back into the coding loop and implement it into the project. 

## Known Limitations

- Email validation requires a valid `RESEND_API_KEY`; without one, accounts
  must be validated directly in the database.
- The upload endpoint `POST /api/user/shop/items` is **still** not
  restricted by role — any authenticated user can upload catalog items. This
  is a known, deferred security fix; the gateway needs to filter on
  `users.role` before submission.
- HTTPS is delivered through a Cloudflare Tunnel, not a locally generated
  certificate — see the [HTTPS delivery](#https-delivery) box above for what
  that means for the defence.
- 19 of the 31 cards only have tier-0 (Common) artwork and fall back to a
  placeholder at higher rarities; `traitre` and `zone_mortelle` have 2 of 3
  tiers. 10 cards (`bastion`, `canon`, `champ_de_bataille`, `fog`,
  `magnetisme`, `ninjaaaa`, `roue_de_la_fortune`, `sniper`,
  `veteran_cavalier`, `veteran_tour`) have complete artwork across all three
  tiers.
- Piece-modifier sprites (the chess-piece art shown when a card transforms a
  piece) are now complete for every modifier the backend defines — `cannon`,
  `sniper`, `veteran_knight`, `veteran_rook`, `veteran_bishop` all have
  black/white sprites for every rarity tier their card art supports; `ninja`
  uses a cloud overlay instead of a piece sprite by design.
- **Naming mismatch, worth fixing before it's relied on**: the frontend's
  sprite-selection code (`front/src/features/games/ChessGame/components/utils/pieces.tsx`)
  builds piece sprite paths as `{type}_{color}--{modifier}--{rarity}.svg`
  (English backend key, double dash before rarity), but every shipped sprite
  file — old and new — is actually named
  `{type}_{color}--{frenchCardName}_{rarity}.svg` (single underscore, French
  card word, e.g. `bishop_black--sniper_0.svg`). Whether pieces actually
  render correctly in the running app has not been verified end-to-end.
- The real-money currency packs (`front/src/features/shop/data/moneyPacks.ts`)
  have no payment-provider integration — likely a mock/demo purchase flow.
- The tournament backend (`Room-API`) has no frontend to drive it — see
  [Modules](#modules).
- `argon2` is a declared backend dependency that is never actually used
  (`bcrypt` is what password hashing actually runs on) — a dead dependency
  worth removing or explaining.
- The separate `infra/Grafana` and `infra/Prometheus` directories contain
  dashboards that are **not** referenced by any compose file — the actually
  deployed monitoring stack lives under `infra/metrics/monitoring/` instead.
  These two directories look like earlier, now-orphaned work.


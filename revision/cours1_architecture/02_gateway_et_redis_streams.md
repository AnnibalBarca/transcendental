# Cours 1 — Chapitre 2 : Gateway-API et Redis Streams

Jour 1 — 2026-08-23

Objectif : pouvoir dessiner au tableau le trajet complet d'une requête HTTP
et d'une connexion WebSocket depuis le navigateur jusqu'au service back
concerné, en expliquant pourquoi deux mécanismes de transport coexistent.

## 1. Le concept : passerelle API (reverse proxy applicatif)

Une gateway API est le point d'entrée unique d'un système multi-services :
elle reçoit toutes les requêtes externes et décide vers quel service interne
les router, sans que le client n'ait jamais besoin de connaître l'adresse
réelle de ce service. **Analogie C/C++** : c'est un `switch` sur un
identifiant de commande qui appelle des fonctions différentes selon le cas,
sauf que chaque « fonction » est un processus séparé, potentiellement sur une
autre machine, et que l'appel doit franchir le réseau au lieu d'un simple
`call`.

Ici, Gateway-API (axum, Rust) est **la seule porte d'entrée** du back (voir
`AGENTS.md` : « seule entrée back »). Le reste des services Rust n'exposent
aucun port HTTP consommé directement par le navigateur (à l'exception notable
des ports WS de Chess-API exposés directement — voir §5 « contradiction 6 »
plus bas, un écart volontaire à connaître).

## 2. Le concept : deux mécanismes de transport, un seul point d'entrée

Un système peut connecter deux services de deux façons : **synchrone directe**
(HTTP : j'appelle, j'attends la réponse sur la même connexion) ou
**asynchrone via un intermédiaire** (file de messages : je dépose une requête
dans une queue, un autre processus la consomme à son rythme, dépose la
réponse dans une autre queue, et j'attends cette réponse-là). Ce projet utilise
les deux, **par choix explicite selon le service**, pas par incohérence.

**Analogie C/C++/Python** : le mode direct, c'est un appel de fonction
classique ou un `requests.get()` — le mode par streams Redis, c'est plus
proche d'un **pipe nommé (FIFO POSIX) avec un identifiant de corrélation** :
au lieu d'un `read()` bloquant qui récupère la prochaine ligne écrite par
n'importe qui, chaque message porte un identifiant (ici un UUID) qui permet à
celui qui a écrit la requête de retrouver *sa* réponse au milieu de toutes les
réponses qui transitent sur le canal partagé — un peu comme des messages IPC
System V taggés par `mtype` dans une même file, filtrés à la lecture.

### 2.1 Pourquoi Redis Streams et pas HTTP direct pour tout ?

Question d'examen typique (jury 42) : *pourquoi ne pas simplement faire du
HTTP direct partout ?* Éléments de réponse vérifiés dans le code :

- **Découplage temporel** : un service qui redémarre ne fait pas échouer
  immédiatement les requêtes en vol — elles restent dans le stream Redis
  jusqu'à ce qu'un worker soit de nouveau disponible pour les consommer
  (dans la limite du timeout de 30 s côté gateway, voir §3).
- **Un seul mécanisme de découverte** : les services `user`/`room`/`social`/
  `chess` n'ont pas besoin d'exposer une adresse IP/port stable que la
  gateway devrait connaître — ils consomment un stream nommé, point.
  `back/Chess-API` en a même deux instances (`chess-1`, `chess-2`) qui
  peuvent consommer le même mécanisme sans configuration réseau
  supplémentaire côté gateway pour le proxy HTTP (le websocket, lui, a besoin
  d'un mécanisme de découverte séparé — voir §4).
- **Contrepartie mesurée dans l'audit** (`audit/ARCHITECTURE.md`, §2.1) :
  « une panne peut venir du service, de Redis ou de la gateway, et les traces
  sont réparties sur trois conteneurs » — c'est le prix du découplage :
  observabilité plus difficile qu'un appel HTTP direct traçable de bout en
  bout.

## 3. Ce projet précis : le trajet exact d'une requête `/api/user/...`

Fichiers réels, dans l'ordre du trajet :

**a) Dispatch initial** — `back/Gateway-API/src/http/handlers/router.rs`,
fonction `api_handler` (route unique `"/api/*rest"`, déclarée dans
`create_gateway_router`) :

```rust
if service == "notifications" { /* proxy_http direct */ }
if service == "auth" { /* rate-limit puis proxy_http direct */ }
if service == "user" { /* rate-limit puis proxy_redis */ }
if service == "chess" || service == "room" || service == "social"
    || service == "permission" { /* rate-limit puis proxy_redis */ }
```

Donc **exactement deux services passent en HTTP direct** (`auth`,
`notifications`, via `proxy_http.rs`) et **tous les autres passent par Redis
Streams** (via `proxy_redis.rs`). C'est un `if`/`match` explicite dans le
code, pas une convention implicite — la lecture du fichier suffit à trancher
n'importe quel doute sur le chemin pris par un service donné.

**b) Publication de la requête** — `back/Gateway-API/src/http/handlers/proxy_redis.rs`,
fonction `proxy_redis` :

1. Génère un `request_id` (`Uuid::new_v4()`) — c'est le correlation-id.
2. Valide le token (`validate_token_for_service`, voir Cours 1 Chapitre 3
   pour le détail JWT — hors périmètre du Jour 1).
3. Sérialise la requête (`ServiceRequest { id, method, action, cookies, body,
   ... }`) et l'écrit dans le stream Redis `<service>:requests` via
   `XADD <service>:requests * data <payload_json>` (fonction `push_to_queue`,
   commande Redis `XADD` — l'équivalent Redis d'un `append` atomique sur un
   log distribué).
4. Enregistre un canal de réponse en mémoire locale (`state.request_router.
   register(request_id, tx)` — un `mpsc::unbounded_channel` associé au
   `request_id` dans une `DashMap` partagée, voir point d).
5. Attend sur ce canal avec un **timeout de 30 secondes**
   (`timeout(Duration::from_secs(30), rx.recv())`) — au-delà, renvoie 504
   (`GATEWAY_TIMEOUT`) au client.

**c) Traitement côté service** (hors périmètre du code lu ce Jour 1, à
creuser en Cours 1 Chapitre 6 pour User-API) : le service consomme
`<service>:requests`, traite, publie sa réponse sur le stream
**`gateway:responses`** — un seul stream de retour, partagé par tous les
services.

**d) Réception de la réponse** — `back/Gateway-API/src/http/response_listener.rs`,
`ResponseListener::run` : tourne dans une tâche tokio séparée, lancée au
démarrage du process (`main.rs` : `tokio::spawn(... listener.run() ...)`,
**avant** même que le serveur HTTP ne commence à écouter). Utilise un
`RedisStreamManager` en mode **consumer group**
(`"gateway:responses"`, groupe `"gateway-response-group"`, consommateur
`"gateway-consumer-1"`) — le groupe de consommateurs Redis garantit qu'un
message n'est traité qu'une fois même si plusieurs instances de gateway
tournaient en parallèle (ce n'est pas le cas ici, une seule instance gateway,
mais le mécanisme est prévu pour). Pour chaque message reçu : parse le JSON,
extrait le `status`, puis appelle `router.send_response(&id_field,
response)`.

**e) Le routeur de corrélation** — `back/Gateway-API/src/http/router.rs`,
struct `Router` : une simple `DashMap<String, mpsc::UnboundedSender<Response>>`
(une table de hachage concurrente — équivalent Rust d'une `std::unordered_map`
protégée par mutex fine-grained plutôt qu'un lock global). `register` insère
le canal d'attente, `send_response` le retrouve par `request_id` et y pousse
la réponse, `cleanup` le retire. C'est **tout le mécanisme de corrélation** :
pas de file d'attente FIFO globale, chaque requête a son canal dédié, retrouvé
par UUID.

**Analogie C/C++** : `Router` est l'équivalent d'une table de callbacks
indexée par un identifiant de transaction — comme un client RPC maison qui
garderait `std::map<uint64_t, std::promise<Response>>` et résoudrait la
`promise` correspondante quand la réponse arrive, au lieu de bloquer le thread
appelant sur un `recv()` direct.

## 4. Le cas particulier : découverte dynamique pour le WebSocket d'échecs

Le jeu d'échecs a **deux instances** (`chess-1`, `chess-2`, voir
`docker-compose.yml`) : une partie donnée vit en mémoire sur *une seule* des
deux. La gateway doit donc savoir, pour un `game_id` donné, quelle instance
contacter — **avant** même d'établir la connexion WebSocket (le WS n'est pas
un flux de requêtes corrélées comme au §3, c'est une connexion longue durée
qu'il faut proxy-forwarder vers la bonne instance dès l'upgrade).

`back/Gateway-API/src/chess_discovery.rs`, fonction `resolve_game_ws_url` :

1. `GET chess:game:<game_id>` → l'identifiant d'instance (`chess-1` ou
   `chess-2`), écrit par Chess-API à la création de la partie.
2. `HGET chess:instances <instance_id>` → vérifie que l'instance est toujours
   vivante dans le registre (sinon `None`, partie considérée orpheline).
3. Construit l'URL interne `ws://<instance_id>:8082` — `instance_id` sert
   directement de nom DNS Docker (voir Chapitre 1, §2.1 : résolution par nom
   de service sur le réseau `transcendence-net`).

Appelé depuis `http/handlers/router.rs`, `api_handler` : si la requête est un
upgrade WebSocket **et** `service == "chess"`, extrait `game_id` de la query
string, appelle `resolve_game_ws_url`, et **si la résolution échoue, retombe
sur `state.config.game_api_url`** (une URL statique de config, chargée depuis
la variable d'environnement `CHESS_API_URL`). Point vérifié précisément (la
réponse intuitive serait fausse, ne la devine pas à l'oral) : `CHESS_API_URL`
n'est référencée **nulle part** dans `docker-compose.yml` (confirmé par
grep), mais elle existe dans `.env` avec une **valeur vide**
(`CHESS_API_URL=`). Le service `gateway` charge `.env` en entier via
`env_file:`, donc la variable est bien présente dans l'environnement du
conteneur — avec une chaîne vide. Résultat : `config.game_api_url` vaut
`Some("")`, **pas** `None`. Le code de `router.rs` teste `Some(url) =>
proxy_websocket(...)` / `None => 503` : avec `Some("")` c'est donc la branche
`Some(url)` qui est prise, avec une URL vide passée telle quelle à
`proxy_websocket` (`http/handlers/proxy_websocket.rs`). Ce que fait
concrètement `proxy_websocket` avec une URL de service vide — le upgrade WS
côté client réussit-il quand même ? où l'échec se produit-il précisément ? —
fait l'objet de l'exercice 5 ci-dessous : relis le fichier plutôt que de
supposer. C'est la même famille de problème que la contradiction 7
documentée dans `audit/ARCHITECTURE.md` (résolution qui échoue sans remonter
de cause claire au client), même si le symptôme exact diffère de celui décrit
dans l'audit (mapping Redis expiré → mauvaise instance) : ici c'est l'absence
totale de mapping qui déclenche un fallback vers une URL vide plutôt que vers
`None`.

## 5. Contribution almeekel ici

**Aucune.** `revision/00_CONTRIBUTIONS.md` recense 0 commit almeekel sur
`back/Gateway-API`. Tout le mécanisme décrit dans ce chapitre (dispatch HTTP
vs Redis Streams, corrélation par UUID, discovery Chess) est à maîtriser en
lecture seule pour l'oral : savoir l'expliquer et le dessiner, sans revendiquer
en avoir écrit une ligne. C'est cohérent avec le périmètre réel des
contributions (`front` shop/Tailwind, `back/User-API` shop/storage,
`infra/Nginx` HTTPS) — Gateway-API est un module à comprendre pour situer où
le code d'almeekel s'insère dans le système, pas un module à défendre comme
auteur.

## 6. Exercices pratiques (après-midi Jour 1) — lecture de code Gateway-API

Consignes : pour chaque exercice, **relis le fichier réel** avant de répondre
(grep/ouvre le fichier, ne réponds pas de mémoire sur la base du résumé
ci-dessus). L'objectif est la compréhension, pas la mémorisation — pas de
contribution almeekel dans ce périmètre, donc pas d'enjeu de « défendre son
code », uniquement de savoir le lire vite sous pression orale.

1. **Trace un chemin.** Une requête `DELETE /api/social/friends/42` arrive
   sur la gateway. Sans relire le résumé ci-dessus, retrouve dans
   `http/handlers/router.rs` quelle branche du `if` la prend en charge, et
   liste dans l'ordre les fonctions appelées jusqu'à ce que la requête soit
   effectivement publiée sur un stream Redis. Note le nom exact du stream
   ciblé.

2. **Trouve le timeout.** Combien de temps la gateway attend-elle une réponse
   avant de renvoyer une erreur au client sur le chemin Redis Streams ? Le
   `match` sur `timeout(...).await` a une branche `_` qui couvre **deux** cas
   Rust différents (`rx.recv()` retourne `Ok(None)`, ou le `timeout` retourne
   `Err`) et distingue ensuite le code HTTP renvoyé selon
   `start_time.elapsed().as_secs() >= 30`. Question ouverte, pas de réponse
   évidente dans le fichier lui-même : le cas `Ok(None)` (canal fermé sans
   réponse) te semble-t-il atteignable en pratique avec le code actuel, ou
   est-ce une branche défensive qui protège juste contre l'exhaustivité du
   `match` ? Cherche dans tout `proxy_redis.rs` et `router.rs` (Router) qui
   pourrait faire disparaître l'émetteur du canal sans jamais envoyer de
   réponse.

3. **Cherche la limite du mécanisme de corrélation.** Le `Router`
   (`http/router.rs`) associe un `request_id` à un canal en mémoire locale du
   process gateway. Si la gateway elle-même redémarre pendant qu'une requête
   est en vol (déposée sur `user:requests`, réponse pas encore arrivée), que
   devient cette réponse quand elle finit par arriver sur
   `gateway:responses` ? Indice : relis `response_listener.rs`, la branche
   `Err(e) => if e.to_string().contains("not found")`.

4. **Explique un choix d'architecture à un jury.** `auth` et `notifications`
   passent en HTTP direct, tout le reste en Redis Streams. Propose une
   hypothèse sur *pourquoi ces deux services précisément* ont été exemptés du
   mécanisme Redis Streams (l'audit ne le dit pas explicitement — c'est à toi
   de formuler une hypothèse défendable, pas de la deviner au hasard). Indice
   pour construire l'hypothèse : que se passe-t-il pour `notifications` si la
   réponse doit rester ouverte plus de 30 secondes (SSE, voir Cours 1
   Chapitre 5) ? Et pour `auth`, quel est l'enjeu de latence sur un
   `login`/`register` par rapport à un appel `user`/`room`/`social` typique ?

5. **Discovery Chess — cas limite.** `resolve_game_ws_url` retourne `None`
   dans deux cas distincts (relis `chess_discovery.rs`). Lesquels,
   précisément ? Et dans le code appelant (`router.rs`, `api_handler`), que
   se passe-t-il si `resolve_game_ws_url` renvoie `None` — sachant que le
   fallback `state.config.game_api_url` vaut en réalité `Some("")` (chaîne
   vide, voir §4 : `CHESS_API_URL=` existe dans `.env` sans valeur, chargée
   telle quelle via `env_file:`) et **pas** `None` ? Relis
   `proxy_websocket.rs`, fonction `proxy_websocket` puis
   `handle_websocket_proxy`, avec `service_url = ""` : à quelle ligne exacte
   la construction de `target_url`/`ws_url` produit-elle quelque chose
   d'invalide, et à quelle étape (`ws.on_upgrade`, ou `connect_async`
   ensuite) l'échec se manifeste-t-il concrètement ? Le client reçoit-il un
   code HTTP d'erreur propre, ou autre chose ? Justifie avec les lignes du
   fichier, ne suppose pas.

*(Pas de corrigé écrit ici à dessein — fais l'exercice en relisant le vrai
code, puis vérifie tes réponses à l'oral avec quelqu'un ou en te relisant à
voix haute. Note dans `fiches_du_soir/JOUR_01.md` les questions où tu as
hésité, pour le point « à revoir demain matin en échauffement ».)*

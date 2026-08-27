# Cours 1 — Chapitre 1 : superprojet git et orchestration Docker

Jour 1 — 2026-08-23

Objectif de ce chapitre : pouvoir dessiner au tableau « qu'est-ce qui tourne,
où, et comment c'est démarré » pour l'ensemble de ft_transcendence, sans
notes.

## 1. Le concept : un superprojet à sous-modules git

### 1.1 Le problème que ça résout

Le dépôt `ft_transcendence` ne contient quasiment aucun code applicatif à sa
racine. C'est une coquille qui référence **12 autres dépôts git indépendants**
(chacun avec son propre historique, ses propres branches, ses propres
contributeurs), plus une entrée `front/front_DEV` déclarée mais inactive
(voir §1.3). Chaque service back, le front, chaque brique d'infra vit dans
son propre repo GitHub, avec son propre cycle de vie.

**Analogie C/C++** : c'est l'équivalent d'un projet qui ne vendorise pas le
code source d'une bibliothèque externe, mais se contente de noter *quel commit
précis* de cette bibliothèque il faut checkouter avant de compiler — un peu
comme un `Makefile` qui ferait `git -C libfoo checkout <hash_precis>` avant
`make`, plutôt que d'inclure `libfoo/` directement dans l'archive. Le
superprojet ne stocke jamais le code des sous-modules, seulement leur nom, leur
URL, et le hash de commit exact à checkouter.

### 1.2 Le mécanisme réel : `.gitmodules` + pointeurs

Fichier `.gitmodules` (racine) — extrait représentatif :

```ini
[submodule "back/Gateway-API"]
    path = back/Gateway-API
    url = git@github.com:ft-transcendence-tkt-on-vera/Gateway-Api.git
[submodule "back/Auth-API"]
    path = back/Auth-API
    url = git@github.com:ft-transcendence-tkt-on-vera/Auth-Api.git
    branch = fix/2fa
```

Chaque entrée = un nom de dossier + une URL de dépôt (+ éventuellement une
branche à suivre, comme `back/Auth-API` qui suit `fix/2fa` plutôt que `main`).
Le superprojet, lui, ne commite qu'un **pointeur** : un objet git spécial de
type *gitlink* qui vaut « ce dossier = le commit `<hash>` du dépôt distant
déclaré dans `.gitmodules` ». Aucune ligne de code du sous-module ne vit dans
l'historique du superprojet.

**Le piège, vérifié dans `AGENTS.md` et `AUDIT_PROMPT.md`** :

> `git pull` ne met à jour que les pointeurs : toujours suivre de
> `git submodule update --init --recursive`, sinon on lit du code périmé.

Autrement dit : `git pull` sur le superprojet peut faire avancer le pointeur
`back/Chess-API` vers un nouveau commit, mais le dossier `back/Chess-API/` sur
le disque reste checkouté sur l'ancien commit tant que
`git submodule update --init --recursive` n'a pas été lancé. Lire le code sans
cette étape, c'est lire une version qui n'est plus celle référencée par le
superprojet.

**Analogie C/C++** : c'est exactement le décalage entre *mettre à jour un
numéro de version dans un fichier de manifeste* (`package.json`,
`Cargo.lock`, un `#define VERSION`) et *effectivement recompiler/relinker*
contre cette version. Le pointeur qui change ne change rien tant qu'on n'a pas
« rebuild » — ici, `submodule update` est le `make` qui synchronise le disque
avec le manifeste.

### 1.3 Vérification sur ce dépôt précis (2026-08-23)

`git submodule status` (lecture seule, aucune modification) montre **12
sous-modules initialisés** sur le disque :

```
back/Auth-API, back/Chess-API, back/Gateway-API, back/Notification-API,
back/Room-API, back/Social-API, back/User-API, back/api-core,
documentation, front, infra/Nginx, infra/metrics
```

`.gitmodules` en déclare un 13e : `front/front_DEV` →
`git@github.com:ft-transcendence-tkt-on-vera/Front_DEV.git`. **Ce sous-module
n'existe pas sur le disque** (`front/front_DEV` n'existe pas). C'est
probablement un résidu de l'historique du projet : `front/front_DEV` a
vraisemblablement été l'ancien nom du sous-module frontend avant d'être
remplacé par `front` (déclaré séparément, actif). Point à signaler à l'oral si
la question « pourquoi 13 sous-modules déclarés mais 12 utilisés » tombe —
c'est exactement le genre d'écart entre la description et le code réel que le
jury 42 aime faire justifier.

**Trois sous-modules ont des modifications non commitées** au moment de cet
audit (`git status --short` dans chacun) : `back/User-API`
(`src/db/shop.rs`, `src/http/handlers/shop.rs`, `src/services/storage.rs`),
`front` (fichiers `src/features/shop/**`), et `infra/metrics` (fichiers de
sauvegarde Elasticsearch, hors périmètre applicatif). Les deux premiers
correspondent exactement aux chantiers Shop d'almeekel qui seront reconstruits
en Cours 2 — c'est du travail en cours, pas une anomalie : **aucune commande
d'écriture git n'a été lancée** pendant cette session pour ne pas perturber ce
travail.

## 2. Le concept : orchestration multi-services avec Docker Compose

### 2.1 Le problème que ça résout

Le projet fait tourner ~17 processus long-running en parallèle (base
Postgres, Redis, 7 services Rust, 2 instances Chess-API, MinIO, nginx, le
front en dev, Kibana/Grafana/Elasticsearch, un tunnel Cloudflare…), chacun
dans son propre conteneur, qui doivent se trouver les uns les autres par nom
et démarrer dans le bon ordre.

**Analogie C/C++/Python** : pense à un `Makefile` qui ne compile pas des
fichiers objets mais démarre des processus serveur, avec une règle de
dépendance explicite (`depends_on`) qui joue le rôle d'un `target: prerequis`
— sauf que la « compilation » ici, ce sont des `docker build` par service, et
l'exécution reste attachée (les processus tournent en continu, ils ne
terminent pas comme un `make` classique). Le réseau `transcendence-net`
déclaré dans `docker-compose.yml` (`networks: transcendence-net: driver:
bridge`) est un réseau virtuel privé : chaque conteneur peut joindre les
autres par leur **nom de service** comme s'il s'agissait d'une entrée DNS —
équivalent applicatif de résoudre `gethostbyname("redis")` et d'obtenir
l'IP du conteneur Redis, sans jamais coder d'IP en dur.

### 2.2 Ce projet précis : `docker-compose.yml` et `Makefile`

Extrait représentatif (`docker-compose.yml`, service `gateway`) :

```yaml
gateway:
  build:
    context: back
    dockerfile: Gateway-API/Dockerfile
  container_name: gateway_service
  ports:
    - "127.0.0.1:${GATEWAY_API_PORT}:${GATEWAY_API_PORT}"
  networks:
    - transcendence-net
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:${GATEWAY_API_PORT}/health"]
  depends_on:
    redis:
      condition: service_healthy
    postgres:
      condition: service_healthy
```

Points à retenir, vérifiés dans le fichier réel :

- **`depends_on` avec `condition: service_healthy`** ne veut pas dire
  « démarre après », mais « démarre seulement quand le healthcheck de l'autre
  service répond OK ». `nginx` dépend ainsi de `gateway`, `frontend`,
  `chess-1`, `chess-2`, `room`, `kibana`, `grafana` — nginx est délibérément le
  dernier à être prêt, puisqu'il route vers tout le reste.
- **Contradiction à signaler explicitement (audit vs code actuel).**
  `AUDIT_PROMPT.md` liste comme bug « services internes exposés publiquement »
  avec des bindings `0.0.0.0` explicites (`postgres_db 0.0.0.0:5432->5432`,
  `redis_db 0.0.0.0:6380->6379`, `minio_storage 0.0.0.0:9000-9001`, `grafana
  0.0.0.0:3000`, `kibana 0.0.0.0:5601`, `adminer_ui 0.0.0.0:8081`), et
  `audit/ADR/ADR-004-exposition-reseau-tls.md` reprend ce constat (« une
  dizaine de ports publiés en `0.0.0.0` »). **Relecture directe de
  `docker-compose.yml` (+ `infra/metrics/elk/docker-compose.yml`,
  `infra/metrics/monitoring/docker-compose.yml`) le 2026-08-23** : *tous* les
  ports actuellement publiés sont préfixés `127.0.0.1:` (postgres, adminer,
  gateway, auth, user, chess-1/2, room, social, notification, frontend,
  nginx, scalar, kibana, grafana) — et **Redis comme MinIO n'ont même plus de
  section `ports:` du tout**, donc ne sont plus publiés sur l'hôte. Aucun
  `0.0.0.0` trouvé dans ces trois fichiers. Deux lectures possibles, à
  vérifier plutôt qu'à trancher ici : soit le correctif recommandé par
  l'option 2 de l'ADR-004 (retirer/restreindre les `ports:`) a déjà été
  appliqué sur ce point précis sans que `BUGS.md`/`ADR-004` aient reçu la
  mention « MISE À JOUR » qu'ont d'autres bugs résolus dans
  `audit/ARCHITECTURE.md` ; soit l'audit initial décrivait un état antérieur
  (ex. avant un rebase/nettoyage du fichier). À trancher au Jour 3 (nginx /
  observabilité / sécurité) en comparant avec `git log -p -- docker-compose.yml`
  plutôt qu'en le supposant ici — mais pour l'oral dès maintenant : ne pas
  réciter « les ports sont exposés en 0.0.0.0 » sans avoir vérifié l'état du
  fichier au moment de la question, c'est précisément le genre d'écart que
  le jury peut faire constater en direct.
- **`frontend` monte deux volumes** : `./front:/app` (bind mount du code
  source, pour le hot-reload Vite) et `/app/node_modules` (volume anonyme).
  C'est exactement le piège documenté dans `AGENTS.md` : le volume anonyme
  masque le `node_modules` du dossier monté, donc `npm install` fait sur
  l'hôte (ou via un `git pull` qui ajoute une dépendance) n'a aucun effet
  dans le conteneur — il faut `docker exec frontend_dev npm install`.
- **Le service `mc-public`** est une boucle shell (`while true; do … sleep
  60; done`) qui rend publics tous les buckets MinIO toutes les 60 secondes.
  Ce n'est pas un service Rust : c'est un choix d'ops pragmatique pour ne pas
  gérer de policy IAM MinIO fine.

`Makefile` — cibles à connaître par cœur pour l'oral :

- `make up` = `check-env` + `check-docker-root` + `kibana-password` puis
  `docker compose up -d --build`, puis `make publish-assets` puis
  `make kibana-import`. **`make publish-assets` n'est pas automatique en
  dehors de `make up`/`make dev`/`make prod`** : un fichier ajouté dans
  `assets/` reste invisible du navigateur tant que cette cible n'a pas
  tourné (confirmé par `scripts/publish-assets.sh` et par `AGENTS.md`).
- `make prod` = `front-build` (`npm run build` dans `front/`, sort dans
  `front/dist`) + `docs-build` (build Scalar dans `documentation/`) + `up`.
  C'est la cible par défaut (`all: prod`).
- `make kibana-password` régénère `kibana.env` à **chaque** `make up`/`dev`/
  `build` (dépendance de ces trois cibles) — donc le mot de passe Kibana
  change à chaque redémarrage complet de la stack, par design.
- `make ultraclean` est la seule cible destructive (supprime volumes +
  images + réseaux orphelins) — à ne jamais lancer par réflexe.

### 2.3 Piège d'exploitation à ne pas confondre avec un bug de code

**nginx résout les noms de services une seule fois, au démarrage.** Si un
service back redémarre seul (ex. après un crash ou un rebuild ciblé), nginx
garde l'ancienne IP du conteneur et **toutes** les routes qui passent par lui
renvoient 502 — alors que le service lui-même est parfaitement sain. Remède :
`docker compose restart nginx`. C'est un comportement DNS de nginx (résolution
au démarrage du process, pas de re-résolution périodique par défaut), pas un
bug applicatif — vérifié dans `AGENTS.md` et `AUDIT_PROMPT.md`, à ne pas
chercher à « corriger » dans le code back en soutenance.

## 3. Contribution almeekel ici

**Aucune.** D'après `revision/00_CONTRIBUTIONS.md`, les commits d'almeekel se
concentrent dans `front` (80 commits — feature Shop + migration Tailwind),
`back/User-API` (11 commits — endpoints shop, stockage MinIO) et
`infra/Nginx` (4 commits — bascule HTTPS). Aucun commit sur `docker-compose.yml`,
le `Makefile` racine, ou `.gitmodules` n'est recensé dans l'audit git. Ce
chapitre est donc à maîtriser **en lecture, pas en tant qu'auteur** : sur ce
périmètre précis, la bonne réponse en soutenance est « je comprends comment
c'est orchestré, mais je n'ai pas écrit cette couche » — c'est une information
utile pour le jury, pas un aveu de faiblesse.

À l'inverse, les trois sous-modules où almeekel a contribué (`front`,
`back/User-API`, `infra/Nginx`) sont orchestrés par les fichiers vus dans ce
chapitre : le service `frontend` (§2.2) construit exactement le code que
Cours 2 / Chapitre 1 va faire reconstruire, et le service `user` (§2.2,
`User-API/Dockerfile`) fait tourner le code shop/storage du Chapitre 2. Pour
`infra/Nginx` (Chapitre 3, HTTPS) : `audit/ADR-004` note que « nginx n'écoute
qu'en HTTP clair sur :8000 » et que le TLS réel est terminé hors dépôt (tunnel
Cloudflare) — à réconcilier avec les 3 commits almeekel « Start https » /
« Serve the application over HTTPS » lorsque le Chapitre 3 lira réellement
`nginx.conf` et le `Dockerfile` (ne pas trancher ici sans avoir lu ces
fichiers). Le lien entre « ce que j'ai écrit » et « comment ça démarre en
pratique » est le fil à tenir pour l'oral.

## 4. Questions de contrôle rapide (auto-évaluation, pas l'examen du soir)

1. Pourquoi `git pull` seul ne suffit-il jamais à mettre à jour le code d'un
   sous-module, même si le pointeur a changé ?
2. Que signifie concrètement `condition: service_healthy` dans un
   `depends_on`, et pourquoi `nginx` en dépend-il pour sept services ?
3. Un fichier ajouté dans `assets/` n'apparaît pas côté navigateur après un
   `git pull` + redémarrage des conteneurs. Deux causes possibles vues dans ce
   chapitre — lesquelles, et comment les distinguer ?
4. `front/front_DEV` est déclaré dans `.gitmodules` mais absent du disque.
   Est-ce un bug à corriger avant la soutenance ? Justifie.

*(Pas de corrigé écrit ici à dessein — ce chapitre n'est pas un chantier
almeekel : reformule les réponses toi-même à voix haute avant de vérifier
contre les sections ci-dessus, c'est l'exercice.)*

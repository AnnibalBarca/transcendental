# ADR-004 — Exposition réseau, TLS, en-têtes et CSRF

> **Mise à jour (re-vérification du 20/08)** : le point « logout en GET »
> ci-dessous est résolu — la route est désormais `post(logout_handler)`
> (`back/Auth-API/src/http/router.rs:76`) et tous les appelants front sont
> alignés en POST (voir bug #29 dans `BUGS.md`). Le vecteur CSRF trivial via
> `<img src="/api/auth/logout">` n'existe donc plus pour cette route
> spécifique ; la recommandation `SameSite=Strict` reste pertinente pour la
> surface CSRF restante (autres routes mutantes en POST, qui restent
> vulnérables à un CSRF « classique » form-based sans protection dédiée).
> Le reste du contexte (ports exposés, absence de TLS/en-têtes dans le
> dépôt, WS chess anonyme via les ports directs) reste vrai en l'état — le
> WS chess n'est plus « anonyme » via la gateway/le chemin applicatif normal
> (voir bug #35 mis à jour), mais les ports 8082/8083 publiés directement
> continuent de contourner la gateway pour le transport (pas pour l'auth,
> désormais vérifiée côté Chess-API lui-même).

## Contexte

Constats (tous vérifiés dans le dépôt) :

- une dizaine de ports publiés en `0.0.0.0` : postgres 5432, redis 6380,
  adminer 8081, minio 9000/9001, kibana 5601, grafana 3000, WS chess
  8082/8083, metrics 9101-9106, scalar 5050 (`docker-compose.yml` + includes
  metrics) ;
- nginx n'écoute qu'en HTTP clair sur :8000 (`infra/Nginx/nginx.conf:44`) et
  n'émet aucun en-tête de sécurité (aucun `add_header` dans tout le fichier) ;
- le TLS réel est terminé hors dépôt (domaine `chicken-exe.com`, un tunnel
  Cloudflare a existé — token commité) ; le signalement « TLS 1.0/1.1
  acceptés » concerne cette terminaison externe ;
- le logout est un GET avec effets de bord (`Auth-API/src/http/router.rs:76`,
  `LogoutButton.tsx:19-23`) : CSRF trivial (`<img src="/api/auth/logout">`) ;
  l'autre appelant front en POST prend d'ailleurs un 405 ;
- le WS d'échecs est joignable anonymement via la gateway, et directement via
  les ports 8082/8083 publiés.

## Options envisagées

1. **Pare-feu hôte uniquement** (bloquer les ports en entrée côté machine) :
   - Avantages : zéro changement de dépôt.
   - Inconvénients : non versionné, perdu à chaque réinstallation, ne protège
     pas les déploiements d'autres membres.
2. **Moindre exposition déclarée dans compose + edge unique** (recommandé) :
   - supprimer les `ports:` de tout service consommé uniquement sur le réseau
     `transcendence-net` (postgres, redis, minio console 9001, kibana,
     grafana, chess WS, metrics) ; garder uniquement nginx (8000/443) et, si
     besoin d'inspection locale, préfixer `127.0.0.1:` ;
   - exposer adminer/grafana/kibana, si réellement nécessaires à distance,
     derrière la gateway avec auth (et non en direct) ;
   - terminer le TLS au plus près : soit nginx lui-même (certbot, `listen
     443 ssl`, `ssl_protocols TLSv1.2 TLSv1.3`), soit assumer l'edge externe
     mais alors fixer par contrat « Minimum TLS 1.2 » et le vérifier par un
     test de déploiement (`openssl s_client -tls1` doit échouer) ;
   - ajouter sur nginx : `Strict-Transport-Security` (si TLS),
     `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY` (hors pages
     l'embarquant), `Content-Security-Policy` (commencer permissif :
     `default-src 'self'`, autoriser `wss:` connect, img MinIO, styles
     inline Vite en dev) ;
   - logout en POST (+`SameSite=Strict` sur les cookies si compatible) et
     aligner les deux appelants front.
3. **Réseau fermé + VPN/SSH tunnel pour tout l'outillage** : la variante la
   plus stricte de 2, moins pratique pour une équipe d'étudiants.

## Décision recommandée

Option **2**. Pour la couche TLS, **porter la terminaison dans nginx du
dépôt** (option par défaut), quitte à garder l'edge Cloudflare en proxy : le
réglage min-TLS devient alors auditable dans le repo (`ssl_protocols`), ce qui
manque aujourd'hui. Les en-têtes et le POST-logout sont des S immédiats,
indépendants du reste.

## Conséquences

- `docker compose ps` ne montre plus que l'entrée nginx : les tests locaux
  d'adminer/grafana passent par tunnel SSH ou `127.0.0.1` — à documenter dans
  le README sous peine de tickets « plus accès à grafana ».
- La gateway devient le seul point d'auth pour l'outillage admin : prévoir
  les routes manquantes (aujourd'hui adminer/kibana ne sont pas derrière).
- CSP : prévoir une passe de correction du front (Vite injecte des styles
  inline ; `unsafe-inline` en transition ou build avec nonce).
- Le retrait des ports 8082/8083 rend `CHESS_PUBLIC_WS_URL` définitivement
  mort — à supprimer pour éviter la confusion (le front passe déjà par
  `/api/chess/chess`).
- Effort estimé : M (compose S, nginx TLS S/M, headers S, logout S, CSP M).

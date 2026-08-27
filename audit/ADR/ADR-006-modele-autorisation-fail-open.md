# ADR-006 — Le modèle d'autorisation est « fail-open » par construction (gateway + handlers)

*Ajoutée le 20/08, lors de la repasse de vérification de l'audit initial —
ce n'était pas un problème structurel identifié dans la première version.*

## Contexte

Deux découvertes indépendantes de cette repasse (bug #30 « rate-limit auth
partiel » et bug #36 « shop upload sans authentification ») partagent en
réalité une seule cause structurelle : le modèle d'autorisation du projet
est conçu pour **laisser passer par défaut**, à deux niveaux différents.

1. **Niveau gateway** — `back/Gateway-API/src/http/rate_limit.rs::enforce_access` :
   ```rust
   let Some(user_id) = extract_user_sub(redis_pool, access_token.as_deref()).await
   else {
       return Ok(());   // ← laisse passer, sans rate-limit ni contrôle de permission
   };
   ```
   Toute requête sans cookie `access_token` valide saute *entièrement* le
   rate-limit **et** la vérification de permission
   (`enforce_permission`, qui n'est même pas appelée). Ce n'est pas un
   oubli isolé sur une route : c'est la façon dont `enforce_access` est
   écrite pour **tous** les services qui l'appellent (`auth`, `user`,
   `chess`, `room`, `social`, `permission`).

2. **Niveau permission applicative** — même quand un `user_id` est présent,
   `enforce_permission` (`rate_limit.rs:55-99`) ne bloque que si la route
   est explicitement liée à une permission dans la table
   `permission_routes` (peuplée uniquement pour `/api/user/admin/%` par la
   migration `044_link_default_permission_routes`). Une route qui n'a
   **jamais** été ajoutée à cette table — par oubli, comme `shop/items`,
   ou parce que personne n'a pensé à la protéger — est accessible à
   **tout utilisateur authentifié, quel que soit son rôle**. Il n'existe
   aucun mécanisme qui liste les routes *non couvertes* pour alerter une
   équipe qui ajoute un nouvel endpoint sensible.

3. **Niveau handler** — certains handlers de `back/User-API/src/http/handlers/shop.rs`
   font *en plus* leur propre vérification (`authed_user_id`, appelée par
   `handle_purchase_collection`, optionnelle dans `handle_get_shop`), et
   d'autres n'en font **aucune** (`handle_upload_item`). Rien dans la
   structure du routeur (`register_public(...)` — le nom lui-même est un
   signal : toutes les routes User-API sont déclarées « publiques » au sens
   du framework, l'auth étant sensée être un choix du handler) ne force à
   penser à cette vérification pour un nouveau endpoint.

Ces trois niveaux se combinent pour qu'une route mutante (écriture en base,
upload de fichier) puisse rester totalement ouverte sans qu'aucune des
couches ne le signale — ni au moment du développement (pas de type/trait
qui obligerait à déclarer un handler comme public ou protégé), ni à
l'exécution (pas de log d'alerte quand une route sans permission liée reçoit
une requête mutante).

## Options envisagées

1. **Statu quo + audit manuel ponctuel** : corriger `shop/items` (bug #36)
   et le trou `register`/anonyme (bug #30) au cas par cas.
   - Avantages : rapide, zéro risque de régression sur le reste.
   - Inconvénients : ne corrige pas la cause ; le prochain endpoint sensible
     ajouté par l'équipe aura le même défaut par défaut.
2. **Deny-by-default au niveau gateway pour les méthodes mutantes**
   (recommandé) :
   - toute route `POST`/`PATCH`/`PUT`/`DELETE` proxyée par la gateway exige
     un `user_id` valide sauf si elle figure explicitement dans une
     allowlist de routes publiques (register, login, refresh, etc.) ;
   - `enforce_access` retourne 401 (pas `Ok(())`) quand `user_id` est
     absent et que la route n'est pas dans cette allowlist ;
   - `enforce_permission` gagne un mode strict optionnel par route
     (« doit être liée à au moins une permission ») activable pour les
     routes marquées sensibles (admin, upload, mutation de catalogue),
     avec un test/lint CI qui échoue si une route sensible n'a pas de
     permission liée.
3. **Wrapper d'authentification obligatoire au niveau handler** (en
   complément, pas en remplacement de 2) : un helper partagé dans
   `api-core` (`require_auth(request) -> Result<Claims, ErrorResponse>`)
   que chaque handler mutant appelle en première ligne, avec une revue de
   code qui grep les handlers `POST`/`PATCH`/`DELETE` sans cet appel.

## Décision recommandée

Option **2**, complétée par l'option **3** pour la défense en profondeur :
la gateway ne doit fail-open que pour une liste explicite et courte de
routes publiques (auth de base), jamais par défaut ; toute route mutante
côté `user`/`room`/`social`/`chess` doit soit être dans cette allowlist,
soit exiger un `user_id` valide. La vérification de permission par route
reste utile pour le contrôle fin des rôles (admin vs joueur), mais ne doit
plus être la seule ligne de défense contre l'accès anonyme.

## Conséquences

- Corrige d'un coup la classe de bugs illustrée par #30 et #36, plutôt que
  deux correctifs isolés — et prémunit contre la récidive sur un futur
  endpoint.
- Nécessite de dresser la liste exhaustive des routes réellement publiques
  (register, login, google/42 oauth, refresh, forgot/reset password,
  validate_email, health) — travail de recensement à faire une fois, déjà
  presque disponible via la table `api_routes` (migration `035_seed_api_routes`).
- Risque de régression si l'allowlist est mal recensée (une route
  légitimement publique basculerait en 401) — à dérisquer par un passage en
  mode « log seulement » avant activation stricte.
- Effort estimé : S/M (allowlist + inversion du défaut dans
  `enforce_access`) + M (mode strict par route + lint CI, optionnel mais
  recommandé pour la durabilité du correctif).

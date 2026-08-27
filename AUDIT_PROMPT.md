# Mission d'audit — ft_transcendence

Tu interviens en **mode plan**. Tu ne modifies aucun fichier, tu n'ouvres aucune
PR, tu n'exécutes aucune commande qui écrit. Ton livrable est un ensemble de
documents d'analyse.

L'objectif : comprendre l'architecture réelle du projet, puis rattacher chaque
bug signalé aux fichiers exacts qui en sont responsables. Un bug mal attribué
coûte plus cher qu'un bug non traité, donc **quand tu n'es pas sûr, dis-le**.

---

## 1. L'architecture, pour ne pas la redécouvrir

Le dépôt est un superprojet git avec **13 sous-modules**. Un `git pull` met à
jour les pointeurs mais ne récupère pas leur contenu : sans
`git submodule update --init --recursive`, tu lis du code périmé sans le savoir.

```
back/Gateway-API      routeur d'entrée, proxy websocket, RBAC, rate-limit
back/Auth-API         inscription, connexion, OAuth Google et 42, JWT
back/User-API         profil, inventaire, boutique, packs, admin, migrations SQL
back/Chess-API        moteur d'échecs, cartes, deux instances chess-1 et chess-2
back/Room-API         salles privées et publiques
back/Social-API       amis, messages
back/Notification-API SSE
back/api-core         bibliothèque partagée : routeur, cache, JWT, métriques
front                 React 19, TypeScript, Vite, Tailwind, react-i18next
infra/Nginx           reverse proxy
infra/metrics         ELK et Prometheus/Grafana
documentation         OpenAPI
```

Les services back communiquent par **streams Redis**, pas par HTTP direct. La
gateway publie une requête sur `<service>:requests` et attend la réponse. Cela
signifie qu'une panne peut venir du service, de Redis, ou de la gateway, et que
les traces sont réparties sur trois conteneurs.

**Le websocket du jeu** passe par la gateway, qui résout dans Redis quelle
instance de Chess-API héberge la partie demandée
(`Gateway-API/src/chess_discovery.rs`). Le front construit son URL depuis
l'adresse de la page : `${protocol}//${host}/api/chess/chess?game_id=...` dans
`front/src/features/games/ChessGame/hooks/useChessGame.ts`. Les variables
`CHESS_PUBLIC_WS_URL` ne servent qu'au serveur.

**Les images** sont servies par MinIO derrière nginx sur `/img/`. Le catalogue
en base ne stocke qu'un `asset_key` du type `mask/1.png` ; l'URL complète est
reconstruite à la volée par `User-API/src/services/storage.rs`.

---

## 2. Pièges d'environnement — vérifiés, à ne pas confondre avec des bugs

Plusieurs symptômes viennent de l'exploitation, pas du code. Si tu tombes
dessus, ne cherche pas de coupable dans les sources.

**nginx résout les noms de services une seule fois au démarrage.** Après le
redémarrage d'un service back, il garde l'ancienne IP et **toutes** les routes
renvoient 502. Remède : `docker compose restart nginx`.

**Le conteneur front monte `/app/node_modules` en volume anonyme**, qui masque
le dossier du dépôt. Une dépendance ajoutée dans `package.json` n'est jamais
installée par un simple pull, et Vite échoue en silence sur le composant qui
l'importe. Remède : `docker exec frontend_dev npm install`.

**Les assets ne partent dans MinIO que sur appel explicite** de
`make publish-assets`. Un fichier ajouté dans `assets/` reste invisible du
navigateur tant que ce n'est pas fait.

**Toute route qui n'est pas `/img/…` renvoie `index.html` en 200** avec le type
`text/html`, parce que nginx envoie le reste au front et que Vite répond par sa
page unique. Une image dont l'URL est mal construite ne produit donc **pas** de
404 : elle produit un 200 contenant du HTML, et le `onError` de la balise ne se
déclenche jamais. C'était la cause d'un bug d'affichage déjà corrigé ; garde ce
mécanisme en tête, il peut en masquer d'autres.

**Une migration SQL déjà appliquée ne se rejoue pas.** Modifier une migration
existante n'a aucun effet sur une base en place, et les schémas divergent alors
entre installations neuves et anciennes.

**Le healthcheck de plusieurs services est cassé** : la sonde appelle `curl`,
absent des images Rust. Le conteneur est marqué `unhealthy` alors que le service
fonctionne, et tout ce qui en dépend refuse de démarrer.

---

## 3. Ce qu'on attend de toi

Produis un dossier `audit/` contenant :

**`audit/ARCHITECTURE.md`** — la cartographie réelle : qui parle à qui, par quel
canal, où vit chaque responsabilité. Signale les endroits où le code contredit
cette description.

**`audit/BUGS.md`** — pour **chaque** bug de la liste ci-dessous :

- les fichiers et lignes concernés, chemins complets
- la cause probable, formulée comme une hypothèse falsifiable
- ton niveau de confiance : `confirmé` si tu as lu le code fautif, `probable` si
  tu déduis, `à investiguer` si tu ne peux pas trancher
- ce qu'il faudrait observer pour confirmer, si tu ne peux pas conclure
- une estimation de l'effort de correction

**`audit/ADR/`** — une décision d'architecture par problème structurel, au format
habituel : contexte, options envisagées, décision recommandée, conséquences.
N'en écris que pour les problèmes qui appellent un choix, pas pour les
correctifs évidents.

**`audit/INCONNUES.md`** — ce que tu n'as pas pu comprendre, et pourquoi. Cette
section a autant de valeur que les autres : elle dit où concentrer l'effort
humain.

---

## 4. La liste des bugs

### Moteur de jeu et cartes

- La carte **Traître** ne fonctionne pas.
- La carte **Pyromane** ne fonctionne pas.
- La carte **Bastion** ne fonctionne pas.
- Le **Maçon en furie** est injouable et ne fait rien.
- Le **Vétéran fou** est bugué.
- Le **Vétéran tour** : la description parle des diagonales alors que l'effet
  concerne les axes verticaux et horizontaux — ou l'inverse. À trancher entre le
  texte et le code.
- Jouer **Vétéran tour** passe parfois simplement le tour.
- **Vétéran cavalier** et **Vétéran fou** sont parfois impossibles à cliquer.
- Le **Sniper** se joue même en l'absence de fou, et consomme le tour.
- Certaines cartes qui demandent de sélectionner une pièce, **Percée** en
  particulier, sautent purement et simplement le tour. Comportement inconstant,
  difficile à reproduire. **C'est probablement le problème le plus grave du
  jeu.** Piste : le serveur websocket.
- Les cartes d'échange de pièces — Maçon, Cheval fou et apparentées — souffrent
  de deux défauts. Elles restent jouables alors que les pièces concernées ne
  sont plus sur l'échiquier. Et lorsque plusieurs paires sont éligibles, le
  choix n'est pas laissé au joueur. Correction souhaitée : conditionner leur
  usage à la présence des **deux types** de pièce, puis laisser le joueur
  sélectionner une pièce de chaque type.
- Jouer une carte alors qu'on est **en échec** passe la main à l'adversaire en
  nous laissant en échec, ce qui est contraire aux règles.
- Jouer une carte peut **nous mettre nous-même en échec**. L'adversaire peut
  alors capturer le roi. Comportement peut-être assumable, mais il faut en
  mesurer les conséquences.

### Interface et état du jeu

- Si une pièce jouable est sélectionnée **avant** de jouer une carte de
  déplacement, rien ne se passe et l'on reste verrouillé sur la pièce et ses
  destinations pendant le tour adverse. Parfois le mouvement a lieu mais la vue
  reste figée sur la sélection. Problème d'actualisation d'état.
- Le **timer adverse** ne décroît pas chez l'hôte de la partie dans certains
  cas, et n'accélère pas quand **Top chrono** a été joué.
- Le surlignage et le timer du joueur actif ne sont pas répercutés chez
  l'adversaire : la page ne se rafraîchit pas.
- À l'inverse, la page du jeu **se rafraîchit sans raison**. À diagnostiquer
  côté websocket.
- La **vue spectateur** est buguée. À vérifier notamment : le spectateur
  voit-il à travers les effets, le brouillard par exemple ?
- Le **défilement entre cartes** dans la zone de sélection est impossible sur
  ordinateur.
- Les descriptions sous les cartes n'ont **pas de zone de défilement**.
  Décision retenue : supprimer le sous-texte.
- Les messages entre amis et les messages privés n'ont **ni zone de défilement
  ni limite de taille**, ce qui étire la page.

### Interface, réglages et traduction

- Dans les réglages, le bouton de **déconnexion est invisible** tant que la
  souris ne le survole pas.
- Les **conditions d'utilisation**, sur la page de connexion comme dans les
  réglages, ne suivent pas la langue choisie par le drapeau.
- Dans les réglages, les champs de liens sociaux attendent un **pseudo**, pas
  une URL, car le lien est préformé. L'étiquette induit en erreur.

### Erreurs serveur et protocole

- Une erreur du back, **non traduite et au format brut**, s'affiche quand on
  demande en ami un utilisateur inexistant.
- Un **409 sans corps** apparaît souvent en tentant de rejoindre une salle
  depuis le lobby, prétendant que l'on est déjà dans une salle alors que ce
  n'est pas le cas, y compris côté serveur. Piste : connexion et socket.
- **Requêtes de rafraîchissement de jeton mal formées.**
- Des **connexions websocket expirent** depuis la console parce que le serveur
  ne les gère pas.
- La **déconnexion passe par une requête GET**. À remplacer par POST, DELETE ou
  PUT, plus sûrs vis-à-vis des attaques CSRF.
- En créant un compte par e-mail, on peut **saturer le serveur** en faisant des
  allers-retours répétés avec le bouton de retour au menu de connexion.

### Sécurité et exposition

- **Services internes exposés publiquement.** À placer derrière un accès
  restreint.
  ```
  postgres_db    0.0.0.0:5432->5432
  redis_db       0.0.0.0:6380->6379
  minio_storage  0.0.0.0:9000-9001
  grafana        0.0.0.0:3000
  kibana         0.0.0.0:5601
  adminer_ui     0.0.0.0:8081
  ```
- **En-têtes de sécurité absents** — `curl -I` n'en renvoie aucun :
  `Strict-Transport-Security`, `X-Content-Type-Options: nosniff`,
  `X-Frame-Options`, `Content-Security-Policy`.
- **TLS 1.0 et 1.1 sont encore acceptés**, alors qu'ils sont dépréciés depuis
  2020.
- **Deux secrets ont été commités puis supprimés**, mais restent lisibles dans
  l'historique git : un `GOOGLE_CLIENT_SECRET` et un `FT_CLIENT_SECRET`. Le
  retrait d'un fichier n'efface pas les commits antérieurs.
- **Audit à mener sur la connexion websocket** : peut-elle être interceptée ou
  rejouée ?

---

## 5. Méthode attendue

Commence par lire le code avant de formuler la moindre hypothèse. Sur ce
projet, plusieurs symptômes évidents avaient une cause inattendue.

Cherche les **causes communes**. Une bonne partie des bugs de jeu — tours
sautés, vue figée, timer bloqué, rafraîchissements intempestifs — sentent la
même racine : la synchronisation d'état par websocket. S'ils partagent une
cause, dis-le et traite-la comme un seul problème.

Distingue **ce que le code fait** de **ce que la description annonce**. Pour
Vétéran tour, l'un des deux est faux, et savoir lequel change le correctif.

N'invente pas de certitude. Sur des bugs décrits comme inconstants, l'honnêteté
sur ton incertitude vaut mieux qu'une hypothèse assurée qui enverrait quelqu'un
sur une fausse piste pendant deux heures.

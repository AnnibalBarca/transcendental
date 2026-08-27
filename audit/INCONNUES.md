# Inconnues — ce que cet audit n'a pas pu trancher

Ces points ont autant de valeur que les réponses : ils indiquent où l'effort
humain (reproduction, accès aux environnements) doit se concentrer.

> **Note de repasse (20/08)** : cette version a été revérifiée contre le
> code fraîchement synchronisé des sous-modules (`back/Room-API`,
> `back/User-API`, `front`, et accessoirement `back/Chess-API`,
> `back/Gateway-API`, `back/Auth-API` qui avaient eux aussi des commits en
> attente). Plusieurs points ci-dessous ont pu être tranchés et sont
> marqués **[RÉSOLU]** ; de nouveaux points ouverts par cette relecture ont
> été ajoutés en fin de fichier.

## 1. TLS 1.0/1.1 — où est la configuration fautive ?

Le dépôt ne contient **aucune** configuration TLS (nginx écoute en clair sur
:8000, `infra/Nginx/nginx.conf:44`). Le signalement ne peut donc concerner
que la terminaison externe du domaine `chicken-exe.com` (`.env`) : hébergeur,
reverse proxy machine, ou Cloudflare (un tunnel y est référencé, commenté,
dans docker-compose.yml:414-421). **À vérifier avec un accès au compte
d'hébergement/Cloudflare** : réglage « Minimum TLS Version », et
`openssl s_client -connect <domaine>:443 -tls1` pour une preuve. Tant que ce
n'est pas fait, corriger dans le dépôt n'aura aucun effet sur ce point.

## 2. « 409 sans corps »

Le code produit toujours un JSON
(`{"status":409,"error":"Already in a room or matchmaking"}`,
`Room-API/src/http/response.rs:4-9`, recopié par la gateway
`response_listener.rs:40-56` ; nginx n'intercepte pas les erreurs). Je n'ai
trouvé **aucun** chemin renvoyant un 409 nu. Hypothèses restantes : perception
devtools (requête annulée par navigation au moment de l'erreur), ou un saut
de réseau intermédiaire. Reproduction nécessaire : `curl -i` sur
`/api/room/join_room` avec session fantôme et inspection du corps réel.

## 3. Percée : repro exact du « saut de tour inconstant »

Trois mécanismes avaient été identifiés (`ends_turn:true` par design, succès
vide `card_no_effect`, désynchronisation du replay SharedWorker après
remontage). **Mise à jour (20/08)** : le troisième facteur (ordre de replay
du SharedWorker) semble corrigé — `shared-ws-worker.js::replayState` rejoue
désormais `started` avant `turn_changed` de façon explicite. Les deux
premiers facteurs (design `ends_turn`, silence serveur) restent en revanche
confirmés et inchangés. Je n'ai toujours pas pu déterminer lequel des deux
domine dans les cas signalés à l'origine, ni si le signalement attend que la
carte permette encore de jouer le pion dans le même tour (interprétation la
plus plausible du texte). **Une partie instrumentée** (logs WS des deux
clients + logs Chess-API) reste nécessaire avant de choisir entre correction
de flag et correction de protocole.

## 4. « La page se rafraîchit sans raison »

Candidats identifiés mais non départagés : reconnexion WS silencieuse du
SharedWorker + replay du cache (le plus probable — mais l'ordre de ce replay
a été corrigé depuis, voir INCONNUES #3, ce qui réduit sans l'éliminer ce
candidat), re-`start` déclenché par `ready` chez un client replacé en état
non-`playing`, ou — en dev uniquement — full-reload Vite/HMR lors de
modifications de fichiers. Il manque une reproduction avec timestamps :
corréler l'instant du « refresh » avec les logs `WebSocket error/close`
console et `docker logs nginx_proxy`.

## 5. « L'adversaire peut capturer le roi » (après auto-échec par carte)

Revérifié sur `board/moves.rs` et `cards/effects.rs` réécrits le 20/08 : le
code interdit toujours toute capture de roi dans les coups autorisés, et
aucune carte active ne supprime de roi. Je n'ai trouvé **aucun** chemin
permettant la capture effective, ni avant ni après la réécriture. Si le
phénomène a réellement été observé, il vient soit d'un build antérieur,
soit d'une confusion avec l'état d'échec prolongé (roi en échec plusieurs
tours de suite après carte). À retester sur l'état actuel avant d'en tenir
compte dans les règles.

## 6. Bouton logout invisible : deux mécanismes candidats — [PROBABLEMENT RÉSOLU]

Le composant `front/src/features/settings/components/LogoutButton.tsx` a été
entièrement réécrit depuis la version examinée initialement : il n'utilise
plus d'override `bg-[#06b6d4]!` ni de variant `secondary` translucide, mais
délègue à `ThemeButton tone="red"`, qui pose un fond dégradé opaque visible
au repos (`hover:brightness-95` seulement en survol). Les deux mécanismes
candidats identifiés initialement (build périmé sans `!important` ; défaut
de contraste au repos) semblent donc caducs. **Reste à faire** : confirmer
visuellement (ou via `curl`) que le build servi correspond bien à ce code
source — cet audit n'exécute pas le stack et ne peut pas vérifier le
contenu réel de `front/dist`.

## 7. « Connexions websocket expirent depuis la console » : quel endpoint ?

Le heartbeat applicatif chess (ping/pong texte) n'est répondu que page montée
→ expiration plausible onglet en arrière-plan ou navigation. Mais les
messages console pourraient aussi concerner le SSE notifications (nginx
`proxy_read_timeout 300s` sans keepalive applicatif identifié) ou le WS HMR
de Vite en dev. Il faut le **texte exact** des messages console (URL du
socket) pour attribuer.

## 8. Devinabilité des `game_id` (impact de l'usurpation de slot WS) — [RÉSOLU, sans objet]

Cette question n'a plus d'importance pour le risque qu'elle visait à
évaluer. Depuis les commits du 19-20/08 (`back/Chess-API/src/websocket/
handler.rs` + `back/Room-API` `set_game_players`), le WS chess **exige un
jeton valide et vérifie que `claims.sub` fait partie des deux joueurs
enregistrés** (`chess:game_players:{game_id}`, écrit par Room-API à la
création de la partie). Un attaquant qui devine un `game_id` — même
trivialement, même séquentiel — n'obtient plus de slot : il reçoit un 403.
La génération des IDs par Room-API reste donc non auditée, mais ce n'est
plus nécessaire pour ce risque précis (voir bug #35, mis à jour).

## 9. Vue spectateur : intention produit non tranchée — [CHANGEMENT DE NATURE, toujours ouvert]

La question du brouillard côté spectateur est devenue secondaire : depuis le
commit `79a258c` (20/08, voir bug #18 mis à jour), **le mode spectateur ne
se connecte plus du tout** (403 au moment de l'upgrade WS, car la
vérification d'appartenance à `chess:game_players` ne laisse passer que les
deux joueurs). La vraie question à trancher est désormais en amont : ce
retrait est-il volontaire (feature mise en pause pendant le durcissement de
la sécurité WS, à restaurer proprement) ou un effet de bord non anticipé du
correctif de sécurité (bug #35) ? Le nom du commit (« update back », sans
mention de spectateur) et l'absence de tout nettoyage côté front
(`spectate=true` toujours envoyé, composants `SpectateGame.tsx`/
`useSpectateGame.ts` toujours présents et inchangés) penchent vers un effet
de bord non anticipé plutôt qu'une décision produit — mais seule l'équipe
peut confirmer. Si la fonctionnalité doit être restaurée, la question
initiale (brouillard subi vs vue « arbitre ») redevient pertinente.

## 10. Impact runtime des `refresh` en POST morts — [RÉSOLU]

Le code mort a disparu : `front/src/api/api.ts` a été réduit à 4 constantes
de chemin, et `authService.refreshToken()` n'existe plus dans
`authService.ts` (seule subsiste `refresh()`, en GET, conforme). Recherche
exhaustive de `refreshToken`/`/refresh` dans `front/src` : aucune occurrence
suspecte restante. Le signalement original « requêtes de refresh mal
formées » n'a donc plus de cause identifiable dans le code actuel — voir
bug #27, marqué résolu.

## 11. Nouveau (20/08) — le retrait du mode spectateur est-il volontaire ?

Doublon volontaire avec le point 9 ci-dessus, formulé différemment pour ne
pas le perdre : c'est la question la plus urgente à poser à l'équipe avant
de toucher au WS chess. Si volontaire (durcissement temporaire), le travail
restant est de restaurer un chemin spectateur explicite. Si non anticipé,
c'est une régression à corriger en priorité (une fonctionnalité citée dans
`AUDIT_PROMPT.md` comme existante — « vue spectateur » — a cessé de
fonctionner entre l'audit initial et cette repasse).

## 12. Shop upload (bug #36) : exploité en pratique ?

L'analyse statique est sans ambiguïté (aucune vérification d'identité dans
`handle_upload_item`), mais je n'ai pas pu exécuter le stack pour confirmer
qu'aucune couche non versionnée (WAF, règle nginx additionnelle en
production, filtrage réseau externe) ne réduit l'exposition réelle. À
vérifier avec un `curl` sans cookie depuis l'extérieur du réseau Docker
avant de considérer le correctif comme urgence absolue vs. urgence relative.

## 13. Timing des signalements de cartes vs. le correctif front « playable »

Plusieurs entrées de `BUGS.md` (#2, #4, #9, #11) ont vu leur gravité perçue
révisée à la baisse parce que le front masque désormais le bouton « Jouer »
quand `playable=false`. Je n'ai aucun moyen de dater ce correctif front par
rapport aux signalements originaux (pas d'accès à l'historique des tickets/
Discord de l'équipe) : si les signalements sont postérieurs au correctif,
soit ils décrivent un tout autre chemin (à investiguer plus avant), soit ils
concernent uniquement les cartes Vétéran (non couvertes par ce correctif,
voir « Cause commune » dans `BUGS.md`). À confirmer avec l'équipe.

## 14. Non vérifié à l'exécution

Tout cet audit est statique. Les chaînes marquées « confirmé » le sont par
lecture croisée du code, sans exécution des services (pas de docker ici).
Les estimations d'effort supposent les correctifs faits par quelqu'un qui
connaît déjà Rust/React du projet.

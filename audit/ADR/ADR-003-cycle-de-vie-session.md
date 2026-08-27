# ADR-003 — Cycle de vie de la session joueur (présence, rooms, identité WS)

> **Mise à jour (re-vérification du 20/08) : la partie « identité WS » de
> l'option 2 recommandée ci-dessous a été implémentée entre-temps.**
> `back/Room-API` écrit désormais `chess:game_players:{game_id}` à la
> création de toute partie (room privée/publique, matchmaking classé,
> tournoi — commit `5a190ef "feat: register game players in redis for slot
> verification"`), et `back/Chess-API/src/websocket/handler.rs` exige un
> jeton valide et vérifie l'appartenance à cette liste avant d'accepter la
> connexion (403 sinon). C'est exactement la phrase de la décision
> recommandée : « le WS chess exige un jeton valide et vérifie que
> `claims.sub` est bien l'un des deux joueurs enregistrés dans la room ».
> Voir `audit/BUGS.md` bug #35 (mis à jour) pour le détail. **Ce qui reste
> à faire de cette ADR** : le canal de présence dérivé du SSE
> (`presence:{user_id}`), l'auto-réparation de `join_room` sur session
> orpheline, la purge de la ZSET de matchmaking, et la réordonnance de
> `leave_room` pour toujours réinitialiser la session — rien de tout cela
> n'a été trouvé dans le code actuel. Effet de bord non anticipé de la
> partie implémentée : le mode spectateur ne peut plus se connecter du tout
> (voir bug #18) — cette ADR devrait, si elle est reprise, inclure
> explicitement comment un spectateur légitime doit s'authentifier sans
> réintroduire la faille d'origine.

## Contexte

Le bug du « 409 déjà dans une salle » révèle qu'aucun composant ne possède la
vérité sur la présence d'un joueur :

- l'état « dans une salle » vit dans Redis `user:session:{id}` (TTL 2 h réarmé)
  écrit par Room-API (`join_room.rs:83-90`, `create_room.rs:102-111`,
  `play_ranked.rs:71-81`) ;
- il n'est libéré que par des appels explicites (`leave_room`, `cancel_ranked`,
  fin de partie via le stream `chess:game:events`) ;
- Room-API n'a **aucune** détection de déconnexion (ses fichiers ws sont
  vides : `src/ws/ws_handler.rs`, `src/ws/socket_manager.rs`) ; le SSE de
  Notification-API ne remonte pas de présence ; le WS chess garde le slot pour
  reconnexion (`lobby.rs:148-157`) ;
- le front n'appelle `leaveRoom` qu'au clic bouton
  (`RoomLobbyPage.tsx:93-103`), aucun `beforeunload`/`pagehide` (0 occurrence
  dans front/src) ;
- cas piégé : si la room expire avant le joueur, `leave_room` échoue en 500
  **avant** la réinitialisation de session (`services/room.rs:317-319`,
  `leave_room.rs:59-65`) : session libérable par personne jusqu'au TTL.

Le même flou permet l'usurpation de slot joueur sur le WS d'échecs : jeton
optionnel (`Chess-API/src/websocket/handler.rs:48-67`), slots attribués
premier arrivé premier servi sans vérification d'appartenance à la room.

## Options envisagées

1. **Cleanup défensif partout** : `beforeunload` front + TTL courts +
   possibilité d'appeler `leave` quand la room n'existe plus (délier la
   réinitialisation de session de l'existence de la room).
   - Avantages : correctifs locaux, aucun changement d'architecture.
   - Inconvénients : `beforeunload` n'est pas fiable (mobile, crash, onglet
     tué) ; le fantôme redevient possible ; l'usurpation de slot reste.
2. **Présence dérivée des connexions vivantes** (recommandé) :
   - le SSE Notification-API devient le canal de présence : chaque utilisateur
     authentifié connecté publie un heartbeat Redis (clé
     `presence:{user_id}` TTL ~30 s, renouvelée par le SSE) ;
   - Room-API consulte/joint cette présence : `join_room` force-la libération
     d'une session dont l'utilisateur est absent ou dont la room n'existe
     plus (auto-réparation du cas piégé) ;
   - le matchmaking purge sa ZSET des joueurs sans présence (elle n'a pas de
     TTL aujourd'hui) avant tout pairing ;
   - le WS chess **exige** un jeton valide et vérifie que `claims.sub` est
     bien l'un des deux joueurs enregistrés dans la room (mapping écrit par
     Room-API à la création de partie) ; sinon `game_full`/`forbidden`.
3. **Superviseur central** : un service « presence » dédié agrégeant SSE+WS.
   - Inconvénient : nouveau composant à exploiter pour un besoin couvert par
     l'existant.

## Décision recommandée

Option **2**, avec correctifs immédiats en amont (indépendants et à faire
même si l'ADR était refusé) :

- `leave_room` doit **toujours** réinitialiser la session, même si la room a
  disparu (réordonner `leave_room.rs:59-81`) ;
- exposer au lobby front l'état de session (déjà renvoyé par `/api/user/me`)
  pour désactiver « Rejoindre » et proposer « Revenir dans ma salle » ou
  « Quitter la salle fantôme » ;
- TTL de session réduit (par ex. 15 min) et non réarmé par des écritures
  non liées à l'activité du joueur.

## Conséquences

- Le SSE devient critique pour le matchmaking et le join : il faut un
  heartbeat et une reconnexion propre (voir ADR-002 pour le modèle de tick).
- La vérification d'appartenance au WS chess fait de Room-API le guichet
  unique de la vérité « qui joue quoi » — cohérent avec le fait qu'il crée
  déjà les parties.
- L'auto-réparation (join qui purge la session orpheline) supprime la
  catégorie entière de tickets « je suis coincé hors de ma salle ».
- Effort estimé : L (SSE présence S, Room-API M, WS chess M, front S).

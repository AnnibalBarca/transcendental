# Annexe — rapport détaillé du bug 409 « déjà dans une salle »

Rapport d'exploration Room-API (exhaustif), intégré par référence dans
`BUGS.md` #26.

## Déclencheur du 409

`back/Room-API/src/http/handlers/join_room.rs:53-69` : le 409 survient quand
le champ `status` du hash Redis `user:session:{user_id}` diffère de `"none"`
(`waiting`, `playing`, `matchmaking`). Gardes identiques dans
`create_room.rs:70-72` et `play_ranked.rs:55-57`. Aucune base SQL, aucun état
en mémoire : uniquement Redis (HSET + EXPIRE 7200 s, `src/user_state.rs:33-50`).

## Écritures vers un état non-`none`

| Emplacement | Statut écrit |
|---|---|
| `join_room.rs:83-90` | `waiting` |
| `create_room.rs:102-111` | `waiting` |
| `play_ranked.rs:71-81` | `matchmaking` (room_id=`"matchmaking"`) |
| `services/room.rs:154-163` (start_room) | `playing` |
| `services/matchmaking.rs:75-89` | `playing` |
| `services/tournament.rs:133-137,166`, `tournament_matchmaking.rs:58` | `playing` |

## Réinitialisations existantes (liste exhaustive)

- `leave_room.rs:71-81` — seulement si status `waiting`/`playing` ;
- `cancel_ranked.rs:46-59`, `cancel_ranked_tournament.rs:46-53` ;
- kick : `services/room.rs:222-228` — `let _ = save_session(...)` **ignore les
  erreurs Redis** ;
- fin de partie : `services/game_result.rs:366-387` (via stream
  `chess:game:events`), tous chemins (victoire/nul/annulé, l.246-289) ;
- Chess-API `cleanup_user_session` (`game.rs:48-71`) : uniquement à la fin
  réelle de partie, jamais à la déconnexion WS.

## Voies de non-nettoyage (cœur du bug)

1. Room-API n'a **aucun** websocket : `src/ws/ws_handler.rs` et
   `src/ws/socket_manager.rs` sont vides (1 ligne). Rien n'observe une
   déconnexion.
2. Front : `leaveRoom()` uniquement au bouton Quitter/Annuler
   (`front/src/pages/Room/RoomLobbyPage.tsx:93-103`). Aucun
   `beforeunload`/`pagehide`, aucun leave dans les cleanups d'`useEffect`.
   Le TTL 2 h est réarmé par toute écriture de session.
3. Piège majeur : si la room a expiré, `services/room.rs:317-319` renvoie
   `"Room not found"` et le handler `leave_room.rs:59-65` répond 500 **avant**
   la réinitialisation (l.71-81) : la session orpheline devient non libérable
   via l'API. Et si la page de room ne charge pas, le bouton Quitter n'est
   même pas rendu (`RoomLobbyPage.tsx:152-171`).
4. `clean_stale_public` (`src/cache/room.rs:274-316`) ne purge que la ZSET de
   listing : les sessions restent `waiting`.
5. Statut `matchmaking` résiduel : `leave_room.rs:41-43` rejette tout statut
   hors `waiting`/`playing` ; la ZSET de file n'a pas de TTL
   (`cache/matchmaking.rs:17-52`) et le pairing force après 30 s
   (`find_best_pair`, l.156-176) → un fantôme peut être apparié à un vrai
   joueur.
6. WS chess : la fermeture du socket garde le slot « for reconnection »
   (`Chess-API/src/websocket/lobby.rs:148-157`), sans cleanup de session.
7. SSE Notification (`Notification-API/src/http/handlers.rs:43-87`) : la
   déconnexion retire juste la connexion d'une map (`api-core/sse/connection.rs:107`),
   rien n'est notifié à Room-API.

## Scénario reconstitué

join/création → `waiting` → refresh ou fermeture d'onglet sans `leave_room` →
retour au lobby → « Rejoindre » → 409. Si la room a expiré entre-temps, le
`leave_room` de secours renvoie 500 et le joueur reste bloqué jusqu'à
expiration du hash (≤ 2 h).

## Sur le « 409 sans corps »

Room-API renvoie toujours `{"status":409,"error":"Already in a room or matchmaking"}`
(`src/http/response.rs:4-9`), sérialisé dans le stream `gateway:responses`
(`service.rs:95-105`) ; la gateway recopie le JSON en body avec
`content-type: application/json` (`Gateway-API/src/http/response_listener.rs:40-56`).
Aucun `StatusCode::CONFLICT` nu n'existe sur ce chemin ; nginx ne supprime pas
les corps d'erreur. Voir INCONNUES #2.

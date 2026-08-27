# ADR-002 — Synchronisation d'état du jeu par websocket et horloges

> **Mise à jour (re-vérification du 20/08)** : `Chess-API/src/game.rs`
> n'existe plus (éclaté en `game/commands.rs`, `game/session.rs`,
> `game/redis_helpers.rs` — toutes les lignes `game.rs:NNNN` citées
> ci-dessous sont donc obsolètes, voir `audit/BUGS.md`/`ARCHITECTURE.md`
> pour les chemins à jour). Sur le fond : **un des correctifs de l'option 1
> a été appliqué** — le replay du SharedWorker suit désormais un ordre
> explicite et correct (`shared-ws-worker.js::replayState`, `started` avant
> `turn_changed`) — donc ce point précis de la section « Décision
> recommandée » est fait. Les autres (horloge autoritaire diffusée
> périodiquement, séquenceur `seq`, ne pas renvoyer `start` sur `ready`) ne
> le sont pas. **Le volet spectateur a régressé plutôt qu'avancé** : ce
> n'est plus une question de « brouillard/évènements exposés au spectateur
> dieu », le spectateur ne peut désormais plus se connecter du tout (voir
> bug #18 dans `BUGS.md`) — toute reprise de ce chantier doit d'abord
> statuer sur la restauration d'un accès spectateur légitime (cf. ADR-003
> mis à jour) avant de traiter le format des messages qu'il reçoit.

## Contexte

Une famille de symptômes (timer adverse figé, Top chrono sans accélération,
surlignage/tour non répercutés, page qui « se rafraîchit », Percée « inconstant »)
provient de la même architecture client-serveur :

- le serveur ne diffuse l'horloge et le multiplicateur **que dans
  `turn_changed`**, émis uniquement quand `ends_turn`
  (`Chess-API/src/game/game.rs:1082-1125`) : une carte comme Top chrono
  (`ends_turn=false`) modifie le temps sans jamais prévenir ;
- l'horloge est extrapolée côté client (`useChessGame.ts:466-477`) à partir du
  dernier `turn_changed` ; le tick serveur (`runner.rs` chaque seconde)
  n'est jamais diffusé tant que personne ne joue ;
- le SharedWorker du front met en cache les messages d'état et les rejoue au
  remontage dans un ordre qui remet `started` **après** `turn_changed`
  (`shared-ws-worker.js:52-79` + `useChessGame.ts:292-302` : `timerStarted=false`,
  compte à rebours fantôme) ;
- la reconnexion d'un joueur rediffuse `ready` aux deux clients
  (`game.rs:222-255`), et le client replacé en état non-`playing` renvoie
  `start` (`useChessGame.ts:257-265`) : redémarrages d'UI intempestifs ;
- les spectateurs reçoivent un plateau « dieu » (voir ADR annexe spectateur :
  `to_json_for_color(None)` ignore le brouillard, `board/mod.rs:42-47`) mais
  pas l'évènement `check` (`game.rs:904-915`).

## Options envisagées

1. **Correctifs localisés** : envoyer `turn_changed` après chaque carte ;
   réordonner le replay du worker ; ne pas renvoyer `start` sur `ready`.
   - Avantages : chaque patch est trivial.
   - Inconvénients : l'horloge reste non autoritaire (dérive client jusqu'à
     100 % pendant l'inactivité du flux), les reprises de session restent
     fragiles.
2. **Horloge serveur autoritaire diffusée périodiquement** (recommandé) :
   - le `run_game_loop` qui tick déjà chaque seconde diffuse un message
     léger `clock {white_ms, black_ms, multiplier, turn, seq}` 1×/s aux
     joueurs et spectateurs ; le client n'extrapole plus qu'entre deux ticks ;
   - tout évènement modifiant l'état (carte, coup, connexion) déclenche la
     diffusion immédiate du même message : un seul format ;
   - séquenceur `seq` croissant : le client (et le worker) ignorent tout
     message d'état plus ancien que le dernier appliqué — le problème
     d'ordonnancement du replay disparaît par construction.
3. **Migration vers un canal d'état différenciel** (diffs par coup) :
   - Avantages : bande passante minimale.
   - Inconvénients : protocole complet à réécrire, fragile aux reprises ;
     surdimensionné.

## Décision recommandée

Option **2**, avec ces compléments :

- corriger le replay du worker (définir un ordre canonique `connected →
  players_info → game_state → turn_changed/clock` et ne jamais rejouer
  `started` après un état de jeu), ou plus simple : ne mettre `started` en
  cache qu'en l'absence de `game_state` ;
- ne renvoyer `start` que si aucune partie n'est connue localement (état
  « connecting » pur), jamais sur `ready` reçu en cours de partie ;
- répondre au `ping` applicatif **dans le worker** (page ou non) pour
  éliminer les expirations d'onglet en arrière-plan ;
- décider explicitement la politique spectateur (brouillard subi ou non,
  évènements `check`/`hand` exposés) et l'implémenter dans le même format
  `clock`/`game_state` — la vue SpectateGame front doit alors consommer les
  mêmes champs (`fogRemaining`, `squareModifiers`, `lastMove`).

## Conséquences

- Trafic : ~1 message/s/partie/client — négligeable, et il remplace une
  extrapolation fausse (moins de litiges d'horloge en fin de partie).
- Le timer devient testable côté serveur (assertion sur `clock` diffusé).
- Le séquenceur `seq` impose que **tous** les producteurs d'état passent par
  le même point d'émission : refactor léger de `game.rs` (aujourd'hui chaque
  branche construit son `turn_changed` à la main, 4 occurrences).
- Effort estimé : M serveur + S/M front.

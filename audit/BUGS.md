# Bugs — analyse, attribution et confiance

Légende confiance : **confirmé** = code fautif lu et mécanisme reconstitué de
bout en bout ; **probable** = mécanisme établi mais chaîne complète non
vérifiée à l'exécution ; **à investiguer** = pistes sans certitude.

Échelle d'effort : **S** (< 2 h), **M** (½ - 2 jours), **L** (3 - 5 jours).

---

## Cause commune identifiée : le contrat de jeu « silencieux »

**Mise à jour (re-vérification post-refactor, code synchronisé le 20/08) :
le front a été corrigé depuis la rédaction de l'audit initial — la moitié
« UI » de ce problème est désormais traitée.** `back/Chess-API/src/game.rs`
n'existe plus : il a été éclaté en `back/Chess-API/src/game/commands.rs`,
`.../session.rs` et `.../redis_helpers.rs`. Toutes les citations `game.rs:NNNN`
ci-dessous et dans les entrées suivantes ont été remappées vers ces fichiers.

Le serveur silencie toujours l'échec de jouabilité :

`back/Chess-API/src/game/game_loop/core.rs:360-363`

```rust
let result = if is_card_playable(&board, player, card_id) {
    board.apply_card_effect(card_id, card_rarity, player, target)?
} else {
    CardResult::new("card_no_effect")   // ← succès silencieux
};
```

puis `back/Chess-API/src/game/commands.rs:479-480` (`ends_turn`) et
`:561-577` (le tour passe et `turn_changed` n'est diffusé que dans ce cas).
La carte est retirée de la main sans condition
(`core.rs:383-386`, deuxième occurrence après les effets spéciaux Poubelle/
Roue de la fortune ; la première `hand.remove` à `core.rs:300` est un chemin
différent non lié à `play_card`).

**Mais le front ne l'ignore plus.** `CardRegister` porte désormais
`playable?: boolean`
(`front/src/features/games/ChessGame/types/cardTypes.ts:9`) ; le serveur le
calcule et l'envoie pour chaque carte de la main
(`back/Chess-API/src/game/cards/types.rs:124` :
`"playable": is_card_playable(board, color, *c)`) ; et
`front/src/features/games/ChessGame/components/CardsModal.tsx:110`
(`const isPlayable = centeredCard?.playable !== false;`) conditionne
l'affichage même du bouton « Jouer » (`:182-206` : le bouton n'est **rendu**
que si `isPlayable`, sinon seul « Défausser » reste visible). Pour la voie
normale (clic dans le carrousel de cartes, `Cards.tsx` ouvre toujours la
modale avant de jouer), une carte que le serveur jugerait `card_no_effect`
n'est donc plus jouable depuis l'UI dans la majorité des cas — le
`card_no_effect` silencieux et la perte de tour associée ne devraient plus se
produire pour un joueur qui n'interagit qu'avec le bouton.
**Conséquence pour les entrées ci-dessous — nuance importante selon le type
de condition** : `is_card_playable`/`playable_if`
(`back/Chess-API/src/game/cards/registry.rs`) ne vérifie que l'**existence**
d'une pièce du bon type/couleur (`PieceRequirement::Own(PieceType)`, etc.),
**jamais son `has_moved`**. Le nouveau garde-fou front (bouton masqué si
`playable=false`) corrige donc bien les cartes dont le seul problème était
« la pièce requise n'existe plus » (Pyromane #2, Maçon en furie #4, Sniper
#9, cartes d'échange #11), mais **ne corrige pas** les cartes Vétéran (#5,
#7, #8) : leur carte reste affichée comme jouable (une tour/un fou/un
cavalier existe), le bouton apparaît, le joueur choisit une cible ayant déjà
bougé, et c'est *seulement là* que le serveur rejette
(`err_veteran_piece_moved`, toujours silencieux côté client — voir #5). Pour
Percée/Breakthrough (#10) et les cartes en échec (#12/#13), le problème est
ailleurs (flag `ends_turn`, absence de garde d'échec) et n'est pas affecté
par ce correctif front. Un correctif serveur explicite (rejet + `err_*`,
et si possible sérialiser `has_moved` pour une pré-validation ciblée) reste
donc nécessaire pour la famille Vétéran — voir ADR-001.

---

# Moteur de jeu et cartes

## 1. La carte Traître ne fonctionne pas

- **Fichiers** (chemins/lignes revérifiés sur le code synchronisé du
  20/08 ; le nom interne est désormais `Traitor`, `clear_traitre()` a été
  renommé `clear_traitor()`) :
  - `back/Chess-API/src/game/cards/effects.rs:673-683` (`apply_traitor` :
    pose `card_state.traitor`)
  - `back/Chess-API/src/game/board/mod.rs:63-70` — **cause directe** : pour
    toute pièce ennemie non contrôlée par le traître, le JSON envoyé au
    client écrase `move_set` par `[]` (variable `traitor_controlled` l.63-65)
  - `front/src/features/games/ChessGame/components/ChessBoard.tsx:154-177` —
    un clic ne produit un coup que si `dest` figure dans le `moveSet` reçu
  - `back/Chess-API/src/game/game_loop/move_handler.rs:58-71` (le serveur,
    lui, autorise bien à déplacer un pion adverse en capture intra-camp via
    la condition `is_traitor`)
- **Hypothèse falsifiable** : inchangée — l'effet serveur fonctionne, mais
  comme le client ne reçoit jamais les destinations des pions adverses
  (move_set vidé), il est impossible de matérialiser la capture « traître »
  depuis l'UI ; le joueur joue la carte, rien ne se passe visuellement, puis
  son propre coup efface l'état (`move_handler.rs:100` `clear_traitor()`).
- **Confiance** : confirmé (côté client) / l'intention serveur est cohérente.
- **Pour confirmer** : jouer Traître, tenter en console WS d'envoyer
  `{"action":"move_piece","from":"d7","to":"c6"}` (pion noir capturant pièce
  noire) — accepté par le serveur, donc le blocage est bien purement UI.
- **Effort** : M (exposer les captures traître dans le move_set du JSON pour
  la couleur concernée, + UI de sélection d'une pièce ennemie).

## 2. La carte Pyromane ne fonctionne pas

- **Fichiers** (renommée `Pyromaniac`, id `"12"`) :
  - `back/Chess-API/src/game/cards/registry.rs:92-98` (exige Tour alliée + Fou ennemi)
  - `back/Chess-API/src/game/cards/effects.rs` (swap tour↔fou ennemi — conforme
    à la description `front/src/data/cards.ts:183-190`)
  - `back/Chess-API/src/game/game_loop/core.rs:360-363` (silence `card_no_effect`)
  - **toujours vrai** : `back/User-API/src/db/migrations.rs:527-537`
    (migration `039_seed_default_player_cards`) et le trigger
    `040_seed_default_cards_trigger` (l.539-556) ne sèment que les ids
    `'1','2','3','5','6','7','8','9','10','11'` — Pyromane (id `12`) n'est
    **toujours pas** dans le deck de départ, confirmé à nouveau sur le code
    synchronisé.
- **Hypothèse** : l'effet est correct ; le bug perçu vient de la combinaison
  (a) carte absente du deck par défaut (confirmé ci-dessus), (b) quand le
  joueur l'obtient (shop/pack) et que le fou ennemi n'existe plus,
  `is_card_playable` → false. **Correction depuis la rédaction initiale** :
  côté UI, le bouton « Jouer » n'est plus proposé du tout quand `playable`
  vaut `false` (voir « Cause commune » ci-dessus,
  `CardsModal.tsx:110,182-206`) — la carte ne peut donc plus être « jouée
  pour rien » depuis la modale normale. Le signalement original pourrait
  dater d'avant ce correctif front, ou concerner un état où le serveur et le
  client divergent brièvement sur la présence du fou ennemi (fenêtre entre
  la capture du fou et le rafraîchissement de la main).
- **Confiance** : probable → **à réévaluer à la baisse** (le chemin normal
  est maintenant bloqué côté UI ; seul un scénario de désynchronisation
  reste plausible).
- **Pour confirmer** : journaliser `message_id` de `card_result` en jeu —
  si `card_no_effect` apparaît malgré tout, chercher un décalage entre l'état
  de main affiché et l'état serveur au moment du clic.
- **Effort** : S (rejet explicite serveur reste souhaitable — ADR-001).

## 3. La carte Bastion ne fonctionne pas

- **Mise à jour majeure (re-vérification du 20/08) : cette entrée est
  résolue, mais pas par un correctif — Bastion a été retirée du jeu depuis
  la rédaction de l'audit initial, exactement comme le décrit déjà
  `AGENTS.md` (« Cartes retirées volontairement… commentées… c'est un choix,
  pas un bug »).** Preuves, sur le code synchronisé :
  - `back/Chess-API/src/game/cards/types.rs:35,71,107` — `CardId::Bastion`
    (et `Magnetism`) sont **commentés** dans l'enum, `as_str`/`from_str` :
    id `"26"` n'a plus de variante active.
  - `back/Chess-API/src/game/cards/registry.rs:165` et
    `back/Chess-API/src/game/cards/effects.rs:605-628` — l'entrée de
    registre et `apply_bastion` sont entièrement commentées.
  - `back/User-API/src/db/migrations.rs:886-893` — migration
    `054_disable_magnetisme_bastion` : `UPDATE shop_catalog SET is_active =
    FALSE WHERE item_id IN ('25','26')` **et** suppression de ces cartes de
    `player_cards`/`player_deck` pour tous les comptes existants. C'est une
    désactivation base de données, pas seulement un commentaire de code.
  - Le commit responsable côté Chess-API : `6aa3f4c "remove unstable card"`
    (20/08), qui touche 10 fichiers du moteur de cartes — cohérent avec le
    nom du commit (Bastion/Magnétisme jugées instables, retirées plutôt que
    corrigées).
- **Ancienne hypothèse (toujours vraie historiquement, gardée pour mémoire)** :
  au tour du joueur, le dernier coup enregistré était celui de l'adversaire
  (`lm.moved.color == player` toujours faux), donc Bastion n'était jamais
  jouable — c'est probablement *pourquoi* la carte a été jugée « instable »
  et désactivée plutôt que corrigée.
- **Confiance** : confirmé — retrait volontaire et documenté (migration SQL +
  code commenté), pas un bug résiduel à corriger.
- **Reliquat front confirmé** : `front/src/data/cards.ts` et les 6 fichiers
  `front/src/i18n/locales/*/translation.json` référencent encore Bastion
  (titre/description). Sans appel shop (désactivé) ni présence en main
  (retirée des decks), le risque d'affichage est faible, mais le nettoyage
  n'est pas fait.
- **Effort** : nul côté jeu (déjà fait) ; **S** pour nettoyer les reliquats
  front (`cards.ts` + traductions) devenus inertes.

## 4. Le Maçon en furie est injouable et ne fait rien

- **Fichiers** (renommée `FuriousMason`, id `"7"` ; lignes revérifiées) :
  - `back/Chess-API/src/game/cards/effects.rs` (swap fou↔tour alliés)
  - `back/Chess-API/src/game/cards/registry.rs:52-58` (exige Fou allié + Tour
    alliée — `PieceRequirement::Own(Bishop)` + `Own(Rook)`)
  - `front/src/data/cards.ts:145-151`
- **Hypothèse** : l'effet existe et fonctionne quand les deux pièces sont
  présentes ; « injouable/ne fait rien » correspond aux cas où l'une des deux
  a été prise. **Mise à jour** : depuis le correctif front décrit dans
  « Cause commune », le bouton « Jouer » disparaît quand `playable=false`
  (`CardsModal.tsx:110,182-206`), donc le clic-sans-effet ne devrait plus se
  reproduire dans le flux normal. Le signalement « injouable » (carte grisée/
  non sélectionnable) pourrait en réalité décrire ce nouveau comportement
  correct (bouton absent) plutôt qu'un bug — à confirmer selon la date du
  signalement par rapport au correctif front.
- **Confiance** : probable → mécanisme confirmé, mais la manifestation exacte
  attendue aujourd'hui a changé (voir INCONNUES).
- **Pour confirmer** : capturer le fou adverse du joueur puis vérifier que
  Maçon n'affiche plus de bouton « Jouer » (comportement attendu) plutôt que
  de chercher un `card_no_effect` en retour.
- **Effort** : S (rejet explicite serveur reste souhaitable — ADR-001).

## 5. Le Vétéran fou est bugué

- **Fichiers** (lignes revérifiées sur le code du 20/08 — nom interne
  `VeteranBishop`) :
  - `back/Chess-API/src/game/board/moves.rs:90-91` — un fou vétéran gagne un
    pas **orthogonal** `[(1,0),(-1,0),(0,1),(0,-1)]`
  - `front/src/data/cards.ts:263-270` — la description annonce des pas **diagonaux**
  - `back/Chess-API/src/game/cards/effects.rs:499-502` — rejet
    `err_veteran_piece_moved` si la pièce a déjà bougé
  - `back/Chess-API/src/game/board/types.rs:107-110` — `has_moved` est
    toujours `#[serde(skip_serializing)]` : le front ne peut pas pré-valider
  - `front/src/features/games/ChessGame/hooks/useChessGame.ts:344-347` — un
    `card_result` invalide reste un no-op silencieux (`// no-op: action
    rejetée silencieusement`, littéralement commenté ainsi dans le code actuel)
- **Hypothèse** : double défaut — (1) description inversée avec Vétéran tour
  (voir #6) ; (2) dès que le fou visé a bougé, le serveur rejette
  silencieusement et l'UI n'affiche rien : « bugué ». **Toujours valable** :
  contrairement à Pyromane/Maçon/Sniper (voir note dans « Cause commune »),
  `playable_if` pour `VeteranBishop` ne vérifie que la présence d'un fou
  (`registry.rs:145-149`), pas son `has_moved` — le bouton « Jouer » reste
  donc affiché même quand tous les fous ont déjà bougé, et le rejet muet se
  produit après sélection de la cible.
- **Confiance** : confirmé (inversion texte/code) + confirmé (absence totale
  de feedback d'erreur carte, non corrigée par le fix front décrit plus haut).
- **Effort** : S (texte) ; M (chaîne de feedback des erreurs de carte, mutualisée).

## 6. Vétéran tour : diagonales vs axes — trancher

- **Fichiers** (lignes revérifiées) :
  - code : `back/Chess-API/src/game/board/moves.rs:89` — la **tour** vétéran
    gagne un pas **diagonal** ; le **fou** vétéran (l.91) gagne un pas
    **orthogonal** (le cavalier vétéran, l.87, gagne lui aussi un pas
    orthogonal — non couvert par le signalement original)
  - texte : `front/src/data/cards.ts:255-262` (Vétéran tour : « de 1 sur les
    axes verticaux et horizontaux ») et `:263-270` (Vétéran fou : « de 1 dans
    chaque diagonale »)
- **Analyse** : les deux descriptions sont **inversées** par rapport au code.
  Le code est cohérent en soi (une tour qui gagnerait un pas orthogonal
  n'obtiendrait rien de nouveau ; le gain utile est le pas diagonal — et
  symétriquement pour le fou). Les tests du dépôt
  (`board/mod.rs:284-302`) valident le comportement code.
- **Verdict recommandé** : le **texte** est faux — échanger les deux phrases.
- **Confiance** : confirmé.
- **Effort** : S.

## 7. Jouer Vétéran tour passe parfois simplement le tour

- **Fichiers** (lignes revérifiées) :
  - `back/Chess-API/src/game/cards/registry.rs:140-144` (`VeteranRook`) —
    `ends_turn: false` : aucun chemin serveur ne fait passer le tour sur
    cette carte
  - `front/src/features/games/ChessGame/components/ChessBoard.tsx` — la
    prévalidation front exige une pièce alliée du bon type, mais ne connaît
    pas `has_moved` (non sérialisé, voir #5)
  - `useChessGame.ts:344-347` (rejet muet)
- **Hypothèse** : le tour ne « passe » pas au sens serveur ; le joueur cible
  une tour ayant bougé → rejet muet côté serveur → l'utilisateur, croyant la
  carte jouée, joue ensuite son coup normal → le tour change. Perception :
  « j'ai joué Vétéran tour, mon tour est passé ». Autre contributeur : après un
  replay du SharedWorker (`shared-ws-worker.js`), un `turn_changed`
  périmé peut réafficher un tour obsolète. **Non affecté** par le correctif
  front du bouton « Jouer » masqué (voir « Cause commune ») puisque
  `is_card_playable` pour cette carte ne teste pas `has_moved`.
- **Confiance** : probable (scénario reconstitué, pas reproduit).
- **Pour confirmer** : activer les logs `[Chess-WS] Received from`
  (`back/Chess-API/src/game/commands.rs:240`, désormais dans `commands.rs` et
  non plus `game.rs`) et rejouer : chercher `play_card` suivi immédiatement
  de `move_piece`.
- **Effort** : S (feedback d'erreur) — sinon couvert par ADR-001/002.

## 8. Vétéran cavalier / Vétéran fou parfois impossibles à cliquer

- **Fichiers** (lignes revérifiées) : mêmes que #5/#7 — `ChessBoard.tsx`
  (prévalidation type+couleur mais pas `has_moved`),
  `back/Chess-API/src/game/board/types.rs:107-110` (`has_moved` masqué),
  `back/Chess-API/src/game/cards/effects.rs:499-502` (rejet),
  `useChessGame.ts:344-347` (muet).
- **Hypothèse** : « impossible à cliquer » = la case visée est refusée par le
  serveur (pièce ayant bougé) sans aucun retour ; toutes les cases semblent
  mortes. Se produit dès que les candidats restants ont bougé — d'où « parfois ».
  Toujours d'actualité malgré le correctif front décrit dans « Cause commune »
  (même raison que #5/#7 : condition non couverte par `is_card_playable`).
- **Confiance** : probable.
- **Effort** : S/M (sérialiser `has_moved` + désactiver les cases invalides ou
  afficher la raison).

## 9. Le Sniper se joue même sans fou, et consomme le tour

- **Mise à jour (re-vérification du 20/08) : probablement corrigé côté UI
  depuis la rédaction initiale.**
- **Fichiers** (lignes revérifiées) :
  - `back/Chess-API/src/game/cards/registry.rs:105-108` — exige Fou allié
    (`PieceRequirement::Own(PieceType::Bishop)`), `ends_turn: true`
  - `back/Chess-API/src/game/cards/effects.rs:332` (`apply_sniper`) — ne
    renvoie **jamais d'erreur** : la boucle ne trouve aucun fou, l'effet
    « réussit » à vide (mécanisme serveur inchangé)
  - `core.rs:360-363` (`card_no_effect`) + `commands.rs:479-480,561-577`
    (tour passé)
  - **front, corrigé** : `CardsModal.tsx:110,182-206` — le bouton « Jouer »
    n'est désormais **plus affiché du tout** quand `playable=false`, et
    `playable` pour Sniper ne dépend que de l'existence d'un fou allié (pas
    de `has_moved`, contrairement aux cartes Vétéran) — donc ce cas est bien
    couvert par le nouveau garde-fou front.
- **Hypothèse** : sans fou, `is_card_playable` = false → en écriture WS brute
  le serveur accepterait toujours silencieusement (`card_no_effect` + tour
  consommé, mécanisme inchangé), mais depuis le poste client normal le
  bouton n'apparaît plus dès qu'aucun fou allié n'est sur l'échiquier. Le
  symptôme original ne devrait plus se reproduire via l'UI.
- **Confiance** : confirmé (mécanisme serveur) ; probable-résolu (UI) — à
  confirmer en rejouant la scène exacte du signalement.
- **Effort** : S (rejet serveur explicite reste recommandé pour fermer la
  voie WS brute — ADR-001).

## 10. Percée et cartes à ciblage « sautent le tour » — problème le plus grave

- **Fichiers** (lignes revérifiées ; `Percee` renommée `Breakthrough`, id `"29"`) :
  - `back/Chess-API/src/game/cards/registry.rs:180-183` — **`Breakthrough` a
    `ends_turn: true`** : jouer la carte termine le tour par conception (ou
    par erreur de flag)
  - `core.rs:360-363` (silence si non jouable — pions tous pris)
  - `back/Chess-API/src/game/commands.rs:561-577` (déplacé depuis
    `game.rs:1082-1125`) — `turn_changed` n'est diffusé **que** si `ends_turn` :
    les cartes `ends_turn=false` ne produisent aucun évènement de tour
  - `front/public/shared-ws-worker.js:58-81` + `useChessGame.ts` — **mise à
    jour** : l'ordre de replay du cache est maintenant explicite et correct
    (`replayState`, l.68-71 : `["game_state","hand","started","turn_changed"]`
    — `started` est bien rejoué **avant** `turn_changed`, contrairement à ce
    que décrivait l'audit initial). Ce facteur 3 semble donc corrigé ; il n'a
    pas pu être confirmé comme la cause dominante à l'origine, donc son
    retrait ne clôt pas l'entrée.
  - `useChessGame.ts:344-347` — rejet muet (`err_percee_requires_pawn`,
    `err_need_target`), toujours vrai
- **Hypothèse (multi-facteurs, hiérarchisée)** :
  1. **Design** : `ends_turn=true` sur Breakthrough (et Frog, Cannon, tous
     les échanges) fait passer le tour après la carte — perçu comme « le
     tour saute » alors que le joueur attendait de pouvoir encore bouger.
     Incohérent avec Vétéran/Ninja/Zone mortelle (`ends_turn=false`) : c'est
     l'incohérence qui rend le comportement « inconstant ». **Non affecté**
     par le correctif front du bouton masqué (voir « Cause commune ») —
     Breakthrough exige un pion allié, généralement présent, donc le bouton
     reste offert ; le problème est le flag `ends_turn`, pas la jouabilité.
  2. **Silence serveur** : conditions non remplies → tour consommé à vide.
  3. **Réordonnancement du cache worker au remontage** : probablement
     corrigé (voir ci-dessus) — dégradé en facteur mineur/historique.
- **Confiance** : confirmé pour 1 et 2 (code lu) ; le facteur 3 est
  maintenant probablement non reproductible (ordre de replay corrigé).
- **Pour confirmer** : instrumentation WS (logger brut des frames dans les
  deux onglets) pendant une partie où le bug se produit encore.
- **Effort** : décision de game-design + M (voir ADR-001).

## 11. Cartes d'échange : jouables à tort + pas de choix de pièces

- **Fichiers** (lignes revérifiées) :
  - `back/Chess-API/src/game/cards/effects.rs:252-283` — `apply_swap` choisit
    la **première** pièce trouvée de chaque type (`find_piece_index`, l.284)
    : aucun choix laissé au joueur — **toujours vrai, non affecté** par le
    correctif front (celui-ci évite de jouer la carte pour rien, mais ne
    résout pas l'absence de ciblage)
  - `core.rs:360-363` — restent « jouables » côté serveur (succès à vide)
    quand une des deux pièces manque
  - **mise à jour front** : le champ `playable` renvoyé dans chaque `hand`
    (`back/Chess-API/src/game/cards/types.rs:124`) est désormais lu par le
    front (`cardTypes.ts:9`, `CardsModal.tsx:110,182-206`) : le bouton
    « Jouer » est masqué dès qu'une des deux pièces requises manque. Le
    volet « jouable à tort » de ce signalement est donc probablement résolu
    pour le flux normal ; **le volet « pas de choix de pièce » reste
    entier**, c'est un problème de protocole, pas de jouabilité.
- **Hypothèse** : confirmé pour le manque de ciblage (protocole à deux étapes
  inexistant) ; probable-résolu pour le « jouable à tort » perçu depuis
  l'UI normale.
- **Confiance** : confirmé (mécanisme serveur) / probable (statut UI actuel).
- **Effort** : M-L (protocole de sélection + UI — voir ADR-001).

## 12. Jouer une carte en étant échec passe la main

- **Fichiers** (lignes revérifiées) :
  - `back/Chess-API/src/game/game_loop/core.rs:326-397` — `play_card` ne
    vérifie toujours jamais `is_in_check(player)` — confirmé à nouveau sur
    le code du 20/08
  - `back/Chess-API/src/game/commands.rs:479-481` — `ends_turn` →
    `record_move()` → `isWhiteTurn` inversé (déplacé depuis `game.rs:1002-1006`)
- **Hypothèse** : aucune garde d'échec → on peut « passer » en étant échec.
  L'adversaire joue, puis `make_move` recalcule `check`/`checkmate`
  (`move_handler.rs:118-120`) : le joueur en échec peut même être déclaré mat
  sur le coup adverse — le jeu ne reste pas incohérent, mais la règle est
  violée.
- **Confiance** : confirmé.
- **Effort** : S (rejeter `err_in_check` quand `is_in_check(player)` et que la
  carte ne résout pas l'échec) — s'intègre à l'ADR-001.

## 13. Jouer une carte peut nous mettre soi-même en échec

- **Fichiers** (`effects.rs` a été réécrit le 20/08 — numéros de ligne
  indicatifs, la structure ligne par ligne des cartes a bougé) :
  - `back/Chess-API/src/game/cards/effects.rs` — aucun `apply_*` ne valide
    l'échec soi après modification ; revérifié sur les effets encore actifs
    (Voyage/`Journey`, échanges/`apply_swap` l.252-283, Roulette/
    `RussianRoulette`) — toujours aucun appel à `is_in_check` après
    application d'un effet
  - `board/moves.rs` — toute capture de roi reste interdite dans
    `is_move_legal` ; le canon exclut aussi le roi de ses cibles
- **Hypothèse** : se mettre en échec soi-même est possible (confirmé), mais la
  capture du roi annoncé dans le ticket paraît **impossible** via un coup
    normal (garde l.451-455) — aucune carte ne supprime de roi non plus. La
  conséquence réelle est un état illégal prolongé (échec non traitable tant
  que le joueur n'a pas de coup légal → mat détecté au coup adverse suivant).
- **Confiance** : confirmé (auto-échec) ; à investiguer (capture de roi —
  jamais vue dans le code, vraisemblablement une interprétation d'un état
  d'échec persistant).
- **Effort** : décision de règles + S pour valider post-effet (rejouer
  `is_move_legal`-like sur l'état résultant).

---

# Interface et état du jeu

## 14. Pièce sélectionnée avant une carte → vue figée / verrouillage

- **Fichiers** :
  - `front/src/features/games/ChessGame/components/ChessBoard.tsx:27-28` —
    `moveSelection`/`selectedSquare` sont un état **local au composant**,
    jamais réinitialisés ni par le tour, ni par une carte, ni par `game_state`
  - `ChessBoard.tsx:163` (`if (!isMyTurn) return;`) — les clics suivants sont
    muets pendant le tour adverse, mais les surlignages (l.225) restent rendus
  - `ChessGame.tsx:112-120` — jouer une carte ne reset pas la sélection
- **Hypothèse** : la sélection survit au changement de tour ; les destinations
    restent affichées et aucune interaction ne les efface (le clic sur une
    case est bloqué par `!isMyTurn`). Variante « le mouvement a lieu mais la
    vue reste figée » : le drag a réussi (onDragEnd reset l.91-92) mais un
    **clic** antérieur laisse `selectedSquare` en place quand le coup part
    par un autre chemin (promotion, carte).
- **Confiance** : confirmé.
- **Effort** : S (effacer la sélection sur `pieces`/`currentPlayer`/`pendingCardTarget`).

## 15. Timer adverse figé chez l'hôte ; n'accélère pas avec Top chrono

- **Mise à jour (re-vérification du 20/08) : `game.rs` n'existe plus (éclaté
  en `commands.rs`/`session.rs`/`redis_helpers.rs`), toutes les citations
  ci-dessous sont remappées. Un des trois mécanismes semble corrigé.**
- **Fichiers** :
  - `back/Chess-API/src/game/commands.rs:561-577` (déplacé depuis
    `game.rs:1082`) — `turn_changed` (qui porte `timer_running` et
    `time_multiplier`) n'est toujours diffusé **que si `ends_turn`** :
    TopChrono/`TimeBoost` (`ends_turn=false`) modifie le multiplicateur sans
    jamais prévenir les clients — **toujours vrai**.
  - `front/src/features/games/ChessGame/hooks/useChessGame.ts:362-368`ish — le
    multiplicateur n'est mis à jour que sur `turn_changed` — inchangé.
  - **Probablement corrigé** : l'audit initial blâmait un rejeu de `started`
    **après** `turn_changed` au reconnect. Sur le code actuel,
    `front/public/shared-ws-worker.js:68-71` (`replayState`) définit un ordre
    de rejeu explicite `["game_state","hand","started","turn_changed"]` —
    `started` est bien rejoué **avant** `turn_changed`. Le troisième
    contributeur documenté par l'audit semble donc résolu ; il reste à
    vérifier que le serveur (`back/Chess-API/src/game/session.rs:162-175`,
    bloc de reconnexion) envoie ses propres messages dans un ordre cohérent
    — lu : il envoie `game_state`, `hand`, puis `turn_changed`, sans
    `started` du tout pendant une reconnexion à une partie déjà démarrée
    (`session.rs:111-176`), ce qui est cohérent.
  - `back/Chess-API/src/game/game_loop/timer.rs` — le multiplicateur est
    remis à 1 par `record_move` : correct serveur, mais toujours invisible
    client tant qu'aucun `turn_changed` n'est émis pour une carte
    `ends_turn=false`.
- **Hypothèse** : le mécanisme central (pas de diffusion périodique du
  multiplicateur/horloge, dépendance à `ends_turn`) reste confirmé ; le
  facteur « ordre de replay » n'est plus reproductible en l'état.
- **Confiance** : confirmé pour le défaut de diffusion sur cartes
  `ends_turn=false` ; le facteur replay est maintenant probablement non
  reproductible.
- **Effort** : M (diffuser un `turn_changed`/`clock` après toute carte,
  idéalement tick serveur périodique — ADR-002).

## 16. Surlignage/timer du joueur actif non répercutés chez l'adversaire

- **Fichiers** (remappés) : mêmes que #15 — `turn_changed` est bien
  broadcast (`commands.rs:571-576`, déplacé depuis `game.rs:1115-1124`) ; le
  défaut d'affichage vient du client qui n'applique pas l'état à temps
  (reconnexions silencieuses du worker).
- **Hypothèse** : le message arrive mais l'état local peut être écrasé par un
  replay ; ou la socket a été recyclée et le client n'a pas reçu le
  `turn_changed` émis pendant la reconnexion. **Atténué** par la correction
  probable de l'ordre de replay (voir #15) mais pas éliminé (le cas d'un
  message manqué pendant une coupure reste possible indépendamment de
  l'ordre du cache).
- **Confiance** : probable (symptôme = #15/#17, même racine, gravité réduite).
- **Effort** : couvert par ADR-002.

## 17. La page du jeu se rafraîchit sans raison

- **Fichiers** (remappés) :
  - `front/public/shared-ws-worker.js` — reconnexion automatique avec
    backoff ; chaque reconnexion rejoue le cache (`replayState`, l.58-81)
    → remise à zéro visuelle partielle (`game_state` → re-rendu complet) ;
    l'ordre de rejeu de `started` est désormais correct (voir #15), ce qui
    réduit un des contributeurs possibles sans éliminer le symptôme racine
    (cycle déconnexion/reconnexion lui-même).
  - `useChessGame.ts` — sur `ready` (broadcasté aux deux joueurs à chaque
    reconnexion de l'un, `back/Chess-API/src/game/session.rs:88-99`, déplacé
    depuis `game.rs:222-255`), le client non-joueur peut renvoyer `start`.
- **Hypothèse** : un « refresh » perçu = cycle déconnexion/reconnexion WS
  (nginx coupé, service redémarré, réseau) + replay du cache. Alternative non
  exclue (dev) : hot-reload Vite (HMR full reload). Inchangée dans son
  principe malgré la correction de l'ordre de replay.
- **Confiance** : probable.
- **Pour confirmer** : compter les `WebSocket error/close` dans la console au
  moment du phénomène ; vérifier `docker logs nginx_proxy` (502 pendant les
  redémarrages).
- **Effort** : M (ADR-002 : pas de re-`start` intempestif ; le replay ordonné
  semble déjà traité).

## 18. Vue spectateur buguée (brouillard, effets)

- **RÉGRESSION MAJEURE découverte lors de cette relecture (20/08) : le mode
  spectateur n'est plus « bugué », il est désormais entièrement
  non-fonctionnel — la connexion WS est rejetée avant même le handshake.**
  Ce n'est pas une aggravation cosmétique : c'est un changement de nature du
  problème, à traiter en priorité avant tout correctif de brouillard/effets.
- **Ce qui s'est passé** : le commit `back/Chess-API` `79a258c "update back"`
  (20/08, le tout dernier commit du sous-module) a supprimé le paramètre
  `spectate` et toute la logique d'autorisation spectateur du WS handler.
  Avant ce commit, `back/Chess-API/src/websocket/handler.rs` acceptait un
  spectateur si `is_registered_player(...) || is_tournament_game(...)` ;
  après, **une seule condition inconditionnelle** s'applique :
  ```rust
  // back/Chess-API/src/websocket/handler.rs:60-68 (actuel)
  if !is_registered_player(&state.redis_pool, &game_id, &user_id).await {
      return (StatusCode::FORBIDDEN, "You are not a player of this game").into_response();
  }
  ```
  `is_registered_player` ne contient que les 2 user_id enregistrés comme
  joueurs par Room-API (`chess:game_players:{game_id}`, voir bug #35) — un
  spectateur n'y figure jamais. Confirmé par recherche exhaustive : **le mot
  « spectat » n'apparaît plus nulle part dans `back/Chess-API/src`** (ni
  dans `back/Room-API/src`, ni ailleurs dans `back/`) alors que le front
  s'attend encore à ce mode :
  - `front/src/features/tournament/hooks/useSpectateGame.ts:139` construit
    toujours `.../api/chess/chess?game_id=...&spectate=true`, mais ce
    paramètre n'est lu par **aucun** code serveur désormais — il est mort.
  - `front/src/pages/Games/SpectateGame.tsx` et `useSpectateGame.ts`
    n'existent plus servis : la connexion échoue en HTTP 403 au moment du
    upgrade WS, donc `connected` reste `false` indéfiniment côté client —
    l'utilisateur voit « Connecting to game server... » sans jamais
    progresser (pas d'erreur explicite affichée non plus, `error` state
    n'est mis à jour que sur des erreurs applicatives distinctes du refus
    HTTP).
  - `back/websocket/lobby.rs` : `LobbyState` n'a plus que
    `players: [Option<PlayerSlot>; 2]` — la notion même de spectateur a
    disparu de la structure de données, pas seulement de l'autorisation.
- **Anciennes observations (toujours vraies pour référence historique,
  décrivaient un mode qui fonctionnait encore alors)** :
  `board/mod.rs:39-70` (`to_json_for_color(None)` → pas de filtrage
  brouillard) ; `SpectateGame.tsx` codait en dur `fogRemaining={0}`. Ces
  points redeviendront pertinents **seulement si** le mode spectateur est
  restauré.
- **Hypothèse falsifiable** : toute tentative de spectate (bouton
  « regarder » depuis un tournoi ou une room publique) échoue désormais
  systématiquement avec un refus de connexion, indépendamment du brouillard.
- **Confiance** : confirmé (lecture directe du commit et du code résultant).
- **Pour confirmer** : ouvrir les devtools réseau lors d'un clic « regarder »
  — la requête d'upgrade WS doit répondre 403.
- **Effort** : M — restaurer une autorisation spectateur explicite (jeton
  requis + vérification que la game existe/est publique ou liée à un
  tournoi, PAS d'accès anonyme comme avant), potentiellement via une
  décision d'architecture (ADR-002, annexe spectateur : accès en lecture
  seule et traitement du brouillard) plutôt qu'un simple retour en arrière.

## 19. Défilement entre cartes impossible (desktop)

- **Fichiers** (fichier CSS revérifié, désormais 36 lignes) :
  - `front/src/features/games/ChessGame/components/styles/Cards.module.css:1-21`
    — `.cards` : `display:flex; justify-content:center` **toujours sans
    `overflow-x`**
  - `Cards.tsx` — le carrousel utilise `onPointerDown/Move/Up` pour un
    drag-to-scroll tactile/souris (confirmé présent), mais toujours aucune
    molette (`onWheel`) ni flèches pour un scroll clavier/desktop pur
  - `front/src/assets/styles/globals.css` — scrollbars masquées globalement
    (même un scroll fonctionnel serait invisible)
- **Confiance** : confirmé (mécanisme toujours présent), avec une nuance :
  un drag-to-scroll à la souris existe désormais dans `CardsModal.tsx`
  (`onPointerDown` etc.) — à re-tester avant de considérer le clic-glissé
  impossible ; le signalement portait spécifiquement sur l'absence de
  molette/clavier, qui reste valide.
- **Effort** : S (overflow-x + wheel handler ou flèches + scrollbar visible).

## 20. Descriptions sous les cartes : pas de défilement — décision : supprimer

- **Fichiers** (revérifié — toujours non corrigé) :
  - `front/src/features/games/ChessGame/components/Card.tsx:53-57`
    (`.cardDescription`, affiché uniquement quand `zoom` est vrai)
  - styles : `front/src/features/games/ChessGame/components/styles/Card.module.css:44-47,164-173`
    — toujours aucun `overflow`/`max-height` sur `.cardDescription`
  - Le bloc dupliqué `CardsModal.tsx` (`selectedInfo`/`selectedDescription`
    cité dans l'audit initial) a disparu de `CardsModal.tsx` (le composant a
    été réécrit), mais **la description sur la carte zoomée elle-même
    (`Card.tsx`) est toujours là**, donc le problème persiste sous une forme
    légèrement différente (une seule occurrence désormais, pas deux).
- **Décision déjà actée, toujours pas appliquée** : supprimer le sous-texte
  → retirer le bloc `.cardDescription` de `Card.tsx:53-57` (et la prop
  `description` si elle devient inutile) + nettoyer le CSS associé.
- **Confiance** : confirmé.
- **Effort** : S.

## 21. Messages privés/amis : ni scroll ni limite de taille

- **Fichiers/mécanisme mis à jour (le composant a changé depuis la rédaction
  initiale)** :
  - `front/src/features/home/components/HomeLayoutDesktop.tsx:69-89` — la
    structure a changé : le slide entier est maintenant
    `<div className="h-full w-full overflow-y-auto">` (l.75) et englobe à la
    fois le contenu de page et `<div className="mt-20 w-[380px]
    shrink-0"><FriendPanel /></div>` (l.83-85). L'`overflow-y-auto` est donc
    posé sur le **conteneur composite**, pas sur la colonne du panneau ami
    elle-même, qui reste sans hauteur propre bornée.
  - `front/src/features/play/components/friends/FriendChat.tsx` — le
    `overflow-y-auto` interne existe toujours pour la zone de messages.
  - **Conséquence probable, différente de l'audit initial** : plutôt que
    « toute la page s'étire sans aucun scroll », c'est maintenant plus
    probablement « toute la colonne (page + panneau ami) scrolle comme un
    seul bloc » au lieu que seule la liste de messages défile à l'intérieur
    de sa propre zone — un défaut voisin mais pas identique, à confirmer
    visuellement (non exécuté ici).
- **Confiance** : probable (mécanisme déplacé, pas revérifié à l'écran).
- **Effort** : S (donner sa propre hauteur bornée + `overflow-y-auto` à la
  colonne `w-[380px]` plutôt qu'au slide entier).

---

# Interface, réglages et traduction

## 22. Bouton de déconnexion invisible hors survol

- **Mise à jour (re-vérification du 20/08) : probablement corrigé — le
  composant a été entièrement réécrit depuis la rédaction initiale.**
- **Fichiers** :
  - `front/src/features/settings/components/LogoutButton.tsx:1-56` — le
    bouton n'utilise plus d'override Tailwind `bg-[#06b6d4]!` ; il délègue à
    `<ThemeButton tone="red">`
    (`front/src/features/play/components/ThemeButton.tsx`), qui pose un
    fond dégradé **statique et opaque**
    (`bg-linear-to-br from-red-900 via-red-950 to-slate-950`, l.32-34) avec
    seulement `hover:brightness-95` en survol — le bouton est donc visible
    au repos par construction, plus de dépendance à `:hover` pour exister
    visuellement.
  - `front/src/components/ui/Button/Button.tsx` n'est plus utilisé par ce
    composant (le variant `secondary` cité dans l'audit initial n'est plus
    dans le chemin de LogoutButton).
- **Hypothèse** : le correctif (adoption de `ThemeButton`) élimine le
  mécanisme d'invisibilité décrit initialement (build périmé sans
  `!important`, ou contraste au repos insuffisant) — les deux causes
  possibles disparaissent avec la réécriture.
- **Confiance** : probable-résolu — non revérifié à l'écran (aucune
  exécution du stack dans cet audit), mais le code ne contient plus aucun
  des mécanismes d'invisibilité identifiés.
- **Effort** : nul si confirmé visuellement ; sinon S.

## 23. Les CGU ne suivent pas la langue du drapeau

- **Mise à jour majeure (re-vérification du 20/08) : très largement corrigé
  depuis la rédaction initiale.**
- **Fichiers** :
  - `front/src/pages/Legal/TermsOfServicePage.tsx:1-53` — **entièrement
    réécrite**, 100 % pilotée par `t("legal.terms...")`, plus aucun texte en
    dur (idem `PrivacyPolicyPage.tsx`, 20 appels `t(...)`).
  - `front/src/i18n/locales/{de,en,es,fr,it,sr}/translation.json` — les 6
    locales contiennent désormais la clé `legal.terms` (confirmé par
    recherche dans les 6 fichiers).
  - `front/src/features/settings/components/LegalSection.tsx:7-18` — libellés
    déjà en `t("settings.legal")`, `t("legal.privacy.title")`,
    `t("legal.terms.title")` : conforme.
  - **Reste non traduit** : `front/src/components/Footer.tsx:30` — le lien de
    la page de connexion affiche toujours littéralement « Conditions
    d'utilisation » en dur, sans `t()`. C'est le seul point résiduel de ce
    signalement.
- **Hypothèse** : la page elle-même et les réglages sont désormais
  internationalisés ; seul le lien du pied de page de connexion ne suit pas
  la langue.
- **Confiance** : confirmé (contenu principal résolu, résidu Footer confirmé
  par lecture).
- **Effort** : S (ajouter une clé i18n pour le libellé du lien Footer —
  le gros du travail, initialement estimé M, est déjà fait).

## 24. Étiquettes « liens sociaux » trompeuses (pseudo attendu, pas URL)

- **Mise à jour (re-vérification du 20/08) : corrigé.**
- **Fichiers** :
  - `front/src/features/settings/components/SocialLinksSection.tsx:54-57` —
    un texte d'aide explicite a été ajouté juste au-dessus des champs :
    `{t("settings.socialsHint")}`, dont la valeur (`fr`) est *« Entrez votre
    pseudo uniquement, pas d'URL complète (ex. johndoe, pas
    https://github.com/johndoe) »* — exactement la clarification que
    l'audit initial recommandait.
  - `front/src/pages/Profile/PlayerProfilePage.tsx:81,113` — la concaténation
    `https://github.com/${profile.github}` / `https://x.com/${profile.twitter}`
    est inchangée ; toujours cassée si un utilisateur colle une URL malgré
    la nouvelle indication (pas de `normalize` défensif côté affichage).
- **Hypothèse** : le risque de confusion est désormais couvert côté
  formulaire (le libellé/l'aide dit explicitement « pseudo, pas URL ») ;
  il reste un défaut de robustesse mineur si l'utilisateur ignore l'aide.
- **Confiance** : confirmé.
- **Effort** : S restant (normaliser l'entrée pour retirer un domaine/`@`
  collé par erreur — non bloquant, la confusion d'origine est réglée).

---

# Erreurs serveur et protocole

## 25. Erreur back brute et non traduite (ami inexistant)

- **Fichiers** (revérifié, toujours vrai) :
  - `back/Social-API/src/http/handlers/send_friend_request.rs:50` —
    `json_error(404, "User not found")`
  - `front/src/features/friends/services/friendService.ts:42-49`
    (`fetchJson`) — le corps de réponse **brut** (texte de la réponse ou
    `HTTP {status}`) est jeté dans `Error.message` sans parse JSON ni
    mapping i18n
  - `front/src/features/play/components/FriendPanel.tsx` — rendu brut
    toujours présent
- **Hypothèse** : confirmé, mécanisme inchangé.
- **Effort** : S (parser `.error` + mapper vers clés i18n existantes).

## 26. 409 « déjà dans une salle » fantôme

- **Revérifié sur le code synchronisé du 20/08** : le mécanisme de fond est
  inchangé (`ws_handler.rs`/`socket_manager.rs` toujours des stubs vides).
  Un ajout notable depuis la rédaction initiale : Room-API enregistre
  désormais `chess:game_players:{game_id}` au démarrage d'une partie
  (`back/Room-API/src/services/room.rs`, `src/cache/room.rs`,
  `src/services/tournament.rs` — voir bug #35) ; ceci ne touche pas
  directement le cycle de vie de `user:session:{id}` décrit ici, donc
  l'analyse reste valable.
- **Fichiers** (voir rapport complet en annexe de ce dossier) :
  - `back/Room-API/src/http/handlers/join_room.rs:68` — 409 si
    `user:session:{id}.status != "none"`
  - Écritures sans cleanup : `join_room.rs:83-90` (`waiting`),
    `create_room.rs:102-111`, `play_ranked.rs:71-81` (`matchmaking`) ;
    TTL 7200 s réarmé (`back/Room-API/src/user_state.rs:33-50`)
  - **Aucune détection de déconnexion** : `back/Room-API/src/ws/ws_handler.rs`
    et `ws/socket_manager.rs` sont vides ; le SSE Notification ne remonte pas
    de présence ; le WS chess garde le slot « for reconnection »
    (`Chess-API/src/websocket/lobby.rs:148-157`)
  - Front : `front/src/pages/Room/RoomLobbyPage.tsx:93-103` — `leaveRoom()`
    uniquement au clic bouton ; aucun `beforeunload` (grep : 0 dans front/src) ;
    `PublicRooms.tsx` ne désactive pas « Rejoindre » si session active
  - Piège aggravant : si la room a expiré, `leave_room` échoue en 500
    **avant** la réinitialisation (`services/room.rs:317-319` +
    `leave_room.rs:59-65`) → session libérable par personne
- **Sur le « sans corps »** : Room-API renvoie toujours un JSON
  (`{"status":409,"error":"Already in a room or matchmaking"}`,
  `src/http/response.rs:4-9`) que la gateway recopie — le « 409 sans corps »
  est probablement un artefact d'observation (devtools/ requête annulée).
- **Confiance** : confirmé (mécanique complète) ; à investiguer (perception
  « sans corps »).
- **Effort** : L (ADR-003 : source de vérité de présence + libération
  automatique).

## 27. Requêtes de refresh mal formées

- **Mise à jour majeure (re-vérification du 20/08) : corrigé — plus aucune
  discordance dans le code actuel.**
- **Fichiers** :
  - `back/Auth-API/src/http/router.rs:75` — route toujours **GET
    uniquement** (`get(refresh_token_handler)`).
  - `front/src/features/auth/services/authService.ts:87-97` (méthode
    `refresh()`) — **GET**, conforme.
  - `front/public/refresh-worker.js:53-56` — **GET**, conforme, et **sans**
    le `Content-Type: application/json` cosmétique signalé initialement (il
    a été retiré).
  - `front/src/api/api.ts` — le fichier a été réduit à 4 constantes de
    chemin (`API_AUTH`, `API_USER`, `API_SOCIAL`, `API_NOTIFICATIONS`) : le
    code POST fautif cité dans l'audit initial (`api.ts:6-9`) n'existe plus.
  - `authService.refreshToken()` (la seconde méthode POST citée) n'existe
    plus dans `authService.ts` — recherche exhaustive de `refreshToken`/
    `/refresh` dans `front/src` : aucune occurrence restante hors
    `refresh()`/`refresh-worker.js`.
- **Confiance** : confirmé — le front a été nettoyé, une seule voie GET
  cohérente subsiste de bout en bout.
- **Effort** : nul (déjà fait).

## 28. Connexions websocket « expirent » (console)

- **Fichiers** (remappés — `game.rs` a été éclaté, voir note en tête de
  section « Cause commune ») :
  - `back/Chess-API/src/game/session.rs:199-221` (déplacé depuis
    `game.rs:388-407`) — heartbeat applicatif inchangé : ping texte toutes
    les 15 s (`interval`), coupure à 30 s sans pong (`is_pong`, l.22-28) ;
    le pong n'est envoyé **que par la page montée**
    (`front/src/features/games/ChessGame/hooks/useChessGame.ts:438-439`) :
    onglet en arrière-plan prolongé, navigation hors page avec socket encore
    ouvert → expiration
  - `infra/Nginx/nginx.conf:57-59` — `proxy_read_timeout 300s` : toute WS
    sans trafic 5 min (ex. SSE silencieux) est coupée par nginx
- **Hypothèse** : le serveur « ne gère pas » les connexions inactives au
  niveau protocole (pas de ping WS natif ni en gateway
  `proxy_websocket.rs` ni en Chess-API) ; la charge repose sur l'applicatif,
  qui ne tourne que page ouverte.
- **Confiance** : probable (à confirmer par l'endpoint exact des messages
  console — voir INCONNUES).
- **Effort** : S (répondre au ping dans le worker lui-même, pas dans la page).

## 29. Déconnexion en GET

- **Mise à jour majeure (re-vérification du 20/08) : corrigé — la route et
  tous les appelants sont désormais uniformément en POST.**
- **Fichiers** :
  - `back/Auth-API/src/http/router.rs:76` — **`post(logout_handler)`**
    (n'est plus `get`).
  - `front/src/features/settings/components/LogoutButton.tsx:21-26` —
    `fetch(..., { method: "POST" })`.
  - `front/src/features/auth/services/authService.ts:143-150`
    (`logout()`) — **POST**, cohérent avec le back.
  - Recherche exhaustive : aucune occurrence restante d'un appel GET vers
    `/logout` dans `front/src`.
- **Confiance** : confirmé — le mécanisme CSRF-fragile décrit initialement
  (déconnexion déclenchable par une simple navigation/`<img>` GET) n'existe
  plus ; POST + cookies restent à sécuriser par `SameSite` (cf. ADR-004,
  toujours pertinent pour le reste de la surface CSRF).
- **Effort** : nul (déjà fait) ; ADR-004 reste utile pour `SameSite`/CSRF au
  sens large.

## 30. Saturation serveur via allers-retours inscription

- **Mise à jour (re-vérification du 20/08) : partiellement corrigé, avec une
  limite importante qui laisse le scénario d'origine largement ouvert.**
- **Fichiers** :
  - Coûteux par cycle : `back/Auth-API/src/http/handlers/register.rs`
    (bcrypt + 2 INSERT + 2 JWT) et `send_validation_email_code.rs` (code
    Redis + e-mail Resend) — inchangé.
  - **Nouveau** : `back/Gateway-API` a reçu le commit `feat: enforce rate
    limit on auth service` (`206b727`, fusionné le 19/08) — la branche
    `service == "auth"` de `back/Gateway-API/src/http/handlers/router.rs:142-153`
    appelle désormais `crate::http::rate_limit::enforce_access(...)` comme
    les autres services, ce qui contredit directement le constat initial
    (« la gateway n'applique `enforce_access` qu'à user/chess/room/social/
    permission — pas auth »).
  - **Mais** : `back/Gateway-API/src/http/rate_limit.rs:17-23` — `enforce_access`
    commence par extraire `user_id` du cookie `access_token` ; **si aucun
    cookie valide n'est présent, la fonction retourne `Ok(())` sans limiter
    ni journaliser quoi que ce soit** (`let Some(user_id) = ... else { return
    Ok(()) };`). Le rate-limit ajouté est donc **par utilisateur déjà
    authentifié**, pas par IP/anonyme. Or `POST /api/auth/register` est par
    construction le tout premier appel, sans cookie — il reste **totalement
    non limité** par ce nouveau mécanisme. `send_validation_email_code`,
    elle, est appelée *après* que `register` ait posé le cookie
    `access_token` (`register.rs` : `Set-Cookie: access_token=...`), donc
    **cette route-là est bien couverte** par le nouveau garde-fou.
  - La middleware interne `back/Auth-API/src/http/rate_limit.rs`
    (`rate_limit_middleware`, par IP) existe toujours mais **n'est
    toujours pas branchée** — confirmé à nouveau : aucun `.layer(...)` sur
    le routeur `create_auth_router` (`back/Auth-API/src/http/router.rs`),
    seuls `TraceLayer`/`CompressionLayer` sont posés dans `main.rs:101-102`.
  - Le bouton retour (`ValidateEmailPage.tsx`) : `handleBackToLogin` appelle
    toujours `logout()` (désormais en **POST cohérent**, voir bug #29 —
    n'échoue donc plus en 405) puis navigue ; le compte non validé reste
    (pas de purge).
- **Hypothèse** : le scénario précis décrit (aller-retour avec le bouton
  retour) déclenche un nouveau `register` à chaque cycle si l'utilisateur
  ressaisit un e-mail — ce chemin **anonyme** reste non protégé par le
  nouveau rate-limit gateway (qui exige un cookie), donc la saturation par
  `register` répétés reste plausible. Le sous-cas où l'abus se ferait via
  des rappels répétés de `send_validation_email_code` sur un **même**
  compte déjà enregistré est, lui, désormais limité par utilisateur.
- **Confiance** : confirmé (mécanisme de contournement lu précisément :
  `enforce_access` no-op sans cookie).
- **Effort** : S — brancher la middleware IP existante d'Auth-API
  (`rate_limit_middleware`) suffirait à couvrir le trou anonyme sans
  nouveau développement ; alternative : limiter aussi par IP dans
  `enforce_access` côté gateway quand `user_id` est absent.

---

# Sécurité et exposition

> **Cause commune identifiée entre #30 et #36 (ajout du 20/08)** : les deux
> bugs de cette section qui concernent une absence de contrôle d'accès
> applicatif partagent la même racine — le modèle d'autorisation
> (`Gateway-API::enforce_access` + table `permission_routes`) est
> **fail-open** : il ne bloque que ce qu'on lui a explicitement dit de
> bloquer (un `user_id` présent *et* une permission liée), et laisse
> passer par défaut dans tous les autres cas (pas de cookie → laissé
> passer ; route non liée à une permission → laissée passer). Voir
> **ADR-006** pour l'analyse et la décision recommandée.

## 31. Services internes exposés publiquement

- **Fichier** : `docker-compose.yml` (+ les deux compose de metrics inclus) —
  tous les `ports:` publient sur `0.0.0.0` par défaut :
  - postgres l.16-17 (5432), redis l.34-35 (6380), adminer l.53-54 (8081),
    minio l.374-376 (9000-9001), WS chess l.177-178/210-211 (8082-8083),
    metrics par service (9101-9106), scalar l.426-427 (5050)
  - ELK `infra/metrics/elk/docker-compose.yml:54-55` (kibana 5601),
    monitoring `infra/metrics/monitoring/docker-compose.yml:92-93` (grafana 3000)
- **Hypothèse** : confirmé — bindings d'express dev jamais restreints.
- **Effort** : M (préfixer `127.0.0.1:` ou retirer les `ports` (réseau interne
  suffit), sauf nginx — cf. ADR-004).

## 32. En-têtes de sécurité absents

- **Fichier** : `infra/Nginx/nginx.conf` (tout le `server` l.43-128) — aucun
  `add_header` : pas de HSTS, `X-Content-Type-Options`, `X-Frame-Options`,
  CSP. Confirmé (cohérent avec le `curl -I` du ticket).
- **Effort** : S (bloc `add_header` + politique CSP à définir — ADR-004).

## 33. TLS 1.0/1.1 acceptés

- **Constat dépôt** : aucun TLS dans le dépôt — nginx écoute en clair sur
  :8000 (`nginx.conf:44`). Le domaine réel (`.env` : `DOMAIN_NAME=chicken-exe.com`)
  est terminé ailleurs (hébergeur/proxy — un tunnel Cloudflare a existé,
  docker-compose.yml:414-421). Le réglage min-TLS est donc **hors dépôt**.
- **Confiance** : à investiguer (côté hébergement/Cloudflare).
- **Pour confirmer** : `openssl s_client -connect chicken-exe.com:443 -tls1`
  et vérifier la configuration « Minimum TLS Version » du compte edge.
- **Effort** : S (un réglage côté edge) + S pour la trace ADR-004.

## 34. Deux secrets OAuth dans l'historique git

- **Fichiers/commits** : le `.env.example` historique contient des valeurs
  réelles : commit `b0cc0c7` (GOOGLE `GOCSPX…`, 42 `s-s4t2…`), également
  touchés `5e2030e`, `761d1c2`, `57d7810`, `63e20a6`, `da983bd`
  (`git log --all -S GOOGLE_CLIENT_SECRET` / `-S FT_CLIENT_SECRET`).
  Bonus non signalé : un **token de tunnel Cloudflare** commité (commenté)
  dans docker-compose.yml:414-421.
- **Hypothèse** : confirmé — les secrets restent récupérables par clone.
- **Effort** : M (révocation/rotation chez Google/42 + purge historique
  (`git filter-repo`) + force-push + coordination équipe — ADR-005).

## 35. Audit de la connexion websocket (interception/rejeu)

- **Mise à jour majeure (re-vérification du 20/08) : le trou de sécurité
  décrit ci-dessous a été comblé depuis la rédaction initiale — exactement
  selon la recommandation de l'option 2 de l'ADR-003.** C'est l'un des
  correctifs les plus significatifs identifiés pendant cette relecture.
- **Ce qui a changé** :
  - `back/Chess-API/src/websocket/handler.rs:22-67` (actuel) — le jeton est
    désormais **obligatoire** : `None => return 401 "Missing access token"`
    (l.47-54), et validé côté Chess-API lui-même via
    `jwt_manager().validate_access_token(&token)` (l.37-44), pas seulement
    transféré par la gateway.
  - **Nouveau garde-fou d'appartenance** :
    `is_registered_player(&state.redis_pool, &game_id, &user_id)`
    (`handler.rs:60-68,95-121`) — si le `claims.sub` de l'utilisateur ne
    figure pas dans la clé Redis `chess:game_players:{game_id}`, la
    connexion est rejetée en **403 Forbidden** avant même l'upgrade WS.
  - Cette clé Redis est désormais écrite par **Room-API** au moment où il
    crée la partie (commit `5a190ef "feat: register game players in redis
    for slot verification"`, 19/08) — trois points d'écriture :
    `back/Room-API/src/cache/room.rs` (matchmaking classé),
    `src/services/room.rs` (room privée/publique), `src/services/tournament.rs`
    (bracket de tournoi), tous via `chess_client::set_game_players(pool,
    game_id, &[player1, player2])`
    (`back/Room-API/src/services/chess_client.rs`, `SETEX
    chess:game_players:{game_id} 7200 [...]`). Le commentaire du code est
    explicite : *« Ce mapping est vérifié par le Chess-API au moment de la
    connexion websocket pour empêcher l'usurpation de slot. »*
  - `back/Gateway-API/src/http/handlers/router.rs:64-116` — la branche WS de
    la gateway ne valide toujours rien elle-même (transfert simple du
    cookie), mais ce n'est plus un problème puisque Chess-API fait
    maintenant sa propre validation en aval.
- **Conséquence** : l'usurpation de slot par simple connaissance du
  `game_id` (le scénario principal de cette entrée) n'est plus possible —
  un attaquant qui devine/obtient un `game_id` sans être l'un des deux
  `user_id` enregistrés reçoit un 403. La question de la devinabilité des
  `game_id` (INCONNUES #8) devient donc largement sans objet pour ce risque
  précis.
- **Effet de bord non désiré** : ce même changement a **supprimé le mode
  spectateur** (aucun `user_id` de spectateur n'est jamais dans
  `chess:game_players`) — voir la mise à jour de bug #18, qui est
  désormais le principal problème lié au WS chess.
- **Confiance** : confirmé (lecture directe du handler + du commit
  Room-API correspondant).
- **Effort** : nul pour le risque d'usurpation (déjà traité) ; M pour
  restaurer un accès spectateur légitime sans réintroduire de trou (#18).

## 36. Endpoint d'upload du shop entièrement ouvert — aucune authentification, aucune permission

- **Contexte** : une note antérieure signalait que « n'importe quel
  utilisateur connecté peut uploader des items de catalogue, sans filtrage
  sur `users.role` ». La relecture du code confirme le problème et montre
  qu'il est **plus grave que ce qui était supposé** : ce n'est pas une
  question de rôle insuffisamment vérifié, c'est une **absence totale de
  vérification d'identité**, y compris pour un visiteur non connecté.
- **Fichiers** :
  - `back/User-API/src/http/router.rs:21-26` — la route est déclarée
    `router.register_public("POST", "shop/items", |ctx, req| { ...
    shop::handle_upload_item(&ctx, &req).await })`.
  - `back/User-API/src/http/handlers/shop.rs:150-257`
    (`handle_upload_item`) — **ne lit ni ne valide `request.cookies` à
    aucun moment** : contrairement à `handle_get_shop`
    (`authed_user_id`, l.297-302, utilisé en optionnel l.47) ou à
    `handle_purchase_collection` (l.88-91, `authed_user_id` obligatoire),
    `handle_upload_item` ne contient **aucun appel** à
    `validate_and_get_claims`/`authed_user_id`. N'importe quel appelant,
    identifié ou non, atteint directement la logique métier.
  - `back/Gateway-API/src/http/rate_limit.rs:17-23` — `enforce_access`
    retourne `Ok(())` (laisse passer) dès qu'aucun cookie `access_token`
    valide n'est présent, **avant** même la vérification de permission
    (`enforce_permission`, l.55-99) : un appel anonyme ne déclenche donc ni
    le rate-limit ni le contrôle de permission au niveau gateway.
  - `back/User-API/src/db/migrations.rs:420` — la route
    `('POST', '/api/user/shop/items', 'Upload item boutique')` est bien
    enregistrée dans la table `api_routes` (utilisée pour le rate-limit et
    les permissions par route), **mais** la migration `044_link_default_
    permission_routes` (l.623-693) qui relie les permissions aux routes ne
    référence **que** les chemins `/api/user/admin/%` — jamais
    `shop/items`. Aucune permission n'est donc jamais requise pour cette
    route, même pour un utilisateur authentifié sans rôle admin.
  - `back/User-API/src/db/shop.rs:62-89` (`upsert_item`) — `INSERT ...
    ON CONFLICT (item_id, item_type) DO UPDATE SET title=…, price=…,
    asset_key=…` : l'opération n'est pas seulement une création, c'est une
    **réécriture** d'un item existant (y compris les cartes/cosmétiques du
    catalogue par défaut semées en migration, ex. `('1','card','Top
    chrono',...)`).
- **Hypothèse falsifiable, confirmée par lecture directe (pas d'exécution)** :
  une requête `POST /api/user/shop/items` avec un corps JSON valide
  (`item_id`, `item_type` ∈ `{base,hat,mask,clothes,accessory}`, `title`,
  `price`, `image_base64` ≤ 2 Mo) et **sans aucun cookie** aboutit à un
  `200 { "message": "Item uploaded", ... }` : l'image atterrit dans le
  bucket MinIO public (`storage.put_object`, `shop.rs:194`) et la ligne
  `shop_catalog` est créée ou **écrasée** en base. Impact : (a) hébergement
  gratuit d'images arbitraires (≤2 Mo, extension dérivée du
  `content_type` déclaré) sur l'infrastructure du projet ; (b) défacement du
  catalogue existant (changer le titre/prix/image d'un item déjà acheté par
  des joueurs, `item_id`/`item_type` réutilisés) ; (c) création d'items à
  prix nul ou arbitraire.
- **Confiance** : confirmé (lecture complète de la chaîne routeur → handler
  → permissions gateway → migration de permissions → upsert SQL ; aucune
  étape n'exige de token, de cookie ou de permission).
- **Pour confirmer en environnement réel** : `curl -X POST
  http://<host>/api/user/shop/items -H 'Content-Type: application/json' -d
  '{"item_id":"1","item_type":"card","title":"pwned","price":0,
  "image_base64":"..."}'` sans en-tête `Cookie` — un `200` avec
  `"message":"Item uploaded"` confirme l'absence totale de contrôle
  d'accès.
- **Effort** : S — ajouter dans `handle_upload_item` le même garde
  `authed_user_id`/`validate_and_get_claims` que les autres handlers du
  fichier, **et** lier la route à une permission dédiée (ex. `shop.manage`
  ou réutiliser `panel.access`) via une nouvelle migration
  `permission_routes`, pour couvrir aussi bien l'absence de cookie que le
  cas « connecté mais pas admin » évoqué par la note d'origine.

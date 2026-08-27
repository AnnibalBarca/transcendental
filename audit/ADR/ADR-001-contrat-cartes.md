# ADR-001 — Contrat de jouabilité des cartes et protocole de ciblage

> **Mise à jour (re-vérification du 20/08)** : la partie front de l'option 2
> recommandée est désormais en place — `CardRegister.playable` existe côté
> client et conditionne l'affichage du bouton « Jouer »
> (`front/src/features/games/ChessGame/components/CardsModal.tsx:110,182-206`).
> Le serveur, lui, n'a **pas** été modifié : `play_card` renvoie toujours
> `card_no_effect` en silence au lieu de `Err("err_not_playable")`
> (`back/Chess-API/src/game/game_loop/core.rs:360-363`), et `is_card_playable`
> ne teste toujours pas `has_moved` — ce qui laisse les cartes Vétéran
> exposées au même problème malgré le garde-fou front (voir `BUGS.md`,
> section « Cause commune »). Le point « Bastion/Magnétisme : redéfinir la
> sémantique » ci-dessous est devenu sans objet : ces deux cartes ont été
> retirées du jeu (code commenté + `shop_catalog.is_active=FALSE` +
> purgées des decks, migration `054_disable_magnetisme_bastion`) plutôt que
> corrigées — voir bug #3. Le reste de la décision recommandée (rejet
> serveur explicite, protocole de ciblage à deux étapes pour les échanges,
> exposition des captures Traître) reste entièrement à faire.

## Contexte

La moitié des bugs de jeu signalés (Traître, Pyromane, Bastion, Maçon,
Sniper, Percée, cartes d'échange, échecs ignorés) partagent une racine : le
serveur trache une carte non jouable comme un succès vide
(`Chess-API/src/game/game_loop/core.rs:286-290` — `card_no_effect`), retire la
carte de la main, et fait passer le tour si `ends_turn`. Le front, de son
côté, ignore le champ `playable` que le serveur calcule et envoie dans chaque
`hand` (`cards/types.rs:127-139`) : deux vérités divergentes. Enfin, certaines
cartes exigent deux pièces (échanges) ou une pièce adverse (Traître) alors que
le protocole ne connaît qu'une cible unique `Option<String>` et que le JSON
plateau vide les move_sets ennemis (`board/mod.rs:58-70`).

S'y ajoutent deux absences de validation : aucune vérification d'échec avant
de jouer (`core.rs:256-323`) ni après l'effet (auto-échec possible), et des
`ends_turn` incohérents entre cartes similaires (Percée/Frog/Canon terminent
le tour, Vétéran/Ninja non), source de l' sensation « inconstante ».

## Options envisagées

1. **Correctifs ponctuels par carte** : traiter chaque ticket séparément
   (griser Sniper côté front, choix aléatoire éclairci pour les échanges…).
   - Avantages : rapide, aucun changement de protocole.
   - Inconvénients : la validation reste dupliquée front/back et divergera de
     nouveau ; le bug « tour sauté silencieux » reste possible pour toute
     future carte.
2. **Serveur strict + front consommateur** (recommandé) :
   - `play_card` renvoie `Err("err_not_playable")` quand `is_card_playable`
     est faux — plus jamais de succès vide ;
   - le front se contente du champ `playable` (déjà envoyé) pour griser
     les cartes et affiche toute erreur `card_result` (toast) ;
   - garde d'échec : refuser `play_card` si le joueur est en échec et que
     l'effet ne le lève pas ; valider l'absence d'auto-échec après l'effet
     (rejeter et rollback, ou assumer la règle par décision de game design).
3. **Moteur de règles déclaratif complet** : tout (jouabilité, cibles,
   légalité, fin de tour) décrit dans `CardDef` et évalué par un validateur
   unique, front inclus via génération d'un contrat partagé.
   - Avantages : supprime la divergence à long terme.
   - Inconvénients : surdimensionné pour l'état du projet.

## Décision recommandée

L'option **2**, complétée par une extension du protocole de ciblage :

- `play_card` accepte `targets: [String]` (1 ou 2 cases selon la carte) ;
- les cartes d'échange exigent la sélection **d'une pièce de chaque type** et
  le serveur vérifie les types réels (rejet `err_wrong_piece`) ;
- pour Traître, le JSON plateau expose les destinations des captures traître
  sur les pièces ennemies concernées (le move_set « vidé » n'est pas touché
  pour les autres) afin que l'UI puisse offrir la sélection ;
- statuer et documenter `ends_turn` pour chaque carte dans un tableau lu par
  les deux parties (aujourd'hui `registry.rs`) — le front doit pouvoir
  prédire si le tour passera (message « votre tour va se terminer ») ;
- pour Bastion/Magnétisme : redéfinir la sémantique (réaction au coup adverse
  ou pièce alliée désignée) avant tout codage — l'actuelle condition
  « dernier coup = mien » est insatisfaisable pendant mon propre tour.

## Conséquences

- Le front dépend entièrement du serveur pour la jouabilité : cohérence
  garantie, mais toute carte non prévue doit enrichir `is_card_playable`.
- Les erreurs deviennent visibles (rejets explicites) : le bug « le tour
  saute » devient un refus affiché, ce qui change le ressenti joueur même sans
  changement de règles.
- L'extension `targets` casse le format `play_card` : prévoir une version
  (champ `v`) ou une période de double lecture, y compris pour le replay du
  SharedWorker qui met des messages en cache (il ne met pas `play_card` en
  cache, donc sans risque de ce côté).
- Effort estimé : M-L (serveur S/M, protocole+UI M, Traître M).

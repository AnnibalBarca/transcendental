# ADR — index

| # | Titre | Statut |
|---|---|---|
| [001](ADR-001-contrat-cartes.md) | Contrat de jouabilité des cartes et protocole de ciblage | **partiellement implémentée** (garde-fou `playable` front fait, rejet serveur explicite et ciblage à deux étapes restent à faire) |
| [002](ADR-002-synchronisation-websocket.md) | Synchronisation d'état du jeu par websocket et horloges | **partiellement implémentée** (ordre de replay du worker corrigé ; horloge autoritaire et séquenceur restent à faire ; volet spectateur régressé, voir note du 20/08) |
| [003](ADR-003-cycle-de-vie-session.md) | Cycle de vie de la session joueur (presence, rooms, identité WS) | **partiellement implémentée** (identité WS faite, présence/auto-réparation restent à faire — voir note du 20/08 dans le document) |
| [004](ADR-004-exposition-reseau-tls.md) | Exposition réseau, TLS, en-têtes et CSRF | proposé |
| [005](ADR-005-secrets-historique.md) | Secrets : rotation et purge de l'historique git | proposé |
| [006](ADR-006-modele-autorisation-fail-open.md) | Modèle d'autorisation « fail-open » (gateway + permissions applicatives) | proposé (ajoutée le 20/08) |

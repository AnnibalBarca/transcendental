# Planning maître — 10 jours de révision ft_transcendence

Créé le 2026-08-23 à partir du §5 de `REVISION_PROMPT.md`. Jour 1 = aujourd'hui.
Les dates ci-dessous supposent 10 jours **consécutifs** ; si le rythme réel
dérive (jour sauté, sujet qui déborde), mets ce tableau à jour en fin de
session plutôt que d'en garder une version périmée — c'est le fichier relu en
premier au début de chaque « Jour N ».

## Cadre Pomodoro (identique chaque jour)

- **Bloc matin — 11h–13h (120 min)** : 4 pomodoros de 25 min + 4 pauses de
  5 min. Théorie dense (Cours 1, rappels de langage) : charge cognitive plus
  haute juste après l'ouverture de la session.
- **Bloc après-midi — 14h30–19h (270 min)** : 8 pomodoros de 25 min + 7 pauses
  de 5 min + 1 pause longue de 20 min après le 4e pomodoro (255 min), puis
  15 min en fin de bloc pour la fiche du soir. Pratique (reconstruction de
  code, examens).
- Fin de session obligatoire : mise à jour du fichier de chapitre en cours si
  inachevé, puis écriture de `fiches_du_soir/JOUR_N.md`.

## Répartition des 10 jours

| Jour | Date | Matin (théorie) | Après-midi (pratique) | Statut |
|---|---|---|---|---|
| 1 | 2026-08-23 | Cours 1 : superprojet, orchestration Docker, Gateway + Redis Streams | Exercices de lecture de code sur Gateway-API (pas de contribution almeekel, focus compréhension) | **en cours** |
| 2 | 2026-08-24 | Cours 1 : Auth-API (JWT/OAuth), Chess-API (moteur de jeu) | Lecture de code Chess-API/Auth-API + début des questions d'examen Cours 1 | à faire |
| 3 | 2026-08-25 | Cours 1 : Room/Social/Notification-API, front React (routing, i18n, state), nginx/observabilité en général | `09_examen_cours1.md` — passage complet, oral blanc sur l'architecture | à faire |
| 4 | 2026-08-26 | Chapitre 1 : rappel TypeScript/React + fiche mission Shop | Chapitre 1 : étapes data modules → cartes → grilles (commits du 2026-07-20) | à faire |
| 5 | 2026-08-27 | Chapitre 1 (suite) : intégration API, wallet, i18n | `chapitre_1_ts_shop/99_examen.md` | à faire |
| 6 | 2026-08-28 | Chapitre 2 : rappel Rust + fiche mission User-API/shop | Chapitre 2 : storage.rs (client S3 MinIO) → shop.rs → migrations → `99_examen.md` | à faire |
| 7 | 2026-08-29 | Chapitre 3 : rappel nginx/TLS + fiche mission + les 3 commits HTTPS | Chapitre 3 : `99_examen.md`, puis démarrage Chapitre 4 : rappel Tailwind v4 + fiche mission | à faire |
| 8 | 2026-08-30 | Chapitre 4 (suite) : conversion HomeLayout/play views | Chapitre 4 : conversion navbar/wallet/levels + `99_examen.md` | à faire |
| 9 | 2026-08-31 | Révision transversale : relecture croisée Cours 1 ↔ Cours 2 (où chaque contribution almeekel s'insère dans l'archi globale) | Oral blanc complet type soutenance (jury simulé), toutes les questions des 4 chapitres mélangées | à faire |
| 10 | 2026-09-01 | Reconstruction à blanc d'un fichier choisi au hasard par l'utilisateur lui-même (tirage), sans notes | `examen_final/ORAL_BLANC.md` + `examen_final/SYNTHESE.md` : bilan complet, ce qui reste fragile, plan de révision espacée post-J10 | à faire |

## Suivi rapide

- **Déjà écrit avant le Jour 1** : `revision/00_CONTRIBUTIONS.md` (audit git),
  `audit/ARCHITECTURE.md`, `audit/BUGS.md`, `audit/ADR/*.md`,
  `audit/INCONNUES.md` (audit technique).
- **Jour 1** : `cours1_architecture/01_superprojet_et_orchestration.md`,
  `cours1_architecture/02_gateway_et_redis_streams.md` (contient aussi les
  exercices de l'après-midi), `fiches_du_soir/JOUR_01.md`.

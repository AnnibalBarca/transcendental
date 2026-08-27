# Jour 1 — 2026-08-23

## Ce qui a été couvert aujourd'hui

- `revision/planning/PLANNING_10_JOURS.md` — calendrier maître créé à partir
  du §5 du harnais.
- `revision/cours1_architecture/01_superprojet_et_orchestration.md` —
  superprojet à sous-modules git (mécanisme des pointeurs, piège
  `submodule update`), orchestration Docker Compose (`depends_on` +
  `service_healthy`, volumes, `make up`/`make prod`), écart relevé :
  `front/front_DEV` déclaré dans `.gitmodules` mais absent du disque (12
  sous-modules actifs sur 13 déclarés).
- `revision/cours1_architecture/02_gateway_et_redis_streams.md` — dispatch
  HTTP direct (`auth`, `notifications`) vs Redis Streams (`user`, `room`,
  `social`, `chess`, `permission`), mécanisme de corrélation par UUID
  (`push_to_queue` → `XADD`, `ResponseListener` sur `gateway:responses`,
  `Router`/`DashMap`), découverte dynamique de l'instance Chess-API
  (`chess_discovery.rs`). Contient les 5 exercices de lecture de code de
  l'après-midi (§6 du fichier).

*(À compléter : coche/complète chaque point ci-dessus une fois le bloc
correspondant réellement fait, plutôt que de valider en bloc à l'avance.)*

## Les 5 idées à retenir

*(À remplir par toi, dans tes mots, après le bloc après-midi — pas par moi :
c'est le principe du harnais. Si tu veux qu'on le fasse ensemble maintenant
en repassant sur les 2 cours du jour, dis-le et on les formule à l'oral avant
de les écrire ici.)*

1.
2.
3.
4.
5.

## Points faibles identifiés pendant l'examen du jour

*(Le Jour 1 n'a pas d'examen formel — §6 de `02_gateway_et_redis_streams.md`
sert de test de compréhension. Note ici les questions des 5 exercices où tu
as hésité ou t'es trompé, une fois que tu les auras faites en relisant le
vrai code.)*

-

## À revoir demain matin en échauffement (10 min)

*(2-3 points précis tirés de la case précédente, pas une relecture
complète — à remplir après les exercices.)*

-

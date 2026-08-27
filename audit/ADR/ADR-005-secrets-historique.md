# ADR-005 — Secrets : rotation et purge de l'historique git

## Contexte

Vérifié : `GOOGLE_CLIENT_SECRET` (format `GOCSPX…`) et `FT_CLIENT_SECRET`
(format `s-s4t2…`) ont été commités en valeur réelle, notamment dans le
`.env.example` du commit `b0cc0c7`, et restent lisibles dans l'historique
(`git log --all -S GOOGLE_CLIENT_SECRET` : b0cc0c7, 5e2030e, 761d1c2, 57d7810,
63e20a6, da983bd…). S'y ajoute un token de tunnel Cloudflare commité
(commenté) dans `docker-compose.yml:414-421`. Le retrait ultérieur des
fichiers n'efface pas les commits antérieurs : quiconque clone récupère les
secrets.

## Options envisagées

1. **Ne rien faire côté historique, compter sur l'obscurité** : inacceptable —
   le dépôt est clonable par toute personne ayant eu accès (orga GitHub,
   forks éventuels).
2. **Rotation seule** (révoquer/régénérer les trois secrets) : les anciens
   identifiants deviennent inertes ; l'historique reste sale mais inoffensif.
   - Avantages : aucune rupture de git, 30 minutes de travail.
   - Inconvénients : la moindre réapparition d'une vieille valeur (stashes,
     backups, `.env.backup-1635` présent à la racine !) redevient un incident ;
     l'audit « rien d'autre n'a fuité » reste impossible proprement.
3. **Rotation + purge d'historique** (recommandé) :
   1. révoquer/régénérer : Google Cloud (client secret), intra 42 (app), token
      de tunnel Cloudflare ; vérifier aussi `RESEND_API_KEY` et les mots de
      passe par défaut (postgres/minio `admin/password123` visibles dans
      compose) ;
   2. purger avec `git filter-repo` (`--replace-text` sur les motifs
      `GOCSPX-*`, `s-s4t2-*`, `eyJ*` du tunnel) sur **chaque** sous-module
      concerné et le superprojet (l'historique fautif est ici celui du
      superprojet, mais les submodules ont leurs propres histoires) ;
   3. force-push coordonné + re-clone de toute l'équipe ; invalider les
      forks/PR ouvertes ;
   4. prévention : `gitleaks` en pre-commit et en CI, `.env*` réellement
      ignorés (vérifier `.env.backup-1635`), `.env.example` avec valeurs
      factices uniquement.

## Décision recommandée

Option **3**, en traitant la **rotation comme première étape immédiate**
(elle porte seule 90 % du risque et ne dépend de personne).

## Conséquences

- Réécriture d'historique : tous les hashes changent, les submodules doivent
  être réalignés (nouveaux pointeurs dans le superprojet), les PRs ouvertes
  à refaire. À planifier à un moment calme.
- `git filter-repo` sur un superprojet à 13 submodules est délicat : purger
  d'abord chaque sous-module concerné, mettre à jour les pointeurs, puis
  purger le superprojet.
- L'analyse `git log -S` post-purge doit revenir vide : c'est le critère
  d'acceptation.
- Effort estimé : rotation S ; purge + coordination M (humain surtout).

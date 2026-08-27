# Audit des contributions — almeekel / AnnibalBarca

Généré le 2026-08-23 par lecture directe de `git log` dans le superprojet et dans
chacun des 10 sous-modules git (les commits de contenu vivent dans les
sous-modules ; le superprojet ne contient que des commits de pointeurs).

## Identité : une seule personne, deux identités git

`almeekel <almeekel@student.42lyon.fr>` et `AnnibalBarca <alexandremeekel@gmail.com>`
sont **la même personne** (même adresse e-mail que l'utilisateur de cette
session : alexandremeekel@gmail.com). Vérification :

```
git log --author="AnnibalBarca" --stat -p
```

ne retourne que **2 commits**, tous deux des *merge commits* GitHub (PR #31
« feat(assets): google drive », PR #30 « chore: point front at its merged
main ») — **aucune ligne de code authentifiée sous ce nom**. Tout le travail de
fond (features, refactors, fixes) est signé `almeekel`. Le harness et les deux
cours ci-dessous traitent donc `almeekel` comme la source de vérité unique.

**Avant de relancer cet audit** (le dépôt peut avoir évolué) :
```
git submodule update --init --recursive
for s in front back/User-API back/Gateway-API back/Auth-API back/Chess-API \
         back/Room-API back/Social-API back/Notification-API back/api-core \
         infra/Nginx; do
  echo "=== $s ==="; (cd "$s" && git log --author="almeekel" --oneline | wc -l)
done
```

## Répartition par sous-module (nombre de commits almeekel)

| Sous-module | Commits | Rôle des contributions |
|---|---|---|
| `front` | 80 | Feature boutique/shop de bout en bout, migration CSS→Tailwind v4, footer, pages légales, corrections i18n, règles des cartes |
| `back/User-API` | 11 | Endpoints shop, client S3 MinIO, seed des collections, séparation endpoints interne/public, validation username |
| `infra/Nginx` | 4 | Bascule HTTP→HTTPS, redirection sans port explicite |
| `back/Auth-API` | 1 | (pointeur/mineur — à vérifier au prochain audit) |
| `back/api-core` | 1 | (pointeur/mineur — à vérifier au prochain audit) |
| `back/Gateway-API`, `back/Room-API`, `back/Social-API`, `back/Notification-API`, `back/Chess-API` | 0 | Aucune contribution almeekel — hors périmètre des cours ci-dessous, mais à connaître pour le Cours 1 (architecture générale) |

## Détail par chantier (ce qui alimente le Cours 2)

### A. Front TypeScript — feature Shop
Fichiers principaux (nb de commits almeekel) :
`ShopView.tsx`(10) `ItemPackGrid.tsx`(9) `moneyPacks.ts`(8) `shop.css`(8)
`ShopPage.tsx`(6) `MoneyPackGrid.tsx`(6) `ItemPackCard.tsx`(5)
`shopService.ts`(4) `shopData.ts`(4) `MoneyPackCard.tsx`(4) `CollectionList.tsx`(4)

Ordre chronologique réel des commits (du plus ancien au plus récent) —
c'est l'ordre dans lequel le Cours 2 / Chapitre 1 doit faire recoder le
learner :

```
2026-07-20  cca7cc1  Re-pushing renamal of boutique to shop
2026-07-20  355cd71  Adding the Money Packs data module
2026-07-20  4467d86  function for rendering an individual card with all its elements
2026-07-20  a309e98  Adding the Item Packs data module edit+ ItemPackCard function
2026-07-20  5b76d58  Using Moneypack 4 times to render a grid
2026-07-20  b1c6824  shop placeholder css
2026-07-20  3fb31c2  Grid function caller but for Items (moneypacks done)
2026-07-20  4241462→2841b7d→13e4acd→98b7cab→7ec481b  itérations shop fonctionnel
2026-08-04  e817634  pathing vers le backend, prix et wallet réels
2026-08-13  d4412ee  group items by avatar slot
2026-08-13  76c5632  show collection contents and buy a pack
2026-08-13  a153df6  add collection items and pack purchase to the api client
2026-08-13  49d59ee  map avatar slot labels and show item images
2026-08-20  14c6ce8  fix remaining untranslated shop/card/matchmaking text
```

### B. Rust — back/User-API (shop + storage)
```
2026-08-03  84855e3  adding the shop pub in mod rs
2026-08-10  4f893de  feat(storage): add minio s3 client
2026-08-10  f802bbe  feat(shop): add catalog assets and collection bundles
2026-08-10  8db0241  feat(shop): add shop endpoints
2026-08-11  4f1980f  refactor(shop): drop admin token check
2026-08-13  e4643dd  feat(shop): seed the five crew collections
2026-08-13  a491c9d  feat(inventory): return the asset url with each owned item
2026-08-14  1edbc1e  fix(storage): keep the internal and public endpoints apart
2026-08-14  f1c5230  Validate username on rename
```
Fichiers : `src/services/storage.rs`, `src/http/handlers/shop.rs`,
`src/db/shop.rs`, `src/db/migrations.rs`, `src/http/handlers/{add_item,
remove_item,get_inventory,collection,change_username,change_email}.rs`.

### C. Nginx — bascule HTTPS
```
2026-08-15  39be8e1  Start htts
2026-08-15  6206a23  Serve the application over HTTPS
2026-08-16  763b264  Redirect without an explicit port
```
Fichiers : `nginx.conf`, `Dockerfile`. Chantier volontairement petit (3
commits, 2 fichiers) : la difficulté est conceptuelle (TLS, certificats,
redirection), pas volumétrique.

### D. Front — migration Tailwind CSS
```
2026-08-04  6e2b3dd  deplacer les jetons de theme.css vers @theme
2026-08-04  0cd7566  convertir HomeLayout et la mise en page des vues play
2026-08-04  b8b3aab  convertir playButton, navbar et modeSelector en Tailwind
2026-08-04  b5df488  convertir wallet, levels, playView et friendView en Tailwind
```
(Chaque commit existe en double sous deux hash différents mêmes message —
artefact d'un force-push/rebase mentionné dans le superprojet, sans
conséquence : dédupliquer par message dans le tutoriel.)

## Fait notable à réutiliser dans une fiche mission (Cours 2, Ch. 1)

`ShopPage.tsx` **n'est pas** monté par une route react-router dédiée. Il n'y a
pas de `<Route path="/shop">`. La vraie mécanique :

`AppRoutes.tsx` → route `/play` → `ProtectedHome` (`front/src/ProtectedHome.tsx`)
qui monte **simultanément** `ShopPage`, `VestiairePage`, `HomePage`,
`SocialPage`, `SettingsPage` comme enfants directs de `HomeLayout`. La
visibilité de chaque page est pilotée par un slider horizontal
(`SliderContext` / `useSlider`, `activeIndex`), pas par le routeur : toutes
les pages restent montées en permanence. C'est une différence fondamentale
avec un modèle « une route = une page », et une bonne question de fiche
mission : *pourquoi `ShopPage.tsx` ne peut pas être calqué sur un modèle
`HomePage.tsx` classique à base de `useNavigate` ?* (Réponse attendue :
`HomePage.tsx` lui-même n'est pas routé indépendamment non plus — les deux
sont des enfants permanents du même slider, donc le vrai modèle de référence
est `HomeLayout.tsx`, pas une page individuelle.)

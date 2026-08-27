# Assets

Every image the application serves from MinIO. One folder here becomes one
bucket, and the folder layout is the object layout:

```
assets/<bucket>/<path>   ->   <bucket>/<path>
```

| Folder | Used for |
| --- | --- |
| `assets` | interface icons: currency, provider logos, language flags, status |
| `cosmetics` | shop items, `<slot>/<item_id>.png`, keyed by the catalog id |
| `carte`, `card-effects` | card artwork and in-game effects |
| `piece`, `rank` | chess pieces and ladder badges |

## Publishing

```
./scripts/publish-assets.sh
```

Creates each bucket if missing, makes it public-read, and mirrors the folder
into it. Run once after `docker compose up -d`.

## Adding or replacing an image

Drop the file in the right folder and run the script again. For shop items keep
the filename: it is the catalog id stored in `shop_catalog.asset_key`, not a
label.

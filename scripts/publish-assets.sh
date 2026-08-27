#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# shellcheck disable=SC1091
set -a; source .env; set +a

NETWORK="$(docker inspect -f \
  '{{range $n, $_ := .NetworkSettings.Networks}}{{$n}}{{"\n"}}{{end}}' \
  minio_storage 2>/dev/null | head -1)"
if [ -z "$NETWORK" ]; then
  echo "minio_storage is not running -- start the stack first" >&2
  exit 1
fi

echo "==> Publishing assets/ to MinIO"
docker run --rm \
  --network "$NETWORK" \
  -v "$ROOT/assets:/assets:ro" \
  -e MC_HOST_local="http://${MINIO_ROOT_USER}:${MINIO_ROOT_PASSWORD}@minio:9000" \
  --entrypoint /bin/sh \
  minio/mc -c '
    set -e
    for dir in /assets/*/; do
      bucket=$(basename "$dir")
      mc mb --ignore-existing "local/$bucket" >/dev/null
      mc anonymous set download "local/$bucket" >/dev/null
      mc mirror --overwrite "$dir" "local/$bucket" >/dev/null
      echo "  $bucket"
    done
    echo
    echo "--- buckets ---"
    mc ls local/
  '

echo
echo "Done. Check ${MINIO_PUBLIC_ENDPOINT:-http://localhost:9000}/assets/RoyalCoins.svg"

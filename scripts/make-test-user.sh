#!/usr/bin/env bash
set -euo pipefail

EMAIL="${1:-}"
PASSWORD="${2:-Test!Pass2026}"
WALLET="${3:-5000}"
BASE="${BASE_URL:-http://localhost:8000}"

if [ -z "$EMAIL" ]; then
  echo "usage: $0 <email> [password] [wallet]" >&2
  echo "example: $0 player2@test.local" >&2
  exit 1
fi

USERNAME="$(echo "$EMAIL" | cut -d@ -f1 | tr -cd 'a-zA-Z0-9_')_$RANDOM"

echo "==> Registering $EMAIL"
CODE=$(curl -s -o /tmp/reg.json -w '%{http_code}' -X POST "$BASE/api/auth/register" \
  -H 'Content-Type: application/json' \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")

if [ "$CODE" = "409" ] || grep -qi "already" /tmp/reg.json 2>/dev/null; then
  echo "    account already exists, validating it"
elif [ "$CODE" != "201" ] && [ "$CODE" != "200" ]; then
  echo "    registration failed (HTTP $CODE):" >&2
  cat /tmp/reg.json >&2; echo >&2
  exit 1
fi

echo "==> Validating (no mail can be sent, RESEND_API_KEY is a placeholder)"
docker exec -i postgres_db psql -U postgres -d ft_transcendence -q <<SQL
UPDATE users
SET email_validated = TRUE,
    account_validated = TRUE,
    username = COALESCE(NULLIF(username, ''), '$USERNAME'),
    wallet = $WALLET
WHERE email = '$EMAIL';

INSERT INTO user_profile (user_id)
SELECT id FROM users WHERE email = '$EMAIL'
ON CONFLICT (user_id) DO NOTHING;
SQL

echo "==> Checking login"
LOGIN=$(curl -s -o /dev/null -w '%{http_code}' -X POST "$BASE/api/auth/login/email" \
  -H 'Content-Type: application/json' \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")

docker exec -i postgres_db psql -U postgres -d ft_transcendence -qtc \
  "SELECT '    ' || username || '  wallet=' || wallet || '  validated=' || account_validated
   FROM users WHERE email = '$EMAIL';"

echo
if [ "$LOGIN" = "200" ]; then
  echo "==> Ready. Log in with:"
  echo "    $EMAIL"
  echo "    $PASSWORD"
else
  echo "==> Login returned HTTP $LOGIN -- check the password rules" >&2
  exit 1
fi

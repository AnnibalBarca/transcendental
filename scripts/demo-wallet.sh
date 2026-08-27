#!/usr/bin/env bash
set -euo pipefail

EMAIL="${1:-}"
if [ -z "$EMAIL" ]; then
  echo "usage: $0 <email> | $0 --off" >&2
  exit 1
fi

if [ "$EMAIL" = "--off" ]; then
  docker exec -i postgres_db psql -U postgres -d ft_transcendence <<'SQL'
DROP TRIGGER IF EXISTS demo_wallet ON users;
DROP FUNCTION IF EXISTS keep_demo_wallet_full();
SQL
  echo "==> Demo wallet disabled"
  exit 0
fi

docker exec -i postgres_db psql -U postgres -d ft_transcendence <<SQL
DO \$\$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM users WHERE email = '$EMAIL') THEN
    RAISE EXCEPTION 'no account with email %', '$EMAIL';
  END IF;
END \$\$;

CREATE OR REPLACE FUNCTION keep_demo_wallet_full() RETURNS trigger AS \$fn\$
BEGIN
  IF NEW.email = '$EMAIL' THEN
    NEW.wallet := 1000000000;
  END IF;
  RETURN NEW;
END; \$fn\$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS demo_wallet ON users;
CREATE TRIGGER demo_wallet BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION keep_demo_wallet_full();

UPDATE users SET wallet = 1000000000 WHERE email = '$EMAIL';
SELECT email, wallet FROM users WHERE email = '$EMAIL';
SQL

echo
echo "==> $EMAIL now has an unlimited wallet"
echo "    disable it with: $0 --off"

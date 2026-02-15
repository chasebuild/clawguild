#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "DATABASE_URL is required (e.g. postgres://user:pass@host:5432/db)" >&2
  exit 1
fi

MIGRATIONS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../migrations" && pwd)"

if [[ ! -d "${MIGRATIONS_DIR}" ]]; then
  echo "Migrations directory not found: ${MIGRATIONS_DIR}" >&2
  exit 1
fi

AUTO_CREATE_DB="${AUTO_CREATE_DB:-1}"

if [[ "${AUTO_CREATE_DB}" == "1" ]]; then
  if ! command -v psql >/dev/null 2>&1; then
    echo "psql is required to auto-create databases (install postgres client tools or set AUTO_CREATE_DB=0)" >&2
    exit 1
  fi

  if ! command -v python3 >/dev/null 2>&1; then
    echo "python3 is required to parse DATABASE_URL for auto-create (or set AUTO_CREATE_DB=0)" >&2
    exit 1
  fi

  mapfile -t parsed < <(
    python3 - <<'PY' "${DATABASE_URL}"
import sys
from urllib.parse import urlparse, parse_qs

url = urlparse(sys.argv[1])
db = url.path.lstrip("/") or "postgres"
params = parse_qs(url.query)
admin_db = params.get("admin_db", ["postgres"])[0]

admin_url = url._replace(path="/" + admin_db, query="").geturl()
print(db)
print(admin_url)
PY
  )

  db_name="${parsed[0]}"
  admin_url="${parsed[1]}"

  if [[ -n "${db_name}" && -n "${admin_url}" ]]; then
    exists=$(psql "${admin_url}" -tA -c "SELECT 1 FROM pg_database WHERE datname='${db_name}'")
    if [[ "${exists}" != "1" ]]; then
      echo "Database ${db_name} not found. Creating..."
      psql "${admin_url}" -v ON_ERROR_STOP=1 -c "CREATE DATABASE \"${db_name}\";"
      echo "Database ${db_name} created."
    fi
  fi
fi

if ! command -v sqlx >/dev/null 2>&1; then
  echo "sqlx CLI is required to run migrations (cargo install sqlx-cli --no-default-features --features postgres)" >&2
  exit 1
fi

set +e
sqlx_output=$(sqlx migrate run --source "${MIGRATIONS_DIR}" 2>&1)
status=$?
set -e

if [[ ${status} -ne 0 ]]; then
  if echo "${sqlx_output}" | rg -q "no driver found for URL scheme \"postgres\""; then
    echo "sqlx CLI is missing the Postgres driver." >&2
    echo "Install with: cargo install sqlx-cli --no-default-features --features postgres" >&2
    exit 1
  fi
  echo "${sqlx_output}" >&2
  exit "${status}"
fi

echo "${sqlx_output}"

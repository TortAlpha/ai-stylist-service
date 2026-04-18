#!/usr/bin/env bash
set -euo pipefail

SERVICE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEST_SQL="${SERVICE_DIR}/tests/sql/schema_invariants.sql"

CONTAINER_NAME="${CONTAINER_NAME:-postgres-product}"
PGUSER="${PGUSER:-postgres}"
PGADMIN_DB="${PGADMIN_DB:-postgres}"
TEST_DB="${TEST_DB:-product_db_sqltest_$(date +%s)_${RANDOM}}"

if ! docker ps --format '{{.Names}}' | grep -qx "${CONTAINER_NAME}"; then
  echo "Container '${CONTAINER_NAME}' is not running."
  echo "Start it first: docker compose up -d postgres-product"
  exit 1
fi

cleanup() {
  docker exec -i "${CONTAINER_NAME}" \
    psql -U "${PGUSER}" -d "${PGADMIN_DB}" -v ON_ERROR_STOP=1 \
    -c "DROP DATABASE IF EXISTS ${TEST_DB};" >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "Creating temporary test database: ${TEST_DB}"
docker exec -i "${CONTAINER_NAME}" \
  psql -U "${PGUSER}" -d "${PGADMIN_DB}" -v ON_ERROR_STOP=1 \
  -c "CREATE DATABASE ${TEST_DB};"

echo "Applying schema bootstrap (init.sql)"
docker exec -i "${CONTAINER_NAME}" \
  psql -U "${PGUSER}" -d "${TEST_DB}" -v ON_ERROR_STOP=1 \
  -f /docker-entrypoint-initdb.d/init.sql >/dev/null

echo "Running SQL invariant tests"
docker exec -i "${CONTAINER_NAME}" \
  psql -U "${PGUSER}" -d "${TEST_DB}" -v ON_ERROR_STOP=1 < "${TEST_SQL}"

echo "SQL tests passed"

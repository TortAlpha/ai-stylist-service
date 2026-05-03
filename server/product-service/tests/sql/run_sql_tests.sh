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

# Apply runtime migrations on top of bootstrap. Bootstrap mirrors all runtime
# changes, so this pass must be a no-op (every migration is idempotent).
RUNTIME_FILES=$(docker exec -i "${CONTAINER_NAME}" \
  bash -c "ls /docker-entrypoint-initdb.d/runtime/*.sql 2>/dev/null | sort")

if [ -n "${RUNTIME_FILES}" ]; then
  echo "Applying runtime migrations (pass 1: should be no-op on fresh bootstrap)"
  while IFS= read -r migration; do
    [ -z "${migration}" ] && continue
    echo "  -> ${migration}"
    docker exec "${CONTAINER_NAME}" \
      psql -U "${PGUSER}" -d "${TEST_DB}" -v ON_ERROR_STOP=1 \
      -f "${migration}" >/dev/null
  done <<< "${RUNTIME_FILES}"

  echo "Re-applying runtime migrations (pass 2: idempotency check)"
  while IFS= read -r migration; do
    [ -z "${migration}" ] && continue
    echo "  -> ${migration}"
    docker exec "${CONTAINER_NAME}" \
      psql -U "${PGUSER}" -d "${TEST_DB}" -v ON_ERROR_STOP=1 \
      -f "${migration}" >/dev/null
  done <<< "${RUNTIME_FILES}"
else
  echo "No runtime migrations found in /docker-entrypoint-initdb.d/runtime/"
fi

echo "Running SQL invariant tests"
docker exec -i "${CONTAINER_NAME}" \
  psql -U "${PGUSER}" -d "${TEST_DB}" -v ON_ERROR_STOP=1 < "${TEST_SQL}"

echo "SQL tests passed"

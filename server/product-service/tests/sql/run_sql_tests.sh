#!/usr/bin/env bash
set -euo pipefail

SERVICE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEST_SQL="${SERVICE_DIR}/tests/sql/schema_invariants.sql"
RUNTIME_DIR="${SERVICE_DIR}/migrations/runtime"

CONTAINER_NAME="${CONTAINER_NAME:-postgres-product}"
PGUSER="${PGUSER:-postgres}"
PGPASSWORD="${PGPASSWORD:-postgres}"
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
RUNTIME_FILES=$(find "${RUNTIME_DIR}" -maxdepth 1 -type f -name '*.sql' | sort)

if [ -n "${RUNTIME_FILES}" ]; then
  TRANSACTION_CONTROL=$(grep -HnE '^[[:space:]]*(BEGIN|COMMIT|ROLLBACK)[[:space:]]*;[[:space:]]*(--.*)?$' "${RUNTIME_DIR}"/*.sql || true)
  if [ -n "${TRANSACTION_CONTROL}" ]; then
    echo "Runtime migrations must not contain manual transaction control:"
    echo "${TRANSACTION_CONTROL}"
    exit 1
  fi

  SQLX_TEST_DATABASE_URL="${SQLX_TEST_DATABASE_URL:-}"
  if [ -z "${SQLX_TEST_DATABASE_URL}" ]; then
    HOST_PORT="${PGPORT:-}"
    if [ -z "${HOST_PORT}" ]; then
      HOST_PORT=$(docker port "${CONTAINER_NAME}" 5432/tcp 2>/dev/null | awk -F: 'NR == 1 { print $NF }' || true)
    fi
    if [ -n "${HOST_PORT}" ]; then
      SQLX_TEST_DATABASE_URL="postgres://${PGUSER}:${PGPASSWORD}@localhost:${HOST_PORT}/${TEST_DB}"
    fi
  fi

  if [ -n "${SQLX_TEST_DATABASE_URL}" ]; then
    echo "Applying runtime migrations through sqlx::migrate!()"
    (
      cd "${SERVICE_DIR}"
      DATABASE_URL="${SQLX_TEST_DATABASE_URL}" \
        cargo test --test runtime_migrations runtime_migrations_apply_through_sqlx_migrator -- --ignored
    )
  else
    echo "Skipping sqlx::migrate!() runtime check: ${CONTAINER_NAME}:5432 is not published."
    echo "Set SQLX_TEST_DATABASE_URL or PGPORT to enable it in this environment."
  fi

  echo "Applying runtime migrations (pass 1: should be no-op on fresh bootstrap)"
  while IFS= read -r migration; do
    [ -z "${migration}" ] && continue
    echo "  -> ${migration}"
    docker exec -i "${CONTAINER_NAME}" \
      psql -U "${PGUSER}" -d "${TEST_DB}" -v ON_ERROR_STOP=1 \
      < "${migration}" >/dev/null
  done <<< "${RUNTIME_FILES}"

  echo "Re-applying runtime migrations (pass 2: idempotency check)"
  while IFS= read -r migration; do
    [ -z "${migration}" ] && continue
    echo "  -> ${migration}"
    docker exec -i "${CONTAINER_NAME}" \
      psql -U "${PGUSER}" -d "${TEST_DB}" -v ON_ERROR_STOP=1 \
      < "${migration}" >/dev/null
  done <<< "${RUNTIME_FILES}"
else
  echo "No runtime migrations found in ${RUNTIME_DIR}"
fi

echo "Running SQL invariant tests"
docker exec -i "${CONTAINER_NAME}" \
  psql -U "${PGUSER}" -d "${TEST_DB}" -v ON_ERROR_STOP=1 < "${TEST_SQL}"

echo "SQL tests passed"

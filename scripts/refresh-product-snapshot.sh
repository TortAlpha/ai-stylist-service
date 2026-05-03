#!/usr/bin/env bash
# Pulls the prod product_db from the Raspberry Pi and restores it into a
# local snapshot container, leaving prod untouched.
#
# Usage:  ./scripts/refresh-product-snapshot.sh
# Result: postgres://postgres:postgres@localhost:5437/product_db

set -euo pipefail

SSH_HOST="${SSH_HOST:-rpi}"
PROD_CONTAINER="${PROD_CONTAINER:-sc-product-db}"
PROD_DB="${PROD_DB:-product_db}"
PROD_USER="${PROD_USER:-postgres}"

LOCAL_CONTAINER="${LOCAL_CONTAINER:-postgres-product-snapshot}"
LOCAL_VOLUME="${LOCAL_VOLUME:-postgres-product-snapshot-data}"
LOCAL_PORT="${LOCAL_PORT:-5437}"

DUMP=/tmp/${PROD_DB}.dump

echo "→ dumping ${PROD_DB} from ${SSH_HOST}:${PROD_CONTAINER}"
ssh "${SSH_HOST}" "docker exec -i ${PROD_CONTAINER} pg_dump -Fc -U ${PROD_USER} ${PROD_DB}" > "${DUMP}"
echo "  $(du -h "${DUMP}" | cut -f1) downloaded"

echo "→ resetting local snapshot container"
docker rm -f "${LOCAL_CONTAINER}" 2>/dev/null || true
docker volume rm "${LOCAL_VOLUME}" 2>/dev/null || true

docker run -d \
  --name "${LOCAL_CONTAINER}" \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB="${PROD_DB}" \
  -p "${LOCAL_PORT}:5432" \
  -v "${LOCAL_VOLUME}:/var/lib/postgresql/data" \
  postgres:16-alpine >/dev/null

echo "→ waiting for postgres to be ready"
until docker exec "${LOCAL_CONTAINER}" pg_isready -U postgres -d "${PROD_DB}" >/dev/null 2>&1; do
  sleep 1
done

echo "→ restoring dump"
docker cp "${DUMP}" "${LOCAL_CONTAINER}:/tmp/dump"
docker exec "${LOCAL_CONTAINER}" \
  pg_restore -U postgres -d "${PROD_DB}" --clean --if-exists /tmp/dump

echo
echo "✓ snapshot ready"
echo "  postgres://postgres:postgres@localhost:${LOCAL_PORT}/${PROD_DB}"
echo "  (from inside docker:  host.docker.internal:${LOCAL_PORT})"

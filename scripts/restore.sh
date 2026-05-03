#!/usr/bin/env bash
#
# Restore sc state from a backup tar.gz produced by backup.sh.
#
# Usage:
#   scripts/restore.sh /home/tortalpha/Code/sc/backups/20260503_120000.tar.gz
#
# Steps:
#   1. Extract archive into a temp dir
#   2. Stop service containers (DB containers stay up so we can restore into them)
#   3. pg_restore --clean for each Postgres
#   4. mc mirror --overwrite into MinIO
#   5. docker compose up -d
#
# Env (with defaults):
#   SC_REPO_DIR    /home/tortalpha/Code/sc
#   COMPOSE_FILE   docker-compose.prod.yml
#
# WARNING: this overwrites current data. Prompts for confirmation unless
# SC_RESTORE_YES=1.

set -euo pipefail

ARCHIVE="${1:-}"
[ -n "${ARCHIVE}" ] || { echo "usage: $0 <backup.tar.gz>" >&2; exit 1; }
[ -f "${ARCHIVE}" ] || { echo "restore.sh: ${ARCHIVE} not found" >&2; exit 1; }

SC_REPO_DIR="${SC_REPO_DIR:-/home/tortalpha/Code/sc}"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.prod.yml}"
ENV_FILE="${SC_REPO_DIR}/.env"
[ -f "${ENV_FILE}" ] || { echo "restore.sh: ${ENV_FILE} not found" >&2; exit 1; }

set -a
# shellcheck source=/dev/null
. "${ENV_FILE}"
set +a

if [ "${SC_RESTORE_YES:-0}" != "1" ]; then
  echo "About to OVERWRITE current sc data from: ${ARCHIVE}"
  read -r -p "Proceed? [y/N] " ans
  [ "${ans,,}" = "y" ] || { echo "Aborted."; exit 1; }
fi

WORK_DIR="$(mktemp -d -t sc-restore-XXXXXX)"
cleanup() { rm -rf "${WORK_DIR}"; }
trap cleanup EXIT

echo "=> Extracting"
tar -xzf "${ARCHIVE}" -C "${WORK_DIR}"
[ -f "${WORK_DIR}/manifest.txt" ] && cat "${WORK_DIR}/manifest.txt"

echo "=> Stopping app services (DBs stay up)"
( cd "${SC_REPO_DIR}" && docker compose -f "${COMPOSE_FILE}" stop \
    sc-product-service sc-user-service sc-auth-service sc-client-admin nginx )

restore_pg() {
  local container="$1"
  local db_user="$2"
  local db_name="$3"
  local dump="${WORK_DIR}/postgres/${container}.dump"

  [ -f "${dump}" ] || { echo "restore.sh: missing ${dump}" >&2; exit 2; }

  echo "  pg_restore -> ${container} (db=${db_name})"
  docker exec -i "${container}" pg_restore \
    -U "${db_user}" -d "${db_name}" \
    --clean --if-exists --no-owner --no-acl \
    < "${dump}"
}

echo "=> Postgres restore"
restore_pg sc-product-db "${PRODUCT_DB_USER:-postgres}" "${PRODUCT_DB_NAME:-product_db}"
restore_pg sc-user-db    "${USER_DB_USER:-postgres}"    "${USER_DB_NAME:-user_db}"
restore_pg sc-auth-db    "${AUTH_DB_USER:-postgres}"    "${AUTH_DB_NAME:-auth_db}"

echo "=> MinIO restore"
docker run --rm \
  --network sc_default \
  -v "${WORK_DIR}/minio:/backup" \
  -e MC_HOST_local="http://${MINIO_ROOT_USER}:${MINIO_ROOT_PASSWORD}@minio:9000" \
  --entrypoint sh \
  minio/mc:latest \
  -c 'mc mirror --quiet --overwrite --remove /backup local'

echo "=> Bringing services back up"
( cd "${SC_REPO_DIR}" && docker compose -f "${COMPOSE_FILE}" up -d --remove-orphans )

echo "Restore done from: ${ARCHIVE}"

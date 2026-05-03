#!/usr/bin/env bash
#
# Snapshot all sc state (3 Postgres DBs + MinIO data) into a single tar.gz
# under $SC_BACKUP_DIR, then prune backups older than $SC_RETENTION_DAYS.
#
# Run via deploy.sh before pulling a new release; can also be invoked manually.
#
# Env (with defaults):
#   SC_REPO_DIR        /home/tortalpha/Code/sc           git clone of the repo (must contain .env)
#   SC_BACKUP_DIR      /home/tortalpha/Code/sc/backups   where snapshots land
#   SC_RETENTION_DAYS  30                   prune older than N days
#   SC_TAG             auto                 timestamp tag for the snapshot
#
# Exit codes:
#   0  success
#   1  config / preflight error
#   2  dump failed
#   3  archive failed

set -euo pipefail

SC_REPO_DIR="${SC_REPO_DIR:-/home/tortalpha/Code/sc}"
SC_BACKUP_DIR="${SC_BACKUP_DIR:-/home/tortalpha/Code/sc/backups}"
SC_RETENTION_DAYS="${SC_RETENTION_DAYS:-30}"
SC_TAG="${SC_TAG:-$(date -u +%Y%m%d_%H%M%S)}"

ENV_FILE="${SC_REPO_DIR}/.env"
[ -f "${ENV_FILE}" ] || { echo "backup.sh: ${ENV_FILE} not found" >&2; exit 1; }

# Load creds (MINIO_ROOT_USER / _PASSWORD, *_DB_USER, *_DB_NAME, *_DB_PASSWORD)
set -a
# shellcheck source=/dev/null
. "${ENV_FILE}"
set +a

mkdir -p "${SC_BACKUP_DIR}"
WORK_DIR="${SC_BACKUP_DIR}/.work_${SC_TAG}"
mkdir -p "${WORK_DIR}/postgres" "${WORK_DIR}/minio"

cleanup_workdir() {
  rm -rf "${WORK_DIR}"
}
trap cleanup_workdir EXIT

dump_pg() {
  local container="$1"
  local db_user="$2"
  local db_name="$3"
  local out="${WORK_DIR}/postgres/${container}.dump"

  echo "  pg_dump ${container} (db=${db_name})"
  if ! docker exec "${container}" pg_dump \
        -U "${db_user}" -d "${db_name}" -Fc \
        > "${out}"; then
    echo "backup.sh: pg_dump failed for ${container}" >&2
    exit 2
  fi
}

echo "=> Postgres dumps"
dump_pg sc-product-db "${PRODUCT_DB_USER:-postgres}" "${PRODUCT_DB_NAME:-product_db}"
dump_pg sc-user-db    "${USER_DB_USER:-postgres}"    "${USER_DB_NAME:-user_db}"
dump_pg sc-auth-db    "${AUTH_DB_USER:-postgres}"    "${AUTH_DB_NAME:-auth_db}"

echo "=> MinIO mirror"
# mc image runs in the same docker network so it can reach minio:9000.
# We mount the work dir to dump bucket contents into.
docker run --rm \
  --network sc_default \
  -v "${WORK_DIR}/minio:/backup" \
  -e MC_HOST_local="http://${MINIO_ROOT_USER}:${MINIO_ROOT_PASSWORD}@minio:9000" \
  --entrypoint sh \
  minio/mc:latest \
  -c 'mc mirror --quiet --overwrite local /backup'

echo "=> Manifest"
cat > "${WORK_DIR}/manifest.txt" <<EOF
sc backup snapshot
created_at:    $(date -u +%Y-%m-%dT%H:%M:%SZ)
sc_version:    ${SC_VERSION:-unknown}
git_commit:    $(git -C "${SC_REPO_DIR}" rev-parse HEAD 2>/dev/null || echo unknown)
host:          $(hostname)
EOF

echo "=> Archive"
ARCHIVE="${SC_BACKUP_DIR}/${SC_TAG}.tar.gz"
if ! tar -czf "${ARCHIVE}" -C "${WORK_DIR}" .; then
  echo "backup.sh: tar failed" >&2
  exit 3
fi
echo "  -> ${ARCHIVE} ($(du -h "${ARCHIVE}" | cut -f1))"

echo "=> Prune (older than ${SC_RETENTION_DAYS} days, always keep newest)"
# Locate the most recent .tar.gz; never prune it, even if it's older than retention.
LATEST_BACKUP="$(find "${SC_BACKUP_DIR}" -maxdepth 1 -name '*.tar.gz' -type f \
  -printf '%T@ %p\n' 2>/dev/null | sort -rn | head -1 | cut -d' ' -f2-)"

if [ -n "${LATEST_BACKUP}" ]; then
  find "${SC_BACKUP_DIR}" -maxdepth 1 -name '*.tar.gz' -type f \
    -mtime "+${SC_RETENTION_DAYS}" \
    ! -wholename "${LATEST_BACKUP}" \
    -print -delete
fi

echo "Backup done: ${ARCHIVE}"

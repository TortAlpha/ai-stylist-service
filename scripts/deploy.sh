#!/usr/bin/env bash
#
# Pull-based CD entry point. Run on a schedule (systemd timer).
#
# Flow:
#   1. git fetch tags
#   2. Find the latest semver tag (vX.Y.Z) in the repo
#   3. Compare with SC_VERSION in .env
#   4. If different:
#        a. backup.sh       (snapshot before changing anything)
#        b. checkout the tag (so docker-compose.prod.yml + scripts match)
#        c. update SC_VERSION in .env
#        d. docker compose pull
#        e. docker compose up -d --remove-orphans
#        f. docker image prune -f
#
# Idempotent: if SC_VERSION already matches latest tag, exits 0 silently.
#
# Env (with defaults):
#   SC_REPO_DIR     /opt/sc/sc-repo
#   COMPOSE_FILE    docker-compose.prod.yml
#   SC_BACKUP_DIR   /opt/sc/backups        (passed through to backup.sh)
#
# Logs:
#   stdout/stderr — capture via systemd journal.

set -euo pipefail

SC_REPO_DIR="${SC_REPO_DIR:-/home/tortalpha/Code/sc}"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.prod.yml}"

cd "${SC_REPO_DIR}"

ENV_FILE="${SC_REPO_DIR}/.env"
[ -f "${ENV_FILE}" ] || { echo "deploy.sh: ${ENV_FILE} not found" >&2; exit 1; }

current_version() {
  grep '^SC_VERSION=' "${ENV_FILE}" | head -1 | cut -d= -f2- | tr -d '"'
}

git fetch --tags --quiet

LATEST_TAG="$(git tag -l 'v*' --sort=-v:refname | head -1)"
[ -n "${LATEST_TAG}" ] || { echo "deploy.sh: no v* tags found in repo" >&2; exit 1; }

# release.yml uses docker/metadata-action with {{version}}, which strips the
# leading 'v'. Image tags are therefore '0.1.0', not 'v0.1.0' — and SC_VERSION
# in .env follows that convention. Compare/write in the v-less form.
LATEST_VERSION="${LATEST_TAG#v}"
CURRENT_VERSION="$(current_version)"

if [ "${LATEST_VERSION}" = "${CURRENT_VERSION}" ]; then
  # Up to date — quiet exit.
  exit 0
fi

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] sc deploy: ${CURRENT_VERSION:-<unset>} -> ${LATEST_VERSION} (git ${LATEST_TAG})"

# 1. Backup BEFORE touching anything live.
echo "=> Backup"
SC_TAG="$(date -u +%Y%m%d_%H%M%S)_pre_${LATEST_TAG}" \
  bash "${SC_REPO_DIR}/scripts/backup.sh"

# 2. Check out the tag so compose + script files match the target version.
echo "=> git checkout ${LATEST_TAG}"
git checkout --quiet "${LATEST_TAG}"

# 3. Update SC_VERSION in .env (v-less form, matches docker image tag).
echo "=> bumping SC_VERSION in .env"
sed -i.bak "s|^SC_VERSION=.*|SC_VERSION=${LATEST_VERSION}|" "${ENV_FILE}"
rm -f "${ENV_FILE}.bak"

# 4–5. Pull and roll.
echo "=> docker compose pull"
docker compose -f "${COMPOSE_FILE}" pull --quiet

echo "=> docker compose up -d"
docker compose -f "${COMPOSE_FILE}" up -d --remove-orphans

# 6. Reclaim disk: drop images that no service references.
echo "=> docker image prune"
docker image prune -f >/dev/null

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] sc deploy: ${LATEST_VERSION} live"

#!/usr/bin/env bash
# Mirrors prod MinIO buckets from the Raspberry Pi into the local MinIO.
# Uses an ssh tunnel so prod MinIO does not need to be reachable from LAN.
#
# Requirements:
#   - mc (MinIO Client) installed locally:  brew install minio/stable/mc
#   - local MinIO running on http://localhost:9000 (already in your compose)
#   - ssh access to rpi
#
# Usage:  ./scripts/refresh-minio-snapshot.sh

set -euo pipefail

SSH_HOST="${SSH_HOST:-rpi}"

PROD_PORT="${PROD_PORT:-9000}"
TUNNEL_PORT="${TUNNEL_PORT:-9100}"

PROD_KEY="${PROD_KEY:-minioadmin}"
PROD_SECRET="${PROD_SECRET:-minioadmin}"

LOCAL_URL="${LOCAL_URL:-http://localhost:9000}"
LOCAL_KEY="${LOCAL_KEY:-minioadmin}"
LOCAL_SECRET="${LOCAL_SECRET:-minioadmin}"

BUCKETS=(
  "ava-product-images"
  "ava-product-previews"
)

echo "→ opening ssh tunnel ${SSH_HOST}:${PROD_PORT} → localhost:${TUNNEL_PORT}"
ssh -N -L "${TUNNEL_PORT}:localhost:${PROD_PORT}" "${SSH_HOST}" &
TUNNEL_PID=$!
trap 'kill "${TUNNEL_PID}" 2>/dev/null || true' EXIT

# Wait for tunnel to be ready
for i in {1..15}; do
  if curl -sf "http://localhost:${TUNNEL_PORT}/minio/health/live" >/dev/null 2>&1; then
    break
  fi
  sleep 1
  if [ "$i" = 15 ]; then
    echo "✗ tunnel did not become healthy" >&2
    exit 1
  fi
done

echo "→ configuring mc aliases"
mc alias set prod-rpi "http://localhost:${TUNNEL_PORT}" "${PROD_KEY}" "${PROD_SECRET}" >/dev/null
mc alias set local-minio "${LOCAL_URL}" "${LOCAL_KEY}" "${LOCAL_SECRET}" >/dev/null

for bucket in "${BUCKETS[@]}"; do
  echo "→ ensuring local bucket ${bucket}"
  mc mb --ignore-existing "local-minio/${bucket}" >/dev/null

  echo "→ mirroring prod-rpi/${bucket} → local-minio/${bucket}"
  mc mirror --overwrite --remove "prod-rpi/${bucket}" "local-minio/${bucket}"
done

echo
echo "✓ minio snapshot ready at ${LOCAL_URL}"
echo "  console: http://localhost:9001 (login: ${LOCAL_KEY}/${LOCAL_SECRET})"

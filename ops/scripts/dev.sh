#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

bun run workspace:links >/dev/null

HOST_IP="${DEBUGPATH_HOST_IP:-}"

if [[ -z "$HOST_IP" ]]; then
  for iface in en0 en1 bridge0; do
    if ip=$(ipconfig getifaddr "$iface" 2>/dev/null) && [[ -n "$ip" ]]; then
      HOST_IP="$ip"
      break
    fi
  done
fi

if [[ -z "$HOST_IP" ]]; then
  HOST_IP="$(ifconfig | awk '/inet 192\.168\./ {print $2; exit} /inet 10\./ {print $2; exit} /inet 172\.(1[6-9]|2[0-9]|3[0-1])\./ {print $2; exit}')"
fi

export DEBUGPATH_WEB_UPSTREAM="${DEBUGPATH_WEB_UPSTREAM:-${HOST_IP:+${HOST_IP}:4321}}"
export DEBUGPATH_API_UPSTREAM="${DEBUGPATH_API_UPSTREAM:-${HOST_IP:+${HOST_IP}:3000}}"

if [[ -z "${DEBUGPATH_WEB_UPSTREAM}" || -z "${DEBUGPATH_API_UPSTREAM}" ]]; then
  export DEBUGPATH_WEB_UPSTREAM="host.docker.internal:4321"
  export DEBUGPATH_API_UPSTREAM="host.docker.internal:3000"
  echo "warning: could not detect a host LAN IP, falling back to host.docker.internal"
fi

echo "debugpath dev upstreams: web=${DEBUGPATH_WEB_UPSTREAM} api=${DEBUGPATH_API_UPSTREAM}"

docker compose up -d postgres minio caddy

exec bunx concurrently -k -s first -n api,web \
  "cd apps/api && bun run dev" \
  "cd apps/web && bun run dev"

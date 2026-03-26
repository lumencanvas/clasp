#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Starting DefraDB test nodes..."
docker compose up -d --wait

DEFRA1="http://localhost:9181"
DEFRA2="http://localhost:9182"

echo "Waiting for DefraDB nodes to be ready..."
for url in "$DEFRA1" "$DEFRA2"; do
    for i in $(seq 1 30); do
        if curl -sf "${url}/api/v0/" > /dev/null 2>&1; then
            echo "  ${url} ready"
            break
        fi
        if [ "$i" -eq 30 ]; then
            echo "  ${url} failed to start"
            exit 1
        fi
        sleep 1
    done
done

echo "DefraDB test nodes ready"
echo "  Node 1: $DEFRA1"
echo "  Node 2: $DEFRA2"

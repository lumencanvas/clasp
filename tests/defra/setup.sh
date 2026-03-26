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
        if curl -sf "${url}/health-check" > /dev/null 2>&1; then
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

# ---------------------------------------------------------------------------
# Provision the ClaspParam schema on both nodes so collections exist before
# replicator setup. The schema add is idempotent (re-adding is a no-op).
# ---------------------------------------------------------------------------
CLASP_SCHEMA='type ClaspParam {
    address: String @index
    value: String
    valueType: String
    revision: Int
    writer: String
    timestamp: Int
    lastAccessed: Int
    strategy: String
    lockHolder: String
    origin: String
    ttlMode: String
    ttlSecs: Int
}'

echo "Provisioning ClaspParam schema on both nodes..."
docker exec clasp-defra-1 /defradb client collection add \
    --url http://localhost:9181 --no-keyring "$CLASP_SCHEMA" > /dev/null 2>&1 || true
docker exec clasp-defra-2 /defradb client collection add \
    --url http://localhost:9181 --no-keyring "$CLASP_SCHEMA" > /dev/null 2>&1 || true
echo "  Schema provisioned"

# ---------------------------------------------------------------------------
# Set up P2P replication between the two nodes.
#
# DefraDB replication is unidirectional (push): "replicator add" on node A
# pushes A's writes to the remote peer. We set up bidirectional replication
# so writes on either node propagate to the other.
#
# Peer info returns a JSON array of multiaddrs. We need the one with the
# container's Docker-network IP (not 127.0.0.1) so the other container can
# reach it.
# ---------------------------------------------------------------------------
echo "Setting up P2P replication..."

# Get defra1's peer multiaddr (Docker-network IP, not loopback)
DEFRA1_INFO=$(docker exec clasp-defra-1 /defradb client p2p info \
    --url http://localhost:9181 --no-keyring 2>/dev/null || echo "")

# Get defra2's peer multiaddr
DEFRA2_INFO=$(docker exec clasp-defra-2 /defradb client p2p info \
    --url http://localhost:9181 --no-keyring 2>/dev/null || echo "")

if [ -z "$DEFRA1_INFO" ] || [ -z "$DEFRA2_INFO" ]; then
    echo "  WARNING: Could not retrieve P2P info. Peering skipped."
    echo "  Sync tests will fail but other tests will work."
    exit 0
fi

# Extract the non-loopback multiaddr (contains the Docker-network IP).
# The p2p info output is a JSON array like:
#   ["/ip4/127.0.0.1/tcp/9171/p2p/PEER_ID", "/ip4/172.x.x.x/tcp/9171/p2p/PEER_ID"]
# We want the one that does NOT contain 127.0.0.1.
extract_peer_addr() {
    echo "$1" | tr -d '[]"\n ' | tr ',' '\n' | grep -v '127\.0\.0\.1' | head -1
}

DEFRA1_ADDR=$(extract_peer_addr "$DEFRA1_INFO")
DEFRA2_ADDR=$(extract_peer_addr "$DEFRA2_INFO")

if [ -z "$DEFRA1_ADDR" ] || [ -z "$DEFRA2_ADDR" ]; then
    echo "  WARNING: Could not extract peer addresses. Peering skipped."
    echo "  defra1 info: $DEFRA1_INFO"
    echo "  defra2 info: $DEFRA2_INFO"
    exit 0
fi

echo "  defra1 addr: $DEFRA1_ADDR"
echo "  defra2 addr: $DEFRA2_ADDR"

# Node 1 pushes to node 2 (so writes on node 1 appear on node 2)
if docker exec clasp-defra-1 /defradb client p2p replicator add \
    --url http://localhost:9181 --no-keyring \
    -c ClaspParam "$DEFRA2_ADDR" 2>/dev/null; then
    echo "  defra1 -> defra2 replicator: OK"
else
    echo "  WARNING: Failed to add defra1 -> defra2 replicator"
fi

# Node 2 pushes to node 1 (bidirectional, so writes on node 2 appear on node 1)
if docker exec clasp-defra-2 /defradb client p2p replicator add \
    --url http://localhost:9181 --no-keyring \
    -c ClaspParam "$DEFRA1_ADDR" 2>/dev/null; then
    echo "  defra2 -> defra1 replicator: OK"
else
    echo "  WARNING: Failed to add defra2 -> defra1 replicator"
fi

echo "P2P peering setup complete"

---
title: Federation
description: Multi-site state sync via hub-leaf topology
order: 4
---

# Federation

Federation connects multiple CLASP routers so state changes on one site automatically propagate to others. Each site runs its own relay with full local functionality, and a hub router aggregates state across all sites. Clients connect to their local relay and see a unified state tree.

## Architecture

Federation uses a hub-leaf topology. Each leaf router owns a namespace (a set of address patterns) and forwards matching state changes to the hub. The hub redistributes updates to all other leaves.

```
Site A (Leaf)            Hub Router            Site B (Leaf)
owns /site-a/**   <-->   aggregates    <-->   owns /site-b/**
local clients            all state            local clients
```

Clients on Site A can read and subscribe to `/site-b/**` paths. Writes are routed through the hub to the owning leaf. Each leaf is authoritative for its own namespace.

## Setup

Federation requires the `federation` feature flag at build time. All three relays can run on separate machines, in separate containers, or on the same host with different ports.

### 1. Start the Hub

The hub accepts connections from leaves. It does not need its own namespace.

```bash
clasp-relay --auth-port 7350 --features federation
```

### 2. Start Leaf A

Connect to the hub and declare namespace ownership:

```bash
clasp-relay \
  --federation-hub ws://hub:7330 \
  --federation-id site-a \
  --federation-namespace "/site-a/**"
```

### 3. Start Leaf B

```bash
clasp-relay \
  --federation-hub ws://hub:7330 \
  --federation-id site-b \
  --federation-namespace "/site-b/**"
```

### 4. Verify

Set a value on Site A and observe it on Site B:

```bash
# On a client connected to Site A
clasp set /site-a/greeting "hello from site-a"

# On a client connected to Site B
clasp get /site-a/greeting
# Returns: "hello from site-a"
```

## Namespace Ownership

Each leaf declares which address patterns it owns using `--federation-namespace`. The flag is repeatable for multiple namespaces:

```bash
clasp-relay \
  --federation-hub ws://hub:7330 \
  --federation-id lighting-site \
  --federation-namespace "/lights/**" \
  --federation-namespace "/fixtures/**" \
  --federation-namespace "/scenes/**"
```

Namespace rules:

- Wildcards `*` (single segment) and `**` (multi-segment) are supported.
- Namespaces must not overlap between leaves. If two leaves claim the same path, behavior is undefined.
- The hub does not own any namespace. It aggregates and distributes.
- A leaf processes writes to its own namespace locally and forwards the result to the hub.

## Sync Protocol

When a leaf connects (or reconnects) to the hub, a sync handshake occurs:

1. **DeclareNamespaces** -- the leaf tells the hub which namespaces it owns.
2. **RequestSync** -- the leaf sends its RevisionVector (a map of address to last-known revision).
3. **RevisionVector** -- the hub compares vectors and sends any updates the leaf is missing.
4. **SyncComplete** -- the hub signals that initial sync is done.

After sync completes, real-time forwarding begins. Every state change on any leaf is forwarded through the hub to all other leaves whose subscriptions match.

## Loop Prevention

Forwarded messages carry an `origin` field identifying the source router. When the hub forwards a message to leaves, the originating leaf is excluded. This prevents messages from bouncing back to their source.

The origin field also prevents loops in more complex topologies. A message is never forwarded to the router that produced it, regardless of how many hops it has taken.

## Authentication

Secure leaf-to-hub connections with a shared token:

```bash
# Hub: set a federation token
clasp-relay --auth-port 7350 --federation-token ./federation-secret.txt

# Leaf: provide the same token
clasp-relay \
  --federation-hub ws://hub:7330 \
  --federation-id site-a \
  --federation-namespace "/site-a/**" \
  --federation-token ./federation-secret.txt
```

The token file contains a single line of text used as a bearer token during the WebSocket handshake. The hub rejects connections with missing or invalid tokens.

For TLS-encrypted federation links, use `wss://` instead of `ws://`:

```bash
clasp-relay \
  --federation-hub wss://hub.example.com:7330 \
  --federation-id site-a \
  --federation-namespace "/site-a/**" \
  --federation-token ./federation-secret.txt
```

## Reconnection

Leaves automatically reconnect to the hub if the connection drops. On reconnect, the full sync handshake runs again, bringing the leaf up to date with any changes that occurred while it was disconnected.

Reconnection uses exponential backoff starting at 1 second and capping at 30 seconds. No manual intervention is required.

## Docker Compose Example

A multi-site federation deployment:

```yaml
services:
  hub:
    image: clasp-relay
    ports:
      - "7330:7330"
      - "7350:7350"
    command: clasp-relay --auth-port 7350
    volumes:
      - ./federation-secret.txt:/etc/clasp/federation-secret.txt:ro
    environment:
      - CLASP_FEDERATION_TOKEN=/etc/clasp/federation-secret.txt

  leaf-a:
    image: clasp-relay
    ports:
      - "7331:7330"
    command: >
      clasp-relay
        --federation-hub ws://hub:7330
        --federation-id site-a
        --federation-namespace "/site-a/**"
        --federation-token /etc/clasp/federation-secret.txt
    volumes:
      - ./federation-secret.txt:/etc/clasp/federation-secret.txt:ro
    depends_on:
      - hub

  leaf-b:
    image: clasp-relay
    ports:
      - "7332:7330"
    command: >
      clasp-relay
        --federation-hub ws://hub:7330
        --federation-id site-b
        --federation-namespace "/site-b/**"
        --federation-token /etc/clasp/federation-secret.txt
    volumes:
      - ./federation-secret.txt:/etc/clasp/federation-secret.txt:ro
    depends_on:
      - hub
```

Start the federation:

```bash
docker compose up -d
```

Each leaf exposes its own WebSocket port (7331, 7332) for local client connections. The hub runs on 7330 and handles inter-site traffic.

## Next Steps

- [Discovery](./discovery.md) -- automatic router and device discovery
- [App Config](./app-config.md) -- declarative scopes and write rules
- [Architecture](../concepts/architecture.md) -- architecture deep dive

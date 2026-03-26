---
title: Quick Start
description: Get DefraDB running with CLASP in 5 minutes
order: 1
---

# Quick Start

## 1. Start DefraDB

```bash
# Single node (development)
docker run -d --name defra -p 9181:9181 sourcenetwork/defradb:latest \
  start --url 0.0.0.0:9181 --no-keyring

# Verify it's running
curl http://localhost:9181/health-check
# "Healthy"
```

## 2. Start CLASP Router with DefraDB Journal

Build the router with DefraDB support:

```bash
cargo build --release -p clasp-router-server --features journal-defra
```

Start with DefraDB as the journal backend:

```bash
./target/release/clasp-router \
  --listen 0.0.0.0:7330 \
  --journal \
  --journal-backend defra \
  --journal-defra-url http://localhost:9181
```

The router provisions schemas automatically on first connect. All SET operations are journaled to DefraDB asynchronously.

## 3. Connect a Client

```rust
use clasp_client::ClaspBuilder;
use clasp_core::Value;

let client = ClaspBuilder::new("ws://localhost:7330")
    .name("my-app")
    .connect()
    .await?;

// This value persists in DefraDB
client.set("/lights/brightness", Value::Float(0.8)).await?;
```

Kill the router, restart it. The value is still there: recovered from DefraDB on startup.

## 4. Two-Node Setup (P2P Sync)

```bash
cd tests/defra
bash setup.sh
```

This starts two DefraDB nodes, provisions schemas, and configures bidirectional P2P replication. Start two routers:

```bash
# Router A -> DefraDB node 1
clasp-router --listen 0.0.0.0:7330 --journal --journal-backend defra \
  --journal-defra-url http://localhost:9181

# Router B -> DefraDB node 2
clasp-router --listen 0.0.0.0:7331 --journal --journal-backend defra \
  --journal-defra-url http://localhost:9182
```

A client setting `/lights/brightness` on Router A will see the value appear on Router B after DefraDB syncs (typically 1-5 seconds on a local network).

## Available Journal Backends

| Backend | Flag | Use Case |
|---------|------|----------|
| Memory | `--journal-backend memory` | Development, no persistence |
| SQLite | `--journal-backend sqlite --journal-path ./journal.db` | Single node, persistent |
| DefraDB | `--journal-backend defra --journal-defra-url URL` | Multi-node, P2P sync |

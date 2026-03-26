# DefraDB Test Nodes

Two-node DefraDB cluster for CLASP integration testing.

## Prerequisites

- Docker and Docker Compose v2

## Quick start

```bash
# Start nodes (waits for health checks)
./setup.sh

# Run integration tests
cargo test -p clasp-journal-defra -- --ignored --test-threads=1

# Stop and remove volumes
./teardown.sh
```

## Manual usage

```bash
docker compose up -d --wait
docker compose down -v
```

## Endpoints

| Node | HTTP API | P2P |
|------|----------|-----|
| defra1 | http://localhost:9181 | localhost:9171 |
| defra2 | http://localhost:9182 | localhost:9172 |

## Health check

```bash
curl http://localhost:9181/api/v0/
curl http://localhost:9182/api/v0/
```

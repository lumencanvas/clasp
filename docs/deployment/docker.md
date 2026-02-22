---
title: Docker Deployment
description: Build and run CLASP relay with Docker
order: 4
---

# Docker Deployment

The relay ships with a Dockerfile in `deploy/relay/` that builds from published crates on crates.io. Build once, deploy anywhere.

## Building

Build the relay image from the deploy directory:

```bash
cd deploy/relay
docker build -t clasp-relay .
```

With specific features:

```bash
docker build --build-arg FEATURES=journal,rules -t clasp-relay .
```

With all features:

```bash
docker build --build-arg FEATURES=full -t clasp-relay .
```

The `FEATURES` build arg maps directly to Cargo feature flags. If omitted, only the default features (websocket, rendezvous) are compiled.

## Running

**Basic -- WebSocket only:**

```bash
docker run -p 7330:7330 clasp-relay
```

**With auth:**

```bash
docker run \
  -p 7330:7330 \
  -p 7350:7350 \
  clasp-relay --auth-port 7350
```

**With persistence:**

```bash
docker run \
  -p 7330:7330 \
  -v clasp-data:/data \
  clasp-relay \
    --journal /data/journal.db \
    --persist /data/state.db
```

**With TLS:**

```bash
docker run \
  -p 7330:7330 \
  -v ./certs:/certs:ro \
  clasp-relay \
    --cert /certs/cert.pem \
    --key /certs/key.pem
```

## Environment Variables

| Variable     | Default | Description                              |
| ------------ | ------- | ---------------------------------------- |
| `RUST_LOG`   | `info`  | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `LOG_FORMAT` | `text`  | Set to `json` for structured log output  |

Pass environment variables with `-e`:

```bash
docker run -e RUST_LOG=debug -e LOG_FORMAT=json -p 7330:7330 clasp-relay
```

## Volumes

Mount volumes for any data that should survive container restarts.

| Mount point | Purpose                                  |
| ----------- | ---------------------------------------- |
| `/data`     | Persistence: journal, snapshots, auth DB |
| `/config`   | App config JSON files                    |
| `/certs`    | TLS certificates and keys                |

Example with all volumes:

```bash
docker run \
  -p 7330:7330 \
  -p 7350:7350 \
  -v clasp-data:/data \
  -v ./config:/config:ro \
  -v ./certs:/certs:ro \
  clasp-relay \
    --auth-port 7350 \
    --auth-db /data/relay-auth.db \
    --journal /data/journal.db \
    --persist /data/state.db \
    --app-config /config/app.json \
    --cert /certs/cert.pem \
    --key /certs/key.pem
```

## Health Checks

Add a health check that probes the WebSocket port. A successful TCP connection indicates the relay is accepting clients.

```dockerfile
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
  CMD nc -z localhost 7330 || exit 1
```

If the auth HTTP server is enabled, you can also probe that port:

```bash
curl -sf http://localhost:7350/health || exit 1
```

## Docker Compose

A full production-ready Compose configuration:

```yaml
services:
  relay:
    image: clasp-relay
    build:
      context: deploy/relay
      args:
        FEATURES: "full"
    ports:
      - "7330:7330"
      - "7350:7350"
    environment:
      RUST_LOG: "info"
      LOG_FORMAT: "json"
    command: >
      clasp-relay
        --auth-port 7350
        --auth-db /data/relay-auth.db
        --admin-token /secrets/admin.token
        --cors-origin https://app.yourdomain.com
        --journal /data/journal.db
        --persist /data/state.db
        --persist-interval 30
        --app-config /config/app.json
        --param-ttl 3600
        --max-sessions 500
        --cert /certs/cert.pem
        --key /certs/key.pem
    volumes:
      - relay-data:/data
      - ./config:/config:ro
      - ./certs:/certs:ro
      - ./secrets:/secrets:ro
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "nc", "-z", "localhost", "7330"]
      interval: 30s
      timeout: 5s
      retries: 3

volumes:
  relay-data:
```

Start:

```bash
docker compose up -d
```

View logs:

```bash
docker compose logs -f relay
```

## Next Steps

- [Cloud Deployment](cloud.md) -- deploy containers to a cloud provider
- [Production Checklist](production-checklist.md) -- verify your deployment is hardened

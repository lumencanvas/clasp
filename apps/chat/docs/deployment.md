# Deployment

## Architecture

```
                    Internet
                       │
                       v
              ┌────────────────┐
              │     Caddy       │  :443 (HTTPS)
              │  (reverse proxy)│  :80 (redirect)
              │  Auto TLS certs │
              └───────┬────────┘
                      │
          ┌───────────┴───────────┐
          │                       │
          v                       v
  ┌───────────────┐     ┌──────────────────┐
  │  Static Files  │     │   CLASP Relay     │
  │  (chat SPA)    │     │                  │
  │  /srv/chat/    │     │  :7330 WebSocket │
  │                │     │  :7350 Auth HTTP │
  └───────────────┘     │                  │
                        │  /data/ volume   │
                        │  ├── state.json  │
                        │  └── users.db    │
                        └──────────────────┘
```

## Local Development

### Prerequisites

- Node.js 20+ (for the frontend)
- Rust 1.75+ (for the relay)

### Frontend

```bash
cd apps/chat
npm install
npx vite dev          # Dev server at http://localhost:5173
npx vite build        # Production build to dist/
```

Environment variables (`.env` or `VITE_*`):
| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_RELAY_URL` | `wss://relay.clasp.chat` | WebSocket relay URL |
| `VITE_AUTH_API_URL` | `https://relay.clasp.chat` | Auth HTTP API base URL |

### Relay

```bash
cd deploy/relay
cargo build --release
./target/release/clasp-relay \
  --port 7330 \
  --auth-port 7350 \
  --data-dir ./data \
  --cors-origin "http://localhost:5173"
```

CLI arguments:
| Argument | Default | Description |
|----------|---------|-------------|
| `--port` | 7330 | WebSocket server port |
| `--auth-port` | 7350 | Auth HTTP server port |
| `--data-dir` | `./data` | Directory for state.json and users.db |
| `--cors-origin` | (permissive) | Comma-separated allowed CORS origins |

### Docker Compose (Local)

```bash
cd deploy/chat
docker compose up
```

The `deploy/chat/docker-compose.yml` builds and runs both services:

```yaml
services:
  relay:
    build: ../relay
    ports:
      - "7330:7330"   # WebSocket
      - "7350:7350"   # Auth HTTP
    volumes:
      - relay-data:/data
    environment:
      - RUST_LOG=info

volumes:
  relay-data:
```

## Production Deployment (DigitalOcean Droplet)

### Infrastructure

The production setup uses a DigitalOcean Droplet with:
- Ubuntu 22.04
- Attached block storage volume (mounted at `/mnt/data`)
- Caddy reverse proxy with automatic HTTPS
- Docker Compose orchestration

### Setup Script

`deploy/droplet/setup.sh` automates initial server provisioning:
1. Install Docker and Docker Compose
2. Format and mount the data volume
3. Create directory structure
4. Pull and start containers

### Docker Compose (Production)

`deploy/droplet/docker-compose.yml`:

```yaml
services:
  relay:
    image: clasp-relay:latest
    restart: unless-stopped
    volumes:
      - /mnt/data/relay:/data
    environment:
      - RUST_LOG=info
    # Ports NOT exposed to host -- Caddy proxies internally

  caddy:
    image: caddy:2
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - /mnt/data/caddy:/data
      - /srv/chat:/srv/chat    # Built SPA files
```

### Caddyfile

```
chat.example.com {
    # SPA static files
    root * /srv/chat
    try_files {path} /index.html
    file_server

    # WebSocket proxy
    handle /ws* {
        reverse_proxy relay:7330
    }

    # Auth API proxy
    handle /auth/* {
        reverse_proxy relay:7350
    }
}
```

Caddy provides:
- Automatic Let's Encrypt TLS certificates
- HTTP -> HTTPS redirect
- WebSocket upgrade handling
- SPA fallback routing (`try_files`)

### Relay Dockerfile

`deploy/relay/Dockerfile`:

```dockerfile
FROM rust:1.75 AS builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -r -s /bin/false clasp
USER clasp
COPY --from=builder /build/target/release/clasp-relay /usr/local/bin/
ENTRYPOINT ["clasp-relay"]
CMD ["--data-dir", "/data"]
```

Key security features:
- Multi-stage build (no compiler in runtime image)
- Runs as non-root `clasp` user
- Minimal base image (debian-slim)

## State Persistence

### Snapshot Mechanism

The relay periodically persists all CLASP state to disk:

```
Every 60 seconds:
  1. Serialize full state to JSON
  2. Write to {data-dir}/state.json.tmp
  3. Atomic rename to {data-dir}/state.json
```

On startup:
1. If `state.json` exists, load and restore all params
2. Clients reconnecting after restart receive the restored state via SNAPSHOT

### SQLite Database

User accounts are stored in `{data-dir}/users.db`:

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,              -- u-{timestamp}-{random}
    username TEXT UNIQUE NOT NULL,     -- case-insensitive unique
    password_hash TEXT NOT NULL,       -- Argon2 hash
    created_at INTEGER NOT NULL       -- Unix timestamp
);
```

### Data Volume Layout

```
/data/                    (or /mnt/data/relay/ in production)
├── state.json            CLASP state snapshot (all params)
├── state.json.tmp        Temporary file during atomic write
└── users.db              SQLite user database
```

## Monitoring

The relay logs to stdout with configurable verbosity via `RUST_LOG`:

```bash
RUST_LOG=info     # Connection events, auth attempts
RUST_LOG=debug    # All CLASP operations
RUST_LOG=trace    # Binary frame details
```

Key events logged:
- WebSocket connections and disconnections
- Authentication attempts (register, login, guest) with success/failure
- Rate limit hits
- State snapshot writes
- CLASP message processing errors

## Running Tests

### Frontend Tests

```bash
cd apps/chat
npx vitest run              # Run all tests once
npx vitest                  # Watch mode
npx vitest run --coverage   # With coverage report
```

### Relay Tests

```bash
cd deploy/relay
cargo test                  # Run all tests
cargo test -- --nocapture   # With stdout output
```

Test coverage includes:
- Auth endpoint happy paths and error cases
- Rate limiting behavior
- User ID validation
- CORS configuration
- Crypto roundtrip (encrypt/decrypt)
- Key exchange integration
- Password proof gating

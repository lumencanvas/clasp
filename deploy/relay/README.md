# CLASP Relay Server

A standalone CLASP relay server supporting multi-protocol communication, authentication, persistence, federation, and server-side automation.

## Quick Start

### Option 1: Docker (Recommended)

```bash
cd deploy/relay
docker build -t clasp-relay .
docker run -p 7330:7330 clasp-relay

# Test
wscat -c ws://localhost:7330 -s clasp
```

### Option 2: Cargo

```bash
cd deploy/relay
cargo run --release

# With all features
cargo run --release --features full
```

### Option 3: DigitalOcean App Platform

```bash
doctl auth init
doctl apps create --spec deploy/relay/digitalocean/app.yaml
```

## Architecture

```
Internet -> TLS (443) -> Load Balancer -> clasp-relay (7330)
                                      -> auth HTTP  (7350)
```

The relay runs a CLASP router that provides:
- CLASP v3 binary protocol over WebSocket
- State management with revisions
- Pattern-based subscriptions (`*`, `**`)
- Optional CPSK authentication with scoped permissions
- Optional capability token (Ed25519 delegatable) validation
- Optional entity registry with REST API
- Optional journal-based persistence and REPLAY queries
- Optional server-side rules engine
- Optional federation (multi-site state sync)

## Features

Features are opt-in via Cargo feature flags. Default features: `websocket`, `rendezvous`.

| Feature | Flag | What it enables |
|---------|------|-----------------|
| Journal | `journal` | SQLite/memory state persistence, REPLAY queries |
| Capabilities | `caps` | Delegatable Ed25519 capability tokens (`cap_` prefix) |
| Registry | `registry` | Persistent entity identity with REST API (`ent_` tokens) |
| Rules | `rules` | Server-side reactive automation (OnChange, OnThreshold, OnEvent, OnInterval) |
| Federation | `federation` | Multi-site state sync via leaf-hub topology |
| Full | `full` | All features enabled |

```bash
# Build with specific features
cargo build --release --features journal,rules

# Build with everything
cargo build --release --features full
```

## Configuration

### CLI Options

```
clasp-relay [OPTIONS]

Core:
  -p, --ws-port <PORT>         WebSocket listen port [default: 7330]
      --host <HOST>            Listen host [default: 0.0.0.0]
  -n, --name <NAME>            Server name [default: CLASP Relay]
  -v, --verbose                Enable verbose logging
      --max-sessions <N>       Maximum clients [default: 1000]
      --session-timeout <SEC>  Session timeout [default: 300]
      --no-websocket           Disable WebSocket

Protocols:
      --quic-port <PORT>       Enable QUIC (requires --cert and --key)
      --mqtt-port <PORT>       Enable MQTT server
      --mqtt-namespace <NS>    MQTT namespace prefix [default: /mqtt]
      --osc-port <PORT>        Enable OSC server
      --osc-namespace <NS>     OSC namespace prefix [default: /osc]
      --cert <PATH>            TLS certificate file (PEM)
      --key <PATH>             TLS private key file (PEM)

TTL:
      --param-ttl <SEC>        Parameter TTL [default: 3600]
      --signal-ttl <SEC>       Signal TTL [default: 3600]
      --no-ttl                 Disable all TTL expiration

Auth:
      --auth-port <PORT>       Auth HTTP server port (enables authentication)
      --auth-db <PATH>         Auth database path [default: relay-auth.db]
      --cors-origin <ORIGIN>   Allowed CORS origin(s), comma-separated

Persistence:
      --persist <PATH>         State snapshot file path
      --persist-interval <SEC> Snapshot interval [default: 30]

Rendezvous:
      --rendezvous-port <PORT> WAN discovery port [default: 7340]
      --rendezvous-ttl <SEC>   Device registration TTL [default: 300]

Journal (requires --features journal):
      --journal <PATH>         SQLite journal path
      --journal-memory         Use in-memory journal (ring buffer)

Capabilities (requires --features caps):
      --trust-anchor <PATH>    Trust anchor public key file (repeatable)
      --cap-max-depth <N>      Max delegation chain depth [default: 5]

Registry (requires --features registry):
      --registry-db <PATH>     SQLite entity registry database

Rules (requires --features rules):
      --rules <PATH>           JSON file containing rule definitions

App Config:
      --app-config <PATH>      Application config JSON (scopes, write rules, snapshot rules).
                               Auto-detects from /etc/clasp/ or ./config/ if not specified.

Federation (requires --features federation):
      --federation-hub <URL>   Hub WebSocket URL for leaf mode
      --federation-id <ID>     Local router identity
      --federation-namespace <PAT>  Owned namespace pattern (repeatable)
      --federation-token <TOK> Auth token for hub connection
```

### Examples

```bash
# Default (WebSocket only, no auth)
clasp-relay

# With authentication
clasp-relay --auth-port 7350

# With auth + journal persistence
clasp-relay --auth-port 7350 --journal ./journal.db

# With app-specific config (chat, etc.)
clasp-relay --auth-port 7350 --app-config config/chat.json

# With rules engine
clasp-relay --auth-port 7350 --rules ./rules.json

# With entity registry
clasp-relay --auth-port 7350 --registry-db ./registry.db

# With federation (leaf connecting to hub)
clasp-relay --federation-hub ws://hub:7330 --federation-namespace "/local/**"

# Multi-protocol with QUIC
clasp-relay --mqtt-port 1883 --osc-port 8000 --quic-port 7331 --cert cert.pem --key key.pem

# Kitchen sink
clasp-relay --auth-port 7350 --journal ./journal.db --registry-db ./registry.db \
  --rules ./rules.json --trust-anchor ./anchor.pub
```

### TTL Configuration

Parameters and signals expire after 1 hour (3600s) by default:

```bash
clasp-relay --param-ttl 300 --signal-ttl 300   # 5 minute TTL
clasp-relay --no-ttl                             # Disable TTL
```

### Multi-Protocol

When multiple protocols are enabled, they share the same router state:
- MQTT client publishing to `sensors/temp` is received by WebSocket subscribers on `/mqtt/sensors/**`
- OSC messages to `/synth/volume` reach subscribers on `/osc/synth/**`

### Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Log level: error, warn, info, debug, trace |

## Security

### Authentication

Enable with `--auth-port <PORT>`. The auth HTTP server provides:

- `POST /auth/register` -- Create user account (argon2 password hash)
- `POST /auth/login` -- Authenticate and receive a CPSK token
- `POST /auth/guest` -- Get a guest token with limited scopes

CPSK tokens use the format `cpsk_<uuid>` and carry scoped permissions (`action:pattern`):
- `read:/**` -- Subscribe and GET on all addresses
- `write:/lights/**` -- SET and PUBLISH on the lights namespace
- `admin:/**` -- Full access including registry API

### Capability Tokens

With `--features caps` and `--trust-anchor`, the relay accepts delegatable Ed25519 tokens (`cap_` prefix). Each delegation in the chain can only narrow scopes, never widen them. Works alongside CPSK tokens via `ValidatorChain`.

### Entity Registry

With `--features registry` and `--registry-db`, entities (devices, users, services) get persistent Ed25519 identities. Entity tokens (`ent_` prefix) are validated against the registry database.

### Registry REST API

The registry REST API is mounted on the auth HTTP server port. All endpoints require an admin CPSK token via `Authorization: Bearer <token>` header.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/entities` | Create entity |
| GET | `/api/entities` | List entities (?offset=0&limit=100) |
| GET | `/api/entities/{id}` | Get entity by ID |
| DELETE | `/api/entities/{id}` | Delete entity |
| PUT | `/api/entities/{id}/status` | Update entity status |

**Create entity request:**
```json
{
  "entity_type": "Device",
  "name": "My Sensor",
  "public_key": "<64 hex chars, Ed25519 public key>",
  "tags": ["sensor", "temperature"],
  "namespaces": ["/sensors/**"],
  "scopes": ["write:/sensors/**"]
}
```

**Response:** `201 Created` with entity JSON including the generated `clasp:` prefixed ID.

**Auth example:**
```bash
# Without token: 401 Unauthorized
curl -X GET http://localhost:7350/api/entities

# With admin token: 200 OK
curl -X GET http://localhost:7350/api/entities \
  -H "Authorization: Bearer cpsk_..."
```

## App Config

The `--app-config` flag loads a JSON file that defines application-specific behavior without writing Rust code:

- **Scopes** — per-user auth scope templates (e.g. `read:/chat/user/{userId}/**`)
- **Write rules** — declarative validation (who can write where, field checks, state lookups)
- **Snapshot transforms** — field redaction before delivery (e.g. strip password hashes)
- **Snapshot visibility** — owner-only paths, friendship checks, public sub-paths
- **Rate limits** — login/register attempt throttling

### Auto-detection

When `--app-config` is not specified, the relay checks these paths in order:

1. `/etc/clasp/*.json` — system/Docker install
2. `./config/*.json` — local dev (from the relay working directory)

If exactly one `.json` file is found, it is used automatically. If multiple files exist, the relay skips auto-detection and requires an explicit `--app-config` flag.

### Writing a config

See [`config/chat.json`](config/chat.json) for a complete example (the CLASP Chat app config). The schema supports these check types in write rules:

- `state_field_equals_session` — state lookup field must match session identity
- `state_not_null` — state must exist at a path
- `value_field_equals_session` — written value field must match session identity
- `segment_equals_session` — path segment must match session identity
- `either_state_not_null` — at least one of two state paths must exist
- `require_value_field` — written value must contain a field
- `reject_unless_path_matches` — reject writes not matching a sub-pattern

## Development

The production `Dockerfile` builds from published crates on crates.io. For local development, build directly with Cargo:

```bash
cd deploy/relay
cargo run --features full -- --auth-port 7350
```

## Connecting

### JavaScript

```javascript
import { ClaspBuilder } from '@clasp-to/core';

const client = await new ClaspBuilder('wss://relay.clasp.to')
  .name('My App')
  .connect();

client.set('/hello', 'world');
client.on('/hello', (value) => console.log(value));
```

### Rust

```rust
use clasp_client::Clasp;

let client = Clasp::connect("wss://relay.clasp.to").await?;
client.set("/hello", "world").await?;
client.subscribe("/hello", |value, _| println!("{:?}", value)).await?;
```

### Embedded (ESP32)

```rust
use clasp_embedded::{Client, Value};

let mut client = Client::new();
let frame = client.prepare_set("/sensor/temp", Value::Float(25.5));
websocket.send(frame);
```

## Monitoring

### Health Check

The server responds to any WebSocket connection attempt as healthy.

### Logs

```bash
docker logs clasp-relay -f
doctl apps logs <app-id> --follow
```

## Troubleshooting

### "Connection refused"

1. Check the relay is running: `docker ps`
2. Check the port is exposed: `docker port clasp-relay`
3. Check firewall rules

### "Upgrade failed"

WebSocket requires HTTP Upgrade header. Ensure your client uses `ws://` or `wss://`.

### Build fails on DigitalOcean

1. Check `source_dir` in app.yaml points to `deploy/relay`
2. Ensure Cargo.toml exists in deploy/relay/
3. Check build logs: `doctl apps logs <app-id> --type build`

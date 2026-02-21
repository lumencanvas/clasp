# clasp-cli

Command-line interface for CLASP protocol routers and connections.

## Installation

```bash
cargo install clasp-cli
```

Or build from source:

```bash
git clone https://github.com/lumencanvas/clasp.git
cd clasp
cargo install --path crates/clasp-cli
```

## Commands

### Start CLASP Router

**Important:** You need a CLASP router running before protocol connections can work.

```bash
# Start a CLASP router (required - central message hub)
clasp server --port 7330

# Start router with specific transport
clasp server --protocol websocket --bind 0.0.0.0 --port 7330
```

### Start Protocol Connections

**Note:** These commands create **protocol connections** that connect to the CLASP router. Each connection translates bidirectionally between its protocol and CLASP.

```bash
# Start an OSC connection (listens for OSC, routes to CLASP router)
clasp osc --port 9000

# Start an MQTT connection (connects to broker, routes to CLASP router)
clasp mqtt --host localhost --port 1883 --topic "sensors/#"

# Start a WebSocket connection
clasp websocket --mode server --url 0.0.0.0:8080

# Start an HTTP REST API connection
clasp http --bind 0.0.0.0:3000
```

**How it works:**
```
External Protocol ←→ Protocol Connection ←→ CLASP Router ←→ Other Connections/Clients
```

For example, `clasp osc --port 9000`:
- Listens for OSC messages on UDP port 9000
- Connects to CLASP router (default: localhost:7330)
- Translates OSC ↔ CLASP bidirectionally
- Routes through CLASP router to other clients/connections

### Publish/Subscribe

```bash
# Publish a value
clasp pub /lights/brightness 0.75

# Subscribe to an address pattern
clasp sub "/lights/**"
```

### Create Bridges

```bash
# Bridge OSC to MQTT
clasp bridge --source osc:0.0.0.0:9000 --target mqtt:localhost:1883
```

### Configuration

```bash
# Show current configuration
clasp info

# Start with config file
clasp server --config clasp.toml
```

## Key Management

Generate and inspect Ed25519 keypairs used for capability tokens and entity identity:

```bash
# Generate a new keypair (hex-encoded, saved with 0600 permissions)
clasp key generate --out root.key

# Show the public key
clasp key show root.key

# Show in did:key format
clasp key show root.key --format did
```

## Capability Token Commands

Create, delegate, inspect, and verify Ed25519 capability tokens (requires `caps` feature):

```bash
# Create a root token with admin access, valid for 30 days
clasp token cap create --key root.key --scopes "admin:/**" --expires 30d

# Delegate with narrower scopes
clasp token cap delegate <parent-token> --key child.key --scopes "write:/lights/**"

# Inspect a token (decode without validation)
clasp token cap inspect <token>

# Verify a token against a trust anchor
clasp token cap verify <token> --trust-anchor root.key
```

**Scope format:** `action:pattern` where action is `admin`, `write`, `read`, or a custom string, and pattern is a CLASP address with optional wildcards.

**Delegation rules:** Child tokens can only narrow scopes (never widen), and cannot outlive their parent token.

## Entity Token Commands

Generate entity keypairs and mint entity tokens (requires `registry` feature):

```bash
# Generate an entity keypair with metadata
clasp token entity keygen --out sensor.key --name "Sensor A" --type device

# Mint an entity token from a keypair
clasp token entity mint --key sensor.key

# Inspect an entity token
clasp token entity inspect <token>
```

**Entity ID format:** `clasp:<base58>` derived from the Ed25519 public key.

## Options

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Enable verbose logging |
| `--json` | Output in JSON format |
| `--config` | Path to configuration file |

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

---

Maintained by [LumenCanvas](https://lumencanvas.studio) | 2026

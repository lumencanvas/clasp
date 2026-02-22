---
title: CLASP CLI Reference
description: Complete clasp command-line reference
order: 3
---

# CLASP CLI Reference

Reference for the `clasp` CLI tool. Used for running a development router, launching protocol bridges, publishing/subscribing to signals, and managing keys and tokens.

The CLI is built from the `clasp-cli` crate in `crates/clasp-cli/`.

## Global Options

| Flag | Default | Description |
|------|---------|-------------|
| `-c`, `--config` | none | Configuration file path |
| `-l`, `--log-level` | `info` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |
| `--json-logs` | off | Output logs as JSON |

## clasp server

Start a development CLASP server.

```
clasp server [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-p`, `--protocol` | `quic` | Protocol to serve (`quic`, `tcp`, `websocket`) |
| `-b`, `--bind` | `0.0.0.0` | Bind address |
| `-P`, `--port` | `7331` | Port number |

In QUIC mode, a self-signed certificate is automatically generated for development. In WebSocket mode, the server uses the `clasp-bridge` WebSocket bridge internally.

```bash
# Start a QUIC dev server
clasp server

# Start a WebSocket dev server on port 8080
clasp server --protocol websocket --port 8080
```

## clasp osc

Start an OSC server that bridges OSC messages to CLASP signals.

```
clasp osc [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-p`, `--port` | `9000` | UDP port to listen on |
| `-b`, `--bind` | `0.0.0.0` | Bind address |

```bash
clasp osc --port 9000
```

## clasp mqtt

Connect to an MQTT broker and bridge messages to CLASP signals.

```
clasp mqtt [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-H`, `--host` | `localhost` | MQTT broker host |
| `-p`, `--port` | `1883` | MQTT broker port |
| `-c`, `--client-id` | auto-generated | Client ID for the MQTT connection |
| `-t`, `--topic` | `#` | Topics to subscribe to (supports MQTT wildcards, repeatable) |

```bash
clasp mqtt --host broker.local --topic "sensors/#" --topic "lights/#"
```

## clasp http

Start an HTTP REST API server that bridges HTTP requests to CLASP signals.

```
clasp http [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-b`, `--bind` | `0.0.0.0:3000` | Bind address (host:port) |
| `--base-path` | `/api` | Base path for API endpoints |
| `--cors` | `true` | Enable CORS headers |

Endpoints created:

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `{base}/signals` | List all signals |
| `GET` | `{base}/*path` | Get signal value |
| `PUT` | `{base}/*path` | Set signal value |
| `POST` | `{base}/*path` | Publish event |
| `GET` | `{base}/health` | Health check |

```bash
clasp http --bind 0.0.0.0:3000 --base-path /api
```

## clasp websocket

Start a WebSocket server or client.

```
clasp websocket [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-m`, `--mode` | `server` | Mode: `server` or `client` |
| `-u`, `--url` | `0.0.0.0:8080` | URL (ws://... for client) or bind address for server |

```bash
# Server mode
clasp websocket --mode server --url 0.0.0.0:8080

# Client mode
clasp websocket --mode client --url ws://router.local:7330
```

## clasp midi

Start a MIDI bridge (via the generic bridge command).

```
clasp bridge --bridge-type midi [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-o`, `--opt` | none | Configuration key=value pairs (repeatable) |

## clasp pub

Publish a value to a CLASP address.

```
clasp pub [OPTIONS] <ADDRESS> <VALUE>
```

| Flag | Default | Description |
|------|---------|-------------|
| `-s`, `--server` | `quic://localhost:7331` | CLASP server URL |

The value is parsed as JSON. If it is not valid JSON, it is treated as a string.

```bash
clasp pub /lights/main/brightness 0.75 --server quic://localhost:7331
clasp pub /cue/fire '{"scene":"intro"}' --server ws://router:7330
```

## clasp sub

Subscribe to a signal pattern and print received values.

```
clasp sub [OPTIONS] [PATTERN]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-s`, `--server` | `quic://localhost:7331` | CLASP server URL |
| `PATTERN` | `/**` | Address pattern to subscribe to |

```bash
clasp sub '/lights/**' --server ws://router:7330
clasp sub '/sensors/temperature/*'
```

## clasp bridge

Generic bridge launcher for protocol bridges.

```
clasp bridge --bridge-type <TYPE> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-b`, `--bridge-type` | required | Bridge type: `osc`, `midi`, `artnet`, `mqtt`, `websocket`, `http` |
| `-o`, `--opt` | none | Configuration key=value pairs (repeatable) |

For protocol-specific options, use the dedicated subcommands (`clasp osc`, `clasp mqtt`, `clasp http`, `clasp websocket`) instead.

## clasp key

Manage Ed25519 keypairs for capability and entity tokens.

### clasp key generate

Generate a new Ed25519 keypair. The signing key is hex-encoded (64 hex characters = 32 bytes).

```
clasp key generate [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-o`, `--out` | stdout | Output file path for the signing key |

If `--out` is specified, the signing key is written to the file with `0600` permissions (Unix). The public key is printed to stderr in both cases.

```bash
# Generate and save to file
clasp key generate --out mykey.sk

# Generate and pipe to another command
clasp key generate > mykey.sk
```

### clasp key show

Display the public key for a signing key file.

```
clasp key show [OPTIONS] <PATH>
```

| Flag | Default | Description |
|------|---------|-------------|
| `--format` | `hex` | Output format: `hex` or `did` (did:key multicodec) |

```bash
clasp key show mykey.sk
clasp key show mykey.sk --format did
```

## clasp token

Manage CPSK (pre-shared key) tokens. Tokens are stored in a local JSON file.

```
clasp token [OPTIONS] <SUBCOMMAND>
```

| Flag | Default | Description |
|------|---------|-------------|
| `--file` | `~/.config/clasp/tokens.json` | Token file path |

### clasp token create

Create a new CPSK token.

```
clasp token create [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-s`, `--scopes` | required | Scopes (comma-separated, e.g., `"read:/**,write:/lights/**"`) |
| `-e`, `--expires` | none (never) | Expiration duration (e.g., `7d`, `24h`, `30m`) |
| `--subject` | none | Subject/description for the token |

The generated token string (prefixed `cpsk_`) is printed to stdout.

```bash
clasp token create --scopes "read:/**,write:/lights/**" --expires 7d --subject "lighting-desk"
```

### clasp token list

List all stored CPSK tokens.

```
clasp token list [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--show-expired` | off | Include expired tokens in the listing |

### clasp token show

Show details of a specific token (by exact match or prefix).

```
clasp token show <TOKEN>
```

### clasp token revoke

Remove a token from the local store (by exact match or prefix).

```
clasp token revoke <TOKEN>
```

### clasp token prune

Remove all expired tokens from the local store.

```
clasp token prune
```

## clasp token cap

Capability token operations (delegatable Ed25519 tokens).

Requires: `--features caps`

### clasp token cap create

Create a new root capability token.

```
clasp token cap create [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-k`, `--key` | required | Path to signing key file |
| `-s`, `--scopes` | required | Scopes (comma-separated, e.g., `"admin:/**"`) |
| `-e`, `--expires` | `30d` | Expiration duration (e.g., `30d`, `24h`) |
| `--audience` | none | Audience public key (hex). Omit for bearer token. |

The encoded token string (prefixed `cap_`) is printed to stdout.

```bash
clasp token cap create --key root.sk --scopes "admin:/**" --expires 30d
```

### clasp token cap delegate

Delegate (attenuate) a capability token to create a child token.

```
clasp token cap delegate <PARENT_TOKEN> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-k`, `--key` | required | Path to child signing key file |
| `-s`, `--scopes` | required | Child scopes (must be a subset of parent scopes) |
| `-e`, `--expires` | `7d` | Expiration duration (clamped to parent's expiry) |
| `--audience` | none | Audience public key (hex) |

```bash
clasp token cap delegate "$ROOT_TOKEN" \
  --key child.sk \
  --scopes "write:/lights/**" \
  --expires 7d
```

### clasp token cap inspect

Inspect a capability token (decode without full verification).

```
clasp token cap inspect <TOKEN>
```

Displays version, issuer, audience, scopes, expiration, nonce, chain depth, delegation chain, and signature validity.

### clasp token cap verify

Verify a capability token against a trust anchor.

```
clasp token cap verify <TOKEN> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--trust-anchor` | required | Trust anchor public key file or hex string |
| `--max-depth` | `5` | Maximum delegation chain depth |

Exits with code 0 if valid, code 1 if expired/invalid.

```bash
clasp token cap verify "$TOKEN" --trust-anchor root.pub
```

## clasp token entity

Entity token operations (device/user/service identity tokens).

Requires: `--features registry`

### clasp token entity keygen

Generate a new entity keypair. This does not register the entity in the registry -- use the registry API to register the public key after generation.

```
clasp token entity keygen [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-o`, `--out` | stdout | Output key file path |
| `-n`, `--name` | none | Entity name (informational, printed to stderr) |
| `-t`, `--entity-type` | `device` | Entity type: `device`, `user`, `service`, `router` |

```bash
clasp token entity keygen --out sensor.sk --name "temp-sensor-1" --entity-type device
```

### clasp token entity mint

Mint an entity token from a keypair.

```
clasp token entity mint [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-k`, `--key` | required | Path to signing key file |

The encoded token string (prefixed `ent_`) is printed to stdout.

```bash
clasp token entity mint --key sensor.sk
```

### clasp token entity inspect

Inspect an entity token (decode without full verification).

```
clasp token entity inspect <TOKEN>
```

Displays entity ID, timestamp, signature length, and creation time.

## clasp info

Show version and system information, including platform, architecture, and supported protocols.

```
clasp info
```

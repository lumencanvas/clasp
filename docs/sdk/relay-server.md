---
title: Relay Server SDK
description: Programmatically manage CLASP relay servers from Node.js
order: 4
---

# Relay Server SDK

The `@clasp-to/relay` package lets you start, manage, and stop CLASP relay servers from Node.js. It spawns the `clasp-relay` Rust binary as a managed subprocess.

## Installation

```bash
npm install @clasp-to/relay
```

The `clasp-relay` binary must be available. Options:
- Install a platform package: `npm install @clasp-to/relay-darwin-arm64`
- Build from source: `cargo install clasp-relay`
- Add to PATH or set `CLASP_RELAY_BIN` env var

## Quick Start

```typescript
import { createRelay } from '@clasp-to/relay'

const relay = await createRelay({ port: 7330 })
console.log(`Relay at ${relay.url}`)  // ws://localhost:7330

// Use the relay...
await relay.stop()
```

`createRelay()` spawns the binary, waits for readiness, and returns a `RelayServer` instance.

## Configuration

### Config Object

Pass relay options directly:

```typescript
const relay = await createRelay({
  port: 7330,
  authPort: 7350,
  healthPort: 7360,
  name: 'Production Relay',
  persist: './state.db',
  verbose: true,
  maxSessions: 100,
  drainTimeout: 30,
})
```

### Builder Callback

Use the fluent `RelayBuilder` API for full control:

```typescript
const relay = await createRelay(r => r
  .port(7330)
  .authPort(7350)
  .healthPort(7360)
  .name('My Relay')
  .corsOrigin('https://app.example.com')
  .persist('./state.db', { interval: 30 })
  .tls('./cert.pem', './key.pem')
  .verbose()
)
```

See [Relay CLI Reference](../reference/relay-cli.md) for all configuration options.

### Inline App Config

Pass app config as an object -- it gets written to a temp file automatically:

```typescript
const relay = await createRelay({
  port: 7330,
  appConfig: {
    scopes: ['read:/**', 'write:/public/**'],
    rate_limits: { max_messages_per_second: 100 },
  },
})
```

## RelayServer API

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `url` | `string` | WebSocket URL (`ws://localhost:7330`) |
| `authUrl` | `string \| null` | Auth HTTP URL, or null |
| `healthUrl` | `string \| null` | Health check URL, or null |
| `pid` | `number` | OS process ID |
| `stopped` | `boolean` | Whether the process has exited |
| `exitCode` | `number \| null` | Exit code, or null if running |
| `exitSignal` | `string \| null` | Signal that killed the process |
| `logs` | `readonly string[]` | Last 500 log lines |
| `process` | `ChildProcess` | Underlying Node.js child process |

### Events

```typescript
relay.on('log', (line: string) => { })
relay.on('error', (err: Error) => { })
relay.on('exit', (code: number | null, signal: string | null) => { })
relay.on('ready', () => { })
```

### Lifecycle

```typescript
// Graceful stop: SIGTERM, then SIGKILL after timeout
await relay.stop()          // default 5s timeout
await relay.stop(10_000)    // custom timeout

// Force kill
relay.kill()
```

## Server Options

```typescript
createRelay(config, {
  binary: '/path/to/clasp-relay',  // explicit binary path
  cwd: '/tmp',                     // working directory
  env: { RUST_LOG: 'debug' },      // extra env vars
  readyTimeout: 15_000,            // readiness probe timeout (ms)
  inherit: false,                  // pipe stdio to parent process
})
```

## Binary Resolution

The binary is located automatically in this order:

1. `options.binary` -- explicit path passed to `createRelay()`
2. `CLASP_RELAY_BIN` -- environment variable
3. Platform npm package -- `@clasp-to/relay-darwin-arm64`, etc.
4. PATH lookup -- `which clasp-relay`

You can also use `resolveBinary()` directly:

```typescript
import { resolveBinary } from '@clasp-to/relay'

const path = resolveBinary()
console.log(`Found binary at: ${path}`)
```

## Health & Readiness

`createRelay()` automatically waits for the relay to become ready before returning. The probe strategy adapts to the configuration:

1. **Health port** configured: polls `GET /healthz`
2. **Auth port** configured: polls `GET /`
3. **WS-only**: TCP connect probe to the WebSocket port

You can use the probe functions directly:

```typescript
import { waitForHttp, waitForTcp } from '@clasp-to/relay'

await waitForHttp(7360, 'localhost', 10_000, '/healthz')
await waitForTcp(7330, 'localhost', 10_000)
```

## Use Cases

### Integration Testing

Spin up ephemeral relays for each test:

```typescript
import { createRelay } from '@clasp-to/relay'
import { afterEach, it, expect } from 'vitest'

let relay

afterEach(async () => {
  if (relay && !relay.stopped) await relay.stop(3000)
})

it('client connects and exchanges data', async () => {
  relay = await createRelay({ port: 0, drainTimeout: 1 })

  const client = await connect(relay.url)
  client.set('/test', 42)
  expect(await client.get('/test')).toBe(42)
})
```

### Dev Server

Start a relay alongside your dev workflow:

```typescript
const relay = await createRelay(r => r
  .port(7330)
  .authPort(7350)
  .verbose()
)

relay.on('log', line => console.log(`[relay] ${line}`))

process.on('SIGINT', async () => {
  await relay.stop()
  process.exit(0)
})
```

### Multiple Relays

Run several relays for multi-region or multi-tenant testing:

```typescript
const [us, eu] = await Promise.all([
  createRelay({ port: 7330, name: 'US-East' }),
  createRelay({ port: 7331, name: 'EU-West' }),
])

console.log(us.url)  // ws://localhost:7330
console.log(eu.url)  // ws://localhost:7331
```

## Next Steps

- [JavaScript SDK](javascript.md) -- client-side API
- [Relay CLI Reference](../reference/relay-cli.md) -- all relay configuration options
- [Deployment](../deployment/README.md) -- production deployment guides
- [Auth & Security](../auth/README.md) -- authentication and E2E encryption

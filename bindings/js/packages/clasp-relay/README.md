# @clasp-to/relay

Programmatic CLASP relay server for Node.js. Spawn and manage `clasp-relay` processes with a clean API.

## Install

```bash
npm install @clasp-to/relay
```

Requires `clasp-relay` binary on PATH, or install a platform package:
```bash
npm install @clasp-to/relay-darwin-arm64  # macOS Apple Silicon
npm install @clasp-to/relay-linux-x64     # Linux x64
```

## Quick Start

```typescript
import { createRelay } from '@clasp-to/relay'

const relay = await createRelay({ port: 7330 })
console.log(`Relay running at ${relay.url}`)

// ... use the relay ...

await relay.stop()
```

## Builder API

Use the full `RelayBuilder` fluent API via callback:

```typescript
const relay = await createRelay(r => r
  .port(7330)
  .authPort(7350)
  .name('My Relay')
  .persist('./state.db')
  .verbose()
)
```

## Server API

```typescript
relay.url        // ws://localhost:7330
relay.authUrl    // http://localhost:7350 (or null)
relay.pid        // OS process ID
relay.stopped    // boolean
relay.logs       // recent log lines (up to 200)

relay.on('log', line => console.log(line))
relay.on('error', err => console.error(err))
relay.on('exit', (code, signal) => { ... })

await relay.stop()   // SIGTERM + graceful wait
relay.kill()         // SIGKILL (force)
```

## Options

```typescript
const relay = await createRelay(config, {
  binary: '/path/to/clasp-relay',  // explicit binary path
  cwd: '/tmp',                     // working directory
  env: { RUST_LOG: 'debug' },      // extra env vars
  readyTimeout: 15_000,            // readiness timeout (ms)
  inherit: false,                  // pipe stdio to parent
})
```

## Binary Resolution

The binary is found in this order:
1. `options.binary` (explicit path)
2. `CLASP_RELAY_BIN` environment variable
3. Platform-specific npm package (`@clasp-to/relay-<platform>`)
4. PATH lookup (`which clasp-relay`)

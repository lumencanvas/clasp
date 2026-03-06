# Handoff - 2026-03-06: @clasp-to/relay — Node.js Relay Server SDK

## Overview

Implemented `@clasp-to/relay` — a Node.js package that spawns and manages `clasp-relay` Rust binary processes with a clean programmatic API. Paired with `@clasp-to/sdk` additions (`toArgs()`, `healthPort()`).

**Status: All changes uncommitted on branch `update-bridge-app-1`.** 72 relay tests + 294 SDK tests pass (366 total).

---

## Package: `bindings/js/packages/clasp-relay/`

### New Files

| File | Purpose | Lines |
|------|---------|-------|
| `package.json` | @clasp-to/relay v0.1.0, deps: @clasp-to/sdk | ~50 |
| `tsconfig.json` | TS config, target ES2020, strict | ~15 |
| `vitest.config.ts` | Vitest with SDK source alias for dev | ~15 |
| `src/index.ts` | Public exports | ~4 |
| `src/types.ts` | RelayServerOptions, RelayEvents, ConfigInput | ~25 |
| `src/binary.ts` | 4-step binary resolution (explicit, env, platform pkg, PATH) | ~95 |
| `src/health.ts` | 3-tier readiness: HTTP health, HTTP auth, TCP probe + port parser | ~128 |
| `src/server.ts` | RelayServer class + createRelay() factory | ~315 |
| `tests/binary.test.ts` | 8 binary resolution tests | ~60 |
| `tests/health.test.ts` | 11 parsePortFromOutput tests | ~48 |
| `tests/server.test.ts` | 32 mocked unit tests (spawn, lifecycle, config-to-args) | ~350 |
| `tests/integration.test.ts` | 21 real integration tests against clasp-relay binary | ~434 |
| `README.md` | Quick start, builder API, server API, options, binary resolution | ~80 |

### Modified Files in @clasp-to/sdk

| File | Changes |
|------|---------|
| `src/relay.ts` | Added `toArgs()` method (~90 lines), `healthPort()` fluent method, `--health-port` flag |
| `src/types.ts` | Added `healthPort?: number` to RelayConfig |
| `tests/relay.test.ts` | Added 24 tests for toArgs() and healthPort (56 -> 80 tests) |

---

## API Surface

### Creating a Relay

```typescript
import { createRelay } from '@clasp-to/relay'

// Config object
const relay = await createRelay({ port: 7330, authPort: 7350 })

// Builder callback (full fluent API from @clasp-to/sdk)
const relay = await createRelay(r => r
  .port(7330).authPort(7350).name('My Relay').verbose()
)
```

### RelayServer Properties & Methods

```typescript
relay.url          // ws://localhost:7330
relay.authUrl      // http://localhost:7350 | null
relay.healthUrl    // http://localhost:7360 | null
relay.pid          // OS process ID
relay.stopped      // boolean
relay.exitCode     // number | null
relay.exitSignal   // string | null
relay.logs         // readonly string[] (last 500 lines)
relay.process      // ChildProcess

relay.on('log', line => {})
relay.on('error', err => {})
relay.on('exit', (code, signal) => {})
relay.on('ready', () => {})

await relay.stop()     // SIGTERM, wait for drain, SIGKILL after timeout
relay.kill()           // Immediate SIGKILL
```

### Options

```typescript
createRelay(config, {
  binary: '/path/to/clasp-relay',  // skip auto-detection
  cwd: '/tmp',
  env: { RUST_LOG: 'debug' },
  readyTimeout: 15_000,
  inherit: false,                  // pipe stdio to parent
})
```

---

## Architecture Decisions

1. **Managed subprocess**: Spawn `clasp-relay` binary via `child_process.spawn`, not NAPI bindings. Zero Rust build complexity, works with existing binary.
2. **3-tier readiness detection**: Health port HTTP `/healthz` > Auth port HTTP > TCP probe to WS port. Adapts to whatever ports are configured.
3. **Temp file materialization**: Inline `appConfig({...})` and `rules({...})` objects written to temp dir, paths passed as CLI flags. Cleaned up on process exit.
4. **Port 0 auto-detection**: Parses relay stdout for "listening on", "WebSocket:", "ws port:" patterns to discover the actual bound port.
5. **toArgs() vs toCommand()**: `toArgs()` returns `string[]` for `spawn()` (no shell quoting issues). `toCommand()` returns formatted string for display. Kept independent to avoid breaking each other.

## Bugs Found & Fixed During Audit

1. **stop() resolves before exit**: Timeout handler called `resolve()` after `kill()`. Fixed: only `onExit` listener resolves.
2. **Health check polled WS port via HTTP**: WS-only mode failed readiness. Fixed: added TCP probe as fallback.
3. **30s relay drain timeout in tests**: Tests timed out. Fixed: `drainTimeout: 1` on all test configs.
4. **Auth username collisions**: Same test usernames across runs. Fixed: unique names with `Date.now()` + counter.
5. **earlyExit listener leak**: `server.once('exit', ...)` not cleaned up on success. Fixed: readyPromise.then removes listener.
6. **URL host mismatch**: Tests expected `localhost` but code produced `127.0.0.1`. Fixed: map both to `localhost`.
7. **Port detection path wrong**: `__dirname` needed 5 levels up, not 4. Fixed.
8. **Missing healthUrl/exitSignal/process getters**: Added to RelayServer class.

---

## Verification

```bash
cd bindings/js/packages/clasp-relay

# Unit tests (no binary needed)
npx vitest run tests/binary.test.ts tests/health.test.ts tests/server.test.ts

# Integration tests (requires clasp-relay binary)
npx vitest run tests/integration.test.ts

# All tests
npx vitest run
```

## Not Implemented (Future)

- **Platform binary npm packages**: `@clasp-to/relay-darwin-arm64` etc. (Phase 2)
- **Federation configuration**: RelayBuilder supports federation flags but no dedicated federation helper
- **Cluster mode**: Multiple relay processes behind load balancer
- **Binary auto-download**: Fetch binary from releases if not installed
- **Windows support**: `where` instead of `which`, different signal handling

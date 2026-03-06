# Handoff - 2026-03-06: @clasp-to/sdk — Human-Level Abstraction

## Overview

Implemented `@clasp-to/sdk` — a single JS/TS package that wraps `@clasp-to/core` and `@clasp-to/crypto` to make every CLASP feature accessible in 1-5 lines of human-readable code. Also added `RelayBuilder` for programmatic relay/router configuration generation.

**Status: All changes uncommitted on branch `update-bridge-app-1`.** 92 unit tests pass, 17 integration tests require a running router.

---

## Package: `bindings/js/packages/clasp-sdk/`

### New Files

| File | Purpose | Lines |
|------|---------|-------|
| `package.json` | @clasp-to/sdk v4.1.0, deps: @clasp-to/core + @clasp-to/crypto, build: tsup | ~30 |
| `tsconfig.json` | TS config matching core package | ~15 |
| `src/index.ts` | Re-exports all public API | ~40 |
| `src/easy.ts` | `clasp()` entry point + `EasyClient` class | ~310 |
| `src/device.ts` | `Device` + `CredentialBundle` classes (register/provision/revoke) | ~215 |
| `src/room.ts` | `Room` class — encrypted group wrapper over CryptoClient + E2ESession | ~90 |
| `src/bridge.ts` | `BridgeCommand` — CLI/Docker/env generation for protocol bridges | ~110 |
| `src/discovery.ts` | `discover()` and `watch()` wrappers | ~115 |
| `src/rules.ts` | `buildRuleJSON()` — programmatic rules builder | ~110 |
| `src/relay.ts` | `RelayBuilder` — fluent builder for relay CLI/Docker/env config | ~370 |
| `src/duration.ts` | `parseDuration()`, `parseDurationToSeconds()`, `parseDurationToWholeSeconds()` | ~25 |
| `src/types.ts` | All public types (ClaspOptions, AppConfig, RelayConfig, etc.) | ~200 |
| `tests/easy.test.ts` | 51 tests: EasyClient, inferAuthUrl, parseDuration, lifecycle | ~300 |
| `tests/room.test.ts` | 10 tests: Room creation, password rooms, key rotation, close | ~120 |
| `tests/relay.test.ts` | 31 tests: CLI command, Docker Compose, env vars, app config JSON | ~335 |
| `tests/integration.test.ts` | 17 tests: real router connection, pub/sub, auth (skippable) | ~370 |
| `README.md` | Full documentation with examples for every feature | ~200 |

---

## API Surface

### Entry Point
```typescript
import clasp from '@clasp-to/sdk'
const c = await clasp('ws://localhost:7330')              // anonymous
const c = await clasp('ws://localhost:7330', { name: 'My App', encrypted: true })
```

### Core Operations
```typescript
c.set('/path', value)           // persistent parameter
c.get('/path')                  // Promise<Value>
c.on('/path/**', cb)            // subscribe (returns unsubscribe fn)
c.emit('/path', payload)        // fire-and-forget event
c.stream('/path', data)         // high-rate streaming
c.bundle([...])                 // atomic multi-operation
c.cached('/path')               // last known value
c.time()                        // server time (microseconds)
```

### Auth & Devices
```typescript
const device = await c.register({ name: 'Sensor', scopes: ['write:/data/**'] })
const guest = await c.guest({ scopes: ['read:/**'] })
const me = await c.login({ username, password })
const child = await device.createChild({ name: 'Sub', scopes: ['write:/data/sub/**'] })
const creds = await device.provision({ name: 'IoT', scopes: [...], expires: '30d' })
const conn = await guest.connect()
```

### Encrypted Rooms
```typescript
const room = await c.room('/chat/private')
const room = await c.room('/chat/secret', { password: 'shhh', rotateKeys: '1h' })
room.set('/chat/private/msg', { text: 'hello' })  // auto-encrypted
room.on('/chat/private/**', cb)                     // auto-decrypted
```

### Bridges
```typescript
const osc = c.bridge('osc', { port: 9000 })
osc.command    // CLI string
osc.toDockerCompose()
```

### Rules
```typescript
c.rule('high-temp', {
  when: '/sensors/temp', above: 30,
  then: [{ set: ['/hvac/fan', true] }],
  cooldown: '60s',
})
```

### Relay Builder
```typescript
import { RelayBuilder } from '@clasp-to/sdk'
const relay = new RelayBuilder()
  .port(7330).authPort(7350)
  .corsOrigin('https://app.example.com')
  .persist('./state.db', { interval: 30 })
  .mqtt(1883).tls('./cert.pem', './key.pem')

relay.toCommand()         // CLI command string
relay.toDockerCompose()   // Docker Compose YAML
relay.toEnv()             // environment variables
relay.toAppConfigJSON()   // app config as JSON
```

RelayBuilder supports all relay flags: port, host, name, authPort, corsOrigin, adminTokenPath, tokenTtl, maxSessions, sessionTimeout, paramTtl, signalTtl, noTtl, verbose, persist, journal, tls, appConfig, rules, capabilityTokens, entityRegistry, mqtt, osc, quic, federation, rendezvous, drainTimeout.

### Discovery
```typescript
import { discover, watch } from '@clasp-to/sdk'
const routers = await discover()
watch((event) => { /* found/lost */ })
```

---

## Key Design Decisions

1. **Single default export**: `import clasp from '@clasp-to/sdk'` — function IS the entry point
2. **Everything returns promises**: No builders or connection states to manage
3. **Smart defaults**: Reconnect on, auth URL auto-inferred (port+20), encryption opt-in
4. **Device hierarchy via token delegation**: `createChild()` narrows scopes
5. **Rooms abstract E2E**: No key stores, ECDH, TOFU to think about
6. **Duration strings**: `'1h'`, `'30s'`, `'5m'` instead of milliseconds
7. **Auth URL inference**: `ws://host:7330` → `http://host:7350` (preserves path)
8. **RelayBuilder keeps flag+value together**: Each CLI flag+value is one string in parts array

## Bugs Found & Fixed During Audit

1. **Encrypted mode routing**: `set()`/`emit()` now route through CryptoClient when `encrypted: true`
2. **Duplicated duration parser**: Extracted to shared `src/duration.ts`
3. **Auth URL path loss**: `inferAuthUrl()` now preserves URL pathname
4. **Room race condition**: `roomPending` Map prevents duplicate concurrent room creation
5. **Missing API wrappers**: Added `getSignals()`, `getLastError()`, `clearError()`, `onReconnect()`, `inner`
6. **Rules builder gaps**: Added `onEvent` trigger, operator conditions, `setFrom`/`delay` actions, `name`/`enabled`
7. **Relay toCommand() format**: Changed from separate parts (`'--flag', 'value'`) to joined (`'--flag value'`) to keep pairs together in multiline output

---

## Verification

```bash
cd bindings/js/packages/clasp-sdk

# Type-check
npx tsc --noEmit

# Unit tests (92 pass)
SKIP_INTEGRATION=1 npx vitest run

# Integration tests (requires running router on localhost:7330)
npx vitest run

# Integration with auth (requires --auth-port 7350)
CLASP_AUTH_URL=http://localhost:7350 npx vitest run
```

## Not Implemented

- **P2P**: `c.p2p()` — mentioned in plan, deferred (requires P2PManager from core)
- **Auto-discovery connect**: `await clasp()` with no URL — discovery + auto-connect
- **Rules API endpoint**: POST rules to relay (generates JSON only, no HTTP endpoint)
- **Bridge process spawning**: Returns CLI command strings only, no child_process
- **provisionBatch**: Bulk device provisioning (single provision works)

---
title: Easy Client SDK
description: Human-friendly CLASP client for JavaScript and TypeScript
order: 2
---

# Easy Client SDK

The `@clasp-to/sdk` package wraps `@clasp-to/core` with a human-friendly API. Devices, rooms, rules, bridges, and discovery -- all in 1-5 lines.

If `@clasp-to/core` is the driver, `@clasp-to/sdk` is the car.

## Installation

```bash
npm install @clasp-to/sdk
```

## Connect

```typescript
import clasp from '@clasp-to/sdk'

// Anonymous
const c = await clasp('ws://localhost:7330')

// Named with auth
const c = await clasp('ws://localhost:7330', {
  name: 'Light Controller',
  token: 'cpsk_...',
})

// With E2E encryption on everything
const c = await clasp('ws://localhost:7330', { encrypted: true })
```

## Pub/Sub

```typescript
await c.set('/lights/brightness', 0.8)
await c.get('/lights/brightness')     // -> 0.8

c.on('/lights/**', (val, addr) => console.log(addr, val))

// subscribe() is an alias for on()
c.subscribe('/sensors/**', (val, addr) => {}, { maxRate: 30, epsilon: 0.01 })

await c.emit('/cues/go', { scene: 'Act 2' })

c.stream('/sensors/accel', { x: 0.1, y: 0.2, z: 9.8 })
```

Both `set()` and `emit()` return `Promise<void>`. In encrypted mode the value is encrypted before sending. Always `await` these calls to catch encryption errors.

### `get()` vs `cached()`

- `get(address)` fetches the current value from the server (returns a Promise)
- `cached(address)` returns the last value received via subscription (synchronous, may be `undefined`)

### Atomic Bundles

```typescript
c.bundle([
  { set: ['/a', 1] },
  { emit: ['/b', 'go'] },
])
```

## Devices

Register devices, create child credentials, and provision firmware:

```typescript
const device = await c.register({
  name: 'Living Room Lights',
  scopes: ['write:/lights/living-room/**', 'read:/**'],
})

// Create a child with narrower scopes
const child = await device.createChild({
  name: 'Dimmer Switch',
  scopes: ['write:/lights/living-room/dimmer'],
})

// Connect as that device
const dimmer = await child.connect()
await dimmer.set('/lights/living-room/dimmer', 0.5)

// Provision credentials for firmware
const creds = await device.provision({
  name: 'Kitchen Sensor',
  scopes: ['write:/sensors/kitchen/**'],
  expires: '30d',
})
creds.toJSON()   // for QR code or config file
creds.toEnv()    // CLASP_URL=... CLASP_TOKEN=...
creds.connect()  // connect immediately

// Bulk provision
const batch = await device.provisionBatch([
  { name: 'Sensor 1', scopes: ['write:/sensors/1/**'] },
  { name: 'Sensor 2', scopes: ['write:/sensors/2/**'] },
])
```

## Encrypted Rooms

Create E2E encrypted rooms with optional password protection, key rotation, and TOFU verification:

```typescript
const room = await c.room('/chat/private')
await room.set('/chat/private/messages/1', { text: 'hello', from: 'alice' })
room.on('/chat/private/messages/**', (msg) => console.log(msg))

// Password-protected
const room = await c.room('/chat/secret', { password: 'shhh' })

// Auto-rotating keys
const room = await c.room('/chat/secure', { rotateKeys: '1h' })

// TOFU key change callback
const room = await c.room('/chat/verified', {
  onKeyChange: (peerId, oldFP, newFP) => {
    console.warn(`Key changed for ${peerId}`)
    return true // accept the new key
  },
})

// Cleanup
c.destroyRoom('/chat/private')
```

See [E2E Encryption](../auth/e2e-encryption.md) for the full protocol description, key exchange flow, and TOFU verification.

## Bridges

Generate bridge commands for any supported protocol:

```typescript
const osc = c.bridge('osc', { port: 9000 })
osc.command           // 'clasp bridge osc --router ws://...'
osc.toDockerCompose() // Docker service YAML
osc.toEnv()           // env var format

const mqtt = c.bridge('mqtt', {
  broker: 'mqtt://localhost:1883',
  topics: ['sensors/#'],
})

const dmx = c.bridge('dmx', { serial: '/dev/ttyUSB0' })
const artnet = c.bridge('artnet', { universe: 3, subnet: 1 })
```

## Rules

Define server-side reactive automation:

```typescript
c.rule('high-temp-alert', {
  when: '/sensors/temp',
  above: 30,
  if: { '/system/alerts-enabled': true },
  then: [
    { set: ['/hvac/fan', true] },
    { emit: ['/alerts/high-temp', { msg: 'Too hot!' }] },
  ],
  cooldown: '60s',
})

c.rule('heartbeat', {
  every: '30s',
  then: { emit: ['/system/heartbeat', { alive: true }] },
})

// Session lifecycle triggers
c.rule('welcome', {
  onSessionJoin: '/room/**',
  then: { emit: ['/room/welcome', { msg: 'Hello!' }] },
})
```

### Condition Operators

- `{ '/path': value }` -- equality check
- `{ '/path': { gt: 10 } }` -- greater than
- `{ '/path': { lt: 5 } }` -- less than
- `{ '/path': { gte: 0, lte: 100 } }` -- range
- `{ '/path': { ne: 'off' } }` -- not equal

### Transform Types

`identity`, `scale`, `clamp`, `threshold`, `invert`, `map`, `round`, `abs`

```typescript
c.rule('convert', {
  when: '/sensors/temp',
  then: {
    setFrom: ['/display/temp-f'],
    transform: { type: 'scale', factor: 1.8, offset: 32 },
  },
})
```

## Discovery

Find routers on the network:

```typescript
import { discover, discoverLocal, watch } from '@clasp-to/sdk'

const routers = await discover()
// [{ name: 'Local Router', url: 'ws://localhost:7330' }]

// Probe localhost ports 7330-7339
const local = await discoverLocal({ timeout: 2000 })

// Watch for routers appearing/disappearing
const stop = watch((event) => {
  if (event.type === 'found') console.log('Found:', event.name)
  if (event.type === 'lost') console.log('Lost:', event.name)
}, { rendezvousUrl: 'http://rendezvous:7340', timeout: 5000 })

stop()
```

## RelayBuilder

Generate deployment configs without running a relay:

```typescript
import { RelayBuilder } from '@clasp-to/sdk'

const relay = new RelayBuilder()
  .port(7330)
  .authPort(7350)
  .corsOrigin('https://app.example.com')
  .persist('./state.db', { interval: 30 })
  .mqtt(1883)
  .appConfig({ scopes: ['read:/**'] })
  .logLevel('info')

relay.toCommand()         // CLI command string
relay.toDockerCompose()   // Docker Compose YAML
relay.toEnv()             // env var format
relay.toKubernetes()      // K8s Deployment + Service YAML
relay.toSystemd()         // systemd unit file
```

See the [RelayBuilder API](../sdk/relay-server.md) for full builder method reference.

## Auth Helpers

```typescript
const me = await c.login({ username: 'alice', password: 'secret' })
const guest = await c.guest({ scopes: ['read:/**'] })
await device.revoke(child.id)
const children = await device.children()
```

## Type Exports

All types are re-exported from the SDK:

```typescript
import type {
  Value, SubscriptionCallback, Unsubscribe, SignalDefinition,
  ConnectOptions, SubscribeOptions, TimelineKeyframe,
  ClaspOptions, RegisterOptions, RoomOptions,
  RuleDefinition, RuleAction, RuleTransform,
  BridgeProtocol, BridgeOptions, RelayConfig,
} from '@clasp-to/sdk'

import { QoS } from '@clasp-to/sdk'
```

## Examples

Working examples in [`examples/js/`](https://github.com/lumencanvas/clasp/tree/main/examples/js):

| File | Description |
|------|-------------|
| `easy-client.js` | Connect, pub/sub, devices, rooms, rules, bridges |
| `easy-client-devices.js` | Device registration, children, provisioning |
| `easy-client-rooms.js` | Encrypted rooms, passwords, key rotation, TOFU |

## Next Steps

- [JavaScript SDK (Core)](javascript.md) -- lower-level `@clasp-to/core` API
- [Relay Server SDK](relay-server.md) -- manage relay servers from Node.js
- [E2E Encryption](../auth/e2e-encryption.md) -- encryption protocol details
- [Auth](../auth/README.md) -- CPSK tokens and capability delegation

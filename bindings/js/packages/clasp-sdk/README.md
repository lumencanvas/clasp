# @clasp-to/sdk

Human-level SDK for CLASP. Devices, rooms, rules, bridges -- all in 1-5 lines.

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

## Publish & Subscribe

```typescript
// set() and emit() are async (important for encrypted mode)
await c.set('/lights/brightness', 0.8)
await c.get('/lights/brightness')     // -> 0.8

c.on('/lights/**', (val, addr) => console.log(addr, val))

// subscribe() is an alias for on()
c.subscribe('/sensors/**', (val, addr) => {}, { maxRate: 30, epsilon: 0.01 })

await c.emit('/cues/go', { scene: 'Act 2' })

c.stream('/sensors/accel', { x: 0.1, y: 0.2, z: 9.8 })

// Atomic bundle
c.bundle([
  { set: ['/a', 1] },
  { emit: ['/b', 'go'] },
])
```

### `set()` and `emit()` are async

Both `set()` and `emit()` return `Promise<void>`. In encrypted mode, the value
must be encrypted before sending. In non-encrypted mode, the promise resolves
immediately. Always `await` these calls to catch encryption errors.

### `get()` vs `cached()`

- `get(address)` fetches the current value from the server (returns a Promise)
- `cached(address)` returns the last value received via subscription (synchronous, may be `undefined`)

## Subscribe Options

The `on()` and `subscribe()` methods accept optional `SubscribeOptions`:

```typescript
c.on('/sensors/temp', callback, {
  maxRate: 10,     // Max 10 updates/second
  epsilon: 0.1,    // Ignore changes smaller than 0.1
})
```

## Devices

```typescript
// Register a device
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

// Selective cleanup
c.destroyRoom('/chat/private')
```

## Bridges

```typescript
const osc = c.bridge('osc', { port: 9000 })
osc.command           // 'clasp bridge osc --router ws://...'
osc.toDockerCompose() // Docker service YAML
osc.toEnv()           // env var format

const mqtt = c.bridge('mqtt', {
  broker: 'mqtt://localhost:1883',
  topics: ['sensors/#'],
})

// With auth token
const authed = c.bridge('osc', { port: 9000, token: 'cpsk_...' })

// DMX with serial device
const dmx = c.bridge('dmx', { serial: '/dev/ttyUSB0' })

// ArtNet with universe/subnet
const artnet = c.bridge('artnet', { universe: 3, subnet: 1 })
```

## Rules

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

- `{ '/path': value }` - equality check
- `{ '/path': { gt: 10 } }` - greater than
- `{ '/path': { lt: 5 } }` - less than
- `{ '/path': { gte: 0, lte: 100 } }` - range (use separate entries)
- `{ '/path': { ne: 'off' } }` - not equal

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
  if (event.type === 'error') console.error('Poll error:', event.error)
}, { rendezvousUrl: 'http://rendezvous:7340', timeout: 5000 })

// Stop watching
stop()
```

## Relay Builder

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
relay.toAppConfigJSON()   // app config JSON string
relay.toRulesJSON()       // rules JSON string
```

### Builder Methods

| Method | Description |
|--------|------------|
| `port(n)` | WebSocket port (validates 0-65535) |
| `authPort(n)` | Auth HTTP port |
| `host(s)` | Bind address |
| `name(s)` | Relay name |
| `corsOrigin(s)` | CORS origins (string or array) |
| `verbose()` | Enable verbose logging |
| `logLevel(l)` | Set log level (error/warn/info/debug/trace) |
| `persist(path, opts?)` | Enable persistence |
| `journal(path, opts?)` | Enable journaling |
| `tls(cert, key)` | TLS config |
| `mqtt(port, opts?)` | Enable MQTT |
| `osc(port, opts?)` | Enable OSC |
| `quic(port, cert, key)` | Enable QUIC |
| `federation(opts)` | Configure federation |
| `rendezvous(opts?)` | Enable discovery |
| `appConfig(config)` | Set app config |
| `rules(rules)` | Set rules |
| `merge(other)` | Merge another builder |
| `fromConfig(config)` | Static: create from config object |

## Auth Helpers

```typescript
// Login
const me = await c.login({ username: 'alice', password: 'secret' })

// Guest access
const guest = await c.guest({ scopes: ['read:/**'] })

// Revoke a device
await device.revoke(child.id)

// List children
const children = await device.children()
```

## Error Handling

All HTTP operations (register, login, guest, createChild, revoke, children)
include timeouts and descriptive error messages with HTTP status codes:

```typescript
try {
  await c.register({ name: 'Test' })
} catch (err) {
  // "Registration failed (409): Already exists"
  console.error(err.message)
}
```

Non-JSON responses (502 proxy pages, etc.) are caught with clear error messages.

## Type Exports

All types are re-exported from the SDK for convenience:

```typescript
import type {
  Value, SubscriptionCallback, Unsubscribe, SignalDefinition,
  ConnectOptions, SubscribeOptions, TimelineKeyframe,
  ClaspOptions, RegisterOptions, RoomOptions,
  RuleDefinition, RuleAction, RuleTransform,
  BridgeProtocol, BridgeOptions, RelayConfig,
} from '@clasp-to/sdk'

import { QoS } from '@clasp-to/sdk'
// QoS.Fire, QoS.Confirm, QoS.Commit
```

## API Reference

### `clasp(url, options?)`
Connect to a router. Returns `Promise<EasyClient>`.

Options: `name`, `token`, `encrypted`, `reconnect`, `authUrl`

### `EasyClient`
- `set(address, value)` - persistent param (async)
- `get(address)` - server fetch (async)
- `on(pattern, callback, options?)` - subscribe
- `subscribe(pattern, callback, options?)` - alias for on()
- `emit(address, payload?)` - one-shot event (async)
- `stream(address, value)` - high-rate data
- `gesture(address, id, phase, payload?)` - touch/pen input
- `timeline(address, keyframes, options?)` - animation
- `bundle(messages, options?)` - atomic batch
- `register(options)` - register device
- `login(options)` - authenticate
- `guest(options?)` - guest session
- `room(basePath, options?)` - encrypted room
- `destroyRoom(basePath)` - destroy a specific room
- `bridge(protocol, options?)` - bridge command
- `rule(id, definition)` - define rule
- `close()` - disconnect

### `Device`
- `id`, `token`, `name`, `scopes`
- `createChild(options)` - narrower scopes
- `connect()` - connect as this device
- `provision(options)` - credential bundle
- `provisionBatch(devices)` - bulk provision
- `revoke(childId)` - revoke access
- `children()` - list children

### `CredentialBundle`
- `token`, `url`, `name`, `scopes`, `expires`
- `toJSON()` - JSON string
- `toEnv()` - env vars
- `connect()` - connect as device

### `Room`
- `set(address, value)` - encrypted set (async)
- `emit(address, payload?)` - encrypted emit (async)
- `on(pattern, callback)` - auto-decrypt subscribe
- `rotateKey()` - manual key rotation
- `removePeer(id)` - remove peer
- `destroy()` - cleanup

### `BridgeCommand`
- `command` - CLI string
- `toDockerCompose()` - Docker YAML
- `toEnv()` - env vars

### `RelayBuilder`
- All builder methods (see table above)
- `toCommand()` - CLI string
- `toDockerCompose(options?)` - Docker YAML
- `toEnv()` - env vars
- `toKubernetes(options?)` - K8s YAML
- `toSystemd(options?)` - systemd unit
- `toAppConfigJSON()` - app config
- `toRulesJSON()` - rules config

### `discover(options?)`
Find routers on the network.

### `discoverLocal(options?)`
Probe localhost ports 7330-7339.

### `watch(callback, options)`
Watch for routers appearing/disappearing.

### `buildRuleJSON(id, definition)`
Convert human-readable rule to JSON schema.

### `parseDuration(s)`
Parse duration string ('30s', '5m', '1h') to milliseconds.

## Running Tests

```bash
cd bindings/js/packages/clasp-sdk

# Unit tests only
SKIP_INTEGRATION=1 npx vitest run

# With integration tests (requires: clasp-relay --auth-port 7350)
npx vitest run

# With coverage
SKIP_INTEGRATION=1 npx vitest run --coverage
```

## License

MIT

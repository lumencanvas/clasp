# Language Bindings Analysis

## Overview

CLASP provides official bindings for JavaScript/TypeScript and Python with full feature parity.

---

## JavaScript/TypeScript (@clasp-to/core)

### Package Info

**Location:** `bindings/js/packages/clasp-core/`
**Version:** 3.1.0
**Package:** `@clasp-to/core`

### Distribution

```json
{
  "main": "dist/index.js",
  "module": "dist/index.mjs",
  "types": "dist/index.d.ts"
}
```

### Dependencies

- `@msgpack/msgpack`: ^3.0.0

### Clasp Class

```typescript
class Clasp {
    constructor(url: string, options?: ConnectOptions)

    // Connection
    async connect(timeout?: number): Promise<void>
    close(): void
    get connected(): boolean
    get sessionId(): string | null

    // Subscriptions
    subscribe(pattern: string, callback: SubscriptionCallback, options?): Unsubscribe
    on(pattern: string, callback: SubscriptionCallback, options?): Unsubscribe

    // Data Operations
    set(address: string, value: Value): void
    async get(address: string): Promise<Value>
    emit(address: string, payload?: Value): void
    stream(address: string, value: Value): void
    bundle(messages: BundleItem[], options?: { at?: number }): void
    cached(address: string): Value | undefined

    // Utilities
    time(): number
    getSignals(): SignalDefinition[]
    querySignals(pattern: string): SignalDefinition[]
    getLastError(): ErrorMessage | null

    // Events
    onConnect(callback: () => void): void
    onDisconnect(callback: (reason?: string) => void): void
    onError(callback: (error: Error) => void): void
    onReconnect(callback: (attempt: number) => void): void
}
```

### ClaspBuilder

```typescript
class ClaspBuilder {
    constructor(url: string)
    withName(name: string): this
    withFeatures(features: string[]): this
    withToken(token: string): this
    withReconnect(enabled: boolean, intervalMs?: number): this
    async connect(): Promise<Clasp>
}
```

### Type Definitions

```typescript
type Value = null | boolean | number | string | Uint8Array | Value[] | { [key: string]: Value }
type SignalType = 'param' | 'event' | 'stream' | 'gesture' | 'timeline'
type SubscriptionCallback = (value: Value, address: string, meta?: ParamValue) => void
type Unsubscribe = () => void

enum MessageType {
    Hello = 0x01, Welcome = 0x02, Announce = 0x03,
    Subscribe = 0x10, Unsubscribe = 0x11,
    Publish = 0x20, Set = 0x21, Get = 0x22, Snapshot = 0x23,
    Bundle = 0x30, Sync = 0x40, Ping = 0x41, Pong = 0x42,
    Ack = 0x50, Error = 0x51, Query = 0x60, Result = 0x61
}

enum QoS { Fire = 0, Confirm = 1, Commit = 2 }

interface ConnectOptions {
    name?: string
    features?: string[]
    token?: string
    reconnect?: boolean
    reconnectInterval?: number
    connectionTimeout?: number
}
```

### Binary Codec

**Performance:**
- SET: 69 bytes → 32 bytes (54% smaller)
- Encoding: ~10M msg/s
- Decoding: ~12M msg/s

**Message Codes:**
```typescript
MSG = {
  HELLO: 0x01, WELCOME: 0x02, ANNOUNCE: 0x03,
  SUBSCRIBE: 0x10, UNSUBSCRIBE: 0x11,
  PUBLISH: 0x20, SET: 0x21, GET: 0x22, SNAPSHOT: 0x23,
  BUNDLE: 0x30, SYNC: 0x40, PING: 0x41, PONG: 0x42,
  ACK: 0x50, ERROR: 0x51, QUERY: 0x60, RESULT: 0x61
}

VAL = {
  NULL: 0x00, BOOL: 0x01,
  I8: 0x02, I16: 0x03, I32: 0x04, I64: 0x05,
  F32: 0x06, F64: 0x07,
  STRING: 0x08, BYTES: 0x09, ARRAY: 0x0A, MAP: 0x0B
}
```

### Browser Compatibility

- Chrome 68+
- Firefox 63+
- Safari 12+
- Edge 79+
- Bundle: ~15KB min, ~5KB gzip

---

## Python (clasp-to)

### Package Info

**Location:** `bindings/python/`
**Version:** 3.1.0
**Package:** `clasp-to`
**Python:** 3.8+

### Dependencies

```toml
websockets>=10.0
msgpack>=1.0.0
```

### Clasp Class

```python
class Clasp:
    def __init__(
        self,
        url: str,
        name: str = "CLASP Python Client",
        features: Optional[List[str]] = None,
        token: Optional[str] = None,
        reconnect: bool = True,
        reconnect_interval: float = 5.0,
    )

    # Properties
    @property
    def connected(self) -> bool
    @property
    def session_id(self) -> Optional[str]

    # Connection
    async def connect(self) -> None
    async def close(self) -> None

    # Subscriptions
    def subscribe(self, pattern: str, callback: SubscriptionCallback, **options) -> Callable[[], None]
    def on(self, pattern: str, **options) -> Callable[[SubscriptionCallback], SubscriptionCallback]

    # Data Operations
    async def set(self, address: str, value: Value) -> None
    async def get(self, address: str, timeout: float = 5.0) -> Value
    async def emit(self, address: str, payload: Value = None) -> None
    async def stream(self, address: str, value: Value) -> None
    async def bundle(self, messages: List[Dict], at: Optional[int] = None) -> None
    def cached(self, address: str) -> Optional[Value]

    # Utilities
    def time(self) -> int
    def run(self) -> None

    # Events
    def on_connect(self, callback: Callable[[], None]) -> None
    def on_disconnect(self, callback: Callable[[Optional[str]], None]) -> None
    def on_error(self, callback: Callable[[Exception], None]) -> None
```

### ClaspBuilder

```python
@dataclass
class ClaspBuilder:
    url: str
    name: str = "CLASP Python Client"
    features: List[str] = field(default_factory=lambda: ["param", "event", "stream"])
    token: Optional[str] = None
    reconnect: bool = True
    reconnect_interval: float = 5.0

    def with_name(self, name: str) -> "ClaspBuilder"
    def with_features(self, features: List[str]) -> "ClaspBuilder"
    def with_token(self, token: str) -> "ClaspBuilder"
    def with_reconnect(self, enabled: bool, interval: float = 5.0) -> "ClaspBuilder"
    async def connect(self) -> "Clasp"
```

### Type Definitions

```python
Value = Union[None, bool, int, float, str, bytes, List[Any], Dict[str, Any]]
SubscriptionCallback = Callable[[Value, str], None]

class QoS(IntEnum):
    FIRE = 0
    CONFIRM = 1
    COMMIT = 2

class SignalType(str, Enum):
    PARAM = "param"
    EVENT = "event"
    STREAM = "stream"
    GESTURE = "gesture"
    TIMELINE = "timeline"

class MessageType(IntEnum):
    HELLO = 0x01
    # ... (same as JS)
```

### Decorator Pattern

```python
@client.on('/lights/*/brightness')
def on_brightness(value, address):
    print(f'{address} = {value}')
```

---

## Feature Parity

| Feature | JS/TS | Python | Notes |
|---------|:-----:|:------:|-------|
| Client Creation | ✓ | ✓ | |
| Builder Pattern | ✓ | ✓ | |
| Connection | ✓ | ✓ | |
| Authentication | ✓ | ✓ | |
| Subscriptions | ✓ | ✓ | |
| Pattern Wildcards | ✓ | ✓ | |
| set() | ✓ | ✓ | |
| get() | ✓ | ✓ | |
| emit() | ✓ | ✓ | |
| stream() | ✓ | ✓ | |
| gesture() | ✓ | ✓ | Touch/pen input with phases |
| timeline() | ✓ | ✓ | Keyframe automation |
| bundle() | ✓ | ✓ | |
| cached() | ✓ | ✓ | |
| time() | ✓ | ✓ | |
| Signal Queries | ✓ | ✓ | JS: cached only; Python: async server |
| Auto-Reconnect | ✓ | ✓ | |
| Error Handling | ✓ | ✓ | |
| Event Callbacks | ✓ | ✓ | |
| Binary Codec | ✓ | ✓ | |

---

## Usage Examples

### JavaScript

```typescript
import { Clasp, ClaspBuilder } from '@clasp-to/core';

// Direct connection
const client = new Clasp('ws://localhost:7330');
await client.connect();

// Builder pattern
const client = await new ClaspBuilder('ws://localhost:7330')
    .withName('My App')
    .withFeatures(['param', 'event', 'stream'])
    .withReconnect(true, 5000)
    .connect();

// Subscription
client.on('/lights/*/brightness', (value, address) => {
    console.log(`${address} = ${value}`);
});

// Operations
client.set('/lights/kitchen/brightness', 0.75);
client.emit('/cue/fire', { cueId: 'intro' });
client.stream('/sensors/accelerometer/x', 0.342);

// Atomic bundle
client.bundle([
    { set: ['/scene/1/active', true] },
    { set: ['/scene/2/active', false] }
], { at: client.time() + 100000 });

// Get value
const brightness = await client.get('/lights/kitchen/brightness');
```

### Python

```python
import asyncio
from clasp import Clasp, ClaspBuilder

async def main():
    # Direct connection
    client = Clasp('ws://localhost:7330')
    await client.connect()

    # Builder pattern
    client = await (
        ClaspBuilder('ws://localhost:7330')
        .with_name('My App')
        .with_features(['param', 'event', 'stream'])
        .with_reconnect(True, 5.0)
        .connect()
    )

    # Decorator subscription
    @client.on('/lights/*/brightness')
    def on_brightness(value, address):
        print(f'{address} = {value}')

    # Operations
    await client.set('/lights/kitchen/brightness', 0.75)
    await client.emit('/cue/fire', {'cueId': 'intro'})
    await client.stream('/sensors/accelerometer/x', 0.342)

    # Atomic bundle
    await client.bundle([
        {'set': ['/scene/1/active', True]},
        {'set': ['/scene/2/active', False]}
    ], at=client.time() + 100000)

    # Get value
    brightness = await client.get('/lights/kitchen/brightness')

    # Run event loop
    client.run()

asyncio.run(main())
```

---

## Pattern Matching

Both bindings support identical wildcards:

| Pattern | Matches |
|---------|---------|
| `/exact/path` | Exact only |
| `/path/*` | Single segment |
| `/path/**` | Zero or more segments |

Examples:
- `/lights/*/power` matches `/lights/1/power`
- `/layer/**` matches `/layer`, `/layer/1`, `/layer/1/opacity`

---

## Error Handling

### JavaScript

```typescript
try {
    await client.connect();
} catch (error) {
    console.error('Connection failed:', error);
}

client.onError((error) => {
    console.error('Runtime error:', error);
});
```

### Python

```python
from clasp import ClaspError

try:
    await client.connect()
except ClaspError as e:
    print(f'Connection failed: {e}')

client.on_error(lambda e: print(f'Runtime error: {e}'))
```

---

## Installation

### JavaScript

```bash
npm install @clasp-to/core
# or
yarn add @clasp-to/core
```

### Python

```bash
pip install clasp-to
```

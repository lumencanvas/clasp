---
title: Python SDK
description: Build CLASP clients with Python
order: 2
---

# Python SDK

The `clasp-to` package provides an async WebSocket client for connecting to a CLASP router from Python 3.9+. It supports the builder pattern, decorator-based subscriptions, all five signal types, and automatic reconnection.

## Installation

```bash
pip install clasp-to
```

## Connecting

### Builder Pattern

```python
from clasp import Clasp

client = await Clasp.builder('ws://localhost:7330') \
    .with_name('Sensor Hub') \
    .with_reconnect(True) \
    .with_features(['state', 'events']) \
    .connect()

print('Connected, session:', client.session_id)
```

### Direct Constructor

```python
from clasp import Clasp

client = Clasp(
    'ws://localhost:7330',
    name='Sensor Hub',
    features=['state', 'events'],
    reconnect=True,
    reconnect_interval=5.0
)
await client.connect()
```

| Builder Method | Constructor Param | Description |
|---|---|---|
| `.with_name(name)` | `name` | Client display name (default: `'CLASP Python Client'`) |
| `.with_token(token)` | `token` | CPSK auth token |
| `.with_reconnect(bool)` | `reconnect` | Enable auto-reconnect (default: `True`) |
| `.with_features(list)` | `features` | Requested feature set |

`connect()` returns a `Clasp` instance (builder) or `None` (direct), and resolves once the WebSocket handshake and CLASP HELLO exchange complete.

## Setting and Getting State

State is the primary data model in CLASP. Values set at an address persist on the router and are delivered to late joiners.

```python
# Write state
client.set('/lights/brightness', 0.8)
client.set('/lights/color', {'r': 255, 'g': 100, 'b': 0})

# Read state from the router (async round-trip, 5s default timeout)
brightness = await client.get('/lights/brightness')
print(brightness)  # 0.8

# Read state with custom timeout
color = await client.get('/lights/color', timeout=10.0)

# Read from local cache (no network call, returns None if not cached)
cached = client.cached('/lights/brightness')
```

## Subscriptions

Subscribe to addresses or wildcard patterns. CLASP Python supports both a decorator pattern and a function-call pattern.

### Decorator Pattern

```python
@client.on('/sensors/*')
def on_sensor(value, address):
    print(f'{address}: {value}')

@client.on('/sensors/temperature', max_rate=10, epsilon=0.1)
def on_temp(value, address):
    update_display(value)
```

### Function Pattern

```python
def handle_brightness(value, address):
    print(f'Brightness: {value}')

unsub = client.subscribe('/lights/brightness', handle_brightness)

# Unsubscribe when done
unsub()
```

### Wildcard Patterns

| Pattern | Matches | Example Match |
|---|---|---|
| `/sensors/temperature` | Exact address | `/sensors/temperature` |
| `/sensors/*` | Any single level under `/sensors/` | `/sensors/humidity` |
| `/sensors/**` | Any depth under `/sensors/` | `/sensors/room/1/temp` |

### Subscription Options

Pass options as keyword arguments to throttle or filter updates:

| Option | Type | Description |
|---|---|---|
| `max_rate` | `int` | Maximum callback invocations per second |
| `epsilon` | `float` | Minimum change threshold for numeric values |

## Signal Types

CLASP defines five signal types. `set()` handles persistent state (Param signals). The other four are for transient signals.

### Events

Fire-and-forget notifications. Not stored as state, not delivered to late joiners.

```python
client.emit('/alerts/motion-detected', {'zone': 'lobby', 'confidence': 0.95})
client.emit('/cues/go')  # payload is optional
```

### Streams

High-rate continuous data.

```python
import time

while True:
    level = read_audio_level()
    client.stream('/audio/level', level)
    time.sleep(1 / 60)
```

### Gestures

Phased interaction signals with a lifecycle: `begin`, `update`, `end`.

```python
# Start a gesture
client.gesture('/input/fader', 'fader-1', 'begin', {'value': 0.0})

# Update as the user drags
client.gesture('/input/fader', 'fader-1', 'update', {'value': 0.5})

# End the gesture
client.gesture('/input/fader', 'fader-1', 'end')
```

### Timelines

Keyframe-based animations executed by the router.

```python
client.timeline('/lights/brightness', [
    {'time': 0, 'value': 0.0},
    {'time': 1000, 'value': 1.0},
    {'time': 3000, 'value': 1.0},
    {'time': 4000, 'value': 0.0}
], loop=False, start_time=client.time())
```

## Bundles

Group multiple operations into a single message. Bundles are delivered atomically.

```python
# Atomic bundle
client.bundle([
    {'set': ['/lights/1/brightness', 0.8]},
    {'set': ['/lights/2/brightness', 0.6]},
    {'emit': ['/cues/scene-change', {'scene': 'Act 2'}]}
])

# Scheduled bundle -- execute at a specific server time
two_seconds = client.time() + 2_000_000  # microseconds
client.bundle([
    {'set': ['/lights/1/brightness', 1.0]},
    {'set': ['/lights/2/brightness', 1.0]}
], at=two_seconds)
```

`time()` returns the synchronized server time in microseconds as an integer.

## Events

Register callbacks for connection lifecycle events:

```python
@client.on_connect
def connected():
    print('Connected')

@client.on_disconnect
def disconnected():
    print('Disconnected')

@client.on_error
def error(err):
    print(f'Error: {err}')
```

## Running

For long-lived applications, use `run()` to block the main thread on the CLASP event loop:

```python
from clasp import Clasp

client = Clasp('ws://localhost:7330', name='Worker')

@client.on('/tasks/*')
def on_task(value, address):
    print(f'Task: {address} = {value}')

client.run()  # blocks forever, handles connect/reconnect internally
```

For integration with existing async code, use `asyncio` directly:

```python
import asyncio
from clasp import Clasp

async def main():
    client = await Clasp.builder('ws://localhost:7330') \
        .with_name('Async Worker') \
        .connect()

    @client.on('/tasks/*')
    def on_task(value, address):
        print(f'Task: {address} = {value}')

    await asyncio.Event().wait()  # run forever

asyncio.run(main())
```

## Auth

Pass a CPSK token to authenticate with the router:

```python
client = await Clasp.builder('ws://localhost:7330') \
    .with_name('Secure Client') \
    .with_token('cpsk_a1b2c3d4e5f6...') \
    .connect()
```

Or via the direct constructor:

```python
client = Clasp('ws://localhost:7330', token='cpsk_a1b2c3d4e5f6...')
await client.connect()
```

The router validates the token during the HELLO handshake. If the token is invalid or lacks required scopes, `connect()` raises an exception. See [Auth](../auth/README.md) for token generation and scope configuration.

## Value Types

CLASP value types map to Python types:

| CLASP Type | Python Type | Example |
|---|---|---|
| null | `None` | `None` |
| boolean | `bool` | `True` |
| integer (i64) | `int` | `42` |
| float (f64) | `float` | `3.14` |
| string | `str` | `'hello'` |
| bytes | `bytes` | `b'\x01\x02'` |
| array | `list` | `[1, 'two', 3.0]` |
| object | `dict` | `{'r': 255, 'g': 0, 'b': 128}` |

## Query

Inspect the router's current signal state:

```python
# Query signals matching a pattern (5s default timeout)
sensors = await client.query_signals('/sensors/**')

# With custom timeout
all_signals = await client.get_signals('/sensors/**', timeout=10.0)
```

## Cleanup

Close the connection when your application exits:

```python
await client.close()
```

After calling `close()`, the `connected` property returns `False` and no further callbacks fire.

## Reconnection & Connection Lifecycle

The Python client manages reconnection automatically when `reconnect=True` (the default).

### Auto-Reconnect Behavior

When the WebSocket drops, the client retries with exponential backoff starting from `reconnect_interval` (default: 5.0s). On successful reconnect, all subscriptions are re-established and the router sends a fresh SNAPSHOT.

```python
client = Clasp(
    'ws://localhost:7330',
    name='Resilient App',
    reconnect=True,
    reconnect_interval=3.0
)

@client.on_connect
def connected():
    print('Connected (or reconnected)')

@client.on_disconnect
def disconnected():
    print('Lost connection, auto-reconnecting...')

@client.on_error
def error(err):
    print(f'Error: {err}')

await client.connect()
```

### Manual Close

Call `close()` to disconnect without triggering auto-reconnect:

```python
await client.close()
```

## Examples

Working examples in `examples/python/`:

| File | Description |
|------|-------------|
| `signal_types.py` | All five signal types in action |
| `bundles_and_scheduling.py` | Atomic and scheduled bundles |
| `p2p_webrtc.py` | Peer-to-peer communication via WebRTC |
| `late_joiner.py` | Late-joiner state synchronization |
| `security_tokens.py` | CPSK token authentication |
| `embedded_server.py` | Embedded CLASP router in Python |

## Next Steps

- [Core Concepts](../concepts/architecture.md) -- understand signals, state, and the router model
- [Protocol Bridges](../protocols/README.md) -- connect CLASP to OSC, MIDI, MQTT, and more
- [Auth](../auth/README.md) -- CPSK tokens and capability delegation
- [P2P & WebRTC](../core/p2p.md) -- direct peer-to-peer connections
- [JavaScript SDK](javascript.md) -- build CLASP clients with JavaScript
- [Rust SDK](rust.md) -- build CLASP clients with Rust

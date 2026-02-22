---
title: Python API
description: Complete clasp-to API reference
order: 9
---

# Python API

Complete API reference for the `clasp-to` Python package. This covers the `Clasp` class, the `ClaspBuilder`, all method signatures, and type information. For usage patterns and examples, see the [Python SDK guide](../sdk/python.md).

## Installation

```bash
pip install clasp-to
```

## ClaspBuilder

The builder provides a fluent interface for constructing and connecting a `Clasp` client.

```python
from clasp import Clasp

client = await Clasp.builder("ws://localhost:7330") \
    .with_name("My App") \
    .with_features(["lighting"]) \
    .with_token("cpsk_...") \
    .with_reconnect(True) \
    .connect()
```

### Constructor

| Method                      | Returns         | Description                                       |
|-----------------------------|-----------------|---------------------------------------------------|
| `Clasp.builder(url: str)`   | `ClaspBuilder`  | Create a builder targeting the given WebSocket URL |

### Methods

| Method                                      | Returns        | Description                                     |
|---------------------------------------------|----------------|-------------------------------------------------|
| `with_name(name: str)`                      | `ClaspBuilder` | Set client name                                 |
| `with_features(features: list[str])`        | `ClaspBuilder` | Set feature tags                                |
| `with_token(token: str)`                    | `ClaspBuilder` | Set CPSK authentication token                   |
| `with_reconnect(enabled: bool)`             | `ClaspBuilder` | Enable or disable auto-reconnect                |
| `connect()`                                 | `Awaitable[Clasp]` | Connect and return a `Clasp` instance       |

## Clasp

The main client class. Can be constructed directly or via the builder.

### Constructor

```python
client = Clasp(
    url="ws://localhost:7330",
    name="CLASP Python Client",
    features=None,
    token=None,
    reconnect=True,
    reconnect_interval=5.0,
)
```

| Parameter            | Type              | Default                  | Description                          |
|----------------------|-------------------|--------------------------|--------------------------------------|
| `url`                | `str`             | (required)               | WebSocket URL of the router          |
| `name`               | `str`             | `"CLASP Python Client"`  | Client name                          |
| `features`           | `list[str] \| None` | `None`                | Feature tags                         |
| `token`              | `str \| None`     | `None`                   | CPSK authentication token            |
| `reconnect`          | `bool`            | `True`                   | Whether to auto-reconnect on disconnect |
| `reconnect_interval` | `float`           | `5.0`                    | Seconds between reconnection attempts|

### Connection

| Member                | Type / Returns   | Description                                     |
|-----------------------|------------------|-------------------------------------------------|
| `connect()`           | `Awaitable[None]`| Connect (or reconnect) to the router            |
| `close()`             | `None`           | Disconnect and release resources                |
| `connected`           | `bool` (property)| Whether the client is currently connected       |
| `session_id`          | `str` (property) | The session ID assigned by the router           |

### State

| Method                                        | Returns          | Description                                             |
|-----------------------------------------------|------------------|---------------------------------------------------------|
| `set(address: str, value: Value)`             | `None`           | Set a parameter value on the router                     |
| `get(address: str, timeout: float = 5.0)`     | `Awaitable[Value]`| Request the current value from the router (round-trip) |
| `cached(address: str)`                        | `Value \| None`  | Return the locally cached value, or `None`              |

### Subscriptions

| Method                                                                                 | Returns      | Description                                               |
|----------------------------------------------------------------------------------------|--------------|-----------------------------------------------------------|
| `subscribe(pattern: str, callback: Callable, **opts)`                                  | `Callable`   | Subscribe to value changes matching a glob pattern. Returns an unsubscribe callable. |
| `on(pattern: str, **opts)`                                                             | `Callable`   | Decorator for subscribing to event signals matching a glob pattern. |

**Callback signature**: `def callback(address: str, value: Value, meta: dict) -> None`

**Keyword options** (`**opts`):

| Option      | Type    | Description                                         |
|-------------|---------|-----------------------------------------------------|
| `max_rate`  | `float` | Maximum updates per second (client-side throttle)   |
| `epsilon`   | `float` | Minimum numeric change to trigger the callback      |

#### Decorator Example

```python
@client.on("/events/**")
def handle_event(address, payload, meta):
    print(f"Event at {address}: {payload}")
```

### Signals

| Method                                                                             | Returns | Description                                    |
|------------------------------------------------------------------------------------|---------|------------------------------------------------|
| `emit(address: str, payload: Value = None)`                                        | `None`  | Emit a one-shot event signal                   |
| `stream(address: str, value: Value)`                                               | `None`  | Send a continuous stream value (no state storage)|
| `gesture(address: str, id: str, phase: str, payload: Value = None)`                | `None`  | Send a gesture signal with phase tracking      |
| `timeline(address: str, keyframes: list[dict], loop: bool = False, start_time: float = None)` | `None`  | Start a keyframe animation on an address  |

**Gesture phases**: `"begin"`, `"update"`, `"end"`

**Keyframe dict**: `{"time": float, "value": Value, "easing": str}` where easing is `"linear"`, `"ease-in"`, `"ease-out"`, or `"ease-in-out"`.

### Bundles

| Method                                                         | Returns | Description                                                |
|----------------------------------------------------------------|---------|------------------------------------------------------------|
| `bundle(messages: list[dict], at: float = None)`               | `None`  | Send multiple messages atomically, optionally scheduled    |

**Message dict**: `{"address": str, "value": Value}`

### Events

| Method                                        | Returns | Description                                   |
|-----------------------------------------------|---------|-----------------------------------------------|
| `on_connect(callback: Callable)`              | `None`  | Register a connection callback                |
| `on_disconnect(callback: Callable)`           | `None`  | Register a disconnection callback             |
| `on_error(callback: Callable[[Exception], None])` | `None` | Register an error callback                 |

### Time

| Method   | Returns | Description                                                |
|----------|---------|------------------------------------------------------------|
| `time()` | `float` | Current synchronized time in seconds from the router clock |

### Query

| Method                                            | Returns              | Description                                       |
|---------------------------------------------------|----------------------|---------------------------------------------------|
| `query_signals(pattern: str, timeout: float = 5.0)` | `Awaitable[list[dict]]` | Query signal definitions matching a glob pattern |
| `get_signals(pattern: str, timeout: float = 5.0)`   | `Awaitable[list[dict]]` | Alias for `query_signals`                       |

### Run

| Method  | Returns | Description                                                                      |
|---------|---------|----------------------------------------------------------------------------------|
| `run()` | `None`  | Block the current thread, keeping the client alive. Useful for script-style usage.|

#### Run Example

```python
from clasp import Clasp

client = Clasp("ws://localhost:7330", name="Sensor Reader")
client.connect()

@client.on("/sensors/**")
def on_sensor(address, value, meta):
    print(f"{address} = {value}")

client.run()  # blocks until interrupted
```

## Types

### Value

The Python `Value` type maps to native Python types:

| CLASP Type | Python Type  | Example              |
|------------|--------------|----------------------|
| Null       | `None`       | `None`               |
| Boolean    | `bool`       | `True`               |
| Integer    | `int`        | `42`                 |
| Float      | `float`      | `3.14`               |
| String     | `str`        | `"hello"`            |
| Blob       | `bytes`      | `b"\x00\xff"`        |
| Array      | `list`       | `[1, 2, 3]`          |
| Map        | `dict`       | `{"key": "value"}`   |

### Signal Definition Dict

Returned by `query_signals()` and `get_signals()`:

```python
{
    "address": "/path/to/signal",
    "type": "param",        # "param", "event", "stream", "gesture"
    "value": ...,           # current value (for params)
    "meta": {
        "timestamp": 1234567890.123,
        "source": "session-id",
        "sequence": 42,
    }
}
```

## Next Steps

- [Python SDK guide](../sdk/python.md) -- usage patterns and getting started
- [JavaScript API](js-api.md) -- JavaScript/TypeScript client reference
- [Rust Crates](rust-crates.md) -- Rust crate reference

---
title: JavaScript API
description: Complete @clasp-to/core API reference
order: 8
---

# JavaScript API

Complete API reference for the `@clasp-to/core` package (v4.0.0+). This covers the `ClaspBuilder` and `Clasp` classes, all method signatures, and type definitions. For usage patterns and examples, see the [JavaScript SDK guide](../sdk/javascript.md).

## Installation

```bash
npm install @clasp-to/core
```

## ClaspBuilder

The builder is the primary way to create a connected `Clasp` instance.

```typescript
import { Clasp } from '@clasp-to/core';

const client = await Clasp.builder("ws://localhost:7330")
  .withName("My App")
  .withFeatures(["lighting"])
  .withToken("cpsk_...")
  .withReconnect(true)
  .connect();
```

### Constructor

| Method                       | Returns        | Description                            |
|------------------------------|----------------|----------------------------------------|
| `Clasp.builder(url: string)` | `ClaspBuilder` | Create a builder targeting the given WebSocket URL |

### Methods

| Method                                      | Returns        | Description                                          |
|---------------------------------------------|----------------|------------------------------------------------------|
| `name(name: string)`                        | `ClaspBuilder` | Set client name (alias: `withName`)                  |
| `withName(name: string)`                    | `ClaspBuilder` | Set client name                                      |
| `features(features: string[])`              | `ClaspBuilder` | Set feature tags (alias: `withFeatures`)             |
| `withFeatures(features: string[])`          | `ClaspBuilder` | Set feature tags                                     |
| `token(token: string)`                      | `ClaspBuilder` | Set CPSK authentication token (alias: `withToken`)   |
| `withToken(token: string)`                  | `ClaspBuilder` | Set CPSK authentication token                        |
| `reconnect(enabled: boolean)`               | `ClaspBuilder` | Enable or disable auto-reconnect (alias: `withReconnect`) |
| `withReconnect(enabled: boolean)`           | `ClaspBuilder` | Enable or disable auto-reconnect                     |
| `reconnectInterval(ms: number)`             | `ClaspBuilder` | Set reconnect delay in milliseconds (default: 5000)  |
| `connect()`                                 | `Promise<Clasp>` | Connect to the router and return a `Clasp` instance |

## Clasp

The main client class. All state operations, subscriptions, and signal methods are available on this object.

### Connection

| Member                        | Type / Returns     | Description                                     |
|-------------------------------|--------------------|-------------------------------------------------|
| `connect()`                   | `Promise<void>`    | Manually (re)connect to the router              |
| `close()`                     | `void`             | Disconnect and release resources                |
| `connected`                   | `boolean` (getter) | Whether the client is currently connected       |
| `session`                     | `string` (getter)  | The current session ID assigned by the router   |

### State

| Method                             | Returns           | Description                                              |
|------------------------------------|-------------------|----------------------------------------------------------|
| `set(address: string, value: Value)` | `void`          | Set a parameter value on the router                      |
| `get(address: string)`             | `Promise<Value>`  | Request the current value from the router (round-trip)   |
| `cached(address: string)`         | `Value \| undefined` | Return the locally cached value, or `undefined` if not yet received |

### Subscriptions

| Method                                                          | Returns    | Description                                         |
|-----------------------------------------------------------------|------------|-----------------------------------------------------|
| `subscribe(pattern: string, callback: SubscribeCallback, options?: SubscribeOptions)` | `() => void` | Subscribe to value changes matching a glob pattern. Returns an unsubscribe function. |
| `on(pattern: string, callback: EventCallback, options?: SubscribeOptions)` | `() => void` | Subscribe to event signals matching a glob pattern. Returns an unsubscribe function.  |

**SubscribeCallback**: `(address: string, value: Value, meta: SignalMeta) => void`

**EventCallback**: `(address: string, payload: Value | null, meta: SignalMeta) => void`

### SubscribeOptions

| Field      | Type     | Default | Description                                            |
|------------|----------|---------|--------------------------------------------------------|
| `maxRate`  | `number` | --      | Maximum updates per second (client-side throttle)      |
| `epsilon`  | `number` | --      | Minimum change in numeric value to trigger the callback|

### Signals

| Method                                                                          | Returns | Description                                    |
|---------------------------------------------------------------------------------|---------|------------------------------------------------|
| `emit(address: string, payload?: Value)`                                        | `void`  | Emit a one-shot event signal                   |
| `stream(address: string, value: Value)`                                         | `void`  | Send a continuous stream value (no state storage)|
| `gesture(address: string, id: string, phase: GesturePhase, payload?: Value)`    | `void`  | Send a gesture signal with phase tracking      |
| `timeline(address: string, keyframes: Keyframe[], options?: TimelineOptions)`   | `void`  | Start a keyframe animation on an address       |

### Bundles

| Method                                                    | Returns | Description                                              |
|-----------------------------------------------------------|---------|----------------------------------------------------------|
| `bundle(messages: BundleMessage[], options?: BundleOptions)` | `void`  | Send multiple messages atomically, optionally scheduled  |

**BundleOptions**: `{ at?: number }` -- optional timestamp for scheduled delivery.

### Events

| Method                                | Returns    | Description                                      |
|---------------------------------------|------------|--------------------------------------------------|
| `onConnect(callback: () => void)`     | `() => void` | Register a connection callback. Returns unsubscribe. |
| `onDisconnect(callback: () => void)`  | `() => void` | Register a disconnection callback. Returns unsubscribe. |
| `onError(callback: (error: Error) => void)` | `() => void` | Register an error callback. Returns unsubscribe. |
| `onReconnect(callback: () => void)`   | `() => void` | Register a reconnection callback. Returns unsubscribe. |

### Time

| Method   | Returns  | Description                                              |
|----------|----------|----------------------------------------------------------|
| `time()` | `number` | Current synchronized time in seconds from the router clock|

### Query

| Method                              | Returns                    | Description                                      |
|-------------------------------------|----------------------------|--------------------------------------------------|
| `getSignals()`                      | `Promise<SignalDefinition[]>` | Retrieve all signal definitions from the router |
| `querySignals(pattern: string)`     | `Promise<SignalDefinition[]>` | Query signal definitions matching a glob pattern|

### Error

| Method            | Returns            | Description                                  |
|-------------------|--------------------|----------------------------------------------|
| `getLastError()`  | `Error \| null`    | Return the most recent error, or `null`      |
| `clearError()`    | `void`             | Clear the stored error                       |

## Types

### Value

The `Value` type represents any CLASP-compatible value:

```typescript
type Value =
  | null
  | boolean
  | number
  | string
  | Uint8Array
  | Value[]
  | { [key: string]: Value };
```

### SignalType

```typescript
enum SignalType {
  Param = "param",
  Event = "event",
  Stream = "stream",
  Gesture = "gesture",
}
```

### GesturePhase

```typescript
enum GesturePhase {
  Begin = "begin",
  Update = "update",
  End = "end",
}
```

### EasingType

Used in timeline keyframes.

```typescript
enum EasingType {
  Linear = "linear",
  EaseIn = "ease-in",
  EaseOut = "ease-out",
  EaseInOut = "ease-in-out",
}
```

### SignalDefinition

```typescript
interface SignalDefinition {
  address: string;
  type: SignalType;
  value: Value;
  meta: SignalMeta;
}
```

### SignalMeta

```typescript
interface SignalMeta {
  timestamp: number;
  source: string;
  sequence: number;
}
```

### Keyframe

```typescript
interface Keyframe {
  time: number;
  value: Value;
  easing?: EasingType;
}
```

### TimelineOptions

```typescript
interface TimelineOptions {
  loop?: boolean;
  startTime?: number;
}
```

### BundleMessage

```typescript
interface BundleMessage {
  address: string;
  value: Value;
}
```

## Next Steps

- [JavaScript SDK guide](../sdk/javascript.md) -- usage patterns and getting started
- [Python API](python-api.md) -- Python client reference
- [Rust Crates](rust-crates.md) -- Rust crate reference

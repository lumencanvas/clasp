---
title: Wire Protocol
description: CLASP binary protocol specification
order: 4
---

# Wire Protocol

CLASP uses a compact binary protocol over WebSocket (or QUIC/UDP/TCP). This page documents the frame format, message types, value encoding, and connection handshake.

The protocol supports two encoding versions: v0 (legacy MessagePack with named keys) and v1 (compact binary). The decoder auto-detects the format. All new implementations should use v1 binary encoding.

## Frame Format

Every CLASP message is wrapped in a frame:

```
Offset  Size   Field
0       1      Magic byte (0x53 = 'S')
1       1      Flags
2       2      Payload length (big-endian u16)
[4]     [8]    Timestamp (optional, u64 microseconds)
4/12    N      Payload (binary-encoded message)
```

**Flags byte layout:**

```
Bit 7-6:  QoS       (00=Fire, 01=Confirm, 10=Commit, 11=reserved)
Bit 5:    Timestamp present
Bit 4:    Encrypted
Bit 3:    Compressed
Bit 2-0:  Encoding version (0=MessagePack legacy, 1+=binary)
```

**Constants:**

| Name | Value | Description |
|------|-------|-------------|
| `MAGIC_BYTE` | `0x53` (`'S'`) | Frame start marker |
| `HEADER_SIZE` | `4` bytes | Header without timestamp |
| `HEADER_SIZE_WITH_TS` | `12` bytes | Header with timestamp |
| `MAX_PAYLOAD_SIZE` | `65535` bytes | Maximum payload (u16 max) |

## Message Types

19 message types organized by function:

| Code | Name | Direction | Default QoS | Description |
|------|------|-----------|-------------|-------------|
| `0x01` | Hello | C -> S | Fire | Connection initiation |
| `0x02` | Welcome | S -> C | Fire | Connection accepted |
| `0x03` | Announce | S -> C | Fire | Signal namespace advertisement |
| `0x04` | FederationSync | S <-> S | Confirm | Router-to-router federation sync |
| `0x10` | Subscribe | C -> S | Confirm | Subscribe to address pattern |
| `0x11` | Unsubscribe | C -> S | Confirm | Cancel subscription |
| `0x20` | Publish | C -> S, S -> C | varies | Event, stream, gesture, or timeline data |
| `0x21` | Set | C -> S | Confirm | Set parameter value (stateful) |
| `0x22` | Get | C -> S | Fire | Request current value |
| `0x23` | Snapshot | S -> C | Fire | Bulk state delivery |
| `0x24` | Replay | C -> S | Confirm | Request journal replay |
| `0x30` | Bundle | C -> S | Commit | Atomic message group |
| `0x40` | Sync | S -> C | Fire | Clock synchronization |
| `0x41` | Ping | bidirectional | Fire | Keepalive request |
| `0x42` | Pong | bidirectional | Fire | Keepalive response |
| `0x50` | Ack | S -> C | Fire | Acknowledgment |
| `0x51` | Error | S -> C | Fire | Error response |
| `0x60` | Query | C -> S | Fire | Signal introspection query |
| `0x61` | Result | S -> C | Fire | Query response |

Direction legend: C = Client, S = Server (Router).

## Value Types

| Code | Type | Encoding |
|------|------|----------|
| `0x00` | null | No data bytes |
| `0x01` | bool | 1 byte (0 = false, 1 = true) |
| `0x02` | i8 | 1 byte signed |
| `0x03` | i16 | 2 bytes big-endian signed |
| `0x04` | i32 | 4 bytes big-endian signed |
| `0x05` | i64 | 8 bytes big-endian signed |
| `0x06` | f32 | 4 bytes IEEE 754 |
| `0x07` | f64 | 8 bytes IEEE 754 |
| `0x08` | string | u16 length prefix + UTF-8 bytes |
| `0x09` | bytes | u16 length prefix + raw bytes |
| `0x0A` | array | u16 count + (type_code + value_data) per element |
| `0x0B` | map | u16 count + (string_key + type_code + value_data) per entry |

In the Rust SDK, `Value::Int(i64)` maps to type code `0x05` (i64) on encode. Decoding accepts any integer width (`0x02`-`0x05`) and widens to i64. Similarly, `Value::Float(f64)` maps to `0x07` (f64) but decoding accepts `0x06` (f32) and widens.

## QoS Levels

| Level | Code | Behavior |
|-------|------|----------|
| Fire | `0` | Best-effort delivery. No ACK. Suitable for high-rate streams. |
| Confirm | `1` | At-least-once delivery. Server sends ACK on receipt. Used for Set, Subscribe, and Events. |
| Commit | `2` | Exactly-once, ordered delivery. Server sends ACK with ordering guarantees. Used for Bundles and Timelines. |

Default QoS by signal type:

| Signal Type | Default QoS |
|-------------|-------------|
| Param | Confirm |
| Event | Confirm |
| Stream | Fire |
| Gesture | Fire |
| Timeline | Commit |

## Signal Types

| Code | Type | Description |
|------|------|-------------|
| `0` | Param | Stateful parameter with revision tracking. Set via SET messages. |
| `1` | Event | Ephemeral trigger event. Sent via PUBLISH. |
| `2` | Stream | High-rate continuous data (e.g., audio samples). Sent via PUBLISH with samples array. |
| `3` | Gesture | Phased input (touch/pen/motion). Sent via PUBLISH with phase (Start/Move/End/Cancel) and gesture ID. |
| `4` | Timeline | Time-indexed automation with keyframes and easing. Sent via PUBLISH with timeline data. |

## Connection Handshake

```
Client                          Server
  |                                |
  |--- Hello (version, name,      |
  |    features, token) --------->|
  |                                |
  |<--- Welcome (version, session, |
  |     name, features, time) ----|
  |                                |
  |--- Subscribe (pattern) ------>|
  |--- Subscribe (pattern) ------>|
  |                                |
  |<--- Snapshot (params[]) ------|
  |                                |
  |<--- Ack (subscription) ------|
  |<--- Ack (subscription) ------|
  |                                |
```

1. Client sends **Hello** with protocol version, client name, requested feature flags, and optional auth token.
2. Server responds with **Welcome** containing server version, assigned session ID, server name, supported features, and current server time (microseconds).
3. Client sends **Subscribe** messages for address patterns of interest.
4. Server sends a **Snapshot** containing current state for all matched parameters.
5. Server sends **Ack** for each subscription.

If the Hello includes a token that fails validation, the server sends an **Error** message and closes the connection.

## Key Message Structures

### Hello (0x01)

```
[msg_type:u8=0x01]
[version:u8]
[feature_flags:u8]
[name:string]
[token:string]        (empty string = no token)
```

Feature flags bitmask: `param(0x80)`, `event(0x40)`, `stream(0x20)`, `gesture(0x10)`, `timeline(0x08)`, `federation(0x04)`.

### Welcome (0x02)

```
[msg_type:u8=0x02]
[version:u8]
[feature_flags:u8]
[server_time:u64]     (microseconds since epoch)
[session:string]
[name:string]
[token:string]        (optional server-assigned token)
```

### Set (0x21)

```
[msg_type:u8=0x21]
[flags:u8]
  bit 7:    has_revision
  bit 6:    lock
  bit 5:    unlock
  bit 3-0:  value_type_code
[address:string]
[value_data:...]      (type-specific encoding, type from flags)
[revision:u64]        (if has_revision flag set)
```

### Publish (0x20)

```
[msg_type:u8=0x20]
[flags:u8]
  bit 7-5:  signal_type (3 bits)
  bit 4:    has_timestamp
  bit 3:    has_id (gesture ID)
  bit 2-0:  gesture_phase (3 bits)
[address:string]
[value_indicator:u8]  (0=none, 1=value, 2=samples)
  if 1: [vtype:u8][value_data:...]
  if 2: [count:u16][f64 samples...]
[timestamp:u64]       (if has_timestamp)
[gesture_id:u32]      (if has_id)
[rate:u32]            (if remaining bytes)
```

### Subscribe (0x10)

```
[msg_type:u8=0x10]
[id:u32]              (subscription ID)
[pattern:string]      (address pattern with wildcards)
[type_mask:u8]        (0xFF=all, bitmask: param=0x01, event=0x02, stream=0x04, gesture=0x08, timeline=0x10)
[opt_flags:u8]
  if bit 0: [max_rate:u32]
  if bit 1: [epsilon:f64]
  if bit 2: [history:u32]
  if bit 3: [window:u32]
```

### Bundle (0x30)

```
[msg_type:u8=0x30]
[flags:u8]
  bit 7: has_timestamp
[count:u16]           (number of inner messages)
[timestamp:u64]       (if has_timestamp)
[inner messages...]   (each: [length:u16][message_bytes...])
```

### Snapshot (0x23)

```
[msg_type:u8=0x23]
[count:u16]           (number of parameters)
[params...]           (each:)
  [address:string]
  [vtype:u8][value_data:...]
  [revision:u64]
  [opt_flags:u8]
    if bit 0: [writer:string]
    if bit 1: [timestamp:u64]
```

### Error (0x51)

```
[msg_type:u8=0x51]
[code:u16]
[message:string]
[flags:u8]
  if bit 0: [address:string]
  if bit 1: [correlation_id:u32]
```

### Ack (0x50)

```
[msg_type:u8=0x50]
[flags:u8]
  if bit 0: [address:string]
  if bit 1: [revision:u64]
  if bit 2: [locked:u8]
  if bit 3: [holder:string]
  if bit 4: [correlation_id:u32]
```

### String Encoding

All strings in the binary protocol use a 2-byte big-endian length prefix followed by UTF-8 bytes:

```
[length:u16][utf8_bytes:length]
```

Maximum string length: 65535 bytes.

## Error Codes

The server sends ERROR messages (type `0x51`) with a numeric error code, a human-readable message, and optionally the address that caused the error. Error codes are organized into five ranges:

### Protocol Errors (100-199)

| Code | Name | Description |
|------|------|-------------|
| 100 | `InvalidFrame` | Frame header is malformed (bad magic byte, truncated header, payload exceeds max size) |
| 101 | `InvalidMessage` | Message payload cannot be decoded (unknown type code, corrupt encoding) |
| 102 | `UnsupportedVersion` | Client requested a protocol version the server does not support |

### Address Errors (200-299)

| Code | Name | Description |
|------|------|-------------|
| 200 | `InvalidAddress` | Address string does not conform to CLASP path rules (must start with `/`, no empty segments) |
| 201 | `AddressNotFound` | GET or SUBSCRIBE target does not match any known signal |
| 202 | `PatternError` | Subscription pattern contains invalid wildcard syntax |

### Auth Errors (300-399)

| Code | Name | Description |
|------|------|-------------|
| 300 | `Unauthorized` | No token provided and the router requires authentication |
| 301 | `Forbidden` | Token is valid but lacks the required scope for the requested action |
| 302 | `TokenExpired` | Token signature is valid but the token has passed its expiration time |

### State Errors (400-499)

| Code | Name | Description |
|------|------|-------------|
| 400 | `RevisionConflict` | SET included an expected revision that does not match the current revision on the server |
| 401 | `LockHeld` | SET attempted to write to a locked address and the lock is held by a different session |
| 402 | `InvalidValue` | Value does not pass the app config validation rules for this address |

### Server Errors (500-599)

| Code | Name | Description |
|------|------|-------------|
| 500 | `InternalError` | Unexpected server error (bug or resource exhaustion) |
| 501 | `ServiceUnavailable` | Server is shutting down or temporarily unable to process requests |
| 502 | `Timeout` | Server-side operation timed out (e.g., federation sync, lock acquisition) |

### Handling Errors

Clients receive errors as ERROR messages on the WebSocket. In the JavaScript SDK:

```javascript
client.onError((err) => {
  console.error('CLASP error:', err);
});

// Or check the last error after an operation
const lastErr = client.getLastError();
if (lastErr) {
  console.log(`Code ${lastErr.code}: ${lastErr.message}`);
}
```

**Recommended handling by category:**

- **100s (Protocol)**: These indicate a client bug or version mismatch. Log and fix the client code.
- **200s (Address)**: Check your address strings and patterns for typos or invalid characters.
- **300s (Auth)**: Re-authenticate, refresh the token, or request additional scopes.
- **400s (State)**: Re-read the current value and retry (for conflicts), or wait for the lock to release.
- **500s (Server)**: Retry with backoff. If persistent, check server logs.

## Backward Compatibility

The decoder auto-detects v0 (MessagePack) vs v1 (binary) encoding by inspecting the first byte of the payload:

- If the byte matches a MessagePack map marker (`0x80`-`0x8F`, `0xDE`, `0xDF`), the payload is decoded as legacy MessagePack.
- Otherwise, the byte is treated as a message type code and decoded as binary.

The frame flags `version` field (bits 0-2) also indicates the encoding: 0 for MessagePack, 1 for binary.

## Next Steps

- [CLASP CLI Reference](clasp-cli.md) -- use the CLI to test protocol interactions
- [Architecture](../concepts/architecture.md) -- understand how messages flow through the system
- [App Config Schema](app-config-schema.md) -- configure write validation and snapshot filtering

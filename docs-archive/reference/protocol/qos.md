---
title: "QoS Levels"
description: "CLASP defines three Quality of Service levels that control message reliability and acknowledgment behavior."
section: reference
order: 7
---
# QoS Levels

CLASP defines three Quality of Service levels that control message reliability and acknowledgment behavior.

## Overview

| Level | Name | Code | Acknowledgment | Retry | Use Case |
|-------|------|------|----------------|-------|----------|
| Q0 | Fire | `00` | None | No | High-rate streams, non-critical |
| Q1 | Confirm | `01` | Yes | Yes | State changes, events |
| Q2 | Commit | `10` | Yes + Durable | Yes | Critical operations |

## Q0: Fire (Fire-and-Forget)

**Bits:** `00`

Messages are sent without expecting acknowledgment. If the message is lost, no retry occurs.

### Characteristics
- Lowest latency
- No delivery guarantee
- No ordering guarantee
- Suitable for high-rate data where occasional loss is acceptable

### Use Cases
- Stream samples (fader movements, sensor data)
- Gesture move events
- Non-critical status updates
- Any data that will be superseded quickly

### Example

```typescript
// JavaScript - stream uses Q0 by default
client.stream('/fader/1', 0.75);
```

```rust
// Rust - explicit Q0
client.publish_q0("/fader/1", Value::Float(0.75)).await?;
```

## Q1: Confirm (Acknowledged)

**Bits:** `01`

Messages require acknowledgment from the receiver. If no ACK is received within timeout, the message is retried.

### Characteristics
- Guaranteed delivery (with retries)
- Higher latency than Q0
- Default for state changes

### Retry Behavior
- Initial timeout: 100ms
- Exponential backoff: 100ms, 200ms, 400ms, 800ms
- Maximum retries: 5
- Total max wait: ~1.5 seconds

### Use Cases
- SET messages (state changes)
- Events (triggers, cues)
- Subscriptions
- Any message where delivery matters

### Example

```typescript
// JavaScript - set uses Q1 by default
await client.set('/light/1/brightness', 0.8);

// Explicit Q1 for events
await client.emit('/cue/fire', { id: 'intro' });
```

```rust
// Rust - explicit Q1
client.set_q1("/light/1/brightness", Value::Float(0.8)).await?;
```

### ACK Message

When a Q1 message is received, the receiver sends:

```javascript
{
  type: "ACK",
  correlationId: 42,
  revision: 43
}
```

## Q2: Commit (Durable)

**Bits:** `10`

Messages are acknowledged AND persisted before confirmation. Provides strongest delivery guarantee.

### Characteristics
- Guaranteed delivery
- Guaranteed durability (survives router restart)
- Highest latency
- Used for critical state changes

### Persistence Flow

```
Client                              Router
  │                                    │
  │── SET (Q2) ───────────────────────►│
  │                                    │ (persist to disk)
  │                                    │ (apply to state)
  │◄── ACK ────────────────────────────│
  │                                    │
```

### Use Cases
- Configuration changes
- Show/preset saves
- Critical cue triggers
- Any operation that must not be lost

### Example

```typescript
// JavaScript - explicit Q2
await client.set('/config/important', value, { qos: 'commit' });
```

```rust
// Rust - explicit Q2
client.set_q2("/config/important", value).await?;
```

## Default QoS by Signal Type

| Signal Type | Default QoS | Rationale |
|-------------|-------------|-----------|
| Param (SET) | Q1 Confirm | State changes should be reliable |
| Event | Q1 Confirm | Triggers should be delivered |
| Stream | Q0 Fire | High-rate, loss acceptable |
| Gesture | Q0 Fire | High-rate, loss acceptable |
| Timeline | Q2 Commit | Must not be lost |

## QoS in Frame Header

QoS is encoded in bits 7-6 of the flags byte:

```
Flags byte:
┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐
│ QoS │ QoS │ TS  │ Enc │ Cmp │ Encoding...     │
│ [7] │ [6] │ [5] │ [4] │ [3] │ [2] [1] [0]     │
└─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘

QoS values:
  00 = Q0 Fire
  01 = Q1 Confirm
  10 = Q2 Commit
  11 = Reserved
```

## Error Handling

### Timeout

If no ACK is received within the timeout period:

1. Message is retried with exponential backoff
2. After max retries, an error is raised to the application
3. Connection may be considered unhealthy

### Duplicate Detection

Receivers should detect and ignore duplicate messages based on:
- Message sequence number
- Correlation ID
- Content hash

### Out-of-Order

Q1 and Q2 messages may arrive out of order. Applications should:
- Use revision numbers for conflict resolution
- Use bundles for atomic operations
- Use timestamps for ordering when needed

## Transport Interaction

### WebSocket

All QoS levels work over WebSocket. The transport provides reliable delivery, so Q0 messages will typically arrive, but without application-level acknowledgment.

### UDP

Q0 is most appropriate for UDP. Q1/Q2 require application-level retransmission since UDP is unreliable.

### WebRTC DataChannel

Can be configured for ordered/unordered and reliable/unreliable:

```javascript
// Q0 channel - unreliable, unordered
const q0Channel = pc.createDataChannel('clasp-q0', {
  ordered: false,
  maxRetransmits: 0
});

// Q1/Q2 channel - reliable, ordered
const q1Channel = pc.createDataChannel('clasp-q1', {
  ordered: true
});
```

## Performance Considerations

| QoS | Latency | Throughput | Use When |
|-----|---------|------------|----------|
| Q0 | Lowest | Highest | Loss is acceptable |
| Q1 | Medium | High | Delivery matters |
| Q2 | Highest | Lower | Durability required |

For high-throughput scenarios:
- Use Q0 for most traffic
- Use Q1 selectively for important state
- Use Q2 sparingly for critical operations

## Comparison with Other Protocols

| Protocol | CLASP Q0 | CLASP Q1 | CLASP Q2 |
|----------|----------|----------|----------|
| MQTT QoS 0 | Similar | - | - |
| MQTT QoS 1 | - | Similar | - |
| MQTT QoS 2 | - | - | Similar |
| OSC | Similar | N/A | N/A |
| TCP | - | Similar | - |

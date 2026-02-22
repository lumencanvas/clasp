---
title: "Migrating from OSC"
description: "Guide for transitioning from OSC to CLASP."
section: appendix
order: 2
---
# Migrating from OSC

Guide for transitioning from OSC to CLASP.

## Overview

CLASP shares conceptual similarities with OSC but provides additional features. This guide helps you migrate existing OSC-based systems to CLASP.

## Key Differences

| Feature | OSC | CLASP |
|---------|-----|-------|
| Addresses | Hierarchical | Hierarchical |
| State | None | Built-in |
| Subscriptions | None | Pattern-based |
| Transport | UDP/TCP | WS, QUIC, UDP, WebRTC |
| Encoding | Mixed | Binary |
| Reliability | None | QoS levels |

## Address Translation

OSC and CLASP both use hierarchical addresses:

```
OSC:   /1/fader1
CLASP: /1/fader1   (identical)
```

When using the OSC bridge, addresses get a prefix:

```
OSC:   /1/fader1
CLASP: /osc/1/fader1
```

## Value Types

### Direct Mapping

| OSC Type | CLASP Type |
|----------|------------|
| int32 | Int |
| float32 | Float |
| string | String |
| blob | Blob |
| True/False | Bool |
| Nil | Null |

### Multiple Arguments

OSC messages with multiple arguments become CLASP arrays:

```
OSC:   /xy [0.5, 0.8]
CLASP: /xy = [0.5, 0.8]
```

## Migration Strategies

### Strategy 1: OSC Bridge (Gradual)

Keep existing OSC apps, add CLASP via bridge:

```
┌─────────────┐          ┌─────────────┐
│  OSC App    │ ──OSC──► │ OSC Bridge  │
└─────────────┘          └──────┬──────┘
                                │
┌─────────────┐          ┌──────▼──────┐
│ CLASP App   │ ◄───────►│   Router    │
└─────────────┘          └─────────────┘
```

Configuration:
```bash
clasp server --port 7330
clasp osc --port 8000 --target 192.168.1.100:9000
```

### Strategy 2: Native Migration (Full)

Replace OSC with native CLASP:

**Before (OSC):**
```javascript
// Node.js with osc.js
const osc = require('osc');

const udpPort = new osc.UDPPort({
  localAddress: "0.0.0.0",
  localPort: 8000
});

udpPort.on("message", (msg) => {
  console.log(msg.address, msg.args);
});

udpPort.send({ address: "/fader/1", args: [0.5] }, "192.168.1.100", 9000);
```

**After (CLASP):**
```javascript
const { ClaspBuilder } = require('@clasp-to/core');

const client = await new ClaspBuilder('ws://localhost:7330');

client.on('/fader/*', (value, address) => {
  console.log(address, value);
});

await client.set('/fader/1', 0.5);
```

## Feature Comparison

### No State → State Management

**OSC**: No built-in state. Must implement in application.

**CLASP**: State is automatic.

```javascript
// Get current value (impossible in pure OSC)
const value = await client.get('/fader/1');

// Late joiners receive current state automatically
```

### No Subscriptions → Pattern Subscriptions

**OSC**: Receive everything, filter in application.

**CLASP**: Subscribe to patterns, receive only matching messages.

```javascript
// Only receive fader changes
client.on('/fader/*', handler);

// Only receive temperature sensors
client.on('/sensors/*/temperature', handler);
```

### Fire-and-Forget → QoS Levels

**OSC**: UDP, no delivery confirmation.

**CLASP**: Choose reliability level.

```javascript
// Fire and forget (OSC-like)
await client.set('/data', value);

// With confirmation
await client.setWithQos('/critical', value, 'confirm');

// Exactly-once delivery
await client.setWithQos('/transaction', value, 'commit');
```

### No Bundles Timing → Scheduled Bundles

**OSC**: Timetag bundles possible but rarely supported.

**CLASP**: Native scheduled execution.

```javascript
// Execute 5 seconds from now
await client.bundle()
  .set('/light/1', 255)
  .set('/light/2', 255)
  .atTime(Date.now() + 5000)
  .execute();
```

## Common Patterns

### TouchOSC Integration

**Before**: TouchOSC → UDP → Your App

**After**: TouchOSC → OSC Bridge → CLASP Router → Your App

Benefits:
- Multiple apps can receive same data
- State is available for late joiners
- Web interfaces can access same data

### Multi-App Communication

**OSC**: Each app needs to know all other app addresses/ports.

**CLASP**: All apps connect to router.

```
Before (OSC):
App A ──► App B (port 9000)
App A ──► App C (port 9001)
App B ──► App A (port 8000)
...

After (CLASP):
App A ◄──► Router ◄──► App B
              ▲
              │
           App C
```

## Coexistence

Run OSC and CLASP together during migration:

```javascript
// Handle both OSC (via bridge) and native CLASP
client.on('/osc/fader/*', handleLegacyOsc);
client.on('/fader/*', handleNativeClasp);
```

## Checklist

- [ ] Identify all OSC addresses in use
- [ ] Set up CLASP router
- [ ] Configure OSC bridge
- [ ] Test existing OSC apps work through bridge
- [ ] Migrate apps one at a time to native CLASP
- [ ] Update address patterns if needed
- [ ] Remove OSC bridge when migration complete

## See Also

- [OSC Bridge Reference](../../reference/bridges/osc.md)
- [Add OSC Connection](../../how-to/connections/add-osc.md)
- [First Connection Tutorial](../../tutorials/first-connection.md)

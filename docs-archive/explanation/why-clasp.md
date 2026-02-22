---
title: "Why CLASP?"
description: "CLASP exists because creative projects have outgrown existing protocols. This document explains the problems CLASP solves."
section: explanation
order: 16
---
# Why CLASP?

CLASP exists because creative projects have outgrown existing protocols. This document explains the problems CLASP solves.

## The Problem

Creative projects involve a chaotic mix of technologies:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Lighting  │     │    Audio    │     │   Video     │
│   DMX       │     │    OSC      │     │   MIDI      │
│   Art-Net   │     │    MIDI     │     │   Custom    │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    ┌──────▼──────┐
                    │    ???      │
                    │   No good   │
                    │   option    │
                    └─────────────┘
```

### Protocol Proliferation

- **Lighting** speaks DMX, Art-Net, sACN
- **Audio** software uses OSC, MIDI
- **IoT sensors** communicate via MQTT
- **Web interfaces** need WebSocket or HTTP
- **VJ software** has proprietary APIs

Each protocol was designed for a specific domain. Connecting them requires custom translation code for every pair of protocols.

### Limitations of Existing Protocols

| Protocol | Limitation |
|----------|------------|
| **OSC** | No state management, no standard namespaces, UDP-only |
| **MIDI** | 7-bit values, no networking, limited bandwidth |
| **Art-Net** | DMX-focused, no semantic signals, broadcast floods |
| **MQTT** | No creative primitives, no timing, just bytes |
| **Custom WebSocket** | Every app invents its own format |

### The Result

Without a unifying protocol:
- Every integration is custom code
- State synchronization is manual
- Timing coordination is fragile
- Security is an afterthought
- Adding a new tool means N new integrations

## What CLASP Provides

### 1. Protocol Bridge

CLASP acts as a universal translator:

```
┌─────────────┐     ┌─────────────────────────────────┐     ┌─────────────┐
│  TouchOSC   │────►│           CLASP Router          │────►│  Lighting   │
│  (OSC)      │     │  ┌─────┐ ┌─────┐ ┌──────┐      │     │  (Art-Net)  │
└─────────────┘     │  │ OSC │ │MIDI │ │ArtNet│ ...  │     └─────────────┘
                    │  └─────┘ └─────┘ └──────┘      │
                    └─────────────────────────────────┘
```

One integration per protocol, not N² pairwise integrations.

### 2. Semantic Signals

CLASP understands what signals mean:

| Signal Type | Meaning | Behavior |
|-------------|---------|----------|
| **Param** | A stateful value | Stored, versioned, synced |
| **Event** | A trigger | Delivered reliably, not stored |
| **Stream** | Continuous data | May be downsampled, not stored |
| **Gesture** | Phased input | Tracked by ID, coalesced |
| **Timeline** | Automation | Executed at scheduled times |

This enables smart routing, appropriate reliability, and meaningful UI.

### 3. State Management

CLASP maintains authoritative state:

```javascript
// Always know the current value
const brightness = await client.get('/lights/main/brightness');

// Late joiners receive current state automatically
client.on('/lights/**', (value, address) => {
  // Receives SNAPSHOT of current state on connect
  // Then receives updates as they happen
});
```

No more "what's the current value?" uncertainty.

### 4. Timing Coordination

Synchronized clocks and scheduled execution:

```javascript
// Execute in 100ms, synchronized across all clients
client.bundle([
  { set: ['/light/1', 1.0] },
  { set: ['/light/2', 0.0] }
], { at: client.time() + 100000 });
```

Critical for musical timing and coordinated effects.

### 5. Built-in Security

Encryption and access control from the start:

```javascript
// Capability token limits what client can do
{
  "read": ["/public/**"],
  "write": ["/user/123/**"],
  "constraints": {
    "/user/123/volume": { "max": 100 }
  }
}
```

### 6. Transport Flexibility

Works over whatever transport makes sense:

| Scenario | Transport |
|----------|-----------|
| Browser | WebSocket |
| Native apps | QUIC |
| Local network | UDP |
| Microcontrollers | HTTP POST |
| Mobile | BLE |

Same protocol, different transports.

## Why Not Just Use...

### OSC?

OSC is great for what it does, but:
- No state management (what IS the current value?)
- No standard namespaces (every app invents addresses)
- UDP only (no browser support without bridge)
- No security model
- String parsing overhead on embedded

CLASP can bridge OSC while adding what it lacks.

### MQTT?

MQTT is designed for IoT telemetry:
- No semantic signal types
- No timing/synchronization
- No creative primitives (colors, vectors)
- Requires a broker
- Retained messages aren't the same as state management

CLASP can bridge MQTT and add creative features.

### Custom WebSocket JSON?

Every app ends up inventing:
- Message format
- State synchronization
- Subscription patterns
- Error handling

CLASP provides all of this as a standard, so apps interoperate.

### MIDI 2.0?

MIDI 2.0 is excellent for instrument control:
- But it's hardware-focused
- Not designed for network communication
- Complex property exchange
- Slow ecosystem adoption

CLASP bridges MIDI while working over networks.

## Who Should Use CLASP

### Live Performance
VJs, lighting designers, and music producers connecting multiple tools in real-time.

### Installation Art
Interactive installations with sensors, actuators, and visualizations.

### Home Automation
IoT systems that need better-than-MQTT semantics.

### Software Integration
Connecting creative applications that speak different protocols.

### Embedded Systems
Microcontrollers that need lightweight protocol support.

## Summary

CLASP exists because:

1. **Protocol fragmentation** makes integration painful
2. **Existing protocols** lack important features (state, timing, security)
3. **Custom solutions** don't interoperate
4. **Creative applications** have unique requirements

CLASP provides a unified protocol that bridges legacy systems while enabling modern features.

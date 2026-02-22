---
title: "Bridge Architecture"
description: "Bridges connect CLASP to external protocols like OSC, MIDI, DMX, and MQTT."
section: explanation
order: 2
---
# Bridge Architecture

Bridges connect CLASP to external protocols like OSC, MIDI, DMX, and MQTT.

## What Is a Bridge?

A bridge is a CLASP **client** that also speaks another protocol:

```
┌─────────────────┐      ┌──────────────┐      ┌─────────────────┐
│   External      │      │    Bridge    │      │     Router      │
│   Protocol      │◄────►│   (client)   │◄────►│                 │
└─────────────────┘      └──────────────┘      └─────────────────┘
    OSC/MIDI/etc.            CLASP               CLASP
```

The bridge translates messages bidirectionally.

## Bridge vs Router

A common misconception: **bridges are not routers**.

| Component | Role |
|-----------|------|
| Router | Central message hub, routes between clients |
| Bridge | Client that translates external protocols |

```
                    ┌──────────────────┐
                    │     Router       │
                    └────────┬─────────┘
                             │
       ┌─────────────────────┼─────────────────────┐
       │                     │                     │
┌──────▼──────┐       ┌──────▼──────┐       ┌──────▼──────┐
│ OSC Bridge  │       │ MIDI Bridge │       │  JS Client  │
│  (client)   │       │  (client)   │       │  (client)   │
└──────┬──────┘       └──────┬──────┘       └─────────────┘
       │                     │
┌──────▼──────┐       ┌──────▼──────┐
│  TouchOSC   │       │ MIDI Device │
└─────────────┘       └─────────────┘
```

All routing goes through the router, not through bridges.

## Address Mapping

Bridges map between protocol-specific addresses and CLASP addresses.

### OSC → CLASP

```
OSC: /synth/osc1/cutoff ,f 0.5
         │
         ▼
CLASP: SET /osc/synth/osc1/cutoff = 0.5
```

The prefix `/osc/` indicates the source protocol.

### MIDI → CLASP

```
MIDI: CC 7 (volume) on Channel 1, Value 100
         │
         ▼
CLASP: SET /midi/{device}/cc/1/7 = 100
```

MIDI messages become structured addresses.

### CLASP → OSC

```
CLASP: SET /osc/synth/osc1/cutoff = 0.8
         │
         ▼
OSC: /synth/osc1/cutoff ,f 0.8
```

The `/osc/` prefix is stripped when sending.

### MQTT → CLASP

```
MQTT: Topic "sensors/room1/temp", Payload "23.5"
         │
         ▼
CLASP: SET /mqtt/sensors/room1/temp = 23.5
```

Topic paths map directly.

## Bridge Types

### Input Bridge

Receives external protocol, publishes to CLASP:

```
External Device → Bridge → Router → CLASP Clients
```

Example: TouchOSC sending fader movements.

### Output Bridge

Receives from CLASP, sends to external protocol:

```
CLASP Clients → Router → Bridge → External Device
```

Example: CLASP controlling DMX lights.

### Bidirectional Bridge

Most bridges are bidirectional:

```
External ←→ Bridge ←→ Router ←→ Clients
```

Changes from either side propagate.

## Bridge Configuration

### CLI

```bash
# OSC bridge on port 9000
clasp osc --port 9000

# MIDI bridge for specific device
clasp midi --device "Launchpad X"

# MQTT bridge connecting to broker
clasp mqtt --host broker.local --port 1883 --topic "sensors/#"
```

### Desktop App

The CLASP desktop app manages bridges visually:
1. Click "Add Protocol"
2. Select protocol type
3. Configure options
4. Click "Start"

### Programmatic (Rust)

```rust
use clasp_bridge::osc::OscBridge;

let bridge = OscBridge::new(OscConfig {
    bind: "0.0.0.0:9000".parse()?,
    router_url: "ws://localhost:7330",
    prefix: "/osc",
})?;

bridge.run().await?;
```

## Protocol-Specific Behavior

### OSC

- Bundles map to CLASP bundles
- Timetags become timestamps
- All OSC types supported (int, float, string, blob, etc.)

### MIDI

- Note On/Off → Events
- CC → Params
- Program Change → Events
- Clock → Events (24 PPQ)

### DMX / Art-Net

- Channel values → Params (`/dmx/{universe}/{channel}`)
- Typically output-only (CLASP → DMX)
- Art-Net can be bidirectional

### MQTT

- Topics → Addresses
- Retained messages → Params
- Non-retained → Events
- QoS mapping: MQTT QoS 0 → CLASP Fire, etc.

## Signal Type Inference

Bridges infer CLASP signal types:

| External | CLASP Type |
|----------|------------|
| OSC continuous | Param or Stream |
| MIDI CC | Param |
| MIDI Note | Event |
| MQTT retained | Param |
| MQTT non-retained | Event |
| Art-Net DMX | Param |

Override with configuration:

```yaml
# bridge config
mappings:
  /osc/button/*:
    type: event
  /osc/fader/*:
    type: param
```

## Transform Pipelines

Bridges can transform data:

```yaml
transforms:
  - from: /midi/*/cc/1/7
    to: /audio/master/volume
    scale: [0, 127] -> [0, 1]

  - from: /osc/xy/*
    to: /position
    map:
      x: $[0]
      y: $[1]
```

### Common Transforms

| Transform | Description |
|-----------|-------------|
| `scale` | Linear mapping between ranges |
| `rename` | Change address |
| `map` | Restructure values |
| `filter` | Conditional forwarding |
| `threshold` | Only forward if value crosses threshold |

## Bridge Lifecycle

```
1. Bridge starts
2. Connects to CLASP router as client
3. Announces supported addresses
4. Starts listening on external protocol
5. Translates messages in both directions
6. On disconnect, attempts reconnection
7. On shutdown, closes cleanly
```

## Multiple Bridges

Run multiple bridges for different protocols:

```
┌─────────────────────────────────────┐
│            Router                   │
└───────────────┬─────────────────────┘
                │
    ┌───────────┼───────────┬───────────┐
    │           │           │           │
┌───▼───┐   ┌───▼───┐   ┌───▼───┐   ┌───▼───┐
│  OSC  │   │ MIDI  │   │ MQTT  │   │ArtNet │
│Bridge │   │Bridge │   │Bridge │   │Bridge │
└───────┘   └───────┘   └───────┘   └───────┘
```

Each bridge has its own address prefix, preventing conflicts.

## Bridge Discovery

Bridges can be discovered via their ANNOUNCE messages:

```javascript
{
  type: "ANNOUNCE",
  namespace: "/osc",
  meta: {
    bridge: true,
    protocol: "osc",
    port: 9000,
    bidirectional: true
  },
  signals: [
    { address: "/osc/**", type: "param" }
  ]
}
```

Clients can query for available bridges.

## Best Practices

### Use Prefixes

```
/osc/...    - OSC bridge
/midi/...   - MIDI bridge
/mqtt/...   - MQTT bridge
/dmx/...    - DMX/Art-Net
```

Prefixes prevent address collisions and clarify source.

### One Bridge Per Protocol Instance

```bash
# Two OSC sources
clasp osc --port 9000 --prefix /osc/touchosc
clasp osc --port 9001 --prefix /osc/resolume
```

### Handle Disconnections

Bridges should:
- Reconnect automatically
- Buffer messages during disconnect (optional)
- Log connection state

### Monitor Bridge Health

```javascript
// Check bridge status via CLASP
const bridges = await client.query('/_meta/bridges');
```

## See Also

- [OSC Bridge Reference](../reference/bridges/osc.md)
- [MIDI Bridge Reference](../reference/bridges/midi.md)
- [Art-Net Bridge Reference](../reference/bridges/artnet.md)
- [MQTT Bridge Reference](../reference/bridges/mqtt.md)

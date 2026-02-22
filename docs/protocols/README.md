---
title: Protocol Bridges
description: Bridge OSC, MIDI, MQTT, DMX, Art-Net, sACN, HTTP, and WebSocket to CLASP
order: 1
---

# Protocol Bridges

CLASP bridges 8 external protocols bidirectionally. Devices speaking OSC, MIDI, MQTT, Art-Net, DMX, sACN, HTTP, or WebSocket can participate in the same signal routing graph without modification. A lighting desk sending Art-Net, a synthesizer sending MIDI, and a web dashboard using WebSocket all share the same address space and see each other's state.

## How Bridging Works

A bridge sits between an external protocol and a CLASP router. It performs two translations:

**Inbound**: the bridge listens for messages on the external protocol, translates each message into a CLASP signal (address + value + signal type), and forwards it to the router.

**Outbound**: the bridge subscribes to CLASP signals matching its namespace. When a signal arrives from the router, it translates it back into the external protocol's format and sends it to the external device.

This bidirectional translation means any CLASP client can control an OSC synthesizer, any MQTT sensor can feed data to a web dashboard, and any Art-Net lighting fixture can be driven from a Python script -- all without those devices knowing CLASP exists.

## Two Modes

Bridges run in one of two modes depending on your deployment needs.

### Standalone Bridge

A standalone bridge is a separate process that connects to a CLASP router as a client. Use the `clasp <protocol>` CLI commands:

```bash
clasp osc --listen 8000 --target ws://localhost:7330
clasp midi --target ws://localhost:7330
clasp mqtt --broker mqtt://localhost:1883 --target ws://localhost:7330
clasp http --port 8080 --target ws://localhost:7330
```

Standalone bridges can run on different machines from the router, making them suitable for distributed setups where protocol devices are on separate networks.

### Embedded in Relay

MQTT and OSC bridges can run inside the relay process itself via command-line flags:

```bash
clasp-relay --mqtt-port 1883 --mqtt-namespace /mqtt
clasp-relay --osc-port 8000 --osc-namespace /osc
```

Embedded mode provides lower latency (no network hop between bridge and router) and simpler deployment (one process instead of two). Use this when the relay and the protocol devices are on the same machine or network.

## Address Mapping

External protocol addresses appear under a configurable namespace prefix in the CLASP address space. This prevents collisions between protocols and makes the origin of each signal clear.

| Protocol | External Address | CLASP Address |
|---|---|---|
| OSC | `/synth/volume` | `/osc/synth/volume` |
| MQTT | `sensors/temp` | `/mqtt/sensors/temp` |
| MIDI | Channel 1, CC 74 | `/midi/ch/1/cc/74` |
| Art-Net | Universe 1, Channel 1 | `/artnet/1/1` |
| DMX | Channel 100 | `/dmx/100` |
| sACN | Universe 2, Channel 50 | `/sacn/2/50` |
| HTTP | PUT `/v1/state/lights/on` | `/lights/on` |
| WebSocket | `{"address": "/app/status"}` | `/app/status` |

The namespace prefix is configurable per bridge instance. You can run multiple bridges of the same protocol with different namespaces (e.g., `/osc/stage` and `/osc/foh`).

## Signal Type Mapping

Each protocol's native message types map to CLASP signal types:

| Protocol | Message Type | CLASP Signal Type |
|---|---|---|
| OSC | message | Param |
| MIDI | CC | Param |
| MIDI | Note On / Note Off | Event |
| MQTT | retained message | Param |
| MQTT | non-retained message | Event |
| Art-Net | DMX frame | Stream |
| DMX | channel value | Param |
| sACN | DMX frame | Stream |
| HTTP | GET | Get |
| HTTP | PUT | Set |

## Available Bridges

| Bridge | Transport | Page |
|---|---|---|
| OSC | UDP / TCP | [OSC Bridge](osc.md) |
| MIDI | USB / virtual | [MIDI Bridge](midi.md) |
| MQTT | TCP | [MQTT Bridge](mqtt.md) |
| Art-Net | UDP | [Art-Net Bridge](artnet.md) |
| DMX | USB serial | [DMX Bridge](dmx.md) |
| sACN | UDP multicast | [sACN Bridge](sacn.md) |
| HTTP | TCP | [HTTP Bridge](http.md) |
| WebSocket | TCP | [WebSocket Bridge](websocket.md) |

All bridges are provided by the `clasp-bridge` crate, with each protocol behind a feature flag (`osc`, `midi`, `artnet`, `dmx`, `sacn`, `mqtt`, `http`, `websocket`).

## Next Steps

- Read individual bridge pages for protocol-specific details, CLI usage, and configuration
- See [Router Configuration](../reference/router-config.md) for embedded bridge setup
- See the individual bridge pages above for address mapping and value transform options

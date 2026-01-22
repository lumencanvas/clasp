# CLASP

**Creative Low-Latency Application Streaming Protocol**

CLASP is a universal protocol bridge and API gateway for creative applications. It connects disparate protocols like OSC, MIDI, DMX, Art-Net, MQTT, WebSockets, Socket.IO, and HTTP/REST into a unified, routable message system.

## What is CLASP?

CLASP provides:

- **Protocol Bridges** - Connect any protocol to any other protocol
- **Signal Mapping** - Route and transform signals between systems
- **REST API Gateway** - Stand up API endpoints that trigger protocol messages
- **Real-time Monitoring** - Watch signals flow through your system
- **Learn Mode** - Automatically capture addresses from incoming signals

## Use Cases

### Live Performance
Connect lighting (DMX/Art-Net), audio (OSC/MIDI), and video systems together. Map a MIDI controller to lighting cues or OSC messages.

### Installation Art
Bridge sensors and actuators across different protocols. Use MQTT for IoT devices, OSC for sound, and DMX for lighting.

### Home Automation
Create REST APIs that trigger home automation events. Map HTTP endpoints to MQTT topics or OSC messages.

### Software Integration
Connect creative tools that speak different protocols. Bridge TouchDesigner (OSC) with Ableton (MIDI) and custom WebSocket apps.

## Quick Start

```bash
# Install the CLI
cargo install clasp-cli

# Or download the desktop app from releases

# Start the bridge service
clasp serve

# Create an OSC to MIDI bridge
clasp bridge create --source osc:8000 --target midi:default
```

## Documentation

- [Getting Started](./getting-started/installation.md)
- [Protocol Bridges](./protocols/)
- [Signal Mapping](./concepts/mappings.md)
- [REST API Designer](./app/api-designer.md)
- [Examples](./examples/)

## Supported Protocols

| Protocol | Direction | Status |
|----------|-----------|--------|
| OSC | Bidirectional | ✅ Stable |
| MIDI | Bidirectional | ✅ Stable |
| DMX | Output | ✅ Stable |
| Art-Net | Bidirectional | ✅ Stable |
| MQTT | Bidirectional | ✅ Implemented |
| WebSocket | Bidirectional | ✅ Implemented |
| Socket.IO | Bidirectional | 🚧 Planned |
| HTTP/REST | Server + Client | ✅ Implemented |

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   OSC App   │     │  MQTT Hub   │     │  REST API   │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       ▼                   ▼                   ▼
┌──────────────────────────────────────────────────────┐
│                    CLASP Router                       │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  │
│  │OSC Brg  │  │MQTT Brg │  │HTTP Brg │  │MIDI Brg │  │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘  │
│                                                       │
│              Signal Routing & Transforms              │
└──────────────────────────────────────────────────────┘
       │                   │                   │
       ▼                   ▼                   ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│ MIDI Device  │   │   Lighting   │   │   Web App    │
└──────────────┘   └──────────────┘   └──────────────┘
```

## Desktop App

The CLASP desktop app provides a visual interface for:

- Creating and managing protocol bridges
- Designing signal mappings with transforms
- Building REST API endpoints
- Monitoring real-time signal flow
- Learning addresses from incoming signals

![CLASP Desktop App](./assets/app-screenshot.png)

## License

MIT or Apache-2.0

---

*CLASP - Connect everything.*

# CLASP Documentation

**Creative Low-Latency Application Streaming Protocol**

CLASP is a universal protocol bridge and signal router for creative applications. It unifies disparate protocols (OSC, MIDI, DMX, Art-Net, MQTT, WebSocket, HTTP) into a single, routable message system optimized for real-time performance.

---

## Quick Start by Role

### I'm building a web or desktop app
Start with the [First Connection Tutorial](tutorials/first-connection.md), then explore the [JavaScript](reference/api/javascript/clasp-core.md) or [Python](reference/api/python/clasp-to.md) API reference.

### I'm working with live performance / VJ / lighting
See the [Live Performance Guide](use-cases/live-performance.md) for connecting lighting, audio, and video systems. Learn how to [add OSC](how-to/connections/add-osc.md), [MIDI](how-to/connections/add-midi.md), and [DMX](how-to/connections/add-dmx.md) connections.

### I'm building an installation or IoT system
Check the [Installation Art Guide](use-cases/installation-art.md) and [Home Automation Guide](use-cases/home-automation.md). Learn about [MQTT integration](how-to/connections/add-mqtt.md) and [sensor pipelines](tutorials/sensor-to-visualization.md).

### I'm working with embedded systems / microcontrollers
See the [Embedded Systems Guide](use-cases/embedded-systems.md) and [ESP32 Tutorial](tutorials/embedded-sensor-node.md). Reference the [clasp-embedded API](reference/api/rust/clasp-embedded.md).

### I'm deploying to cloud / building a SaaS
Read the [Cloud Deployment Guide](use-cases/cloud-deployment.md) and [Docker setup](how-to/installation/docker.md). Learn about [security](how-to/security/enable-tls.md) and [capability tokens](how-to/security/capability-tokens.md).

### I want to understand the protocol
Start with [Why CLASP?](explanation/why-clasp.md) and [Architecture Overview](explanation/architecture.md), then dive into the [Protocol Reference](reference/protocol/overview.md).

---

## Documentation Sections

### [Tutorials](tutorials/README.md) — Learning-Oriented
Step-by-step guides to learn CLASP by doing.

- [First Connection](tutorials/first-connection.md) — Connect two apps in 5 minutes
- [Control Lights from Web](tutorials/control-lights-from-web.md) — Build a web UI controlling DMX
- [Sensor to Visualization](tutorials/sensor-to-visualization.md) — IoT sensor → visual app pipeline
- [Cross-Language Chat](tutorials/cross-language-chat.md) — JS ↔ Python ↔ Rust communication
- [Embedded Sensor Node](tutorials/embedded-sensor-node.md) — Build an ESP32 sensor with CLASP

### [How-To Guides](how-to/README.md) — Task-Oriented
Solve specific problems with focused instructions.

- **[Installation](how-to/installation/)** — Install CLI, libraries, desktop app, Docker
- **[Connections](how-to/connections/)** — Add OSC, MIDI, DMX, Art-Net, MQTT, HTTP, WebSocket
- **[State](how-to/state/)** — Subscribe, get/set values, handle conflicts, use locks
- **[Timing](how-to/timing/)** — Clock sync, scheduled bundles, atomic operations
- **[Discovery](how-to/discovery/)** — mDNS, UDP broadcast, manual configuration
- **[Security](how-to/security/)** — TLS, capability tokens, pairing
- **[Advanced](how-to/advanced/)** — P2P WebRTC, custom bridges, embed router, performance tuning
- **[Troubleshooting](how-to/troubleshooting.md)** — Common issues and solutions

### [Reference](reference/README.md) — Information-Oriented
Complete, accurate technical details.

- **[Protocol](reference/protocol/)** — Messages, signal types, addressing, data types, frame format, QoS
- **[API](reference/api/)** — Rust, JavaScript, Python library documentation
- **[CLI](reference/cli/)** — Command-line tool reference
- **[Bridges](reference/bridges/)** — OSC, MIDI, Art-Net, DMX, MQTT, sACN, HTTP mapping
- **[Transports](reference/transports/)** — WebSocket, QUIC, UDP, WebRTC, Serial, BLE
- **[Configuration](reference/configuration/)** — Router config, bridge config, feature flags

### [Explanation](explanation/README.md) — Understanding-Oriented
Concepts, background, and design rationale.

- [Why CLASP?](explanation/why-clasp.md) — The problem CLASP solves
- [Architecture](explanation/architecture.md) — System architecture overview
- [Router vs Client](explanation/router-vs-client.md) — Understanding roles
- [Signals Not Messages](explanation/signals-not-messages.md) — Semantic signal types
- [State Management](explanation/state-management.md) — How state works
- [Timing Model](explanation/timing-model.md) — Clock sync and scheduling
- [Security Model](explanation/security-model.md) — Encryption and tokens

### [Use Cases](use-cases/README.md) — Persona-Specific Guides
Real-world applications and workflows.

- [Live Performance](use-cases/live-performance.md) — VJ, lighting, music production
- [Installation Art](use-cases/installation-art.md) — Interactive installations
- [Home Automation](use-cases/home-automation.md) — IoT and smart home
- [Software Integration](use-cases/software-integration.md) — Connecting applications
- [Embedded Systems](use-cases/embedded-systems.md) — Microcontrollers
- [Cloud Deployment](use-cases/cloud-deployment.md) — Docker, Kubernetes, SaaS

### [Integrations](integrations/README.md) — Third-Party Software
Connect CLASP to popular creative tools.

- [TouchOSC](integrations/touchosc.md)
- [Resolume](integrations/resolume.md)
- [QLab](integrations/qlab.md)
- [Ableton Live](integrations/ableton.md)
- [TouchDesigner](integrations/touchdesigner.md)
- [MadMapper](integrations/madmapper.md)
- [Home Assistant](integrations/home-assistant.md)

### [Appendix](appendix/)
- [Glossary](appendix/glossary.md) — Term definitions
- [FAQ](appendix/faq.md) — Frequently asked questions
- [Changelog](appendix/changelog.md) — Version history
- [Migration from OSC](appendix/migration/from-osc.md)
- [Migration from MQTT](appendix/migration/from-mqtt.md)

---

## Install

### CLI

```bash
cargo install clasp-cli
```

### Libraries

| Platform | Package | Install |
|----------|---------|---------|
| **Rust** | [clasp-client](https://crates.io/crates/clasp-client) | `cargo add clasp-client` |
| **JavaScript** | [@clasp-to/core](https://www.npmjs.com/package/@clasp-to/core) | `npm install @clasp-to/core` |
| **Python** | [clasp-to](https://pypi.org/project/clasp-to/) | `pip install clasp-to` |

### Desktop App

Download from [GitHub Releases](https://github.com/lumencanvas/clasp/releases/latest):
- **macOS**: CLASP Bridge.dmg
- **Windows**: CLASP Bridge Setup.exe
- **Linux**: clasp-bridge.AppImage

---

## Quick Example

**Start a router:**
```bash
clasp server --port 7330
```

**JavaScript client:**
```typescript
import { ClaspBuilder } from '@clasp-to/core';

const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('My App')
  .connect();

client.on('/sensors/*', (value, address) => {
  console.log(`${address} = ${value}`);
});

await client.set('/lights/brightness', 0.8);
```

**Python client:**
```python
from clasp import ClaspBuilder

client = await (
    ClaspBuilder('ws://localhost:7330')
    .with_name('Sensor')
    .connect()
)

await client.set('/sensors/temperature', 23.5)
```

---

## Architecture Overview

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  TouchOSC   │     │   Ableton   │     │  LED Strip  │
│  (OSC)      │     │   (MIDI)    │     │  (Art-Net)  │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    ┌──────▼──────┐
                    │    CLASP    │
                    │   Router    │
                    └──────┬──────┘
                           │
       ┌───────────────────┼───────────────────┐
       │                   │                   │
┌──────▼──────┐     ┌──────▼──────┐     ┌──────▼──────┐
│  Web UI     │     │  IoT Hub    │     │  Resolume   │
│ (WebSocket) │     │  (MQTT)     │     │  (OSC)      │
└─────────────┘     └─────────────┘     └─────────────┘
```

---

## Resources

- **Website**: [clasp.to](https://clasp.to)
- **GitHub**: [lumencanvas/clasp](https://github.com/lumencanvas/clasp)
- **Protocol Spec**: [CLASP-Protocol.md](https://github.com/lumencanvas/clasp/blob/main/CLASP-Protocol.md)
- **Quick Reference**: [CLASP-QuickRef.md](https://github.com/lumencanvas/clasp/blob/main/CLASP-QuickRef.md)

---

*CLASP — Connect everything.*

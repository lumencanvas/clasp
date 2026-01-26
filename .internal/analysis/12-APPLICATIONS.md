# Applications Analysis

## Overview

The CLASP repository includes:
- **CLASP Bridge** - Desktop app for protocol routing
- **Marketing Site** - Documentation and playground
- **Examples** - Multi-language reference implementations

---

## CLASP Bridge Desktop App

**Location:** `apps/bridge/`
**Version:** 0.3.0
**Stack:** Electron + Vue.js + Rust backend

### Architecture

```
Renderer Process (Vue.js UI)
    ↓ (ipcRenderer.invoke)
Preload Script (Context Bridge)
    ↓
Main Process (Electron)
    ├→ Native APIs
    ├→ File System
    ├→ Rust Binaries (clasp-service, clasp-router)
    └→ System Integration
```

### Key Files

| File | Lines | Purpose |
|------|-------|---------|
| `electron/main.js` | 2,660 | Main process |
| `src/app.js` | 4,855 | Vue app logic |
| `electron/preload.js` | 113 | IPC bridge |
| `src/lib/config-io.js` | 244 | Config persistence |
| `src/presets/index.js` | 307 | Workflow templates |

### IPC API

**Device/Server:**
- `getDevices()`, `scanNetwork()`
- `addServer()`, `startServer()`, `stopServer()`

**Bridge Management:**
- `getBridges()`, `createBridge()`, `deleteBridge()`

**Server Operations:**
- `getServerLogs()`, `testConnection()`, `getServerStats()`
- `healthCheck()`, `runDiagnostics()`

**Hardware:**
- `listSerialPorts()`, `listMidiPorts()`, `listNetworkInterfaces()`

**Signals:**
- `sendSignal()`, `sendTestSignal()`, `sendTestSignalBatch()`

**Learn Mode:**
- `startLearnMode()`, `stopLearnMode()`

### State Structure

```javascript
state = {
    routers: [],
    servers: [],
    devices: [],
    bridges: [],
    mappings: [],
    signals: [],
    serverLogs: Map,
    signalRate: 0,
    bridgeServiceReady: false,
    activeTab: 'bridges',
    learnMode: false,
    tokens: []
}
```

### Protocol Support

| Protocol | Default Address |
|----------|----------------|
| OSC | 0.0.0.0:9000 |
| MIDI | default |
| Art-Net | 0.0.0.0:6454 |
| DMX | /dev/ttyUSB0 |
| CLASP | localhost:7330 |
| MQTT | localhost:1883 |
| WebSocket | 0.0.0.0:8080 |
| HTTP | 0.0.0.0:3000 |

### Workflow Presets

1. **Latch Flow Designer** - OSC visual design
2. **VJ Setup** - TouchOSC → Resolume
3. **Lighting Console** - OSC/MIDI → Art-Net/DMX
4. **MIDI Hub** - MIDI → OSC + WebSocket
5. **Sensor Network** - MQTT IoT
6. **Web Control** - WebSocket + HTTP
7. **Minimal Setup** - Basic CLASP

### Build Targets

| Platform | Format |
|----------|--------|
| macOS | DMG, ZIP (arm64, x64) |
| Windows | NSIS, Portable |
| Linux | AppImage, DEB |

### Bundled Rust Binaries

```json
"extraResources": [
    { "from": "../../target/release/clasp-service", "to": "bin/clasp-service" },
    { "from": "../../target/release/clasp-router", "to": "bin/clasp-router" }
]
```

---

## Marketing Site

**Location:** `site/`
**Version:** 1.0.0
**Stack:** Vue 3 + Vite + Vue Router

### Routes

| Path | Component |
|------|-----------|
| `/` | HomePage |
| `/playground` | PlaygroundPage |

### Homepage Sections

1. HeroSection
2. LayersSection (Architecture)
3. DownloadsSection
4. ApiSection (SDK examples)
5. SpecSection (Protocol spec)
6. CapabilitiesSection
7. FooterSection

### Playground Features

**Tabs:**
- Explorer - Signal exploration
- Chat - Protocol messaging
- Sensors - Data visualization
- Security - Auth/tokens
- Discovery - Server discovery

**Components:**
- ConnectionPanel - Server connection
- ConsolePanel - Message logging

### useClasp Composable

```javascript
// Reactive state
{
    client: shallowRef,
    connected: ref(false),
    sessionId: ref(null),
    params: reactive(Map),
    messageLog: ref([]),
    settings: reactive({
        url: 'ws://localhost:7330',
        name: 'Playground Client',
        features: ['param', 'event', 'stream', 'gesture', 'timeline']
    }),
    discoveredServers: ref([])
}

// Methods
connect(), disconnect(), scan()
subscribe(), set(), emit(), stream(), get(), bundle()
time(), clearLog()
```

### Server Discovery

Auto-probes:
- localhost:7330, 8080, 9000
- 127.0.0.1:7330, 8080, 9000
- 2-second timeout per port

### Design System

```css
:root {
    --paper: #F5F0E6;     /* Background */
    --ink: #1a1a1a;       /* Text */
    --accent: #FF5F1F;    /* Orange */
    --muted: rgba(0,0,0,0.35);
}
```

---

## Example Applications

**Location:** `examples/`

### JavaScript (11 examples)

| File | Feature |
|------|---------|
| `simple-publisher.js` | Basic publishing |
| `simple-subscriber.js` | Basic subscription |
| `signal-types.js` | All signal types |
| `bundles-and-scheduling.js` | Atomic bundles |
| `gestures.js` | Touch input |
| `discovery.js` | mDNS/UDP discovery |
| `locks.js` | Parameter locking |
| `late-joiner.js` | State sync |
| `security-tokens.js` | Authentication |
| `p2p-webrtc.js` | WebRTC P2P |
| `embedded-server.js` | Embedded router |

### Python (6 examples)

| File | Feature |
|------|---------|
| `signal_types.py` | All signal types |
| `bundles_and_scheduling.py` | Atomic bundles |
| `late_joiner.py` | State sync |
| `security_tokens.py` | Authentication |
| `p2p_webrtc.py` | WebRTC P2P |
| `embedded_server.py` | Embedded router |

### Rust (8 examples)

| File | Feature |
|------|---------|
| `basic-client.rs` | Client basics |
| `signal_types.rs` | All signal types |
| `bundles_and_scheduling.rs` | Atomic bundles |
| `late_joiner.rs` | State sync |
| `security_tokens.rs` | Authentication |
| `p2p_webrtc.rs` | WebRTC P2P |
| `embedded-server.rs` | Embedded router |
| `bundles.rs` | Bundle operations |

### Feature Coverage

| Feature | JS | Python | Rust |
|---------|:---:|:------:|:----:|
| Params | ✓ | ✓ | ✓ |
| Events | ✓ | ✓ | ✓ |
| Streams | ✓ | ✓ | ✓ |
| Gestures | ✓ | ✓ | ✓ |
| Timelines | ✓ | ✓ | ✓ |
| Bundles | ✓ | ✓ | ✓ |
| Wildcards | ✓ | ✓ | ✓ |
| Late Joiner | ✓ | ✓ | ✓ |
| Locks | ✓ | - | - |
| Discovery | ✓ | - | - |
| Security | ✓ | ✓ | ✓ |
| P2P WebRTC | ✓ | ✓ | ✓ |
| Embedded Server | ✓ | ✓ | ✓ |

### Running Examples

**JavaScript:**
```bash
npm install @clasp-to/core
node examples/js/simple-publisher.js
```

**Python:**
```bash
pip install clasp-to
python examples/python/signal_types.py
```

**Rust:**
```bash
cargo run --example basic-client
```

---

## Documentation Structure

**Location:** `docs/`

```
docs/
├── index.md
├── architecture.md
├── api/
├── explanation/
│   ├── why-clasp.md
│   ├── signals-not-messages.md
│   ├── router-vs-client.md
│   ├── state-management.md
│   └── security-model.md
├── getting-started/
├── guides/
├── how-to/
│   └── advanced/
│       └── custom-bridge.md
├── integrations/
│   ├── TouchOSC/
│   ├── Resolume/
│   └── QLab/
├── tutorials/
├── use-cases/
├── reference/
└── appendix/
    ├── changelog.md
    ├── glossary.md
    ├── faq.md
    └── migration/
        ├── from-osc.md
        └── from-mqtt.md
```

---

## Build Commands

### Bridge App

```bash
npm run dev          # Development
npm run build        # Full build
npm run prebuild     # Rust binaries only
npm run test         # Run tests
```

### Marketing Site

```bash
npm run dev          # Dev server
npm run build        # Production build
npm run preview      # Preview build
```

---

## Technology Stack

### Bridge App

| Layer | Technology |
|-------|------------|
| Desktop | Electron 28 |
| UI | Vue 3 / Vanilla JS |
| Build | Vite 5 |
| Backend | Rust binaries |
| Serial | serialport |
| Packaging | electron-builder |

### Marketing Site

| Layer | Technology |
|-------|------------|
| Framework | Vue 3.5 |
| Routing | Vue Router 4.6 |
| Build | Vite 7.3 |
| Syntax | highlight.js |
| Client | @clasp-to/core |

---

## Deployment

### Bridge App

- macOS: Code signed + notarized
- Windows: NSIS installer
- Linux: AppImage (portable)

### Marketing Site

- Static hosting (any provider)
- Relative paths for portability
- Output: `site/dist/`

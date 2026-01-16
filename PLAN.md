# SignalFlow Monorepo - Master Plan

## Executive Summary

**SignalFlow** is a universal protocol for creative tools communication. This plan outlines building a complete ecosystem: a core Rust library (compilable to WASM), language bindings, protocol bridges, and a cross-platform UI application for bridging creative protocols.

---

## Part 1: Protocol Improvements

### 1.1 Specification Refinements

#### Current State
The SF/2 spec is solid but needs:

| Area | Issue | Improvement |
|------|-------|-------------|
| **Embedded Lite** | Under-specified | Full binary format for 2-byte addresses, fixed message types |
| **WebRTC Signaling** | "Use whatever works" | Define minimal signaling protocol for peer exchange |
| **Multicast Streams** | Not addressed | Add sACN-style multicast subscription for high-device-count scenarios |
| **Mesh Topology** | Missing | Define peer routing for decentralized setups |
| **MIDI 2.0** | Not bridged | Add Property Exchange and UMP mapping |
| **Serial Framing** | Unspecified | Define SLIP/COBS framing for serial transport |

#### Proposed Additions

```
SF/2.1 Additions:
├── Embedded Lite Profile (full spec)
├── WebRTC Signaling Protocol
├── Multicast Extension
├── Mesh Routing Extension
├── MIDI 2.0 Bridge Spec
└── Serial Transport Framing
```

### 1.2 New Signal Types

| Type | Purpose | Use Case |
|------|---------|----------|
| **Query** | Request/response patterns | RPC-style calls |
| **Blob** | Large binary transfers | Firmware, images, presets |
| **Log** | Structured logging | Debug streams |

---

## Part 2: Core Library Architecture

### 2.1 Why Rust?

| Requirement | Rust Fit |
|-------------|----------|
| Cross-platform | ✓ Compiles to all major platforms |
| WASM | ✓ First-class wasm32 target |
| Performance | ✓ Zero-cost abstractions |
| Safety | ✓ Memory safety without GC |
| FFI | ✓ Easy C ABI for bindings |
| Existing ecosystem | ✓ Great crates for protocols we need |

**Key Crates Available:**
- `rosc` - OSC parsing
- `midir` / `midi-msg` - MIDI I/O
- `artnet_protocol` - Art-Net
- `mdns-sd` - mDNS discovery
- `rmp-serde` - MessagePack
- `tokio` - Async runtime
- `webrtc` - WebRTC (pure Rust)
- `quinn` - QUIC
- `serialport` - Serial I/O

### 2.2 Crate Structure

```
signalflow/
├── crates/
│   ├── signalflow-core/          # Protocol types, encoding, no I/O
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs          # Signal types, messages
│   │   │   ├── codec.rs          # MessagePack encode/decode
│   │   │   ├── frame.rs          # Binary frame format
│   │   │   ├── address.rs        # Address parsing, wildcards
│   │   │   ├── state.rs          # Param state, conflict resolution
│   │   │   └── time.rs           # Timestamps, sync
│   │   └── Cargo.toml
│   │
│   ├── signalflow-transport/     # Transport abstractions
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── traits.rs         # Transport trait
│   │   │   ├── websocket.rs      # WebSocket (tungstenite)
│   │   │   ├── udp.rs            # Raw UDP
│   │   │   ├── quic.rs           # QUIC (quinn)
│   │   │   ├── webrtc.rs         # WebRTC DataChannel
│   │   │   ├── serial.rs         # Serial port
│   │   │   └── ble.rs            # Bluetooth LE
│   │   └── Cargo.toml
│   │
│   ├── signalflow-discovery/     # Discovery mechanisms
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── mdns.rs           # mDNS/Bonjour
│   │   │   ├── broadcast.rs      # UDP broadcast fallback
│   │   │   └── rendezvous.rs     # Cloud rendezvous client
│   │   └── Cargo.toml
│   │
│   ├── signalflow-bridge/        # Protocol bridges
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── traits.rs         # Bridge trait
│   │   │   ├── midi.rs           # MIDI 1.0/2.0
│   │   │   ├── osc.rs            # OSC
│   │   │   ├── dmx.rs            # DMX-512 (via serial/USB)
│   │   │   ├── artnet.rs         # Art-Net
│   │   │   ├── sacn.rs           # sACN/E1.31
│   │   │   └── serial_raw.rs     # Raw serial protocols
│   │   └── Cargo.toml
│   │
│   ├── signalflow-router/        # Full router implementation
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── router.rs         # Message routing
│   │   │   ├── session.rs        # Session management
│   │   │   ├── state.rs          # Global state store
│   │   │   ├── subscription.rs   # Subscription matching
│   │   │   └── security.rs       # Auth, tokens
│   │   └── Cargo.toml
│   │
│   ├── signalflow-client/        # High-level client API
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── client.rs         # Main client struct
│   │   │   ├── builder.rs        # Builder pattern
│   │   │   └── reactive.rs       # Reactive bindings
│   │   └── Cargo.toml
│   │
│   ├── signalflow-embedded/      # Embedded-friendly (no_std)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── lite.rs           # Lite profile
│   │   │   └── codec.rs          # Minimal MessagePack
│   │   └── Cargo.toml
│   │
│   └── signalflow-wasm/          # WASM bindings
│       ├── src/
│       │   └── lib.rs
│       └── Cargo.toml
```

### 2.3 Feature Flags

```toml
# signalflow-core/Cargo.toml
[features]
default = ["std"]
std = []
alloc = []  # For no_std with allocator
embedded = []  # Lite profile only

# signalflow/Cargo.toml (umbrella)
[features]
default = ["client", "discovery"]
full = ["client", "router", "bridges", "discovery", "webrtc"]
client = ["signalflow-client"]
router = ["signalflow-router"]
bridges = ["signalflow-bridge"]
discovery = ["signalflow-discovery"]
webrtc = ["signalflow-transport/webrtc"]
wasm = ["signalflow-wasm"]
```

### 2.4 Core API Design (Rust)

```rust
// High-level client usage
use signalflow::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect
    let sf = SignalFlow::connect("wss://localhost:7330").await?;

    // Subscribe with pattern
    sf.subscribe("/lumen/scene/*/layer/*/opacity")
        .on_change(|value: f32, address: &str| {
            println!("{} = {}", address, value);
        })
        .with_max_rate(30)
        .build()?;

    // Set value
    sf.set("/lumen/scene/0/layer/0/opacity", 0.75f32).await?;

    // Emit event
    sf.emit("/cue/fire", json!({ "id": "intro" })).await?;

    // Bundle with scheduling
    sf.bundle()
        .set("/light/1", 1.0)
        .set("/light/2", 0.0)
        .at(sf.time() + Duration::from_millis(100))
        .send()
        .await?;

    Ok(())
}
```

---

## Part 3: Language Bindings

### 3.1 Binding Strategy

| Language | Binding Method | Priority |
|----------|---------------|----------|
| **JavaScript/TypeScript** | WASM + wasm-bindgen | P0 |
| **Python** | PyO3 | P0 |
| **C/C++** | cbindgen (C ABI) | P1 |
| **Swift** | UniFFI | P1 |
| **Kotlin/Java** | JNI via UniFFI | P2 |
| **Go** | CGO | P2 |
| **C#/.NET** | P/Invoke | P2 |

### 3.2 Package Structure

```
bindings/
├── js/                           # JavaScript/TypeScript
│   ├── packages/
│   │   ├── signalflow/           # Main package (WASM core)
│   │   │   ├── src/
│   │   │   │   ├── index.ts
│   │   │   │   ├── client.ts
│   │   │   │   └── types.ts
│   │   │   ├── package.json
│   │   │   └── tsconfig.json
│   │   │
│   │   ├── signalflow-node/      # Node.js native addon
│   │   │   └── ...
│   │   │
│   │   └── signalflow-react/     # React hooks
│   │       └── ...
│   │
│   └── package.json              # Workspace root
│
├── python/                       # Python
│   ├── signalflow/
│   │   ├── __init__.py
│   │   ├── _native.pyi           # Type stubs
│   │   └── py.typed
│   ├── src/                      # PyO3 source
│   │   └── lib.rs
│   ├── pyproject.toml
│   └── Cargo.toml
│
├── swift/                        # Swift
│   ├── Sources/
│   │   └── SignalFlow/
│   │       └── SignalFlow.swift
│   ├── Package.swift
│   └── signalflow.udl            # UniFFI definition
│
├── kotlin/                       # Kotlin/Android
│   ├── lib/
│   │   └── src/main/kotlin/
│   │       └── SignalFlow.kt
│   └── build.gradle.kts
│
└── csharp/                       # C#/.NET
    ├── SignalFlow/
    │   └── SignalFlow.cs
    └── SignalFlow.csproj
```

### 3.3 JavaScript/TypeScript API

```typescript
// signalflow/src/index.ts
import init, { SignalFlowCore } from './wasm/signalflow_wasm';

export class SignalFlow {
  private core: SignalFlowCore;

  static async connect(url: string, options?: ConnectOptions): Promise<SignalFlow> {
    await init();
    const sf = new SignalFlow();
    sf.core = await SignalFlowCore.connect(url, options);
    return sf;
  }

  // Reactive API
  subscribe<T = unknown>(
    pattern: string,
    callback: (value: T, address: string) => void,
    options?: SubscribeOptions
  ): Unsubscribe;

  // Shorthand
  on<T = unknown>(
    pattern: string,
    callback: (value: T, address: string) => void,
    options?: SubscribeOptions
  ): Unsubscribe;

  set(address: string, value: unknown): Promise<void>;
  get<T = unknown>(address: string): Promise<T>;
  emit(address: string, payload?: unknown): Promise<void>;
  stream(address: string, value: unknown): void;

  bundle(messages: BundleMessage[], options?: BundleOptions): Promise<void>;

  time(): bigint;

  // Discovery (native only)
  static discover(options?: DiscoverOptions): Promise<Device[]>;
}

// React hooks
export function useSignalFlow(url: string): SignalFlow | null;
export function useParam<T>(sf: SignalFlow, address: string): T | undefined;
export function useSubscription<T>(
  sf: SignalFlow,
  pattern: string,
  callback: (value: T, address: string) => void
): void;
```

### 3.4 Python API

```python
# signalflow/__init__.py
from ._native import SignalFlowCore

class SignalFlow:
    """SignalFlow client for Python"""

    def __init__(self, url: str, **options):
        self._core = SignalFlowCore(url, options)

    async def connect(self) -> None:
        """Connect to SignalFlow server"""
        await self._core.connect()

    def subscribe(
        self,
        pattern: str,
        callback: Callable[[Any, str], None],
        **options
    ) -> Callable[[], None]:
        """Subscribe to address pattern, returns unsubscribe function"""
        return self._core.subscribe(pattern, callback, options)

    # Decorator style
    def on(self, pattern: str, **options):
        """Decorator for subscriptions"""
        def decorator(func):
            self.subscribe(pattern, func, **options)
            return func
        return decorator

    async def set(self, address: str, value: Any) -> None:
        """Set parameter value"""
        await self._core.set(address, value)

    async def get(self, address: str) -> Any:
        """Get current parameter value"""
        return await self._core.get(address)

    async def emit(self, address: str, payload: Any = None) -> None:
        """Emit event"""
        await self._core.emit(address, payload)

    def stream(self, address: str, value: Any) -> None:
        """Send stream sample (fire-and-forget)"""
        self._core.stream(address, value)

    def bundle(
        self,
        messages: list[dict],
        at: int | None = None
    ) -> None:
        """Send atomic bundle, optionally scheduled"""
        self._core.bundle(messages, at)

    def time(self) -> int:
        """Server-synced time in microseconds"""
        return self._core.time()

    # Sync API for simple scripts
    def run(self) -> None:
        """Run event loop (blocking)"""
        asyncio.get_event_loop().run_forever()

# Async context manager
async def connect(url: str, **options) -> SignalFlow:
    sf = SignalFlow(url, **options)
    await sf.connect()
    return sf
```

---

## Part 4: Cross-Platform Application

### 4.1 Technology Choice: Tauri v2

| Option | Pros | Cons | Verdict |
|--------|------|------|---------|
| **Tauri v2** | Rust backend, tiny bundles, native performance | Webview varies by platform | ✓ **Best fit** |
| Electron | Mature, consistent | 150MB+ bundles, memory hog | ✗ |
| Flutter | Good perf, single codebase | Dart ecosystem, not native feel | ✗ |
| Qt | Native, mature | C++, licensing complexity | ✗ |
| .NET MAUI | Native UI | C# only, Windows-centric | ✗ |

**Why Tauri:**
- Rust backend = seamless SignalFlow core integration
- ~10MB bundle vs 150MB Electron
- v2 supports iOS/Android
- We control the webview content entirely

### 4.2 App Architecture

```
apps/
└── bridge/                       # SignalFlow Bridge app
    ├── src-tauri/                # Rust backend
    │   ├── src/
    │   │   ├── main.rs
    │   │   ├── commands.rs       # Tauri commands
    │   │   ├── bridge_manager.rs # Bridge orchestration
    │   │   ├── discovery.rs      # Device discovery
    │   │   └── state.rs          # App state
    │   ├── Cargo.toml
    │   └── tauri.conf.json
    │
    ├── src/                      # Frontend (Solid.js)
    │   ├── index.html
    │   ├── index.tsx
    │   ├── App.tsx
    │   ├── components/
    │   │   ├── Layout.tsx
    │   │   ├── Sidebar.tsx
    │   │   ├── DeviceList.tsx
    │   │   ├── BridgeConfig.tsx
    │   │   ├── ConnectionStatus.tsx
    │   │   ├── SignalMonitor.tsx
    │   │   └── Settings.tsx
    │   ├── stores/
    │   │   ├── devices.ts
    │   │   ├── bridges.ts
    │   │   └── signals.ts
    │   └── styles/
    │       ├── global.css
    │       └── theme.css
    │
    ├── package.json
    └── vite.config.ts
```

### 4.3 UI Design System

#### Design Language: "Paper Brutalist"
- **Foundation**: Clean geometric shapes, sharp edges
- **Texture**: Subtle paper-like noise/grain
- **Depth**: Crisp shadows (no blur), layered cards
- **Typography**: Monospace for data, geometric sans for UI
- **Motion**: Snappy, purposeful (no bounces)

#### Color Palette: Teal

```css
:root {
  /* Primary Teal Scale */
  --teal-50:  #f0fdfa;
  --teal-100: #ccfbf1;
  --teal-200: #99f6e4;
  --teal-300: #5eead4;
  --teal-400: #2dd4bf;
  --teal-500: #14b8a6;  /* Primary */
  --teal-600: #0d9488;
  --teal-700: #0f766e;
  --teal-800: #115e59;
  --teal-900: #134e4a;
  --teal-950: #042f2e;

  /* Neutrals (warm gray for paper feel) */
  --paper-50:  #fafaf9;
  --paper-100: #f5f5f4;
  --paper-200: #e7e5e4;
  --paper-300: #d6d3d1;
  --paper-400: #a8a29e;
  --paper-500: #78716c;
  --paper-600: #57534e;
  --paper-700: #44403c;
  --paper-800: #292524;
  --paper-900: #1c1917;

  /* Semantic */
  --success: #10b981;
  --warning: #f59e0b;
  --error:   #ef4444;
  --info:    var(--teal-500);

  /* Brutalist shadows */
  --shadow-sm:  2px 2px 0 var(--paper-900);
  --shadow-md:  4px 4px 0 var(--paper-900);
  --shadow-lg:  6px 6px 0 var(--paper-900);

  /* Paper texture (optional) */
  --paper-texture: url("data:image/svg+xml,..."); /* noise pattern */
}
```

#### Component Examples

```tsx
// Card component - brutalist style
const Card = ({ children, elevated }) => (
  <div class={`
    bg-paper-50
    border-2 border-paper-900
    ${elevated ? 'shadow-md' : ''}
    transition-transform
    hover:translate-x-[-2px] hover:translate-y-[-2px]
    hover:shadow-lg
  `}>
    {children}
  </div>
);

// Button - solid, geometric
const Button = ({ variant = 'primary', children }) => (
  <button class={`
    px-4 py-2
    font-mono uppercase tracking-wide text-sm
    border-2 border-paper-900
    transition-all duration-100
    ${variant === 'primary'
      ? 'bg-teal-500 text-white hover:bg-teal-600'
      : 'bg-paper-100 hover:bg-paper-200'}
    shadow-sm hover:shadow-md
    hover:translate-x-[-2px] hover:translate-y-[-2px]
    active:translate-x-0 active:translate-y-0
    active:shadow-none
  `}>
    {children}
  </button>
);

// Device badge
const DeviceBadge = ({ device, status }) => (
  <div class="flex items-center gap-2 p-2 border-2 border-paper-900 bg-paper-50">
    <div class={`w-2 h-2 ${status === 'connected' ? 'bg-success' : 'bg-paper-400'}`} />
    <span class="font-mono text-sm">{device.name}</span>
    <span class="text-xs text-paper-500 uppercase">{device.protocol}</span>
  </div>
);
```

### 4.4 App Features

#### Core Features (v1.0)

```
┌─────────────────────────────────────────────────────────────────┐
│  SignalFlow Bridge                                    ─ □ ×    │
├───────────────────┬─────────────────────────────────────────────┤
│                   │                                             │
│  ┌─────────────┐  │  ┌─────────────────────────────────────┐   │
│  │ DISCOVERED  │  │  │ BRIDGES                              │   │
│  ├─────────────┤  │  ├─────────────────────────────────────┤   │
│  │ ● LumenCan  │  │  │                                     │   │
│  │   SignalFlow│  │  │  ┌─────────┐      ┌─────────┐      │   │
│  │             │  │  │  │  OSC    │ ──── │ Signal  │      │   │
│  │ ○ APC40     │  │  │  │  :8000  │      │  Flow   │      │   │
│  │   MIDI      │  │  │  └─────────┘      └─────────┘      │   │
│  │             │  │  │                                     │   │
│  │ ○ Resolume  │  │  │  ┌─────────┐      ┌─────────┐      │   │
│  │   OSC       │  │  │  │  MIDI   │ ──── │ Art-Net │      │   │
│  │             │  │  │  │  APC40  │      │  :6454  │      │   │
│  │ ● DMX King  │  │  │  └─────────┘      └─────────┘      │   │
│  │   Art-Net   │  │  │                                     │   │
│  │             │  │  │  [+ ADD BRIDGE]                     │   │
│  └─────────────┘  │  └─────────────────────────────────────┘   │
│                   │                                             │
│  [SCAN]           │  ┌─────────────────────────────────────┐   │
│                   │  │ SIGNAL MONITOR                       │   │
│  ┌─────────────┐  │  ├─────────────────────────────────────┤   │
│  │ SERVERS     │  │  │ /midi/apc/cc/48     0.72   ████░░  │   │
│  ├─────────────┤  │  │ /osc/resolume/tempo 120.0          │   │
│  │ ● OSC :8000 │  │  │ /dmx/1/47           255    ██████  │   │
│  │ ● SF  :7330 │  │  │ /lumen/layer/0/opacity 0.5 ███░░░  │   │
│  │ ○ DMX :---  │  │  └─────────────────────────────────────┘   │
│  └─────────────┘  │                                             │
│                   │                                             │
│  [+ ADD SERVER]   │                                             │
│                   │                                             │
├───────────────────┴─────────────────────────────────────────────┤
│  ● Connected to 3 devices │ 142 signals/sec │ 2.3ms latency     │
└─────────────────────────────────────────────────────────────────┘
```

#### Feature List

**Discovery & Connection**
- [x] mDNS auto-discovery
- [x] UDP broadcast fallback
- [x] Manual endpoint entry
- [x] QR code scanning (mobile)
- [x] Connection health monitoring

**Protocol Support**
- [x] SignalFlow native (client & server)
- [x] OSC (send & receive)
- [x] MIDI (all message types)
- [x] Art-Net (universes 0-15)
- [x] sACN/E1.31
- [x] DMX via USB (FTDI, Enttec)
- [ ] MIDI 2.0 (future)
- [ ] WebRTC P2P (future)

**Bridging**
- [x] Any-to-any protocol bridging
- [x] Address mapping rules
- [x] Value scaling/transform
- [x] Conditional routing
- [x] Bridge presets (save/load)

**Monitoring**
- [x] Live signal monitor
- [x] Latency graphs
- [x] Throughput stats
- [x] Connection logs

**Configuration**
- [x] YAML/JSON config files
- [x] Import/export settings
- [x] Auto-start bridges
- [x] System tray mode

---

## Part 5: Repository Structure

### 5.1 Complete Monorepo Layout

```
signalflow/
│
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                # Build & test all crates
│   │   ├── release.yml           # Publish to crates.io, npm, PyPI
│   │   └── app-release.yml       # Build app binaries
│   └── CODEOWNERS
│
├── docs/
│   ├── spec/
│   │   ├── signalflow-v2.md      # Protocol specification
│   │   ├── signalflow-lite.md    # Embedded profile
│   │   └── bridges/
│   │       ├── midi.md
│   │       ├── osc.md
│   │       └── dmx.md
│   ├── api/                      # Generated API docs
│   └── guides/
│       ├── getting-started.md
│       ├── building-bridges.md
│       └── embedded.md
│
├── crates/                       # Rust crates (see Part 2)
│   ├── signalflow-core/
│   ├── signalflow-transport/
│   ├── signalflow-discovery/
│   ├── signalflow-bridge/
│   ├── signalflow-router/
│   ├── signalflow-client/
│   ├── signalflow-embedded/
│   └── signalflow-wasm/
│
├── bindings/                     # Language bindings (see Part 3)
│   ├── js/
│   ├── python/
│   ├── swift/
│   ├── kotlin/
│   └── csharp/
│
├── apps/
│   └── bridge/                   # Tauri app (see Part 4)
│
├── tools/
│   ├── sf-cli/                   # CLI tool
│   │   ├── src/main.rs
│   │   └── Cargo.toml
│   ├── sf-router/                # Standalone router binary
│   │   ├── src/main.rs
│   │   └── Cargo.toml
│   └── sf-test/                  # Conformance test suite
│       ├── src/main.rs
│       └── Cargo.toml
│
├── examples/
│   ├── rust/
│   │   ├── simple-client/
│   │   ├── osc-bridge/
│   │   └── embedded-esp32/
│   ├── js/
│   │   ├── browser-client/
│   │   ├── node-server/
│   │   └── react-demo/
│   └── python/
│       ├── simple-client.py
│       └── midi-controller.py
│
├── Cargo.toml                    # Workspace root
├── Cargo.lock
├── package.json                  # For JS tooling (turbo, etc.)
├── pnpm-workspace.yaml
├── rust-toolchain.toml
├── .rustfmt.toml
├── .clippy.toml
├── README.md
├── LICENSE                       # CC0 for spec, MIT/Apache for code
└── CONTRIBUTING.md
```

### 5.2 Workspace Configuration

```toml
# Cargo.toml (workspace root)
[workspace]
resolver = "2"
members = [
    "crates/*",
    "bindings/python",
    "tools/*",
    "apps/bridge/src-tauri",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/lumencanvas/signalflow"
rust-version = "1.75"

[workspace.dependencies]
# Async
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rmp-serde = "1.1"

# Networking
tokio-tungstenite = "0.21"
quinn = "0.10"
webrtc = "0.9"

# Protocols
rosc = "0.10"
midir = "0.9"
artnet_protocol = "0.2"

# Discovery
mdns-sd = "0.10"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

## Part 6: Implementation Phases

### Phase 1: Foundation (Weeks 1-4)

**Goals**: Core protocol implementation, basic client

```
[ ] signalflow-core
    [ ] Types and message definitions
    [ ] MessagePack codec
    [ ] Frame encoding/decoding
    [ ] Address parsing and wildcards
    [ ] Basic state management

[ ] signalflow-transport
    [ ] Transport trait
    [ ] WebSocket transport
    [ ] UDP transport

[ ] signalflow-client
    [ ] Basic client API
    [ ] Connect, subscribe, set, emit
    [ ] Reconnection logic

[ ] Basic tests and documentation
```

**Deliverable**: Working Rust client that can connect to a SignalFlow server

### Phase 2: Discovery & Bridges (Weeks 5-8)

**Goals**: Auto-discovery, protocol bridges

```
[ ] signalflow-discovery
    [ ] mDNS implementation
    [ ] UDP broadcast fallback
    [ ] Device registry

[ ] signalflow-bridge
    [ ] Bridge trait
    [ ] OSC bridge
    [ ] MIDI bridge
    [ ] Art-Net bridge

[ ] signalflow-router
    [ ] Basic routing
    [ ] Subscription matching
    [ ] State store

[ ] Integration tests
```

**Deliverable**: Router with OSC/MIDI bridges, auto-discovery

### Phase 3: WASM & JS Bindings (Weeks 9-11)

**Goals**: Browser support

```
[ ] signalflow-wasm
    [ ] wasm-bindgen setup
    [ ] Core API exposure
    [ ] WebSocket in browser

[ ] bindings/js
    [ ] TypeScript wrapper
    [ ] npm package
    [ ] React hooks
    [ ] Documentation

[ ] Browser examples
```

**Deliverable**: npm package working in browsers

### Phase 4: Python & Native Bindings (Weeks 12-14)

**Goals**: Python support, native bindings foundation

```
[ ] bindings/python
    [ ] PyO3 setup
    [ ] Async/sync API
    [ ] Type stubs
    [ ] PyPI package

[ ] C header generation (cbindgen)

[ ] Swift bindings (UniFFI)

[ ] Python examples
```

**Deliverable**: pip-installable package

### Phase 5: Desktop App (Weeks 15-20)

**Goals**: Cross-platform bridge application

```
[ ] apps/bridge
    [ ] Tauri project setup
    [ ] Solid.js frontend
    [ ] Design system implementation
    [ ] Discovery UI
    [ ] Bridge configuration UI
    [ ] Signal monitor
    [ ] Settings persistence

[ ] Backend commands
    [ ] Bridge manager
    [ ] Discovery integration
    [ ] State management

[ ] Platform builds
    [ ] macOS (Intel + Apple Silicon)
    [ ] Windows (x64)
    [ ] Linux (AppImage, deb)
```

**Deliverable**: Working cross-platform app

### Phase 6: Polish & Ecosystem (Weeks 21-24)

**Goals**: Production readiness

```
[ ] Additional bridges
    [ ] sACN
    [ ] DMX USB
    [ ] Serial protocols

[ ] Additional bindings
    [ ] Kotlin/Android
    [ ] C#/.NET

[ ] Mobile app (Tauri v2)
    [ ] iOS build
    [ ] Android build

[ ] Documentation site
[ ] Conformance test suite
[ ] Performance benchmarks
[ ] Security audit
```

**Deliverable**: Production-ready ecosystem

---

## Part 7: Testing Strategy

### 7.1 Test Categories

```
tests/
├── unit/                 # Per-crate unit tests
├── integration/          # Cross-crate integration
├── conformance/          # Protocol conformance
├── fuzz/                 # Fuzzing (cargo-fuzz)
├── benchmarks/           # Performance (criterion)
└── e2e/                  # End-to-end with real devices
```

### 7.2 CI Matrix

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly]

  wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: aspect-build/rust-wasm-action@v1
      - run: wasm-pack test --headless --chrome

  bindings:
    runs-on: ubuntu-latest
    steps:
      - name: Python
        run: maturin develop && pytest
      - name: Node
        run: pnpm test
```

### 7.3 Conformance Tests

```rust
// tools/sf-test/src/main.rs
#[tokio::main]
async fn main() {
    let url = std::env::args().nth(1).expect("Usage: sf-test <url>");
    let level = std::env::args().nth(2).unwrap_or("standard".into());

    let results = run_conformance_tests(&url, &level).await;

    println!("SignalFlow Conformance Test Results");
    println!("====================================");
    println!("Level: {}", level);
    println!("Passed: {}/{}", results.passed, results.total);

    for test in results.tests {
        println!("{} {}", if test.passed { "✓" } else { "✗" }, test.name);
    }
}
```

---

## Part 8: Documentation Plan

### 8.1 Documentation Types

| Type | Location | Tool |
|------|----------|------|
| API Reference | docs/api/ | rustdoc, typedoc |
| Specification | docs/spec/ | Markdown |
| Guides | docs/guides/ | Markdown + code samples |
| Website | signalflow.dev | Astro/Starlight |

### 8.2 Documentation Site Structure

```
signalflow.dev/
├── /                     # Landing page
├── /docs/
│   ├── /getting-started
│   ├── /protocol
│   │   ├── /overview
│   │   ├── /messages
│   │   ├── /signal-types
│   │   └── /discovery
│   ├── /guides
│   │   ├── /javascript
│   │   ├── /python
│   │   ├── /rust
│   │   └── /embedded
│   ├── /bridges
│   │   ├── /midi
│   │   ├── /osc
│   │   └── /dmx
│   └── /api
│       ├── /rust
│       ├── /javascript
│       └── /python
├── /app                  # Bridge app download
└── /playground           # Interactive WASM demo
```

---

## Part 9: Deployment & Distribution

### 9.1 Package Distribution

| Package | Registry | Name |
|---------|----------|------|
| Rust | crates.io | `signalflow`, `signalflow-*` |
| JavaScript | npm | `@signalflow/core`, `@signalflow/react` |
| Python | PyPI | `signalflow` |
| Swift | Swift Package Registry | `SignalFlow` |

### 9.2 App Distribution

| Platform | Method |
|----------|--------|
| macOS | DMG + Homebrew Cask |
| Windows | MSI + winget |
| Linux | AppImage + Flatpak |
| iOS | App Store (future) |
| Android | Play Store (future) |

### 9.3 Release Automation

```yaml
# .github/workflows/release.yml
on:
  push:
    tags: ['v*']

jobs:
  crates:
    runs-on: ubuntu-latest
    steps:
      - run: cargo publish -p signalflow-core
      - run: cargo publish -p signalflow-transport
      # ... etc

  npm:
    runs-on: ubuntu-latest
    steps:
      - run: wasm-pack build
      - run: npm publish

  pypi:
    runs-on: ubuntu-latest
    steps:
      - run: maturin publish

  app:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: universal-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
    steps:
      - run: pnpm tauri build --target ${{ matrix.target }}
      - uses: softprops/action-gh-release@v1
```

---

## Part 10: Success Metrics

### 10.1 Technical Metrics

| Metric | Target |
|--------|--------|
| Core library size (WASM) | < 100KB gzipped |
| Connection latency | < 10ms (LAN) |
| Message throughput | > 10,000/sec |
| Memory usage (router) | < 50MB for 100 clients |
| App bundle size | < 20MB |

### 10.2 Adoption Metrics

| Metric | Year 1 Target |
|--------|---------------|
| GitHub stars | 1,000 |
| npm weekly downloads | 500 |
| PyPI weekly downloads | 200 |
| App downloads | 1,000 |
| Community bridges | 5 |

---

## Appendix A: Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| WebRTC complexity | High | Defer to Phase 2, use established crate |
| WASM size bloat | Medium | Aggressive tree-shaking, feature flags |
| Cross-platform bugs | Medium | Extensive CI matrix, beta testing |
| Protocol adoption | Medium | OSC/MIDI bridges as onramp |
| Maintenance burden | Low | Modular architecture, clear ownership |

---

## Appendix B: Alternatives Considered

### Core Language

| Language | Considered | Rejected Because |
|----------|------------|------------------|
| C++ | Yes | Memory safety, build complexity |
| Go | Yes | No WASM without GC, weaker FFI |
| Zig | Yes | Ecosystem too young |
| TypeScript | Yes | Performance for native, no embedded |

### UI Framework

| Framework | Considered | Rejected Because |
|-----------|------------|------------------|
| Electron | Yes | Bundle size, memory |
| Flutter | Yes | Dart ecosystem, not truly native |
| Qt | Yes | C++, licensing |
| egui | Yes | Not mature enough for complex UI |

---

*This plan provides a comprehensive roadmap for building the SignalFlow ecosystem. Each phase delivers working software while building toward the complete vision.*

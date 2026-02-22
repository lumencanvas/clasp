---
title: Transports
description: Wire-level carriers for CLASP frames -- WebSocket, QUIC, TCP, UDP, Serial, BLE
order: 5
---

# Transports

Transports are the wire layer that carries CLASP binary frames between clients and routers. They are distinct from [protocol bridges](../protocols/README.md) -- a bridge translates between an external protocol (OSC, MIDI, MQTT) and CLASP signals, while a transport carries native CLASP frames directly.

CLASP is transport-agnostic. A client on WebSocket and a client on QUIC communicate through the same router with no difference in behavior. All transports implement the same `Transport` trait and produce the same binary frames.

## Choosing a Transport

| Transport | Best For | Reliable | Ordered | Browser | Encryption | Feature Flag |
|-----------|----------|----------|---------|---------|------------|--------------|
| WebSocket | Web apps, general use | Yes | Yes | Yes | WSS (TLS) | `websocket` (default) |
| QUIC | Mobile, high-performance native | Yes | Yes | No | TLS 1.3 (built-in) | `quic` (default) |
| TCP | Server-to-server, LAN | Yes | Yes | No | None (add TLS externally) | `tcp` (default) |
| UDP | Discovery, fire-and-forget | No | No | No | None | `udp` (default) |
| Serial | Hardware, microcontrollers | Yes | Yes | No | None | `serial` |
| BLE | Wireless controllers, battery devices | Yes | Yes | No | BLE pairing | `ble` |
| WebRTC | P2P, browser-to-browser | Yes | Yes | Yes | DTLS | `webrtc` |

Default features include `websocket`, `tcp`, `udp`, and `quic`. Serial, BLE, and WebRTC require explicit feature flags:

```bash
cargo build --features serial,ble
# or enable everything:
cargo build --features full
```

## WebSocket

The default transport for browser clients and the most widely supported option. The relay listens on WebSocket by default.

**URLs:** `ws://host:port` (plaintext) or `wss://host:port` (TLS)

**Configuration (`WebSocketConfig`):**

| Field | Default | Description |
|-------|---------|-------------|
| `subprotocol` | `"clasp"` | WebSocket subprotocol for handshake negotiation |
| `max_message_size` | 64KB | Maximum frame size before rejection |
| `ping_interval` | 30s | Keep-alive ping interval |
| `channel_buffer_size` | 1000 | Internal send/receive queue depth |

**Relay CLI:**

```bash
clasp-relay --ws-port 7330          # default
clasp-relay --no-websocket          # disable WebSocket entirely
```

**Client code:**

```js
// JavaScript
import { ClaspBuilder } from '@clasp-to/core'
const client = new ClaspBuilder('ws://localhost:7330').build()
```

```rust
// Rust
use clasp_transport::websocket::WebSocketTransport;
use clasp_transport::traits::Transport;

let (sender, receiver) = WebSocketTransport::connect("ws://localhost:7330").await?;
```

**WASM:** In browser builds, use the `wasm-websocket` feature instead of `websocket`. This swaps the tokio-tungstenite backend for a `web-sys` WebSocket implementation. The `wasm` feature flag enables this automatically.

**Reverse proxy (nginx):**

```nginx
location /clasp {
    proxy_pass http://127.0.0.1:7330;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
}
```

## QUIC

A modern transport built on UDP with built-in TLS 1.3, connection migration, and stream multiplexing. Ideal for mobile apps and high-performance native clients.

**ALPN:** `clasp/2`

**Configuration (`QuicConfig`):**

| Field | Default | Description |
|-------|---------|-------------|
| `enable_0rtt` | `true` | 0-RTT connection establishment for repeat connections |
| `keep_alive_ms` | 5000 | Keep-alive interval (0 to disable) |
| `idle_timeout_ms` | 30000 | Connection dropped after this idle period |
| `initial_window` | 10 | Initial congestion window in packets |
| `cert_verification` | `SystemRoots` | Certificate verification mode |

**Certificate verification modes (`CertVerification`):**

- `SystemRoots` -- Use OS root certificates (recommended for production)
- `SkipVerification` -- Skip verification (development only, vulnerable to MITM)
- `CustomRoots(Vec<Vec<u8>>)` -- Custom root certificates (DER-encoded)

**Relay CLI:**

```bash
clasp-relay --quic-port 7331 --cert cert.pem --key key.pem
```

**Client code (Rust):**

```rust
use clasp_transport::quic::{QuicTransport, QuicConfig};

// Production (system root certs)
let client = QuicTransport::new_client()?;
let conn = client.connect(addr, "relay.example.com").await?;
let (sender, receiver) = conn.open_bi().await?;

// Development (skip verification)
let client = QuicTransport::new_client_with_config(QuicConfig::insecure())?;
```

**Advantages over WebSocket:** 0-RTT reconnection, seamless network migration (Wi-Fi to cellular), no head-of-line blocking across multiplexed streams, unreliable datagrams via `send_datagram()` / `recv_datagram()`.

## TCP

Raw TCP with length-prefixed framing. Each CLASP message is preceded by a 4-byte big-endian length prefix. Useful for server-to-server links and LAN setups where WebSocket overhead is unnecessary.

**Configuration (`TcpConfig`):**

| Field | Default | Description |
|-------|---------|-------------|
| `max_message_size` | 64KB | Maximum message size |
| `read_buffer_size` | 8192 | Read buffer size in bytes |
| `keepalive_secs` | 30 | TCP keep-alive interval (0 to disable) |

**Client code (Rust):**

```rust
use clasp_transport::tcp::TcpTransport;

let transport = TcpTransport::new();
let (sender, receiver) = transport.connect("192.168.1.10:7330").await?;
```

**Server:**

```rust
use clasp_transport::tcp::TcpServer;

let mut server = TcpServer::bind("0.0.0.0:7330").await?;
let (sender, receiver, peer_addr) = server.accept().await?;
```

## UDP

Connectionless, fire-and-forget transport. Best for discovery broadcasts and scenarios where occasional packet loss is acceptable. UDP provides no reliability or ordering guarantees -- use it only for Fire QoS signals.

**Configuration (`UdpConfig`):**

| Field | Default | Description |
|-------|---------|-------------|
| `recv_buffer_size` | 65536 | Receive buffer size in bytes |
| `max_packet_size` | 65507 | Maximum UDP payload (protocol limit) |

**Client code (Rust):**

```rust
use clasp_transport::udp::{UdpTransport, UdpBroadcast};

// Point-to-point
let transport = UdpTransport::bind("0.0.0.0:0").await?;
transport.send_to(b"data", target_addr).await?;

// Broadcast (for discovery)
let broadcast = UdpBroadcast::new(7340).await?;
broadcast.broadcast(b"announce").await?;
```

> **Warning:** UDP provides no delivery guarantees. Messages may be lost, duplicated, or arrive out of order. Only use UDP for signals where loss is acceptable (discovery, sensor telemetry with high send rates).

## Serial

Direct serial port communication for hardware integration. Requires the `serial` feature flag.

**Configuration (`SerialConfig`):**

| Field | Default | Description |
|-------|---------|-------------|
| `baud_rate` | 115200 | Serial baud rate |
| `data_bits` | 8 | Data bits per frame |
| `stop_bits` | 1 | Stop bits |
| `parity` | `None` | `None`, `Odd`, or `Even` |
| `flow_control` | `None` | `None`, `Hardware`, or `Software` |

**Port discovery:**

```rust
use clasp_transport::serial::SerialTransport;

let ports = SerialTransport::list_ports()?;
// Linux:  ["/dev/ttyUSB0", "/dev/ttyACM0"]
// macOS:  ["/dev/tty.usbserial-1420", "/dev/tty.usbmodem14201"]
// Windows: ["COM3", "COM4"]
```

**Connect:**

```rust
let (sender, receiver) = SerialTransport::connect("/dev/ttyUSB0").await?;

// With custom config
let config = SerialConfig { baud_rate: 9600, ..Default::default() };
let (sender, receiver) = SerialTransport::connect_with_config("/dev/ttyUSB0", config).await?;
```

Typical use cases: DMX controllers over USB, Arduino/ESP32 CLASP bridges, direct microcontroller communication.

## BLE

Bluetooth Low Energy transport using GATT services. Designed for wireless controllers and battery-powered devices. Requires the `ble` feature flag.

**CLASP GATT Service:**

| UUID | Role |
|------|------|
| `0x7330` | CLASP Service |
| `0x7331` | TX Characteristic (client writes to peripheral) |
| `0x7332` | RX Characteristic (peripheral notifies client) |

**Configuration (`BleConfig`):**

| Field | Default | Description |
|-------|---------|-------------|
| `device_name_filter` | `None` | Filter scan results by device name substring |
| `scan_duration_secs` | 5 | How long to scan for devices |
| `mtu` | 512 | Maximum transmission unit (BLE 5.0) |
| `write_without_response` | `true` | Lower latency writes (no ACK from peripheral) |

**Scan and connect workflow:**

```rust
use clasp_transport::ble::{BleTransport, BleConfig};

let config = BleConfig {
    device_name_filter: Some("CLASP".into()),
    ..Default::default()
};
let ble = BleTransport::with_config(config).await?;

// Scan for devices
let devices = ble.scan().await?;
for device in &devices {
    println!("{:?} (CLASP: {})", device.name, device.has_clasp_service);
}

// Connect to a device
let (sender, receiver) = ble.connect(&devices[0]).await?;
```

## WebRTC

WebRTC support is documented in the [P2P & WebRTC](p2p.md) page. It requires the `webrtc` feature flag and is used for direct peer-to-peer communication that bypasses the relay entirely.

## Transport Trait

All transports implement the same trait hierarchy from `clasp-transport::traits`:

```rust
#[async_trait]
pub trait TransportSender: Send + Sync {
    async fn send(&self, data: Bytes) -> Result<()>;
    fn try_send(&self, data: Bytes) -> Result<()>;
    fn is_connected(&self) -> bool;
    async fn close(&self) -> Result<()>;
}

#[async_trait]
pub trait TransportReceiver: Send {
    async fn recv(&mut self) -> Option<TransportEvent>;
}

#[async_trait]
pub trait Transport: Send + Sync {
    type Sender: TransportSender;
    type Receiver: TransportReceiver;
    async fn connect(addr: &str) -> Result<(Self::Sender, Self::Receiver)>;
}

#[async_trait]
pub trait TransportServer: Send + Sync {
    type Sender: TransportSender;
    type Receiver: TransportReceiver;
    async fn accept(&mut self) -> Result<(Self::Sender, Self::Receiver, SocketAddr)>;
}
```

`TransportEvent` carries connection lifecycle events: `Connected`, `Disconnected { reason }`, `Data(Bytes)`, and `Error(String)`.

## Mixed-Transport Systems

A router accepts connections from multiple transports simultaneously. Clients on different transports interact transparently through the shared state store:

```
Browser (WSS) ──────┐
                     │
Mobile App (QUIC) ──┼── Router ── State Store
                     │
ESP32 (Serial) ─────┘
```

The relay exposes WebSocket and QUIC on separate ports. TCP and UDP are available when embedding `clasp-router` directly. See [Relay Server](../deployment/relay.md) for multi-protocol configuration.

## Next Steps

- [Relay Server](../deployment/relay.md) -- production deployment with multi-protocol support
- [P2P & WebRTC](p2p.md) -- direct peer-to-peer connections bypassing the relay
- [Architecture](../concepts/architecture.md) -- how transports fit into the crate layers
- [Signals](signals.md) -- the signal types carried by transports

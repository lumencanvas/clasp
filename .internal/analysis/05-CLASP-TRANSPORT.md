# CLASP-TRANSPORT Crate Analysis

## Overview

**Location:** `crates/clasp-transport`
**Purpose:** Transport abstraction layer with 7 implementations

## Feature Flags

```toml
default = ["websocket", "tcp", "udp", "quic"]
full = ["websocket", "tcp", "udp", "quic", "serial", "ble", "webrtc"]

# Individual features
websocket        # tokio-tungstenite
wasm-websocket   # web-sys (WASM only)
tcp              # Native TCP
udp              # Native UDP
quic             # quinn + rustls
serial           # tokio-serial
ble              # btleplug
webrtc           # webrtc-rs
```

---

## Core Traits (traits.rs)

### TransportEvent

```rust
#[derive(Debug, Clone)]
pub enum TransportEvent {
    Connected,
    Disconnected { reason: Option<String> },
    Data(Bytes),
    Error(String),
}
```

### TransportSender

```rust
#[async_trait]
pub trait TransportSender: Send + Sync {
    async fn send(&self, data: Bytes) -> Result<()>;
    fn try_send(&self, data: Bytes) -> Result<()>;  // Non-blocking
    fn is_connected(&self) -> bool;
    async fn close(&self) -> Result<()>;
}
```

### TransportReceiver

```rust
#[async_trait]
pub trait TransportReceiver: Send {
    async fn recv(&mut self) -> Option<TransportEvent>;
}
```

### Transport

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    type Sender: TransportSender;
    type Receiver: TransportReceiver;

    async fn connect(addr: &str) -> Result<(Self::Sender, Self::Receiver)>
        where Self: Sized;
    fn local_addr(&self) -> Option<SocketAddr>;
    fn remote_addr(&self) -> Option<SocketAddr>;
}
```

### TransportServer

```rust
#[async_trait]
pub trait TransportServer: Send + Sync {
    type Sender: TransportSender;
    type Receiver: TransportReceiver;

    async fn accept(&mut self) -> Result<(Self::Sender, Self::Receiver, SocketAddr)>;
    fn local_addr(&self) -> Result<SocketAddr>;
    async fn close(&self) -> Result<()>;
}
```

### TransportError

```rust
pub enum TransportError {
    ConnectionFailed(String),
    ConnectionClosed,
    BindFailed(String),
    AcceptFailed(String),
    SendFailed(String),
    BufferFull,
    ReceiveFailed(String),
    InvalidUrl(String),
    Timeout,
    Io(std::io::Error),
    Protocol(String),
    NotConnected,
    AlreadyConnected,
    Other(String),
}
```

---

## WebSocket Transport (websocket.rs)

**Feature:** `#[cfg(all(feature = "websocket", not(target_arch = "wasm32")))]`

### Configuration

```rust
pub struct WebSocketConfig {
    pub subprotocol: String,        // "clasp"
    pub max_message_size: usize,    // 64KB
    pub ping_interval: u64,         // 30s
    pub channel_buffer_size: usize, // 1000
}
```

### Client

```rust
pub struct WebSocketTransport {
    config: WebSocketConfig,
}

impl WebSocketTransport {
    pub fn new() -> Self
    pub fn with_config(config: WebSocketConfig) -> Self
}

impl Transport for WebSocketTransport {
    type Sender = WebSocketSender;
    type Receiver = WebSocketReceiver;
    async fn connect(url: &str) -> Result<(Self::Sender, Self::Receiver)>
}
```

### Server

```rust
pub struct WebSocketServer {
    listener: TcpListener,
    config: WebSocketConfig,
}

impl WebSocketServer {
    pub async fn bind(addr: &str) -> Result<Self>
    pub fn with_config(self, config: WebSocketConfig) -> Self
}

impl TransportServer for WebSocketServer {
    async fn accept(&mut self) -> Result<(WebSocketSender, WebSocketReceiver, SocketAddr)>
}
```

**Features:**
- Subprotocol negotiation
- Automatic Ping/Pong handling
- Binary and text message support
- Configurable buffer sizes

---

## TCP Transport (tcp.rs)

**Feature:** `#[cfg(all(feature = "tcp", not(target_arch = "wasm32")))]`

### Configuration

```rust
pub struct TcpConfig {
    pub max_message_size: usize,  // 64KB
    pub read_buffer_size: usize,  // 8192
    pub keepalive_secs: u64,      // 30
}
```

### Frame Format

```
[4 bytes: length (u32 BE)][payload]
```

### Client

```rust
pub struct TcpTransport {
    config: TcpConfig,
}

impl TcpTransport {
    pub fn new() -> Self
    pub fn with_config(config: TcpConfig) -> Self
    pub async fn connect(&self, addr: &str) -> Result<(TcpSender, TcpReceiver)>
}
```

### Server

```rust
pub struct TcpServer {
    listener: TcpListener,
    config: TcpConfig,
}

impl TcpServer {
    pub async fn bind(addr: &str) -> Result<Self>
    pub async fn bind_with_config(addr: &str, config: TcpConfig) -> Result<Self>
}
```

**Features:**
- Length-prefixed framing
- TCP keepalive support
- Configurable buffer sizes

---

## UDP Transport (udp.rs)

**Feature:** `#[cfg(all(feature = "udp", not(target_arch = "wasm32")))]`

### Configuration

```rust
pub struct UdpConfig {
    pub recv_buffer_size: usize,  // 65536
    pub max_packet_size: usize,   // 65507
}
```

### Transport

```rust
pub struct UdpTransport {
    socket: Arc<UdpSocket>,
    config: UdpConfig,
}

impl UdpTransport {
    pub async fn bind(addr: &str) -> Result<Self>
    pub async fn bind_with_config(addr: &str, config: UdpConfig) -> Result<Self>
    pub fn local_addr(&self) -> Result<SocketAddr>
    pub fn sender_to(&self, remote: SocketAddr) -> UdpSender
    pub fn start_receiver(&self) -> UdpReceiver
    pub async fn send_to(&self, data: &[u8], target: SocketAddr) -> Result<()>
    pub fn set_broadcast(&self, enable: bool) -> Result<()>
}
```

### Receiver

```rust
pub struct UdpReceiver {
    rx: mpsc::Receiver<(TransportEvent, SocketAddr)>,
}

impl UdpReceiver {
    pub async fn recv_from(&mut self) -> Option<(TransportEvent, SocketAddr)>
}
```

**Note:** Connectionless - no Transport trait impl, sender per remote address

### Broadcast Support

```rust
pub struct UdpBroadcast {
    socket: Arc<UdpSocket>,
    broadcast_addr: SocketAddr,
}

impl UdpBroadcast {
    pub async fn new(port: u16) -> Result<Self>
    pub async fn broadcast(&self, data: &[u8]) -> Result<()>
}
```

---

## QUIC Transport (quic.rs)

**Feature:** `#[cfg(feature = "quic")]`

### Constants

```rust
pub const CLASP_ALPN: &[u8] = b"clasp/2";
```

### Certificate Verification

```rust
pub enum CertVerification {
    SkipVerification,           // Dev only (insecure)
    SystemRoots,                // rustls_native_certs
    CustomRoots(Vec<Vec<u8>>),  // DER-encoded CAs
}
```

### Configuration

```rust
pub struct QuicConfig {
    pub enable_0rtt: bool,             // true
    pub keep_alive_ms: u64,            // 5000
    pub idle_timeout_ms: u64,          // 30000
    pub initial_window: u32,           // 10
    pub cert_verification: CertVerification,
}

impl QuicConfig {
    pub fn with_system_roots() -> Self
    pub fn insecure() -> Self
    pub fn with_custom_roots(certs: Vec<Vec<u8>>) -> Self
}
```

### Transport

```rust
pub struct QuicTransport {
    config: QuicConfig,
    endpoint: Endpoint,
}

impl QuicTransport {
    // Client
    pub fn new_client() -> Result<Self>
    pub fn new_client_with_config(config: QuicConfig) -> Result<Self>

    // Server
    pub fn new_server(bind_addr, cert_der, key_der) -> Result<Self>
    pub fn new_server_with_config(bind_addr, cert_der, key_der, config) -> Result<Self>

    // Connection
    pub async fn connect(&self, addr: SocketAddr, server_name: &str) -> Result<QuicConnection>
    pub async fn accept(&self) -> Result<QuicConnection>
    pub fn local_addr(&self) -> Result<SocketAddr>
}
```

### Connection (Stream Multiplexing)

```rust
pub struct QuicConnection {
    connection: Connection,
}

impl QuicConnection {
    // Bidirectional streams (reliable, ordered)
    pub async fn open_bi(&self) -> Result<(QuicSender, QuicReceiver)>
    pub async fn accept_bi(&self) -> Result<(QuicSender, QuicReceiver)>

    // Unidirectional streams
    pub async fn open_uni(&self) -> Result<QuicSender>
    pub async fn accept_uni(&self) -> Result<QuicReceiver>

    // Unreliable datagrams
    pub fn send_datagram(&self, data: Bytes) -> Result<()>
    pub async fn recv_datagram(&self) -> Result<Bytes>

    // Management
    pub fn remote_address(&self) -> SocketAddr
    pub fn close(&self, code: u32, reason: &str)
}
```

---

## Serial Transport (serial.rs)

**Feature:** `#[cfg(all(feature = "serial", not(target_arch = "wasm32")))]`

### Configuration

```rust
pub enum SerialParity { None, Odd, Even }
pub enum SerialFlowControl { None, Hardware, Software }

pub struct SerialConfig {
    pub baud_rate: u32,              // 115200
    pub data_bits: u8,               // 8
    pub stop_bits: u8,               // 1
    pub parity: SerialParity,        // None
    pub flow_control: SerialFlowControl, // None
}
```

### Transport

```rust
pub struct SerialTransport {
    config: SerialConfig,
    port_name: String,
}

impl SerialTransport {
    pub fn list_ports() -> Result<Vec<String>>
    pub async fn connect(port_name: &str) -> Result<(SerialSender, SerialReceiver)>
    pub async fn connect_with_config(
        port_name: &str,
        config: SerialConfig,
    ) -> Result<(SerialSender, SerialReceiver)>
}
```

---

## BLE Transport (ble.rs)

**Feature:** `#[cfg(all(feature = "ble", not(target_arch = "wasm32")))]`

### GATT UUIDs

```rust
pub const CLASP_SERVICE_UUID: Uuid = 0x00007330...;  // Based on port 7330
pub const CLASP_TX_CHAR_UUID: Uuid = 0x00007331...;  // Write to send
pub const CLASP_RX_CHAR_UUID: Uuid = 0x00007332...;  // Notify to receive
```

### Configuration

```rust
pub struct BleConfig {
    pub device_name_filter: Option<String>,
    pub scan_duration_secs: u64,     // 5
    pub mtu: usize,                  // 512 (BLE 5.0)
    pub write_without_response: bool, // true
}
```

### Transport

```rust
pub struct BleTransport {
    config: BleConfig,
    adapter: Adapter,
}

impl BleTransport {
    pub async fn new() -> Result<Self>
    pub async fn with_config(config: BleConfig) -> Result<Self>
    pub async fn scan(&self) -> Result<Vec<BleDevice>>
    pub async fn connect(&self, device: &BleDevice) -> Result<(BleSender, BleReceiver)>
}
```

### Device

```rust
pub struct BleDevice {
    pub name: Option<String>,
    pub address: String,
    pub rssi: Option<i16>,
    pub has_clasp_service: bool,
    peripheral: Peripheral,
}
```

**Connection Flow:**
1. Connect to peripheral
2. Discover services
3. Find CLASP characteristics by UUID
4. Subscribe to RX notifications
5. Spawn notification receiver task

---

## WebRTC Transport (webrtc.rs)

**Feature:** `#[cfg(feature = "webrtc")]` (cross-platform)

### Configuration

```rust
pub struct WebRtcConfig {
    pub ice_servers: Vec<String>,    // Google STUN by default
    pub unreliable_channel: bool,    // true (streams, QoS Fire)
    pub reliable_channel: bool,      // true (params, QoS Confirm)
}
```

### DataChannels

- **"clasp"**: Unreliable, unordered (max_retransmits=0) - high-frequency streams
- **"clasp-reliable"**: Reliable, ordered - parameters, events

### Transport

```rust
pub struct WebRtcTransport {
    config: WebRtcConfig,
    peer_connection: Arc<RTCPeerConnection>,
    unreliable_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
    reliable_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
    connection_callback: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>,
    ice_candidate_callback: Arc<Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>,
}

impl WebRtcTransport {
    // Offerer (initiator)
    pub async fn new_offerer() -> Result<(Self, String)>  // Returns SDP offer
    pub async fn new_offerer_with_config(config) -> Result<(Self, String)>

    // Answerer (responder)
    pub async fn new_answerer(remote_offer: &str) -> Result<(Self, String)>  // Returns SDP answer
    pub async fn new_answerer_with_config(remote_offer, config) -> Result<(Self, String)>

    // SDP/ICE handling
    pub async fn set_remote_answer(&self, remote_answer: &str) -> Result<()>
    pub async fn add_ice_candidate(&self, candidate: &str) -> Result<()>

    // Callbacks
    pub fn on_connection_ready<F>(&self, callback: F)
    pub fn on_ice_candidate<F>(&self, callback: F)

    // Channel access
    pub fn unreliable_channel(&self) -> Option<(WebRtcSender, WebRtcReceiver)>
    pub fn reliable_channel(&self) -> Option<(WebRtcSender, WebRtcReceiver)>

    // Direct send
    pub async fn send_reliable(&self, data: Bytes) -> Result<()>
    pub async fn send_unreliable(&self, data: Bytes) -> Result<()>
}
```

---

## WASM WebSocket (wasm_websocket.rs)

**Feature:** `#[cfg(all(feature = "wasm-websocket", target_arch = "wasm32"))]`

**Note:** WASM cannot act as server, client only

### Configuration

```rust
pub struct WasmWebSocketConfig {
    pub subprotocol: String,  // "clasp"
}
```

### Transport

```rust
pub struct WasmWebSocketTransport {
    config: WasmWebSocketConfig,
}

impl WasmWebSocketTransport {
    pub fn new() -> Self
    pub fn with_config(config: WasmWebSocketConfig) -> Self
}

#[async_trait(?Send)]  // WASM doesn't support Send
impl Transport for WasmWebSocketTransport {
    type Sender = WasmWebSocketSender;
    type Receiver = WasmWebSocketReceiver;
    async fn connect(url: &str) -> Result<(Self::Sender, Self::Receiver)>
}
```

**Connection Flow:**
1. Create JS Array of subprotocols
2. Call `web_sys::WebSocket::new_with_str_sequence()`
3. Set binary_type to ArrayBuffer
4. Register event handlers (onopen, onmessage, onerror, onclose)
5. Wait with 10-second timeout

---

## Transport Summary

| Transport | Native | WASM | Server | Key Features |
|-----------|--------|------|--------|--------------|
| WebSocket | ✓ | ✓ | ✓ | Subprotocol, ping/pong |
| TCP | ✓ | - | ✓ | Length-prefixed, keepalive |
| UDP | ✓ | - | - | Connectionless, broadcast |
| QUIC | ✓ | - | ✓ | Streams, datagrams, 0-RTT |
| Serial | ✓ | - | - | Configurable baud/parity |
| BLE | ✓ | - | - | GATT discovery, MTU |
| WebRTC | ✓ | ✓ | - | P2P, dual DataChannels |

## Common Patterns

1. **Async Communication**: All use `tokio::mpsc` channels internally
2. **Shared State**: `Arc<Mutex<bool>>` for connected status
3. **Background Tasks**: Spawned with `tokio::spawn` for IO
4. **Event-Driven**: TransportEvent enum for state changes
5. **Configuration**: Config structs with sensible defaults
6. **Feature Gating**: Complete stub implementations when disabled

# CLASP-CLIENT Crate Analysis

## Overview

**Location:** `crates/clasp-client`
**Purpose:** User-facing client API with subscriptions and P2P support

## Feature Flags

```toml
default = []
p2p = ["clasp-transport/webrtc"]
```

---

## Clasp Client (client.rs)

### Struct Definition

```rust
pub struct Clasp {
    // Configuration
    url: String,
    name: String,
    features: Vec<String>,
    token: Option<String>,
    reconnect: bool,
    reconnect_interval_ms: u64,

    // State Management
    session_id: RwLock<Option<String>>,
    connected: Arc<RwLock<bool>>,
    sender: RwLock<Option<mpsc::Sender<Bytes>>>,

    // Cache & Subscriptions
    params: Arc<DashMap<String, Value>>,
    subscriptions: Arc<DashMap<u32, (String, SubscriptionCallback)>>,
    next_sub_id: AtomicU32,

    // Synchronization
    clock: RwLock<ClockSync>,
    pending_gets: Arc<DashMap<String, oneshot::Sender<Value>>>,

    // Server State
    signals: Arc<DashMap<String, SignalDefinition>>,
    last_error: Arc<RwLock<Option<ErrorMessage>>>,

    // Reconnection
    reconnect_attempts: Arc<AtomicU32>,
    max_reconnect_attempts: u32,
    intentionally_closed: Arc<AtomicBool>,
    reconnect_notify: Arc<Notify>,

    // P2P (feature-gated)
    #[cfg(feature = "p2p")]
    p2p_config: Option<P2PConfig>,
    #[cfg(feature = "p2p")]
    p2p_manager: Option<Arc<p2p::P2PManager>>,
}
```

### Constructor & Connection

```rust
impl Clasp {
    pub fn builder(url: &str) -> ClaspBuilder
    pub async fn connect_to(url: &str) -> Result<Self>

    pub fn is_connected(&self) -> bool
    pub fn session_id(&self) -> Option<String>
    pub fn time(&self) -> u64
    pub async fn close(&self)
}
```

### Subscription API

```rust
pub type SubscriptionCallback = Box<dyn Fn(Value, &str) + Send + Sync>;

pub async fn subscribe<F>(&self, pattern: &str, callback: F) -> Result<u32>
where
    F: Fn(Value, &str) + Send + Sync + 'static

pub async fn on<F>(&self, pattern: &str, callback: F) -> Result<u32>
where
    F: Fn(Value, &str) + Send + Sync + 'static

pub async fn unsubscribe(&self, id: u32) -> Result<()>
```

### Parameter Operations

```rust
pub async fn set(&self, address: &str, value: impl Into<Value>) -> Result<()>
pub async fn set_locked(&self, address: &str, value: impl Into<Value>) -> Result<()>
pub async fn set_unlocked(&self, address: &str, value: impl Into<Value>) -> Result<()>
pub async fn get(&self, address: &str) -> Value
pub fn cached(&self, address: &str) -> Option<Value>
```

### Signal Emission

```rust
pub async fn emit(&self, address: &str, payload: impl Into<Value>) -> Result<()>
pub async fn stream(&self, address: &str, value: impl Into<Value>) -> Result<()>
pub async fn gesture(
    &self,
    address: &str,
    id: u32,
    phase: GesturePhase,
    payload: impl Into<Value>,
) -> Result<()>
```

### Timeline & Bundling

```rust
pub async fn timeline(&self, address: &str, timeline_data: TimelineData) -> Result<()>
pub async fn bundle(&self, messages: Vec<Message>) -> Result<()>
pub async fn bundle_at(&self, messages: Vec<Message>, time: u64) -> Result<()>
```

### Signal Introspection

```rust
pub fn signals(&self) -> Vec<SignalDefinition>
pub fn query_signals(&self, pattern: &str) -> Vec<SignalDefinition>
pub fn last_error(&self) -> Option<ErrorMessage>
pub fn clear_error(&self)
```

### P2P Methods (feature-gated)

```rust
#[cfg(feature = "p2p")]
pub async fn connect_to_peer(&self, peer_session_id: &str) -> Result<()>

#[cfg(feature = "p2p")]
pub fn on_p2p_event<F>(&self, callback: F)
where
    F: Fn(p2p::P2PEvent) + Send + Sync + 'static

#[cfg(feature = "p2p")]
pub fn is_peer_connected(&self, peer_session_id: &str) -> bool
```

---

## ClaspBuilder (builder.rs)

### Struct Definition

```rust
pub struct ClaspBuilder {
    url: String,
    name: String,                         // "Clasp Client"
    features: Vec<String>,                // ["param", "event", "stream"]
    token: Option<String>,
    reconnect: bool,                      // true
    reconnect_interval_ms: u64,           // 5000

    #[cfg(feature = "p2p")]
    p2p_config: Option<P2PConfig>,
}
```

### Fluent API

```rust
pub fn new(url: &str) -> Self
pub fn name(mut self, name: &str) -> Self
pub fn features(mut self, features: Vec<String>) -> Self
pub fn token(mut self, token: &str) -> Self
pub fn reconnect(mut self, enabled: bool) -> Self
pub fn reconnect_interval(mut self, ms: u64) -> Self

#[cfg(feature = "p2p")]
pub fn p2p_config(mut self, config: P2PConfig) -> Self

pub async fn connect(self) -> Result<Clasp>
```

---

## Reconnection Logic

### Exponential Backoff

```rust
let delay_ms = (base_ms as f64 * 1.5_f64.powi(attempts as i32))
    .min(30000.0) as u64;
```

### Reconnection Flow

1. Wait for disconnect notification via `reconnect_notify`
2. Check `intentionally_closed` flag
3. Calculate delay with exponential backoff
4. Attempt reconnection with 10-second timeout
5. On success: reset attempts, resubscribe all patterns
6. Max attempts: 10 (configurable, 0 = unlimited)

---

## State Caching

### Cache Structure

```rust
params: Arc<DashMap<String, Value>>
```

### Get with Fallback

```rust
pub async fn get(&self, address: &str) -> Result<Value> {
    // Check cache first
    if let Some(value) = self.params.get(address) {
        return Ok(value.clone());
    }

    // Request from server with 5-second timeout
    let (tx, rx) = oneshot::channel();
    pending_gets.insert(address, tx);

    match timeout(Duration::from_secs(5), rx).await { ... }
}
```

---

## P2P Manager (p2p.rs)

### Struct Definition

```rust
pub struct P2PManager {
    session_id: RwLock<Option<String>>,
    config: P2PConfig,
    connections: Arc<DashMap<String, P2PConnection>>,
    known_peers: Arc<DashMap<String, Vec<String>>>,
    event_callback: RwLock<Option<P2PEventCallback>>,
    signal_tx: mpsc::Sender<Message>,
    routing_mode: RwLock<RoutingMode>,
    relay_fallback_peers: Arc<DashMap<String, Instant>>,
    p2p_retry_interval_secs: u64,         // 60
}
```

### P2PConnection

```rust
pub struct P2PConnection {
    pub peer_session_id: String,
    pub correlation_id: String,
    pub state: P2PConnectionState,
    transport: Option<WebRtcTransport>,
    pending_candidates: Vec<String>,
}
```

### P2PEvent

```rust
pub enum P2PEvent {
    PeerAnnounced { session_id: String, features: Vec<String> },
    Connected { peer_session_id: String },
    ConnectionFailed { peer_session_id: String, reason: String },
    Disconnected { peer_session_id: String, reason: Option<String> },
    Data { peer_session_id: String, data: Bytes, reliable: bool },
}
```

### SendResult

```rust
pub enum SendResult {
    P2P,    // Sent via P2P
    Relay,  // Sent via server relay
}
```

### Methods

```rust
impl P2PManager {
    // Initialization
    pub fn new(config: P2PConfig, signal_tx: mpsc::Sender<Message>) -> Self
    pub fn set_session_id(&self, session_id: String)
    pub fn on_event<F>(&self, callback: F)
    pub fn set_routing_mode(&self, mode: RoutingMode)

    // Relay Fallback
    pub fn should_use_relay(&self, peer_session_id: &str) -> bool
    pub fn mark_p2p_failed(&self, peer_session_id: &str, reason: &str)
    pub fn clear_relay_fallback(&self, peer_session_id: &str)

    // Connection
    pub async fn connect_to_peer(self: &Arc<Self>, peer_session_id: &str) -> Result<()>
    pub async fn disconnect_peer(&self, peer_session_id: &str) -> Result<()>
    pub fn is_peer_connected(&self, peer_session_id: &str) -> bool
    pub fn known_peers(&self) -> Vec<String>

    // Data Transmission
    pub async fn send_to_peer(
        &self,
        peer_session_id: &str,
        data: Bytes,
        reliable: bool,
    ) -> Result<SendResult>

    // Signal Handling
    pub async fn announce(&self) -> Result<()>
    pub async fn handle_signal(self: &Arc<Self>, address: &str, payload: &Value) -> Result<()>
    pub fn handle_announce(&self, payload: &Value)
}
```

---

## Error Handling (error.rs)

```rust
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("not connected")]
    NotConnected,

    #[error("already connected")]
    AlreadyConnected,

    #[error("send failed: {0}")]
    SendFailed(String),

    #[error("timeout")]
    Timeout,

    #[error("protocol error: {0}")]
    Protocol(#[from] clasp_core::Error),

    #[error("transport error: {0}")]
    Transport(#[from] clasp_transport::TransportError),

    #[error("P2P not connected to peer: {0}")]
    P2PNotConnected(String),

    #[error("client error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ClientError>;
```

---

## Message Handling

### Handler Function

```rust
fn handle_message(
    msg: &Message,
    params: &Arc<DashMap<String, Value>>,
    subscriptions: &Arc<DashMap<u32, (String, SubscriptionCallback)>>,
    pending_gets: &Arc<DashMap<String, oneshot::Sender<Value>>>,
    signals: &Arc<DashMap<String, SignalDefinition>>,
    last_error: &Arc<RwLock<Option<ErrorMessage>>>,
)
```

### Handled Messages

- **Set**: Update cache, invoke matching subscription callbacks
- **Snapshot**: Batch update cache, trigger subscriptions, complete pending gets
- **Publish**: Invoke matching subscription callbacks
- **Error**: Store in last_error
- **Ack**: Log acknowledgment
- **Announce**: Register signals
- **Sync**: Clock synchronization
- **Result**: Process query results
- **Bundle**: Recursively handle nested messages
- **Ping/Pong**: Keepalive

---

## Subscription Matching

```rust
// Uses clasp_core::address::glob_match
glob_match(pattern, address)

// * matches exactly one segment
// ** matches zero or more segments
```

---

## Concurrency Model

- **DashMap**: Thread-safe concurrent maps
- **RwLock**: Read-heavy state (session_id, clock)
- **AtomicU32**: Subscription ID generation
- **AtomicBool**: Intentionally closed flag
- **Notify**: Reconnection signaling
- **oneshot**: Pending GET requests

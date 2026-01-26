# CLASP-ROUTER Crate Analysis

## Overview

**Location:** `crates/clasp-router`
**Purpose:** Central message hub - routing, sessions, subscriptions, state management

## Module Structure

```
clasp-router/
├── src/
│   ├── lib.rs
│   ├── router.rs         # Main router logic
│   ├── session.rs        # Client sessions
│   ├── subscription.rs   # Pattern matching
│   ├── state.rs          # Parameter storage
│   ├── gesture.rs        # Move coalescing
│   ├── p2p.rs            # P2P capabilities
│   └── adapters/
│       ├── mod.rs
│       ├── mqtt_server.rs
│       └── osc_server.rs
└── tests/
```

## Feature Flags

```toml
default = []
websocket = ["clasp-transport/websocket"]
quic = ["clasp-transport/quic"]
mqtt-server = ["mqttbytes"]
osc-server = ["rosc"]
```

---

## Router Module (router.rs)

### RouterConfig

```rust
pub struct RouterConfig {
    pub name: String,                          // "Clasp Router"
    pub features: Vec<String>,                 // ["param", "event", "stream", "timeline", "gesture"]
    pub max_sessions: usize,                   // 100
    pub session_timeout: u64,                  // 300 seconds
    pub security_mode: SecurityMode,           // Open
    pub max_subscriptions_per_session: usize,  // 1000
    pub gesture_coalescing: bool,              // true
    pub gesture_coalesce_interval_ms: u64,     // 16 (60fps)
    pub max_messages_per_second: u32,          // 1000
    pub rate_limiting_enabled: bool,           // true
}
```

### Router Struct

```rust
pub struct Router {
    config: RouterConfig,
    sessions: Arc<DashMap<SessionId, Arc<Session>>>,
    subscriptions: Arc<SubscriptionManager>,
    state: Arc<RouterState>,
    running: Arc<RwLock<bool>>,
    token_validator: Option<Arc<dyn TokenValidator>>,
    p2p_capabilities: Arc<P2PCapabilities>,
    gesture_registry: Option<Arc<GestureRegistry>>,
}
```

### Public Methods

```rust
impl Router {
    // Initialization
    pub fn new(config: RouterConfig) -> Self
    pub fn with_validator<V: TokenValidator>(self, validator: V) -> Self
    pub fn set_validator<V: TokenValidator>(&mut self, validator: V)

    // Server lifecycle
    pub async fn serve_on<S: TransportServer>(server: S) -> Result<()>
    pub async fn serve_websocket(addr: &str) -> Result<()>
    pub async fn serve_quic(addr, cert_der, key_der) -> Result<()>
    pub async fn serve_multi(transports: Vec<TransportConfig>) -> Result<()>
    pub async fn serve_all(config: MultiProtocolConfig) -> Result<()>
    pub fn stop(&self)

    // Diagnostics
    pub fn session_count(&self) -> usize
    pub fn subscription_count(&self) -> usize
    pub fn active_gesture_count(&self) -> usize
    pub fn state(&self) -> &RouterState
}
```

### Message Flow

```
Client → [Hello] → Authenticate → Create Session → [Welcome]
       → [Subscribe] → Validate Pattern → Add Subscription → [Snapshot]
       → [Set] → Validate Scope → Apply State → Broadcast → [ACK]
       → [Publish] → Check P2P/Gesture → Broadcast → Subscribers
       → [Bundle] → Phase1:Validate → Phase2:Apply → [ACK]
```

### Hello Processing

1. Validate token (if authenticated mode)
2. Create session with UUID
3. Set authentication state (token, subject, scopes)
4. Send Welcome with session ID and server time
5. Send full snapshot (chunked if >35KB)

### Set Processing

1. Validate scope (write access required)
2. Apply to RouterState with revision tracking
3. Broadcast SET to subscribers (non-blocking)
4. Send ACK with new revision

### Bundle Processing (Two-Phase Commit)

**Phase 1 - Validation:**
- Check write scope for all SET messages
- Check write scope for all PUBLISH messages
- Reject entire bundle if any validation fails

**Phase 2 - Application:**
- Apply all SET messages to state
- Broadcast to subscribers
- Send single ACK with final revision

### Background Tasks

```rust
// Session cleanup (every timeout/4, min 10s)
fn start_session_cleanup_task()
  - Removes sessions idle > timeout
  - Cleans up subscriptions

// Gesture flush (every 16ms)
fn start_gesture_flush_task(registry: Arc<GestureRegistry>)
  - Flushes stale buffered moves
  - Cleanup gestures > 5 minutes old
```

---

## Session Module (session.rs)

### Session Struct

```rust
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub features: Vec<String>,
    sender: Arc<dyn TransportSender>,
    subscriptions: RwLock<HashSet<u32>>,
    pub created_at: Instant,
    pub last_activity: RwLock<Instant>,
    pub authenticated: bool,
    pub token: Option<String>,
    pub subject: Option<String>,
    scopes: Vec<Scope>,
    messages_this_second: AtomicU32,
    last_rate_limit_second: AtomicU64,
}
```

### Session Methods

```rust
impl Session {
    // Creation
    pub fn new(sender, name, features) -> Self
    pub fn set_authenticated(&mut self, token, subject, scopes)

    // Message operations
    pub async fn send(&self, data: Bytes) -> Result<()>
    pub fn try_send(&self, data: Bytes) -> Result<()>  // Non-blocking
    pub async fn send_message(&self, message: &Message) -> Result<()>

    // Subscription management
    pub fn add_subscription(&self, id: u32)
    pub fn remove_subscription(&self, id: u32) -> bool
    pub fn subscriptions(&self) -> Vec<u32>

    // Permission checking
    pub fn has_scope(&self, action: Action, address: &str) -> bool
    pub fn scopes(&self) -> &[Scope]

    // State
    pub fn is_connected(&self) -> bool
    pub fn touch(&self)
    pub fn idle_duration(&self) -> Duration
    pub fn welcome_message(server_name, features) -> Message
}
```

### Rate Limiting

```rust
pub fn check_rate_limit(&self, max_per_second: u32) -> bool
  // Uses Unix timestamp for per-second windows
  // AtomicU32 counter for message count
  // Returns true if within limit

pub fn messages_per_second(&self) -> u32
```

**Algorithm:**
1. Get current Unix timestamp
2. If new second: reset counter, return true
3. Else: increment counter, check against max

---

## Subscription Module (subscription.rs)

### Subscription Struct

```rust
pub struct Subscription {
    pub id: u32,
    pub session_id: SessionId,
    pub pattern: Pattern,
    pub types: HashSet<SignalType>,
    pub options: SubscribeOptions,
}

impl Subscription {
    pub fn matches(&self, address: &str, signal_type: Option<SignalType>) -> bool
}
```

### SubscriptionManager

```rust
pub struct SubscriptionManager {
    subscriptions: DashMap<(SessionId, u32), Subscription>,
    by_prefix: DashMap<String, Vec<(SessionId, u32)>>,
}
```

**Prefix Indexing:**
```
Pattern "/lumen/scene/*/layer/*/opacity"
  → First segment: "lumen"
  → Prefix: "/lumen"

Pattern "/**"
  → First segment: "*"
  → Prefix: "/"
```

### Find Subscribers

```rust
pub fn find_subscribers(address: &str, signal_type: Option<SignalType>) -> Vec<SessionId>
```

**Algorithm:**
1. Extract address prefix (first segment)
2. Collect candidate subscription keys from:
   - Prefix matching address
   - Root prefix "/"
   - Globstar prefix "/**"
   - Single-level wildcard "/*"
3. Check each candidate with `matches()`
4. Return unique session IDs

---

## State Module (RouterState)

### RouterState Struct

```rust
pub struct RouterState {
    params: RwLock<StateStore>,
    listeners: DashMap<String, Vec<Box<dyn Fn(&str, &Value) + Send + Sync>>>,
    pub signals: DashMap<String, SignalDefinition>,
}
```

### Methods

```rust
impl RouterState {
    // Parameter operations
    pub fn get(&self, address: &str) -> Option<Value>
    pub fn get_state(&self, address: &str) -> Option<ParamState>
    pub async fn apply_set(&self, msg: &SetMessage, writer: &SessionId) -> Result<u64>

    // Snapshot generation
    pub fn snapshot(&self, pattern: &str) -> SnapshotMessage
    pub fn full_snapshot(&self) -> SnapshotMessage

    // Signal registry
    pub fn register_signals(&self, signals: Vec<SignalDefinition>)
    pub fn query_signals(&self, pattern: &str) -> Vec<SignalDefinition>
    pub fn all_signals(&self) -> Vec<SignalDefinition>
}
```

### Snapshot Chunking

```rust
const MAX_SNAPSHOT_CHUNK_SIZE: usize = 800;  // params per chunk
const MAX_FRAME_SIZE: usize = 65535;         // bytes

// If params <= 800: single message
// Else: split into 800-param chunks
```

---

## Gesture Module (gesture.rs)

### GestureRegistry

```rust
pub struct GestureRegistry {
    gestures: DashMap<GestureKey, BufferedGesture>,
    flush_interval: Duration,
}

#[derive(Hash, PartialEq, Eq)]
pub struct GestureKey {
    pub address: String,
    pub gesture_id: u32,
}

struct BufferedGesture {
    pending_move: Option<PublishMessage>,
    started_at: Instant,
    last_move_at: Option<Instant>,
}
```

### Move Coalescing (16ms window)

```rust
pub fn process(&self, msg: &PublishMessage) -> GestureResult

GesturePhase::Start
  → Register new gesture → Forward immediately

GesturePhase::Move
  → Buffer (replaces previous) → GestureResult::Buffered

GesturePhase::End/Cancel
  → Remove gesture → Forward pending_move + end/cancel
```

### Flush and Cleanup

```rust
pub fn flush_stale(&self) -> Vec<PublishMessage>
  // Returns moves older than flush_interval

pub fn cleanup_stale(&self, max_age: Duration)
  // Remove gestures older than max_age (default: 5 min)
```

---

## P2P Module (p2p.rs)

### P2PCapabilities

```rust
pub struct P2PCapabilities {
    p2p_capable: DashSet<String>,
}

impl P2PCapabilities {
    pub fn register(&self, session_id: &str)
    pub fn unregister(&self, session_id: &str)
    pub fn is_capable(&self, session_id: &str) -> bool
    pub fn all_capable(&self) -> Vec<String>
    pub fn count(&self) -> usize
}
```

### P2P Address Routing

```rust
pub enum P2PAddressType {
    NotP2P,
    Signal { target_session: String },
    Announce,
}

pub fn analyze_address(address: &str) -> P2PAddressType

// /clasp/p2p/announce → Broadcast
// /clasp/p2p/signal/{session} → Direct route
// /other → Standard routing
```

---

## MQTT Server Adapter (adapters/mqtt_server.rs)

### Configuration

```rust
pub struct MqttServerConfig {
    pub bind_addr: String,           // "0.0.0.0:1883"
    pub namespace: String,           // "/mqtt"
    pub require_auth: bool,
    pub tls: Option<TlsConfig>,
    pub max_clients: usize,
    pub session_timeout_secs: u64,
}
```

### Protocol Translation

| MQTT | CLASP |
|------|-------|
| CONNECT | Hello → Session |
| SUBSCRIBE `sensors/#` | Subscribe `/mqtt/sensors/**` |
| PUBLISH `sensors/temp` | Set `/mqtt/sensors/temp` |
| CONNACK | Welcome |

### Topic Mapping

```rust
mqtt_topic_to_clasp_pattern("/mqtt", "sensors/+/temp")
  → "/mqtt/sensors/*/temp"  (+ → *, # → **)

mqtt_topic_to_clasp_address("/mqtt", "sensors/temp")
  → "/mqtt/sensors/temp"
```

### Payload Conversion

```
MQTT → CLASP:
  JSON parse → Value
  Float parse → Value::Float
  Boolean → Value::Bool
  String → Value::String
  Binary → Value::Bytes

CLASP → MQTT:
  Bool → "true"/"false"
  Int/Float → decimal string
  Complex → JSON
```

---

## OSC Server Adapter (adapters/osc_server.rs)

### Configuration

```rust
pub struct OscServerConfig {
    pub bind_addr: String,           // "0.0.0.0:8000"
    pub namespace: String,           // "/osc"
    pub session_timeout_secs: u64,
    pub auto_subscribe: bool,
}
```

### Session Model

- Each UDP source (IP:port) = one session
- Tracked by peer address
- Auto-cleaned after timeout
- Bidirectional via UDP

### Address Mapping

```rust
OSC "/synth/volume" + namespace "/osc"
  → CLASP "/osc/synth/volume"

CLASP "/osc/synth/volume"
  → OSC "/synth/volume"
```

### Type Conversion

```rust
OSC → CLASP:
  OscType::Int(i)     → Value::Int(i as i64)
  OscType::Float(f)   → Value::Float(f as f64)
  OscType::String(s)  → Value::String(s)
  OscType::Blob(b)    → Value::Bytes(b)
  OscType::Color(c)   → Value::Array([r, g, b, a])
  OscType::Midi(m)    → Value::Array([port, status, d1, d2])

Multiple args → Value::Array
Single arg   → Value (unwrapped)
Empty args   → Value::Null
```

---

## Concurrency Model

- **DashMap**: Lock-free concurrent hash maps for sessions, subscriptions
- **RwLock**: Exclusive write for state, activity timestamps
- **AtomicU32/U64**: Non-blocking rate limit counters
- **Arc**: Shared ownership across async tasks
- **try_send**: Non-blocking broadcasts to prevent deadlocks

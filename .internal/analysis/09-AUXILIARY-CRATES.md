# Auxiliary Crates Analysis

## Overview

This document covers supporting crates: WASM bindings, embedded implementation, test utilities, CLI tools, and deployment.

---

## CLASP-WASM (`crates/clasp-wasm`)

### Purpose

WebAssembly bindings for browser environments.

### Feature Flags

```toml
default = ["console_error_panic_hook"]
p2p = ["wasm-bindgen-futures"]
```

### ClaspWasm

```rust
#[wasm_bindgen]
pub struct ClaspWasm {
    ws: WebSocket,
    session_id: Rc<RefCell<Option<String>>>,
    connected: Rc<RefCell<bool>>,
    params: Rc<RefCell<HashMap<String, JsValue>>>,
    on_message: Rc<RefCell<Option<js_sys::Function>>>,
    on_connect: Rc<RefCell<Option<js_sys::Function>>>,
    on_disconnect: Rc<RefCell<Option<js_sys::Function>>>,
    on_error: Rc<RefCell<Option<js_sys::Function>>>,
    on_auth_error: Rc<RefCell<Option<js_sys::Function>>>,
    sub_id: Rc<RefCell<u32>>,
    token: Rc<RefCell<Option<String>>>,
}

#[wasm_bindgen]
impl ClaspWasm {
    pub fn new(url: &str) -> Result<ClaspWasm, JsValue>
    pub fn new_with_token(url: &str, token: Option<String>) -> Result<ClaspWasm, JsValue>
    pub fn set_token(&self, token: String)

    #[wasm_bindgen(getter)]
    pub fn connected(&self) -> bool

    #[wasm_bindgen(getter)]
    pub fn session_id(&self) -> Option<String>

    pub fn subscribe(&self, pattern: &str) -> u32
    pub fn unsubscribe(&self, id: u32)
    pub fn set(&self, address: &str, value: JsValue)
    pub fn emit(&self, address: &str, payload: JsValue)
    pub fn get(&self, address: &str) -> JsValue
    pub fn close(&self)
}
```

### WasmP2PManager

```rust
#[wasm_bindgen]
pub struct WasmP2PManager {
    session_id: Rc<RefCell<Option<String>>>,
    connections: Rc<RefCell<HashMap<String, WasmP2PConnection>>>,
    known_peers: Rc<RefCell<HashMap<String, Vec<String>>>>,
    ice_servers: Option<js_sys::Array>,
    on_peer_announced: Rc<RefCell<Option<js_sys::Function>>>,
    on_connection_state: Rc<RefCell<Option<js_sys::Function>>>,
    signal_callback: Rc<RefCell<Option<js_sys::Function>>>,
}

#[wasm_bindgen]
impl WasmP2PManager {
    pub fn new(ice_servers: Option<Array>) -> Self
    pub fn set_session_id(&self, session_id: String)
    pub fn announce(&self) -> Result<(), JsValue>
    pub async fn connect_to_peer(&self, peer_session_id: &str) -> Result<(), JsValue>
    pub async fn handle_signal(&self, address: &str, payload: &JsValue) -> Result<(), JsValue>
    pub fn known_peers(&self) -> js_sys::Array
    pub fn is_peer_connected(&self, peer_session_id: &str) -> bool
    pub fn disconnect_peer(&self, peer_session_id: &str)
}
```

### WasmP2PConnection

```rust
#[wasm_bindgen]
pub struct WasmP2PConnection {
    peer_session_id: String,
    correlation_id: String,
    pc: RtcPeerConnection,
    reliable_channel: Rc<RefCell<Option<RtcDataChannel>>>,
    unreliable_channel: Rc<RefCell<Option<RtcDataChannel>>>,
    state: Rc<RefCell<WasmP2PState>>,
    pending_candidates: Rc<RefCell<Vec<String>>>,
    on_message: Rc<RefCell<Option<js_sys::Function>>>,
    on_state_change: Rc<RefCell<Option<js_sys::Function>>>,
}

#[wasm_bindgen]
pub enum WasmP2PState {
    Disconnected,
    Connecting,
    Connected,
    Failed,
    Closed,
}
```

### Value Conversion

```rust
pub fn value_to_js(value: &Value) -> JsValue
pub fn js_to_value(js: &JsValue) -> Value
```

---

## CLASP-EMBEDDED (`crates/clasp-embedded`)

### Purpose

Minimal no_std implementation for microcontrollers.

### Feature Flags

```toml
default = ["client"]
alloc = []      # Enable heap allocation
client = []     # Client mode
server = []     # MiniRouter support
```

### Constants

```rust
pub const MAGIC: u8 = 0x53;
pub const VERSION: u8 = 1;
pub const HEADER_SIZE: usize = 4;
pub const MAX_PAYLOAD: usize = 1024;
```

### Value (no_std)

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
}

impl Value {
    pub fn as_int(&self) -> Option<i64>
    pub fn as_float(&self) -> Option<f64>
    pub fn as_bool(&self) -> Option<bool>
}
```

### Message (zero-copy)

```rust
#[derive(Debug)]
pub enum Message<'a> {
    Hello { name: &'a str, version: u8 },
    Welcome { session: &'a str },
    Set { address: &'a str, value: Value },
    Subscribe { id: u32, pattern: &'a str },
    Unsubscribe { id: u32 },
    Ping,
    Pong,
    Error { code: u16, message: &'a str },
    Unknown(u8),
}
```

### StateCache (Fixed Size)

```rust
pub const MAX_CACHE_ENTRIES: usize = 32;
pub const MAX_ADDRESS_LEN: usize = 64;

pub struct StateCache {
    entries: [CacheEntry; MAX_CACHE_ENTRIES],
    count: usize,
}

impl StateCache {
    pub const fn new() -> Self
    pub fn get(&self, address: &str) -> Option<Value>
    pub fn set(&mut self, address: &str, value: Value) -> bool
    pub fn len(&self) -> usize
    pub fn clear(&mut self)
}
```

### Client

```rust
pub const TX_BUF_SIZE: usize = 256;
pub const RX_BUF_SIZE: usize = 512;

pub enum ClientState {
    Disconnected,
    Connecting,
    Connected,
}

pub struct Client {
    pub state: ClientState,
    pub cache: StateCache,
    tx_buf: [u8; TX_BUF_SIZE],
    rx_buf: [u8; RX_BUF_SIZE],
}

impl Client {
    pub const fn new() -> Self
    pub fn prepare_hello(&mut self, name: &str) -> &[u8]
    pub fn prepare_set(&mut self, address: &str, value: Value) -> &[u8]
    pub fn prepare_subscribe(&mut self, pattern: &str) -> &[u8]
    pub fn prepare_ping(&mut self) -> &[u8]
    pub fn process<'a>(&mut self, data: &'a [u8]) -> Option<Message<'a>>
    pub fn is_connected(&self) -> bool
    pub fn get_cached(&self, address: &str) -> Option<Value>
}
```

**Memory:** ~3KB total

### MiniRouter (feature = "server")

```rust
pub const MAX_CLIENTS: usize = 4;
pub const MAX_SUBS_PER_CLIENT: usize = 8;
pub const MAX_PATTERN_LEN: usize = 64;

pub struct MiniRouter {
    pub state: StateCache,
    sessions: [Session; MAX_CLIENTS],
    session_count: u8,
    tx_buf: [u8; TX_BUF_SIZE],
}

impl MiniRouter {
    pub fn process(&mut self, client_id: u8, data: &[u8]) -> Option<&[u8]>
    pub fn get_broadcast_targets(&self, address: &str, sender_id: u8) -> BroadcastList
    pub fn prepare_broadcast(&mut self, address: &str, value: Value) -> &[u8]
    pub fn disconnect(&mut self, client_id: u8)
}
```

**Memory:** ~4KB total

### Memory Budget

| Device | Total RAM | Client | Server |
|--------|-----------|--------|--------|
| ESP32 | 320KB | ~2KB | ~4KB |
| RP2040 | 264KB | ~2KB | ~4KB |

---

## CLASP-TEST-UTILS (`crates/clasp-test-utils`)

### Constants

```rust
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_millis(10);
```

### Port Allocation

```rust
pub async fn find_available_port() -> u16
pub fn find_available_udp_port() -> u16
```

### Wait Functions

```rust
pub async fn wait_for<F, Fut>(check: F, interval: Duration, max_wait: Duration) -> bool
pub async fn wait_for_count(counter: &AtomicU32, target: u32, max_wait: Duration) -> bool
pub async fn wait_for_flag(flag: &AtomicBool, max_wait: Duration) -> bool
pub async fn wait_with_notify(notify: &Notify, max_wait: Duration) -> bool
```

### TestRouter

```rust
pub struct TestRouter {
    port: u16,
    handle: Option<JoinHandle<()>>,
    ready: Arc<AtomicBool>,
}

impl TestRouter {
    pub async fn start() -> Self
    pub async fn start_with_config(config: RouterConfig) -> Self
    pub fn url(&self) -> String
    pub fn port(&self) -> u16
    pub fn is_ready(&self) -> bool
    pub async fn connect_client(&self) -> Result<Clasp>
    pub async fn connect_client_named(&self, name: &str) -> Result<Clasp>
    pub fn stop(&mut self)
}

impl Drop for TestRouter {
    fn drop(&mut self) { self.stop(); }
}
```

### Assertions

```rust
pub fn assert_approx_eq(actual: f64, expected: f64, epsilon: f64, msg: &str) -> Result<()>
pub fn assert_that(condition: bool, msg: &str) -> Result<()>
pub fn assert_some<T>(opt: Option<T>, msg: &str) -> Result<T>
pub fn assert_ok<T, E>(result: Result<T, E>, msg: &str) -> Result<T>
pub fn assert_err<T, E>(result: Result<T, E>, msg: &str) -> Result<()>
```

### ValueCollector

```rust
#[derive(Clone)]
pub struct ValueCollector {
    values: Arc<Mutex<Vec<(String, Value)>>>,
    notify: Arc<Notify>,
    count: Arc<AtomicU32>,
}

impl ValueCollector {
    pub fn new() -> Self
    pub fn callback(&self) -> impl Fn(Value, String) + Send + 'static
    pub fn callback_ref(&self) -> impl Fn(Value, &str) + Send + Sync + 'static
    pub fn count(&self) -> u32
    pub async fn wait_for_count(&self, n: u32, max_wait: Duration) -> bool
    pub fn values(&self) -> Vec<(String, Value)>
    pub fn has_address(&self, addr: &str) -> bool
    pub fn values_for(&self, addr: &str) -> Vec<Value>
    pub fn last_value(&self) -> Option<(String, Value)>
    pub fn clear(&self)
}
```

---

## CLASP-CLI (`tools/clasp-cli`)

### Commands

```rust
enum Commands {
    Server {
        protocol: String,    // quic, tcp, websocket
        bind: String,        // 0.0.0.0
        port: u16,           // 7331
    },

    Bridge {
        bridge_type: String, // osc, midi, artnet, mqtt, websocket, http
        opt: Vec<String>,    // key=value pairs
    },

    Osc { port: u16, bind: String },
    Mqtt { host: String, port: u16, client_id: Option<String>, topic: Vec<String> },
    Websocket { mode: String, url: String },
    Http { bind: String, base_path: String, cors: bool },

    Pub {
        server: String,      // quic://localhost:7331
        address: String,
        value: String,       // JSON format
    },

    Sub {
        server: String,
        pattern: String,     // /**
    },

    Info,

    Token {
        file: Option<String>,
        action: TokenAction,
    },
}

enum TokenAction {
    Create { scopes: String, expires: Option<String>, subject: Option<String> },
    List { show_expired: bool },
    Show { token: String },
    Revoke { token: String },
    Prune,
}
```

### Token Management

```rust
pub struct TokenRecord {
    pub token: String,
    pub subject: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<u64>,
    pub created_at: u64,
    pub metadata: HashMap<String, String>,
}

pub struct TokenStore {
    tokens: HashMap<String, TokenRecord>,
}

impl TokenStore {
    pub fn load(path: impl AsRef<Path>) -> Result<Self>
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()>
    pub fn add(&mut self, record: TokenRecord)
    pub fn remove(&mut self, token: &str) -> Option<TokenRecord>
    pub fn to_validator(&self) -> Result<CpskValidator>
    pub fn prune_expired(&mut self) -> usize
}

pub fn default_token_file() -> PathBuf
// ~/.config/clasp/tokens.json
```

---

## CLASP-ROUTER Binary (`tools/clasp-router`)

### CLI

```rust
struct Cli {
    listen: SocketAddr,          // 0.0.0.0:7330
    transport: Transport,        // websocket
    name: String,                // "CLASP Router"
    announce: bool,              // mDNS discovery
    cert: Option<String>,        // TLS cert (QUIC)
    key: Option<String>,         // TLS key (QUIC)
    config: Option<String>,      // TOML config
    auth_mode: AuthMode,         // open
    token_file: Option<String>,  // One token per line
    token: Option<String>,       // Single token
    verbose: bool,
}

enum Transport { Websocket, Quic }
enum AuthMode { Open, Authenticated }
```

### Certificate Generation

```rust
fn generate_self_signed_cert() -> Result<(Vec<u8>, Vec<u8>)>
// Uses rcgen with SANs: ["localhost", "127.0.0.1"]
```

---

## Deployment (`deploy/relay`)

### Cargo.toml

```toml
[package]
name = "clasp-relay"
version = "3.1.0"

[features]
default = ["websocket"]
websocket = ["clasp-router/websocket"]
quic = ["clasp-router/quic", "rustls-pemfile"]
full = ["websocket", "quic", "mqtt-server", "osc-server"]
mqtt-server = ["clasp-router/mqtt-server"]
osc-server = ["clasp-router/osc-server"]
```

### Dockerfile

```dockerfile
FROM rust:1.85-slim-bookworm as builder
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/clasp-relay /usr/local/bin/
EXPOSE 7330
ENTRYPOINT ["clasp-relay"]
CMD ["--port", "7330", "--name", "relay.clasp.to"]
```

### docker-compose.yml

```yaml
services:
  router-websocket:
    # Default: Works on DO App Platform
    # Port: 7330

  router-quic:
    profiles: ["full", "quic", "multi"]
    # Port: 7331/udp

  router-multi-ws:
    profiles: ["multi"]
```

### DigitalOcean App Platform

```yaml
name: clasp-relay
services:
  - name: relay
    dockerfile_path: Dockerfile
    http_port: 7330
    instance_size_slug: basic-xxs  # $5/month
    health_check:
      http_path: /
```

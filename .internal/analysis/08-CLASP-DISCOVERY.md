# CLASP-DISCOVERY Crate Analysis

## Overview

**Location:** `crates/clasp-discovery`
**Purpose:** Device discovery via mDNS, UDP broadcast, and HTTP rendezvous

## Feature Flags

```toml
default = ["mdns", "broadcast"]
rendezvous = ["axum", "tower-http", "reqwest", "dashmap"]
```

---

## Discovery Manager (lib.rs)

### DiscoveryConfig

```rust
pub struct DiscoveryConfig {
    pub mdns: bool,              // true
    pub broadcast: bool,         // true
    pub broadcast_port: u16,     // 9999 (DEFAULT_DISCOVERY_PORT)
    pub timeout: Duration,       // 5 seconds
}
```

### Discovery Struct

```rust
pub struct Discovery {
    config: DiscoveryConfig,
    devices: HashMap<String, Device>,
}

impl Discovery {
    pub fn new() -> Self
    pub fn with_config(config: DiscoveryConfig) -> Self
    pub async fn start(&mut self) -> Result<mpsc::Receiver<DiscoveryEvent>>
    pub fn devices(&self) -> impl Iterator<Item = &Device>
    pub fn get(&self, id: &str) -> Option<&Device>
    pub fn add(&mut self, device: Device)
    pub fn remove(&mut self, id: &str) -> Option<Device>
}
```

### DiscoveryEvent

```rust
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    Found(Device),
    Lost(String),      // Device ID
    Error(String),
}
```

---

## Device (device.rs)

### Device Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub info: DeviceInfo,
    pub endpoints: HashMap<String, String>,
    #[serde(skip)]
    pub discovered_at: Instant,
    #[serde(skip)]
    pub last_seen: Instant,
}

impl Device {
    pub fn new(id: String, name: String) -> Self
    pub fn with_ws_endpoint(mut self, url: &str) -> Self
    pub fn with_udp_endpoint(mut self, addr: SocketAddr) -> Self
    pub fn ws_url(&self) -> Option<&str>
    pub fn udp_addr(&self) -> Option<SocketAddr>
    pub fn touch(&mut self)
    pub fn is_stale(&self, timeout: Duration) -> bool
}
```

### DeviceInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub version: u8,                         // PROTOCOL_VERSION (2)
    pub features: Vec<String>,               // ["param", "event", "stream"]
    pub bridge: bool,                        // false
    pub bridge_protocol: Option<String>,
    pub meta: HashMap<String, String>,
}

impl DeviceInfo {
    pub fn with_features(mut self, features: Vec<String>) -> Self
    pub fn as_bridge(mut self, protocol: &str) -> Self
}
```

---

## mDNS Module (mdns.rs)

### Service Type

```rust
const SERVICE_TYPE: &str = "_clasp._tcp.local.";
```

### Discovery Function

```rust
pub async fn discover(tx: mpsc::Sender<DiscoveryEvent>) -> Result<()>
```

**Process:**
1. Create ServiceDaemon
2. Browse for `_clasp._tcp.local.`
3. Handle ServiceEvent variants:
   - `ServiceResolved`: Parse info, build Device
   - `ServiceRemoved`: Send Lost event

### Feature String Parsing

```
'p' -> "param"
's' -> "stream"
'e' -> "event"
't' -> "timeline"
'g' -> "gesture"
```

### TXT Record Fields

- `name`: Device name
- `features`: Feature string (e.g., "pse")
- `ws`: WebSocket port (default: 7330)

### ServiceAdvertiser

```rust
pub struct ServiceAdvertiser {
    mdns: ServiceDaemon,
    fullname: Option<String>,
}

impl ServiceAdvertiser {
    pub fn new() -> Result<Self>
    pub fn advertise(&mut self, name: &str, port: u16, features: &[&str]) -> Result<()>
    pub fn stop(&mut self) -> Result<()>
}

impl Drop for ServiceAdvertiser {
    fn drop(&mut self) {
        self.stop();
    }
}
```

---

## Broadcast Module (broadcast.rs)

### Discovery Function

```rust
pub async fn discover(port: u16, tx: mpsc::Sender<DiscoveryEvent>) -> Result<()>
```

**Process:**
1. Bind UDP to `0.0.0.0:0`
2. Enable broadcast
3. Send HELLO message to `255.255.255.255:port`
4. Listen with 5-second timeout
5. Decode WELCOME responses
6. Build Device from response

### BroadcastResponder

```rust
pub struct BroadcastResponder {
    transport: UdpTransport,
    name: String,
    features: Vec<String>,
}

impl BroadcastResponder {
    pub async fn bind(port: u16, name: String, features: Vec<String>) -> Result<Self>
    pub async fn run(&self) -> Result<()>
}
```

**Respond Process:**
1. Bind UDP on `0.0.0.0:port`
2. Listen for HELLO messages
3. Send WELCOME with UUID session ID

---

## Rendezvous Module (rendezvous.rs)

### DeviceRegistration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegistration {
    pub name: String,
    pub public_key: Option<String>,
    pub features: Vec<String>,
    pub endpoints: HashMap<String, String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}
```

### RegistrationResponse

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub id: String,
    pub timestamp: u64,
    pub ttl: u64,
}
```

### RegisteredDevice

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredDevice {
    pub id: String,
    pub name: String,
    pub public_key: Option<String>,
    pub features: Vec<String>,
    pub endpoints: HashMap<String, String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub registered_at: u64,
    pub last_seen: u64,
}
```

### DiscoverQuery

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoverQuery {
    pub tag: Option<String>,
    pub feature: Option<String>,
    pub limit: Option<usize>,
}
```

### RendezvousConfig

```rust
pub struct RendezvousConfig {
    pub ttl: u64,                         // 300 (5 minutes)
    pub max_devices_per_source: usize,    // 10
    pub max_total_devices: usize,         // 10000
    pub cleanup_interval: u64,            // 60 seconds
}
```

### RendezvousServer

```rust
pub struct RendezvousServer {
    config: RendezvousConfig,
}

impl RendezvousServer {
    pub fn new(config: RendezvousConfig) -> Self
    pub fn router(&self) -> Router
    pub async fn serve(&self, addr: &str) -> Result<()>
}
```

### HTTP Endpoints

| Method | Route | Handler |
|--------|-------|---------|
| POST | /api/v1/register | Device registration |
| GET | /api/v1/discover | Query devices |
| DELETE | /api/v1/unregister/:id | Remove device |
| POST | /api/v1/refresh/:id | Extend TTL |
| GET | /api/v1/health | Health check |

### RendezvousClient

```rust
pub struct RendezvousClient {
    base_url: String,
    client: reqwest::Client,
}

impl RendezvousClient {
    pub fn new(base_url: &str) -> Self
    pub async fn register(&self, registration: DeviceRegistration)
        -> Result<RegistrationResponse>
    pub async fn discover(&self, tag: Option<&str>)
        -> Result<Vec<RegisteredDevice>>
    pub async fn unregister(&self, id: &str) -> Result<bool>
    pub async fn refresh(&self, id: &str) -> Result<bool>
}
```

---

## Error Handling (error.rs)

```rust
#[derive(Error, Debug)]
pub enum DiscoveryError {
    #[error("mDNS error: {0}")]
    Mdns(String),

    #[error("broadcast error: {0}")]
    Broadcast(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("discovery error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DiscoveryError>;
```

---

## Discovery Methods Comparison

| Method | Transport | Direction | Features |
|--------|-----------|-----------|----------|
| **mDNS** | Multicast DNS | Bidirectional | Auto-discovery, standard protocol |
| **Broadcast** | UDP | Request/Response | Simple, LAN-only |
| **Rendezvous** | HTTP | Client-Server | WAN support, TTL, tags |

---

## Usage Examples

### mDNS Discovery

```rust
let mut discovery = Discovery::new();
let mut rx = discovery.start().await?;

while let Some(event) = rx.recv().await {
    match event {
        DiscoveryEvent::Found(device) => {
            println!("Found: {} at {}", device.name, device.ws_url().unwrap_or("?"));
        }
        DiscoveryEvent::Lost(id) => {
            println!("Lost: {}", id);
        }
        _ => {}
    }
}
```

### mDNS Advertisement

```rust
let mut advertiser = ServiceAdvertiser::new()?;
advertiser.advertise("My Device", 7330, &["param", "event", "stream"])?;

// Runs until dropped
```

### Rendezvous Registration

```rust
let client = RendezvousClient::new("https://rendezvous.clasp.to");

let response = client.register(DeviceRegistration {
    name: "My Device".into(),
    features: vec!["param".into(), "event".into()],
    endpoints: [("ws".into(), "ws://192.168.1.100:7330".into())].into(),
    tags: vec!["lighting".into()],
    ..Default::default()
}).await?;

println!("Registered with ID: {}", response.id);
```

### Rendezvous Discovery

```rust
let client = RendezvousClient::new("https://rendezvous.clasp.to");
let devices = client.discover(Some("lighting")).await?;

for device in devices {
    println!("{}: {:?}", device.name, device.endpoints);
}
```

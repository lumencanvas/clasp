# CLASP-CORE Crate Analysis

## Overview

**Location:** `crates/clasp-core`
**Purpose:** Protocol types, binary codec, frame format - the foundation layer

## Module Structure

```
clasp-core/
├── src/
│   ├── lib.rs
│   ├── types.rs      # Message & Value types
│   ├── codec.rs      # Binary encoding/decoding
│   ├── address.rs    # Address patterns & matching
│   ├── state.rs      # Parameter state & storage
│   ├── time.rs       # Clock sync & jitter buffer
│   ├── security.rs   # Scopes & token validation
│   ├── timeline.rs   # Timeline playback
│   ├── p2p.rs        # P2P signaling
│   ├── error.rs      # Error types
│   └── frame.rs      # Frame format
└── tests/
```

## Feature Flags

```toml
default = ["std"]
std = []           # Standard library
alloc = []         # Alloc-only (no_std)
```

---

## Types Module (types.rs)

### Message Type Codes

```rust
#[repr(u8)]
pub enum MessageType {
    Hello = 0x01,
    Welcome = 0x02,
    Announce = 0x03,
    Subscribe = 0x10,
    Unsubscribe = 0x11,
    Publish = 0x20,
    Set = 0x21,
    Get = 0x22,
    Snapshot = 0x23,
    Bundle = 0x30,
    Sync = 0x40,
    Ping = 0x41,
    Pong = 0x42,
    Ack = 0x50,
    Error = 0x51,
    Query = 0x60,
    Result = 0x61,
}
```

### Quality of Service

```rust
#[repr(u8)]
pub enum QoS {
    Fire = 0,      // Best effort, no confirmation
    Confirm = 1,   // At least once delivery
    Commit = 2,    // Exactly once, ordered
}
```

### Signal Types

```rust
pub enum SignalType {
    Param,     // Stateful, revision-tracked → QoS::Confirm
    Event,     // Ephemeral, one-shot → QoS::Confirm
    Stream,    // High-rate continuous → QoS::Fire
    Gesture,   // Multi-touch phases → QoS::Fire
    Timeline,  // Keyframe automation → QoS::Commit
}
```

### Conflict Resolution

```rust
pub enum ConflictStrategy {
    Lww,    // Last write wins (by timestamp)
    Max,    // Keep maximum value
    Min,    // Keep minimum value
    Lock,   // First writer holds lock
    Merge,  // Application-defined
}
```

### Gesture Phases

```rust
pub enum GesturePhase {
    Start,
    Move,
    End,
    Cancel,
}
```

### Easing Types

```rust
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Step,
    CubicBezier,
}
```

### Value Enum

```rust
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Bytes(Vec<u8>),
}

impl Value {
    pub fn as_f64(&self) -> Option<f64>
    pub fn as_i64(&self) -> Option<i64>
    pub fn as_bool(&self) -> Option<bool>
    pub fn as_str(&self) -> Option<&str>
}
```

### Message Structs

```rust
pub struct HelloMessage {
    pub version: u8,
    pub name: String,
    pub features: Vec<String>,
    pub capabilities: Option<Capabilities>,
    pub token: Option<String>,
}

pub struct WelcomeMessage {
    pub version: u8,
    pub session: String,
    pub name: String,
    pub features: Vec<String>,
    pub time: u64,  // Server time in µs
    pub token: Option<String>,
}

pub struct SetMessage {
    pub address: String,
    pub value: Value,
    pub revision: Option<u64>,
    pub lock: bool,
    pub unlock: bool,
}

pub struct PublishMessage {
    pub address: String,
    pub signal: Option<SignalType>,
    pub value: Option<Value>,
    pub payload: Option<Value>,
    pub samples: Option<Vec<f64>>,
    pub rate: Option<u32>,
    pub id: Option<u32>,
    pub phase: Option<GesturePhase>,
    pub timestamp: Option<u64>,
    pub timeline: Option<TimelineData>,
}

pub struct SubscribeMessage {
    pub id: u32,
    pub pattern: String,
    pub types: Vec<SignalType>,
    pub options: Option<SubscribeOptions>,
}

pub struct SnapshotMessage {
    pub params: Vec<ParamValue>,
}

pub struct BundleMessage {
    pub timestamp: Option<u64>,
    pub messages: Vec<Message>,
}

pub struct SyncMessage {
    pub t1: u64,
    pub t2: Option<u64>,
    pub t3: Option<u64>,
}

pub struct AckMessage {
    pub address: Option<String>,
    pub revision: Option<u64>,
    pub locked: Option<bool>,
    pub holder: Option<String>,
    pub correlation_id: Option<u32>,
}

pub struct ErrorMessage {
    pub code: u16,
    pub message: String,
    pub address: Option<String>,
    pub correlation_id: Option<u32>,
}
```

### Supporting Structs

```rust
pub struct Capabilities {
    pub encryption: bool,
    pub compression: Option<String>,
}

pub struct SignalDefinition {
    pub address: String,
    pub signal_type: SignalType,
    pub datatype: Option<String>,
    pub access: Option<String>,
    pub meta: Option<SignalMeta>,
}

pub struct SignalMeta {
    pub unit: Option<String>,
    pub range: Option<(f64, f64)>,
    pub default: Option<Value>,
    pub description: Option<String>,
}

pub struct SubscribeOptions {
    pub max_rate: Option<u32>,
    pub epsilon: Option<f64>,
    pub history: Option<u32>,
    pub window: Option<u32>,
}

pub struct ParamValue {
    pub address: String,
    pub value: Value,
    pub revision: u64,
    pub writer: Option<String>,
    pub timestamp: Option<u64>,
}

pub struct TimelineKeyframe {
    pub time: u64,
    pub value: Value,
    pub easing: EasingType,
    pub bezier: Option<[f64; 4]>,
}

pub struct TimelineData {
    pub keyframes: Vec<TimelineKeyframe>,
    pub loop_: bool,
    pub start_time: Option<u64>,
    pub duration: Option<u64>,
}
```

---

## Codec Module (codec.rs)

### Constants

```rust
pub const ENCODING_VERSION: u8 = 1;  // Binary v3

pub mod msg {
    pub const HELLO: u8 = 0x01;
    pub const WELCOME: u8 = 0x02;
    pub const ANNOUNCE: u8 = 0x03;
    pub const SUBSCRIBE: u8 = 0x10;
    pub const UNSUBSCRIBE: u8 = 0x11;
    pub const PUBLISH: u8 = 0x20;
    pub const SET: u8 = 0x21;
    pub const GET: u8 = 0x22;
    pub const SNAPSHOT: u8 = 0x23;
    pub const BUNDLE: u8 = 0x30;
    pub const SYNC: u8 = 0x40;
    pub const PING: u8 = 0x41;
    pub const PONG: u8 = 0x42;
    pub const ACK: u8 = 0x50;
    pub const ERROR: u8 = 0x51;
    pub const QUERY: u8 = 0x60;
    pub const RESULT: u8 = 0x61;
}

pub mod val {
    pub const NULL: u8 = 0x00;
    pub const BOOL: u8 = 0x01;
    pub const I8: u8 = 0x02;
    pub const I16: u8 = 0x03;
    pub const I32: u8 = 0x04;
    pub const I64: u8 = 0x05;
    pub const F32: u8 = 0x06;
    pub const F64: u8 = 0x07;
    pub const STRING: u8 = 0x08;
    pub const BYTES: u8 = 0x09;
    pub const ARRAY: u8 = 0x0A;
    pub const MAP: u8 = 0x0B;
}
```

### Public API

```rust
// Message encoding
pub fn encode_message(message: &Message) -> Result<Bytes>
pub fn encode(message: &Message) -> Result<Bytes>
pub fn encode_with_options(
    message: &Message,
    qos: Option<QoS>,
    timestamp: Option<u64>
) -> Result<Bytes>
pub fn encode_payload(message: &Message) -> Result<Vec<u8>>

// Message decoding (auto-detects v2 MessagePack vs v3 binary)
pub fn decode_message(bytes: &[u8]) -> Result<Message>
pub fn decode(bytes: &[u8]) -> Result<(Message, Frame)>
pub fn decode_payload(bytes: &[u8]) -> Result<Message>
```

### Binary Format

**SET Message (0x21)**
```
Flags: [has_rev:1][lock:1][unlock:1][rsv:1][vtype:4]
- msg::SET (1 byte)
- flags byte
- address (u16 len + bytes)
- value (type byte + data)
- optional revision (u64)
```

**PUBLISH Message (0x20)**
```
Flags: [sig_type:3][has_ts:1][has_id:1][phase:3]
- msg::PUBLISH (1 byte)
- flags byte
- address (u16 len + bytes)
- value indicator (1 byte): 0=none, 1=value, 2=samples
- optional value/samples
- optional timestamp/gesture_id/rate
```

### Performance

- SET: 69 bytes (MessagePack) → 32 bytes (binary) = **54% smaller**
- Encoding: ~10M msg/s (vs 1.8M MessagePack)
- Decoding: ~12M msg/s (vs 1.5M MessagePack)

---

## Address Module (address.rs)

### Address Structure

```rust
pub struct Address {
    raw: String,
    segments: Vec<String>,
}

impl Address {
    pub fn parse(s: &str) -> Result<Self>
    pub fn as_str(&self) -> &str
    pub fn segments(&self) -> &[String]
    pub fn namespace(&self) -> Option<&str>
    pub fn property(&self) -> Option<&str>
    pub fn is_pattern(&self) -> bool
    pub fn matches(&self, pattern: &Address) -> bool
}
```

### Pattern Matching

```rust
pub struct Pattern {
    address: Address,
    regex: Option<regex_lite::Regex>,
}

impl Pattern {
    pub fn compile(s: &str) -> Result<Self>
    pub fn matches(&self, addr: &str) -> bool
    pub fn matches_address(&self, addr: &Address) -> bool
    pub fn address(&self) -> &Address
}

pub fn glob_match(pattern: &str, address: &str) -> bool
```

**Wildcard Rules:**
- `*` matches exactly one segment
- `**` matches zero or more segments
- `/a/*/c` matches `/a/b/c`
- `/a/**` matches `/a/b/c/d`

---

## State Module (state.rs)

### Parameter State

```rust
pub struct ParamState {
    pub value: Value,
    pub revision: u64,
    pub writer: String,
    pub timestamp: u64,
    pub strategy: ConflictStrategy,
    pub lock_holder: Option<String>,
    pub meta: Option<ParamMeta>,
}

impl ParamState {
    pub fn new(value: Value, writer: String) -> Self
    pub fn with_strategy(self, strategy: ConflictStrategy) -> Self
    pub fn with_meta(self, meta: ParamMeta) -> Self
    pub fn try_update(
        &mut self,
        new_value: Value,
        writer: &str,
        expected_revision: Option<u64>,
        request_lock: bool,
        release_lock: bool,
    ) -> Result<u64, UpdateError>
    pub fn validate_range(&self, value: &Value) -> bool
}

pub enum UpdateError {
    RevisionConflict { expected: u64, actual: u64 },
    LockHeld { holder: String },
    ConflictRejected,
    OutOfRange,
}
```

### State Store

```rust
pub struct StateStore {
    params: HashMap<String, ParamState>,
}

impl StateStore {
    pub fn new() -> Self
    pub fn get(&self, address: &str) -> Option<&ParamState>
    pub fn get_value(&self, address: &str) -> Option<&Value>
    pub fn set(...) -> Result<u64, UpdateError>
    pub fn get_matching(&self, pattern: &str) -> Vec<(&str, &ParamState)>
    pub fn snapshot(&self) -> Vec<(&str, &ParamState)>
    pub fn len(&self) -> usize
    pub fn remove(&mut self, address: &str) -> Option<ParamState>
    pub fn clear(&mut self)
}
```

---

## Time Module (time.rs)

### Clock Synchronization

```rust
pub type Timestamp = u64;  // Microseconds

pub fn now() -> Timestamp
pub fn to_duration(micros: Timestamp) -> Duration
pub fn from_duration(duration: Duration) -> Timestamp

pub struct ClockSync {
    offset: i64,
    rtt: u64,
    jitter: u64,
    samples: u32,
    last_sync: Instant,
    rtt_history: Vec<u64>,
}

impl ClockSync {
    pub fn new() -> Self
    pub fn process_sync(&mut self, t1: u64, t2: u64, t3: u64, t4: u64)
    pub fn server_time(&self) -> Timestamp
    pub fn to_server_time(&self, local: Timestamp) -> Timestamp
    pub fn to_local_time(&self, server: Timestamp) -> Timestamp
    pub fn offset(&self) -> i64
    pub fn rtt(&self) -> u64
    pub fn jitter(&self) -> u64
    pub fn needs_sync(&self, interval_secs: u64) -> bool
    pub fn quality(&self) -> f64  // 0.0-1.0
}
```

### Session Time

```rust
pub struct SessionTime {
    start: Instant,
    start_unix: Timestamp,
}

impl SessionTime {
    pub fn new() -> Self
    pub fn elapsed(&self) -> Timestamp
    pub fn start_time(&self) -> Timestamp
    pub fn to_unix(&self, session_time: Timestamp) -> Timestamp
    pub fn from_unix(&self, unix_time: Timestamp) -> Timestamp
}
```

### Jitter Buffer

```rust
pub struct JitterBuffer<T> {
    buffer: Vec<(Timestamp, T)>,
    capacity: usize,
    window_us: u64,
}

impl<T> JitterBuffer<T> {
    pub fn new(capacity: usize, window_ms: u64) -> Self
    pub fn push(&mut self, timestamp: Timestamp, value: T)
    pub fn pop(&mut self, playback_time: Timestamp) -> Option<T>
    pub fn drain_ready(&mut self, playback_time: Timestamp) -> Vec<T>
    pub fn len(&self) -> usize
    pub fn depth_us(&self) -> u64
}
```

---

## Security Module (security.rs)

### Action & Scope

```rust
pub enum Action {
    Read,   // SUBSCRIBE, GET
    Write,  // SET, PUBLISH
    Admin,  // All operations
}

impl Action {
    pub fn allows(&self, other: Action) -> bool
}

pub struct Scope {
    action: Action,
    pattern: Pattern,
    raw: String,
}

impl Scope {
    pub fn new(action: Action, pattern_str: &str) -> Result<Self>
    pub fn parse(s: &str) -> Result<Self>  // "action:pattern"
    pub fn allows(&self, action: Action, address: &str) -> bool
}
```

### Token Validation

```rust
pub struct TokenInfo {
    pub token_id: String,
    pub subject: Option<String>,
    pub scopes: Vec<Scope>,
    pub expires_at: Option<SystemTime>,
    pub metadata: HashMap<String, String>,
}

pub enum ValidationResult {
    Valid(TokenInfo),
    NotMyToken,
    Invalid(String),
    Expired,
}

pub trait TokenValidator: Send + Sync + Any {
    fn validate(&self, token: &str) -> ValidationResult;
    fn name(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}
```

### CPSK Validator

```rust
pub struct CpskValidator {
    tokens: RwLock<HashMap<String, TokenInfo>>,
}

pub const PREFIX: &str = "cpsk_";

impl CpskValidator {
    pub fn new() -> Self
    pub fn register(&self, token: String, info: TokenInfo)
    pub fn revoke(&self, token: &str) -> bool
    pub fn exists(&self, token: &str) -> bool
    pub fn generate_token() -> String  // "cpsk_" + 32 base62 chars
}

pub struct ValidatorChain {
    validators: Vec<Box<dyn TokenValidator>>,
}
```

### Utilities

```rust
pub fn parse_scopes(s: &str) -> Result<Vec<Scope>>
pub fn parse_duration(s: &str) -> Result<Duration>  // "7d", "24h", "30m"
pub enum SecurityMode { Open, Authenticated }
```

---

## Timeline Module (timeline.rs)

```rust
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Finished,
}

pub struct TimelinePlayer {
    timeline: TimelineData,
    state: PlaybackState,
    start_time: u64,
    pause_time: Option<u64>,
    loop_count: u32,
}

impl TimelinePlayer {
    pub fn new(timeline: TimelineData) -> Self
    pub fn start(&mut self, current_time_us: u64)
    pub fn start_at(&mut self, start_time_us: u64)
    pub fn pause(&mut self, current_time_us: u64)
    pub fn resume(&mut self, current_time_us: u64)
    pub fn stop(&mut self)
    pub fn state(&self) -> PlaybackState
    pub fn loop_count(&self) -> u32
    pub fn duration(&self) -> u64
    pub fn sample(&mut self, current_time_us: u64) -> Option<Value>
}
```

### Easing Functions

- Linear: `t`
- EaseIn: `t²`
- EaseOut: `1 - (1-t)²`
- EaseInOut: piecewise quadratic
- Step: 0 until t=1, then 1
- CubicBezier: Newton-Raphson iteration

---

## P2P Module (p2p.rs)

```rust
pub const P2P_NAMESPACE: &str = "/clasp/p2p";
pub const P2P_SIGNAL_PREFIX: &str = "/clasp/p2p/signal/";
pub const P2P_ANNOUNCE: &str = "/clasp/p2p/announce";

pub enum P2PSignal {
    Offer { from: String, sdp: String, correlation_id: String },
    Answer { from: String, sdp: String, correlation_id: String },
    IceCandidate { from: String, candidate: String, correlation_id: String },
    Connected { from: String, correlation_id: String },
    Disconnected { from: String, correlation_id: String, reason: Option<String> },
}

pub struct P2PConfig {
    pub ice_servers: Vec<String>,
    pub turn_servers: Vec<TurnServer>,
    pub connection_timeout_secs: u64,
    pub max_retries: u32,
    pub auto_fallback: bool,
}

pub enum P2PConnectionState {
    Disconnected, Connecting, GatheringCandidates, Connected, Failed, Closed,
}

pub enum RoutingMode {
    ServerOnly, P2POnly, PreferP2P,
}

pub fn is_p2p_address(address: &str) -> bool
pub fn signal_address(target_session_id: &str) -> String
```

---

## Error Module (error.rs)

```rust
pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    InvalidMagic(u8),
    PayloadTooLarge(usize),
    BufferTooSmall { needed: usize, have: usize },
    EncodeError(String),
    DecodeError(String),
    UnknownMessageType(u8),
    UnknownSignalType(String),
    InvalidAddress(String),
    InvalidPattern(String),
    RevisionConflict { expected: u64, actual: u64 },
    LockHeld { holder: String },
    PermissionDenied(String),
    ConnectionError(String),
    Timeout,
    Protocol(String),
}

pub enum ErrorCode {
    // 100-199: Protocol errors
    InvalidFrame = 100,
    InvalidMessage = 101,
    UnsupportedVersion = 102,
    // 200-299: Address errors
    InvalidAddress = 200,
    AddressNotFound = 201,
    PatternError = 202,
    // 300-399: Permission errors
    Unauthorized = 300,
    Forbidden = 301,
    TokenExpired = 302,
    // 400-499: State errors
    RevisionConflict = 400,
    LockHeld = 401,
    InvalidValue = 402,
    // 500-599: Server errors
    InternalError = 500,
    ServiceUnavailable = 501,
    Timeout = 502,
}
```

---

## Frame Module (frame.rs)

```rust
pub const HEADER_SIZE: usize = 4;
pub const HEADER_SIZE_WITH_TS: usize = 12;
pub const MAX_PAYLOAD_SIZE: usize = 65535;
pub const MAGIC_BYTE: u8 = 0x53;  // 'S'

pub struct FrameFlags {
    pub qos: QoS,
    pub has_timestamp: bool,
    pub encrypted: bool,
    pub compressed: bool,
    pub version: u8,
}

impl FrameFlags {
    pub fn to_byte(&self) -> u8
    pub fn from_byte(byte: u8) -> Self
    pub fn is_binary_encoding(&self) -> bool
}

pub struct Frame {
    pub flags: FrameFlags,
    pub timestamp: Option<u64>,
    pub payload: Bytes,
}

impl Frame {
    pub fn new(payload: impl Into<Bytes>) -> Self
    pub fn with_qos(self, qos: QoS) -> Self
    pub fn with_timestamp(self, timestamp: u64) -> Self
    pub fn size(&self) -> usize
    pub fn encode(&self) -> Result<Bytes>
    pub fn decode(buf: impl Buf) -> Result<Self>
    pub fn check_complete(buf: &[u8]) -> Option<usize>
}
```

**Frame Format:**
```
Byte 0:     Magic (0x53)
Byte 1:     Flags [qos:2][ts:1][enc:1][comp:1][ver:3]
Byte 2-3:   Payload Length (u16 BE)
[Bytes 4-11]: Timestamp (u64, if flag set)
Payload:    Message data
```

# Complete Function and Class Map

## clasp-core

### Enums

| Enum | Variants | Purpose |
|------|----------|---------|
| `MessageType` | Hello(0x01), Welcome(0x02), Announce(0x03), Subscribe(0x10), Unsubscribe(0x11), Publish(0x20), Set(0x21), Get(0x22), Snapshot(0x23), Bundle(0x30), Sync(0x40), Ping(0x41), Pong(0x42), Ack(0x50), Error(0x51), Query(0x60), Result(0x61) | Binary message type codes |
| `Message` | 17 message variants | Top-level message enum |
| `SignalType` | Param, Event, Stream, Gesture, Timeline | Signal classification |
| `Value` | Null, Bool, Int, Float, String, Array, Map, Bytes | Protocol value types |
| `QoS` | Fire(0), Confirm(1), Commit(2) | Quality of service levels |
| `ConflictStrategy` | Lww, Max, Min, Lock, Merge | State conflict resolution |
| `GesturePhase` | Start, Move, End, Cancel | Touch input phases |
| `EasingType` | Linear, EaseIn, EaseOut, EaseInOut, Step, CubicBezier | Animation curves |
| `Error` | InvalidMagic, PayloadTooLarge, BufferTooSmall, EncodeError, DecodeError, UnknownMessageType, InvalidAddress, RevisionConflict, LockHeld, PermissionDenied, ConnectionError, Timeout, Protocol | Error types |
| `ErrorCode` | 100-502 range codes | Protocol error codes |
| `Action` | Read, Write, Admin | Permission actions |
| `SecurityMode` | Open, Authenticated | Auth modes |
| `ValidationResult` | Valid, NotMyToken, Invalid, Expired | Token validation |
| `P2PSignal` | Offer, Answer, IceCandidate, Connected, Disconnected | WebRTC signaling |
| `P2PConnectionState` | Disconnected, Connecting, GatheringCandidates, Connected, Failed, Closed | P2P states |
| `RoutingMode` | ServerOnly, P2POnly, PreferP2P | Message routing |
| `PlaybackState` | Stopped, Playing, Paused, Finished | Timeline states |

### Structs

| Struct | Fields | Purpose |
|--------|--------|---------|
| `HelloMessage` | version, name, features, capabilities, token | Client handshake |
| `WelcomeMessage` | version, session, name, features, time, token | Server response |
| `SetMessage` | address, value, revision, lock, unlock | Parameter set |
| `PublishMessage` | address, signal, value, payload, samples, rate, id, phase, timestamp, timeline | Publish value |
| `SubscribeMessage` | id, pattern, types, options | Subscribe request |
| `BundleMessage` | timestamp, messages | Atomic group |
| `SnapshotMessage` | params | State dump |
| `ParamValue` | address, value, revision, writer, timestamp | Cached param |
| `FrameFlags` | qos, has_timestamp, encrypted, compressed, version | Frame header |
| `Frame` | flags, timestamp, payload | Binary frame |
| `Address` | raw, segments | Parsed address |
| `Pattern` | address, regex | Compiled pattern |
| `ParamState` | value, revision, writer, timestamp, strategy, lock_holder, meta | Parameter state |
| `StateStore` | params | State storage |
| `ClockSync` | offset, rtt, jitter, samples, last_sync, rtt_history | Time sync |
| `SessionTime` | start, start_unix | Session timing |
| `JitterBuffer<T>` | buffer, capacity, window_us | Stream buffer |
| `Scope` | action, pattern, raw | Permission scope |
| `TokenInfo` | token_id, subject, scopes, expires_at, metadata | Token data |
| `CpskValidator` | tokens | CPSK token store |
| `ValidatorChain` | validators | Validator chain |
| `TimelineData` | keyframes, loop_, start_time, duration | Automation |
| `TimelineKeyframe` | time, value, easing, bezier | Keyframe |
| `TimelinePlayer` | timeline, state, start_time, pause_time, loop_count | Player |
| `P2PConfig` | ice_servers, turn_servers, connection_timeout_secs, max_retries, auto_fallback | P2P config |
| `P2PAnnounce` | session_id, p2p_capable, features | P2P announcement |

### Functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| `encode` | `(&Message) -> Result<Bytes>` | Encode message |
| `decode` | `(&[u8]) -> Result<(Message, Frame)>` | Decode frame |
| `encode_message` | `(&Message) -> Result<Bytes>` | Encode payload |
| `decode_message` | `(&[u8]) -> Result<Message>` | Decode payload |
| `encode_with_options` | `(&Message, Option<QoS>, Option<u64>) -> Result<Bytes>` | Encode with options |
| `now` | `() -> Timestamp` | Current time (µs) |
| `glob_match` | `(&str, &str) -> bool` | Pattern matching |
| `is_p2p_address` | `(&str) -> bool` | Check P2P address |
| `signal_address` | `(&str) -> String` | Build P2P address |
| `parse_scopes` | `(&str) -> Result<Vec<Scope>>` | Parse scope string |

---

## clasp-router

### Structs

| Struct | Fields | Purpose |
|--------|--------|---------|
| `Router` | config, sessions, subscriptions, state, running, token_validator, p2p_capabilities, gesture_registry | Central hub |
| `RouterConfig` | name, features, max_sessions, session_timeout, security_mode, max_subscriptions_per_session, gesture_coalescing, gesture_coalesce_interval_ms, max_messages_per_second, rate_limiting_enabled | Configuration |
| `Session` | id, name, features, sender, subscriptions, created_at, last_activity, authenticated, token, subject, scopes, messages_this_second, last_rate_limit_second | Client session |
| `Subscription` | id, session_id, pattern, types, options | Subscription |
| `SubscriptionManager` | subscriptions, by_prefix | Subscription index |
| `RouterState` | params, listeners, signals | State storage |
| `GestureRegistry` | gestures, flush_interval | Move coalescing |
| `GestureKey` | address, gesture_id | Gesture identifier |
| `P2PCapabilities` | p2p_capable | P2P tracking |
| `MqttServerAdapter` | config, sessions, subscriptions, state, mqtt_sessions, running, tls_acceptor | MQTT server |
| `OscServerAdapter` | config, sessions, subscriptions, state, osc_sessions, running, socket | OSC server |

### Enums

| Enum | Variants | Purpose |
|------|----------|---------|
| `GestureResult` | Forward, Buffered, PassThrough | Coalescing result |
| `P2PAddressType` | NotP2P, Signal, Announce | P2P address type |
| `RouterError` | SessionNotFound, InvalidMessage, Routing, State, Config, Transport, Core, Protocol, Auth, Io, Mqtt, Other | Errors |

### Key Methods

| Struct | Method | Purpose |
|--------|--------|---------|
| `Router` | `new(config)` | Create router |
| `Router` | `serve_websocket(addr)` | Serve WebSocket |
| `Router` | `serve_quic(addr, cert, key)` | Serve QUIC |
| `Router` | `serve_multi(config)` | Multi-protocol |
| `Session` | `send(message)` | Send to client |
| `Session` | `has_scope(action, addr)` | Check permission |
| `Session` | `check_rate_limit(max)` | Rate limiting |
| `SubscriptionManager` | `find_subscribers(addr, type)` | Find matching |
| `GestureRegistry` | `process(message)` | Coalesce moves |

---

## clasp-transport

### Traits

| Trait | Methods | Purpose |
|-------|---------|---------|
| `Transport` | `connect(addr) -> (Sender, Receiver)` | Connection |
| `TransportSender` | `send(data)`, `try_send(data)`, `is_connected()`, `close()` | Sending |
| `TransportReceiver` | `recv() -> Option<TransportEvent>` | Receiving |
| `TransportServer` | `accept() -> (Sender, Receiver, Addr)`, `local_addr()`, `close()` | Server |

### Enums

| Enum | Variants | Purpose |
|------|----------|---------|
| `TransportEvent` | Connected, Disconnected, Data, Error | Events |
| `TransportError` | ConnectionFailed, ConnectionClosed, BindFailed, AcceptFailed, SendFailed, BufferFull, ReceiveFailed, InvalidUrl, Timeout, Io, Protocol, NotConnected, AlreadyConnected, Other | Errors |

### Transport Implementations

| Transport | Config | Sender | Receiver | Server |
|-----------|--------|--------|----------|--------|
| WebSocket | `WebSocketConfig` | `WebSocketSender` | `WebSocketReceiver` | `WebSocketServer` |
| TCP | `TcpConfig` | `TcpSender` | `TcpReceiver` | `TcpServer` |
| UDP | `UdpConfig` | `UdpSender` | `UdpReceiver` | - |
| QUIC | `QuicConfig` | `QuicSender` | `QuicReceiver` | `QuicTransport` |
| Serial | `SerialConfig` | `SerialSender` | `SerialReceiver` | - |
| BLE | `BleConfig` | `BleSender` | `BleReceiver` | - |
| WebRTC | `WebRtcConfig` | `WebRtcSender` | `WebRtcReceiver` | - |
| WASM WS | `WasmWebSocketConfig` | `WasmWebSocketSender` | `WasmWebSocketReceiver` | - |

---

## clasp-bridge

### Traits

| Trait | Methods | Purpose |
|-------|---------|---------|
| `Bridge` | `config()`, `start()`, `stop()`, `send(msg)`, `is_running()`, `namespace()` | Bridge interface |

### Enums

| Enum | Variants | Purpose |
|------|----------|---------|
| `BridgeEvent` | ToClasp(Message), Connected, Disconnected, Error | Bridge events |
| `BridgeError` | ConnectionFailed, Protocol, Mapping, Send, Receive, DeviceNotFound, Io, Other | Errors |
| `Transform` | Identity, Scale, Clamp, Invert, ToInt, ToFloat, Expression, Lookup, Quantize, Modulo, Abs, Negate, Power, Log, Round, Curve, DeadZone, Smooth, RateLimit, Threshold, Chain, Conditional, JsonPath, MapType, Bitwise | 48 variants |
| `CurveType` | Linear, EaseIn, EaseOut, EaseInOut, QuadIn, QuadOut, CubicIn, CubicOut, ExpoIn, ExpoOut, SineIn, SineOut, CircIn, CircOut, ElasticIn, ElasticOut, BounceOut, Bezier | 22 curves |
| `Condition` | GreaterThan, LessThan, Equals, InRange, Expression, And, Or, Not | Conditions |
| `Aggregator` | Average, Sum, Min, Max, Latest, First, Count, MovingAverage, RateOfChange, StdDev | Aggregators |

### Bridge Implementations

| Bridge | Config | Protocol |
|--------|--------|----------|
| `OscBridge` | `OscBridgeConfig` | OSC/UDP |
| `MidiBridge` | `MidiBridgeConfig` | MIDI |
| `ArtNetBridge` | `ArtNetBridgeConfig` | Art-Net/DMX |
| `DmxBridge` | `DmxBridgeConfig` | DMX/Serial |
| `SacnBridge` | `SacnBridgeConfig` | sACN/E1.31 |
| `MqttBridge` | `MqttBridgeConfig` | MQTT |
| `WebSocketBridge` | `WebSocketBridgeConfig` | WebSocket |
| `SocketIOBridge` | `SocketIOBridgeConfig` | Socket.IO |
| `HttpBridge` | `HttpBridgeConfig` | HTTP/REST |

---

## clasp-client

### Structs

| Struct | Fields | Purpose |
|--------|--------|---------|
| `Clasp` | url, name, features, token, reconnect, session_id, connected, sender, params, subscriptions, clock, signals, last_error, pending_gets, reconnect_attempts, intentionally_closed, reconnect_notify, p2p_config, p2p_manager | Client |
| `ClaspBuilder` | url, name, features, token, reconnect, reconnect_interval_ms, p2p_config | Builder |
| `P2PManager` | session_id, config, connections, known_peers, event_callback, signal_tx, routing_mode, relay_fallback_peers, p2p_retry_interval_secs | P2P |
| `P2PConnection` | peer_session_id, correlation_id, state, transport, pending_candidates | Peer connection |

### Enums

| Enum | Variants | Purpose |
|------|----------|---------|
| `ClientError` | ConnectionFailed, NotConnected, AlreadyConnected, SendFailed, Timeout, Protocol, Transport, P2PNotConnected, Other | Errors |
| `P2PEvent` | PeerAnnounced, Connected, ConnectionFailed, Disconnected, Data | P2P events |

### Key Methods

| Struct | Method | Purpose |
|--------|--------|---------|
| `Clasp` | `builder(url)` | Create builder |
| `Clasp` | `connect_to(url)` | Quick connect |
| `Clasp` | `subscribe(pattern, callback)` | Subscribe |
| `Clasp` | `set(addr, value)` | Set param |
| `Clasp` | `get(addr)` | Get param |
| `Clasp` | `emit(addr, payload)` | Emit event |
| `Clasp` | `stream(addr, value)` | Stream data |
| `Clasp` | `gesture(addr, id, phase, payload)` | Gesture input |
| `Clasp` | `timeline(addr, data)` | Timeline |
| `Clasp` | `bundle(messages)` | Bundle |
| `Clasp` | `bundle_at(messages, time)` | Scheduled bundle |
| `Clasp` | `close()` | Close connection |
| `P2PManager` | `connect_to_peer(session)` | P2P connect |
| `P2PManager` | `send_to_peer(session, data, reliable)` | P2P send |

---

## clasp-discovery

### Structs

| Struct | Fields | Purpose |
|--------|--------|---------|
| `Device` | id, name, info, endpoints, discovered_at, last_seen | Discovered device |
| `DeviceInfo` | version, features, bridge, bridge_protocol, meta | Device metadata |
| `Discovery` | config, devices | Discovery manager |
| `DiscoveryConfig` | mdns, broadcast, broadcast_port, timeout | Config |
| `ServiceAdvertiser` | mdns, fullname | mDNS advertiser |
| `BroadcastResponder` | transport, name, features | UDP responder |
| `RendezvousServer` | config | HTTP server |
| `RendezvousClient` | base_url, client | HTTP client |
| `DeviceRegistration` | name, public_key, features, endpoints, tags, metadata | Registration |
| `RegisteredDevice` | id, name, ..., registered_at, last_seen | Registered |

### Enums

| Enum | Variants | Purpose |
|------|----------|---------|
| `DiscoveryEvent` | Found, Lost, Error | Events |
| `DiscoveryError` | Mdns, Broadcast, Network, Io, Other | Errors |

---

## clasp-embedded

### Structs (no_std)

| Struct | Fields | Purpose |
|--------|--------|---------|
| `Client` | state, cache, tx_buf, rx_buf | Embedded client |
| `StateCache` | entries, count | Fixed 32-entry cache |
| `MiniRouter` | sessions, tx_buf | Embedded server (4 clients) |
| `Session` | id, subscriptions | Client session |

### Enums

| Enum | Variants | Purpose |
|------|----------|---------|
| `Value` | Null, Bool, Int, Float | Value types |
| `ClientState` | Disconnected, Connecting, Connected | States |
| `Message` | Hello, Welcome, Subscribe, Unsubscribe, Set, Publish, Ping, Pong, Ack, Error | Messages |

### Functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| `encode_header` | `(buf, flags, len) -> usize` | Frame header |
| `encode_value` | `(buf, value) -> usize` | Value encoding |
| `encode_string` | `(buf, s) -> usize` | String encoding |
| `encode_set` | `(buf, addr, value) -> usize` | SET message |
| `encode_hello` | `(buf, name) -> usize` | HELLO message |
| `decode_header` | `(buf) -> Option<(u8, usize)>` | Parse header |
| `decode_value` | `(buf) -> Option<(Value, usize)>` | Parse value |
| `decode_message` | `(payload) -> Option<Message>` | Parse message |

---

## clasp-test-utils

### Structs

| Struct | Purpose |
|--------|---------|
| `TestRouter` | RAII test server wrapper |
| `ValueCollector` | Thread-safe subscription collector |

### Functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| `find_available_port` | `() -> u16` | Find free TCP port |
| `find_available_udp_port` | `() -> u16` | Find free UDP port |
| `wait_for` | `(check, interval, max) -> bool` | Condition wait |
| `wait_for_count` | `(counter, target, max) -> bool` | Counter wait |
| `wait_for_flag` | `(flag, max) -> bool` | Flag wait |
| `assert_approx_eq` | `(a, b, epsilon, msg) -> Result` | Float compare |
| `assert_that` | `(cond, msg) -> Result` | Assertion |
| `assert_some` | `(opt, msg) -> Result<T>` | Option assert |
| `assert_ok` | `(result, msg) -> Result<T>` | Result assert |

---

## JavaScript Bindings (@clasp-to/core)

### Classes

| Class | Methods |
|-------|---------|
| `Clasp` | `connect()`, `subscribe(pattern, cb)`, `on(pattern, cb)`, `set(addr, value)`, `get(addr)`, `emit(addr, payload)`, `stream(addr, value)`, `bundle(messages)`, `close()`, `onConnect(cb)`, `onDisconnect(cb)`, `onError(cb)` |
| `ClaspBuilder` | `name(n)`, `withName(n)`, `features(f)`, `withFeatures(f)`, `token(t)`, `withToken(t)`, `reconnect(b)`, `withReconnect(b, ms)`, `connect()` |

### Functions

| Function | Purpose |
|----------|---------|
| `encode(msg, qos?)` | Encode message to frame |
| `decode(frame)` | Decode frame to message |
| `encodeFrame(payload, opts?)` | Wrap payload |
| `decodeFrame(frame)` | Extract payload |
| `checkComplete(buffer)` | Check frame complete |

---

## Python Bindings (clasp-to)

### Classes

| Class | Methods |
|-------|---------|
| `Clasp` | `connect()`, `close()`, `subscribe(pattern, cb)`, `on(pattern)`, `set(addr, value)`, `get(addr)`, `emit(addr, payload)`, `stream(addr, value)`, `bundle(messages)`, `cached(addr)`, `time()`, `run()` |
| `ClaspBuilder` | `with_name(n)`, `with_features(f)`, `with_token(t)`, `with_reconnect(b, s)`, `connect()` |

### Dataclasses

| Dataclass | Fields |
|-----------|--------|
| `HelloMessage` | version, name, features, capabilities, token |
| `WelcomeMessage` | version, session, name, features, time, token |
| `SetMessage` | address, value, revision, lock, unlock |
| `PublishMessage` | address, signal, value, payload, timestamp |
| `SubscribeMessage` | id, pattern, types, options |
| `ParamValue` | address, value, revision, writer, timestamp |
| `SnapshotMessage` | params |
| `BundleMessage` | messages, timestamp |
| `ErrorMessage` | code, message, address, correlation_id |

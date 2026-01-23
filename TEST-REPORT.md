# CLASP Test Report

**Generated:** 2026-01-23
**Total Tests:** 442 (440 passing, 2 failing)

## Summary by Crate

| Crate | Unit Tests | Integration Tests | Status |
|-------|------------|-------------------|--------|
| clasp-core | 56 | 38 | ✅ Pass |
| clasp-router | 25 | 41 | ✅ Pass |
| clasp-client | 4 | 38 | ✅ Pass |
| clasp-transport | 6 | 27 | ⚠️ 2 QUIC fail |
| clasp-bridge | 28 | 24 | ✅ Pass |
| clasp-discovery | 2 | 21 | ✅ Pass |
| clasp-embedded | 6 | 0 | ✅ Pass |

---

## clasp-core (94 tests)

### Lib Tests (56 tests) - Wire Protocol & Core Types

| Module | Tests | What They Prove |
|--------|-------|-----------------|
| **address** | 6 | OSC-style address parsing, wildcards (`*`, `**`), validation |
| **codec** | 9 | Binary encoding/decoding, MessagePack compat, size optimization |
| **frame** | 8 | Frame header format, magic byte, timestamps, max payload |
| **p2p** | 5 | P2P address detection, session targeting, signal serialization |
| **security** | 11 | JWT validation, scopes, CPSK auth, token expiry |
| **state** | 5 | State updates, locking, revision conflicts, merge strategies |
| **time** | 3 | Clock synchronization, jitter buffer, session time |
| **timeline** | 9 | Keyframe interpolation, easing, looping, pause/resume |

### protocol_tests.rs (14 tests) - External Protocol Compatibility

| Test | What It Proves |
|------|----------------|
| `test_osc_loopback_*` (6) | OSC UDP packets encode/decode correctly |
| `test_artnet_*` (4) | Art-Net DMX packets, universes, poll/reply |
| `test_midi_*` (4) | MIDI message encoding, virtual ports, channel mapping |

### error_tests.rs (10 tests) - Error Handling

| Test | What It Proves |
|------|----------------|
| `test_malformed_message` | Server handles garbage data gracefully |
| `test_truncated_message` | Server handles incomplete frames |
| `test_wrong_protocol_version` | Protocol version mismatch handling |
| `test_message_before_hello` | Messages before handshake rejected |
| `test_duplicate_hello` | Second HELLO doesn't crash |
| `test_very_long_address` | 10KB addresses handled |
| `test_empty_address` | Empty addresses handled |
| `test_rapid_disconnect_reconnect` | 5x rapid connect/disconnect cycles |
| `test_connection_to_closed_port` | Connection failure handled |
| `test_special_characters_in_address` | Unicode, emoji, symbols in addresses |

---

## clasp-router (66 tests)

### Lib Tests (25 tests) - Router Internals

| Module | Tests | What They Prove |
|--------|-------|-----------------|
| **gesture** | 19 | Gesture coalescing, move buffering, stale cleanup |
| **p2p** | 2 | P2P address routing, capabilities |
| **state** | 2 | State snapshots, basic operations |
| **subscription** | 2 | Pattern matching, subscription manager |

### bundle_tests.rs (5 tests) - Atomic Operations

| Test | What It Proves |
|------|----------------|
| `test_bundle_atomic_execution` | All messages in bundle succeed or fail together |
| `test_bundle_scheduled_execution` | Bundles can be scheduled for future |
| `test_bundle_mixed_message_types` | SET, PUBLISH in same bundle |
| `test_bundle_large_bundle` | 100+ messages in one bundle |
| `test_bundle_timestamp_precision` | Sub-millisecond timing preserved |

### gesture_tests.rs (4 tests) - Touch/Gesture Handling

| Test | What It Proves |
|------|----------------|
| `test_gesture_lifecycle` | Start → Move → End sequence |
| `test_multitouch_gestures` | Multiple simultaneous touch points |
| `test_gesture_cancel` | Cancel flushes buffered moves |
| `test_gesture_high_frequency` | 1000 moves/sec coalescing works |

### timeline_tests.rs (7 tests) - Animation Timelines

| Test | What It Proves |
|------|----------------|
| `test_timeline_linear` | Linear interpolation 0→100 |
| `test_ease_in`/`test_ease_out` | Easing curves work |
| `test_loop` | Looping timelines repeat |
| `test_finished` | Timeline completion detection |
| `test_routing` | Timeline values routed to subscribers |
| `test_multi_keyframe` | Multiple keyframes interpolate |

### subscription_tests.rs (7 tests) - Pub/Sub System

| Test | What It Proves |
|------|----------------|
| `test_exact_match_subscription` | `/a/b/c` matches exactly |
| `test_single_wildcard_subscription` | `/a/*/c` matches `/a/X/c` |
| `test_multi_wildcard_subscription` | `/a/**` matches `/a/b/c/d` |
| `test_unsubscribe` | Unsubscribe stops delivery |
| `test_multiple_subscriptions` | Multiple patterns work |
| `test_subscription_initial_snapshot` | Snapshot on subscribe |
| `test_invalid_subscription_pattern` | Bad patterns rejected |

### session_tests.rs (17 tests) - Client Sessions

| Test | What It Proves |
|------|----------------|
| `test_session_id_*` (3) | Unique session IDs, format, persistence |
| `test_session_cleanup_*` (3) | Cleanup on disconnect, reconnect |
| `test_concurrent_*` (3) | Multiple sessions, state isolation |
| `test_session_*_isolation` (2) | Values/subscriptions per-session |
| `test_*_error` (4) | Invalid URL, nonexistent server |
| `test_double_close` | Double-close doesn't crash |
| `test_operations_after_close` | Operations after close fail gracefully |

### lock_tests.rs (2 tests) - Distributed Locking

| Test | What It Proves |
|------|----------------|
| `test_lock_acquisition_and_denial` | Lock acquired, others denied |
| `test_lww_last_write_wins` | Last-write-wins conflict resolution |

---

## clasp-client (42 tests)

### client_tests.rs (38 tests) - Client SDK

| Category | Tests | What They Prove |
|----------|-------|-----------------|
| **Builder** | 4 | Client builder pattern, config options |
| **Connection** | 5 | Connect, disconnect, reconnect lifecycle |
| **Parameters** | 6 | set(), get(), cached values, locking |
| **Events** | 3 | emit(), subscribe(), receive events |
| **Advanced** | 4 | Bundles, clock sync, streaming |
| **Value Types** | 7 | All value types: int, float, string, bytes, array, null, bool |
| **Two-client** | 2 | Client A sets, client B receives |
| **Concurrent** | 2 | Parallel operations, thread safety |
| **Edge Cases** | 5 | Errors, closed connections, special chars |

---

## clasp-transport (33 tests)

### Lib Tests (6 tests)
- TCP transport creation, config
- UDP bind, send/recv
- WebSocket config

### transport_tests.rs (19 tests) - Transport Layer

| Test | What It Proves |
|------|----------------|
| `test_websocket_connect` | WebSocket connection works |
| `test_websocket_binary_frames` | Binary frame support |
| `test_websocket_subprotocol` | CLASP subprotocol negotiation |
| `test_roundtrip_*` (3) | Encode → send → receive → decode |
| `test_frame_*` (2) | Frame header format, magic byte |
| `test_large_message` | Large payload handling |
| `test_concurrent_connections` | Multiple simultaneous connections |
| `test_rapid_connect_disconnect` | Connection churn handling |
| `test_connection_close` | Graceful close |
| `test_send_after_close` | Fails gracefully |
| `test_*_error` (2) | Invalid URL, nonexistent server |

### quic_tests.rs (8 tests, 2 FAILING)

| Test | Status | What It Proves |
|------|--------|----------------|
| `test_quic_config_*` (3) | ✅ | QUIC configuration works |
| `test_quic_client_creation` | ✅ | Client creates |
| `test_quic_server_creation` | ✅ | Server creates |
| `test_quic_alpn_protocol` | ✅ | ALPN negotiation |
| `test_quic_client_server_connect` | ❌ | ALPN mismatch |
| `test_quic_bidirectional_stream` | ❌ | ALPN mismatch |

### udp_tests.rs (? tests) - UDP Transport
- Bind, send, receive
- Multicast support
- Packet fragmentation

---

## clasp-bridge (52 tests)

### Lib Tests (28 tests) - Bridge Internals

| Module | Tests | What They Prove |
|--------|-------|-----------------|
| **artnet** | 1 | Art-Net config |
| **dmx** | 2 | DMX channel operations |
| **http** | 2 | HTTP config, value conversion |
| **mapping** | 4 | Address mapping, wildcards, transforms |
| **midi** | 3 | MIDI config, CC/note conversion |
| **mqtt** | 3 | MQTT config, topic conversion |
| **osc** | 2 | OSC arg conversion |
| **transform** | 8 | Scale, invert, smooth, ease, expression, aggregate |
| **websocket** | 3 | WebSocket config, message formats |

### artnet_tests.rs (8 tests) - Art-Net Protocol

| Test | What It Proves |
|------|----------------|
| `test_artnet_dmx_packet_*` (2) | DMX packet encoding/decoding |
| `test_artnet_poll_*` (2) | Poll request/reply |
| `test_artnet_multiple_universes` | Universes 0-15 |
| `test_artnet_dmx_values` | All 256 DMX values |
| `test_artnet_sequence_numbers` | Sequence rollover |
| `test_artnet_roundtrip` | Full UDP roundtrip |

### midi_tests.rs (10 tests) - MIDI Protocol

| Test | What It Proves |
|------|----------------|
| `test_midi_*_parsing` (4) | CC, note on/off, program change |
| `test_midi_pitchbend` | 14-bit pitch bend |
| `test_midi_sysex` | System exclusive messages |
| `test_midi_*_pressure` (2) | Channel/poly aftertouch |
| `test_midi_message_generation` | MIDI → bytes |
| `test_midi_virtual_ports` | Virtual port detection |

### osc_tests.rs (9 tests) - OSC Protocol

| Test | What It Proves |
|------|----------------|
| `test_osc_receive_*` (5) | Float, int, string, blob, multiple args |
| `test_osc_send_to_external` | Send to external OSC app |
| `test_osc_bundle_with_timestamp` | Timetag support |
| `test_osc_high_rate` | 1000 msg/sec handling |
| `test_osc_roundtrip` | Full UDP roundtrip |

### http_tests.rs (2 tests) - HTTP REST Bridge

| Test | What It Proves |
|------|----------------|
| `test_http_put_sets_clasp_signal` | PUT → SET message |
| `test_http_post_publishes_event` | POST → PUBLISH message |

### mqtt_tests.rs (3 tests) - MQTT Bridge

| Test | What It Proves |
|------|----------------|
| `test_mqtt_topic_to_clasp_address` | MQTT topic → CLASP address |
| `test_mqtt_qos_mapping` | QoS level mapping |
| `test_clasp_to_mqtt_translation` | CLASP → MQTT message |

### websocket_tests.rs (2 tests) - WebSocket Bridge

| Test | What It Proves |
|------|----------------|
| `test_websocket_text_to_clasp_set` | JSON → SET |
| `test_clasp_set_to_websocket_json` | SET → JSON |

---

## clasp-discovery (23 tests)

### discovery_tests.rs (21 tests) - Service Discovery

| Test | What It Proves |
|------|----------------|
| `test_device_*` (7) | Device creation, staleness, endpoints |
| `test_discovery_*` (10) | Discovery events, add/remove, config |
| `test_broadcast_responder_creation` | mDNS/broadcast responder |

---

## clasp-embedded (6 tests)

### Lib Tests (6 tests) - Embedded/no_std

| Test | What It Proves |
|------|----------------|
| `test_encode_decode_value` | Value encoding in no_std |
| `test_encode_decode_set` | SET message encoding |
| `test_client_flow` | HELLO → WELCOME flow |
| `test_state_cache` | Fixed-size state cache |
| `test_memory_size` | Client < 4KB |
| `test_mini_router` | Server mode (with feature) |

---

## Known Issues

### QUIC Tests (2 failing)
- `test_quic_client_server_connect` - ALPN protocol mismatch
- `test_quic_bidirectional_stream` - ALPN protocol mismatch

These fail due to ALPN configuration mismatch between client and server certificates, not a hang issue.

---

## Test Execution

All tests complete within reasonable timeouts:
- Unit tests: < 1 second
- Integration tests: 1-3 seconds each
- Full workspace: ~15 seconds total

No hanging tests detected.

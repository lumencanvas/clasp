# Testing Infrastructure Handoff

**Date:** 2026-01-26
**Version:** v3.3.0
**Session Focus:** P2P API extensions, data transfer, routing modes, and release

## What Was Accomplished in v3.3.0

### 1. P2P Client API Extensions (`crates/clasp-client/src/client.rs`)
- **`send_p2p(peer_session_id, data, reliable)`** - Send data directly to peers via WebRTC
- **`set_p2p_routing_mode(mode)`** - Control routing behavior
- **`p2p_routing_mode()`** - Get current routing mode
- **Re-exports in `lib.rs`:** `SendResult`, `RoutingMode`, `P2PEvent`

### 2. P2P Data Reception Fix (`crates/clasp-transport/src/webrtc.rs`)
- Added `on_data` callback to `WebRtcTransport`
- Wired callback in `P2PManager` for both offerer and answerer paths
- Data now flows end-to-end: sender → WebRTC DataChannel → receiver → `P2PEvent::Data`

### 3. Connection Timeout (`crates/clasp-client/src/p2p.rs`)
- Added timeout task that monitors pending connections
- Emits `P2PEvent::ConnectionFailed` after configured timeout (default: 30s)
- Configurable via `P2PConfig::connection_timeout_secs`

### 4. New P2P Tests (`clasp-e2e/src/bin/p2p-connection-tests.rs`)
- **Test 6:** `test_p2p_data_transfer()` - Verifies data flows over P2P channel
- **Test 7:** `test_p2p_routing_mode()` - Verifies routing mode affects send path
- **Test 8:** `test_p2p_nonexistent_peer()` - Verifies connection timeout handling

### 5. Documentation Updates
- `crates/clasp-client/README.md` - Added P2P example with new API
- `docs/api/common/p2p.md` - Documented new methods and events
- `docs/testing.md` - Updated test commands for clasp-e2e

### 6. Release v3.3.0
- All Rust crates published to crates.io
- `@clasp-to/core` published to npm
- `clasp-to` published to PyPI
- GitHub release created with tag v3.3.0

## Current State

### Versions
| Component | Version |
|-----------|---------|
| Workspace (Cargo.toml) | 3.3.0 |
| @clasp-to/core (npm) | 3.3.0 |
| clasp-to (PyPI) | 3.3.0 |
| site dependency | ^3.3.0 |
| deploy/relay deps | 3.3 |

### Test Coverage Summary

**Well Covered:**
- ✅ Core protocol & framing (`clasp-core/tests/*`)
- ✅ HTTP/MQTT/WebSocket bridge integration (`clasp-e2e/src/bin/*_integration_tests.rs`)
- ✅ Bundle messages (`bundle_tests.rs`)
- ✅ Lock & basic conflict resolution (`lock_tests.rs`)
- ✅ Security model (JWT/CPSK tokens, scopes) (`security_tests.rs`, `security_pentest.rs`)
- ✅ QUIC transport with TLS (`quic_tests.rs`)
- ✅ **P2P connection, data transfer, routing modes** (`p2p-connection-tests.rs`) - NEW in v3.3.0

**Remaining Gaps (Medium Priority):**
- ⚠️ Token replay attack scenarios
- ⚠️ TCP large message handling (>64KB)
- ⚠️ TLS-encrypted WebSocket (`wss://`) tests
- ⚠️ Multi-peer P2P mesh tests (4+ peers)

**Low Priority / Future:**
- Serial/BLE transport integration tests
- Network degradation simulation (packet loss, latency)
- NAT traversal edge cases (requires specific network topology)

## How to Run Tests

```bash
# All workspace tests (fast)
cargo test --workspace

# P2P tests specifically
cargo run -p clasp-e2e --bin p2p-connection-tests --features p2p

# Full E2E suite
cargo run -p clasp-e2e --bin run-all-tests

# Python binding tests
cd bindings/python && pytest
```

## Files Modified in v3.3.0

| File | Change |
|------|--------|
| `crates/clasp-client/src/client.rs` | Added `send_p2p`, `set_p2p_routing_mode`, `p2p_routing_mode` |
| `crates/clasp-client/src/lib.rs` | Re-exported `SendResult`, `RoutingMode` |
| `crates/clasp-client/src/p2p.rs` | Wired data callbacks, added connection timeout |
| `crates/clasp-transport/src/webrtc.rs` | Added `on_data` callback and `DataCallback` type |
| `clasp-e2e/src/bin/p2p_connection_tests.rs` | Added Tests 6, 7, 8 |
| `crates/clasp-client/README.md` | Added P2P example |
| `docs/api/common/p2p.md` | Documented new API |
| `docs/testing.md` | Updated crate references |
| All `Cargo.toml` / `package.json` / `pyproject.toml` | Version bump to 3.3.0 |

## Next Steps (Priority Order)

### Medium Priority
1. **Security: Token replay tests** (`security_pentest.rs`):
   - Attempt to reuse an expired/revoked token
   - Verify router rejects it

2. **Transports: TCP large message test** (`transport_tests.rs`):
   - Send message >64KB over TCP
   - Verify frame boundary detection still works

3. **Transports: TLS WebSocket test**:
   - If `wss://` support exists, test TLS handshake with self-signed cert
   - Verify CPSK auth still works over encrypted transport

### Low Priority / Future
4. Multi-peer P2P mesh tests (4+ peers, verify all can communicate)
5. Serial/BLE transport integration tests (if those transports are implemented)
6. Network degradation simulation (packet loss, latency injection)

## Environment-Sensitive Tests

Some test scenarios require specific network conditions:
- NAT traversal edge cases
- TURN server failover
- Network degradation

**Recommendation:** Keep these as optional/manual tests:
```rust
if std::env::var("CLASP_P2P_NETWORK_TESTS").is_ok() { ... }
```

---

**Status:** ✅ v3.3.0 RELEASED - All packages published, all tests passing

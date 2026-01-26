# Testing Infrastructure Handoff

**Date:** 2026-01-27  
**Session Focus:** Test structure documentation + P2P test extensions

## What Was Accomplished

### 1. Test Structure Documentation (`docs/testing.md`)
- **Created comprehensive testing overview** that codifies the 3-layer test architecture:
  - **Layer 1:** Per-crate unit tests (`crates/*/src/...` with `#[cfg(test)]` modules, `crates/*/tests/*.rs`)
  - **Layer 2:** Workspace-level smoke tests (`tests/integration/*.rs` - minimal integration coverage)
  - **Layer 3:** System/scenario tests (`test-suite/` crate - comprehensive integration, load, performance)
- **Documented exact commands** for running each category:
  - `cargo test --workspace` (fast unit + small integration)
  - `cargo run -p clasp-test-suite --bin <test-name>` (individual system tests)
  - `cargo run -p clasp-test-suite --bin run-all-tests` (full suite)
  - Python binding tests: `pytest bindings/python/tests`
- **Established clear guidelines** for where new tests should live:
  - Single-crate behavior → inline `#[cfg(test)]` or `crates/<name>/tests/*.rs`
  - One-protocol ↔ CLASP sanity → `tests/integration/*.rs` (smoke) or `test-suite/src/tests/*` (if consolidating)
  - Cross-crate/system scenarios → `test-suite/src/bin/*` (extend existing or add new bins)

### 2. P2P Connection Tests (`test-suite/src/bin/p2p_connection_tests.rs`)
- **Extended Test 1** to verify bidirectional connection state:
  - After `P2PEvent::Connected`, now checks `is_peer_connected()` from both sides
  - Validates that both peers see each other as connected (not just one-way)
  - Still treats successful handshake as pass, but adds diagnostic output

## Current State

### Test Coverage Summary (from previous session analysis)

**Well Covered:**
- ✅ Core protocol & framing (`clasp-core/tests/*`, `test-suite/src/bin/*frame*`)
- ✅ HTTP/MQTT/WebSocket bridge integration (`http_integration_tests.rs`, `mqtt_integration_tests.rs`, `websocket_bridge_tests.rs`)
- ✅ Bundle messages (`bundle_tests.rs`)
- ✅ Lock & basic conflict resolution (`lock_tests.rs`)
- ✅ Security model (JWT/CPSK tokens, scopes) (`security_tests.rs`, `security_pentest.rs`)
- ✅ QUIC transport with TLS (`quic_tests.rs`)
- ✅ Basic P2P connection establishment (`p2p_connection_tests.rs`)

**Partially Covered / Needs Expansion:**
- ⚠️ **P2P WebRTC:** Basic handshake works, but missing:
  - Actual data transfer over P2P channels (reliable/unreliable)
  - Routing mode toggling (`PreferP2P` vs `RouterOnly`)
  - Failure paths (nonexistent peer, timeout handling)
  - Multi-peer scenarios beyond 3 clients
- ⚠️ **Security:** Token/auth flows covered, but missing:
  - TLS/DTLS encryption flows (beyond QUIC's self-signed cert tests)
  - Token replay attack scenarios
  - More nuanced rate-limit enforcement tests
- ⚠️ **Transports:** WebSocket/QUIC covered, but missing:
  - TCP large message handling
  - TCP concurrent client stress tests
  - TLS-encrypted WebSocket (`wss://`) tests
  - Serial/BLE transport integration tests (marked as MEDIUM/LOW priority)

## Blockers / Decisions Needed

### P2P Data Flow Tests
**Status: ✅ RESOLVED in v3.3.0**

The `clasp-client` public API now exposes:
- `Clasp::send_p2p(peer_session_id, data, reliable)` - Send data to peer via P2P
- `Clasp::set_p2p_routing_mode(mode)` - Control routing behavior
- `Clasp::p2p_routing_mode()` - Get current routing mode
- Re-exports: `SendResult`, `RoutingMode` types

Tests added in `clasp-e2e/src/bin/p2p_connection_tests.rs`:
- Test 6: `test_p2p_data_transfer()` - Verifies data flows over P2P channel ✅
- Test 7: `test_p2p_routing_mode()` - Verifies routing mode affects send path ✅
- Test 8: `test_p2p_nonexistent_peer()` - Verifies connection timeout handling ✅

Additional fixes:
- P2P data reception now works (wired `on_data` callback in WebRtcTransport)
- Connection timeout emits `P2PEvent::ConnectionFailed` after configured timeout

### Environment-Sensitive Tests
**Issue:** Some test scenarios (NAT traversal, TURN server variants, network degradation) require:
- Public STUN/TURN server access
- Specific network topologies
- May be flaky in CI

**Recommendation:** Keep these as **optional/manual tests**:
- Wrap in env var check: `if std::env::var("CLASP_P2P_NETWORK_TESTS").is_ok() { ... }`
- Document in `docs/testing.md` that these require internet access and should be run manually
- Don't block CI on them

## Next Steps (Priority Order)

### Immediate (Next Session) - ✅ COMPLETED in v3.3.0
1. **Extend `clasp-client` API for P2P data flow:** ✅
   - Added `send_p2p()` method
   - Added `set_p2p_routing_mode()` method
   - Added `p2p_routing_mode()` getter
   - Re-exported `SendResult`, `RoutingMode` types

2. **Add P2P data flow test:** ✅
   - Added `test_p2p_data_transfer()` in `clasp-e2e/src/bin/p2p_connection_tests.rs`
   - Establishes P2P connection, sends payload, verifies receipt via `P2PEvent::Data`

3. **Add routing mode test:** ✅
   - Added `test_p2p_routing_mode()` in `clasp-e2e/src/bin/p2p_connection_tests.rs`
   - Verifies all routing modes work: `PreferP2P`, `P2POnly`, `ServerOnly`
   - Verifies `SendResult::P2P` vs `SendResult::Relay` based on mode

4. **Add failure path test:** ✅
   - Added `test_p2p_nonexistent_peer()` in `clasp-e2e/src/bin/p2p_connection_tests.rs`
   - Verifies connection timeout emits `P2PEvent::ConnectionFailed`

### Medium Priority
5. **Security: Token replay tests** (`security_pentest.rs`):
   - Attempt to reuse an expired/revoked token
   - Verify router rejects it

6. **Transports: TCP large message test** (`transport_tests.rs`):
   - Send message >64KB over TCP
   - Verify frame boundary detection still works

7. **Transports: TLS WebSocket test** (new `encryption_tests.rs` or extend `transport_tests.rs`):
   - If `wss://` support exists in `clasp-transport`, test TLS handshake with self-signed cert
   - Verify CPSK auth still works over encrypted transport

### Low Priority / Future
8. **Serial/BLE transport integration tests** (if those transports are implemented)
9. **Multi-peer P2P mesh tests** (4+ peers, verify all can communicate)
10. **Network degradation simulation** (packet loss, latency injection) - requires test infrastructure

## Files Modified This Session

- `docs/testing.md` (NEW) - Comprehensive testing overview and guidelines
- `test-suite/src/bin/p2p_connection_tests.rs` (MODIFIED) - Extended Test 1 with bidirectional connection verification

## Key Insights

1. **Test structure is now well-documented** - new contributors can easily find where tests live and how to run them
2. **P2P tests are at a good baseline** - handshake works, but need API extensions to test actual data flow
3. **Security tests are comprehensive** - JWT/CPSK/auth flows are well covered; encryption flows could be expanded
4. **Most gaps are "nice-to-have"** - core functionality is well tested; remaining items are edge cases or advanced scenarios

## Questions for Next Session

1. Should we extend `clasp-client` API for P2P data flow, or test via internals?
2. Do we want to add TLS WebSocket tests now, or wait until `wss://` is more fully implemented?
3. Should Serial/BLE transport tests be added even if those transports are still experimental?

---

**Status:** ✅ COMPLETE - P2P API extended, all tests pass (v3.3.0)

## Testing Overview

CLASP has three main layers of tests. This document explains what each layer is for and how to run them.

---

## 1. Per-crate unit and small integration tests

- **Location**: `crates/*/src` (with `#[cfg(test)]` modules) and `crates/*/tests/*.rs`
- **Purpose**: Fast feedback on individual crates (codec, frame, router, transport, bridge adapters, etc.)
- **Examples**:
  - `crates/clasp-core/tests/frame_tests.rs`
  - `crates/clasp-core/tests/state_tests.rs`
  - `crates/clasp-router/tests/router_tests.rs`
  - Inline `#[cfg(test)]` modules in core/transport/bridge code

**How to run:**

- Whole workspace (fast checks):

```bash
cargo test --workspace
```

- Single crate:

```bash
cargo test -p clasp-core
cargo test -p clasp-router
cargo test -p clasp-transport
```

**When to add tests here:**

- A behavior is local to a single crate and can be exercised via that crate’s public API.
- The test does not need external services (MQTT broker, HTTP server, WebRTC stack, etc.).
- You want tests to run quickly on every PR.

---

## 2. Workspace-level integration tests (Rust)

- **Location**: `tests/integration/*.rs`
- **Purpose**: Lightweight end-to-end checks that exercise real protocol code without the full test-suite harness.
- **Current examples**:
  - `tests/integration/osc_echo_test.rs`
  - `tests/integration/midi_echo_test.rs`
  - `tests/integration/artnet_dmx_test.rs`

These tests are run as part of the normal Rust test command and should stay:

- Deterministic (no flaky timing assumptions).
- Self-contained (no long-running external infrastructure).
- Reasonably fast.

**How to run:**

```bash
cargo test --tests
# or specifically
cargo test --test osc_echo_test
```

**When to add tests here:**

- You want a **smoke-level** check that a given protocol or bridge still works end-to-end.
- The scenario is small enough that it should run on every PR with `cargo test --workspace`.
- You don’t need the richer orchestration/reporting from the test-suite crate.

Over time, deeper protocol scenarios should live in the test-suite (see below), and this folder should mostly remain a small collection of critical smoke tests.

---

## 3. System / scenario / load tests (clasp-test-suite)

- **Location**: `test-suite/`
- **Crate**: `clasp-test-suite`
- **Structure**:
  - Library + custom runner:
    - `test-suite/src/lib.rs` — `TestSuite` and `TestResult` types.
    - `test-suite/src/tests/*.rs` — grouped tests (OSC, MIDI, Art-Net, CLASP-to-CLASP, security, load, helpers).
    - `test-suite/src/main.rs` — `run-all-tests` orchestrator.
  - Binaries (`test-suite/src/bin/*.rs`) — focused suites and scenarios:
    - Bridges/protocols: `http_integration_tests.rs`, `mqtt_integration_tests.rs`, `websocket_bridge_tests.rs`, `osc_integration.rs`, `midi_integration.rs`, `artnet_integration.rs`, etc.
    - Transports: `transport_tests.rs`, `udp_tests.rs`, `quic_tests.rs`, `relay_e2e.rs`, `p2p_connection_tests.rs`.
    - System behavior: `bundle_tests.rs`, `lock_tests.rs`, `timeline_tests.rs`, `session_tests.rs`, `subscription_tests.rs`, `e2e_protocol_tests.rs`, `network_tests.rs`.
    - Performance / soak / security: `load_tests.rs`, `soak_tests.rs`, `real_benchmarks.rs`, `security_tests.rs`, `security_pentest.rs`, `proof_tests.rs`, `hardware_tests.rs`.
  - Benchmarks:
    - `test-suite/benches/throughput.rs`

**How to run:**

- **Run the aggregated suite:**

```bash
cargo run -p clasp-test-suite --bin run-all-tests
```

- **Run a single focused suite:**

```bash
# Bridges
cargo run -p clasp-test-suite --bin http_integration_tests
cargo run -p clasp-test-suite --bin mqtt_integration_tests
cargo run -p clasp-test-suite --bin websocket_bridge_tests

# Transports
cargo run -p clasp-test-suite --bin transport_tests
cargo run -p clasp-test-suite --bin udp_tests
cargo run -p clasp-test-suite --bin quic_tests
cargo run -p clasp-test-suite --bin p2p-connection-tests

# System behavior
cargo run -p clasp-test-suite --bin bundle_tests
cargo run -p clasp-test-suite --bin lock_tests
cargo run -p clasp-test-suite --bin e2e-protocol-tests

# Security / load
cargo run -p clasp-test-suite --bin security-tests
cargo run -p clasp-test-suite --bin security-pentest
cargo run -p clasp-test-suite --bin load-tests
```

- **Run benchmarks:**

```bash
cargo bench -p clasp-test-suite
```

**When to add tests here:**

- The test involves **multiple crates** (router + client + bridge + transport, etc.).
- The test needs **real external protocols** (MQTT brokers, HTTP, WebSocket, WebRTC, QUIC, MIDI/OSC/Art-Net devices or libraries).
- You’re testing **performance, resilience, or long-running behavior**.
- You need richer orchestration or custom reporting than standard `#[test]` provides.

---

## 4. Non-Rust binding tests

- **Python**:
  - Location: `bindings/python/tests/`
  - Run:

```bash
cd bindings/python
pip install -e .
pytest
```

- **JavaScript / TypeScript**:
  - (If/when present) tests should live under `bindings/js/` or in the relevant package (for example `bindings/js/packages/clasp-core/__tests__`).
  - Typical commands (to be confirmed in each package):

```bash
cd bindings/js
npm test
```

These tests verify that language bindings faithfully implement the CLASP protocol semantics exposed by the Rust core.

---

## 5. Guidelines for adding new tests

Use this as a quick rule-of-thumb:

- **Add to a crate’s own tests (`crates/*/src` or `crates/*/tests`) if:**
  - Only one crate’s API is involved.
  - No external services are required.
  - The test should run on every PR as part of `cargo test --workspace`.

- **Add to `tests/integration/` if:**
  - You need a **small, high-value smoke test** that touches multiple crates or a real bridge.
  - It must remain fast and deterministic for CI.
  - You don’t need the full test-suite harness.

- **Add to `test-suite/` if:**
  - The scenario is cross-cutting (multiple crates + external protocols/transports).
  - You’re testing performance, resilience, security, or complex timing.
  - The test is long-running, uses Docker / external tools, or is environment-dependent.

When in doubt, prefer:

1. **Unit tests in crates** for logic.
2. **`tests/integration/`** for quick end-to-end smoke.
3. **`test-suite/`** for everything heavier or more scenario-driven.


# CLASP Testing Strategy

## Test Organization

### Test Levels

```
┌─────────────────────────────────────────────────────────────┐
│                    E2E TESTS (clasp-e2e)                    │
│  40 test binaries, compliance suites, stress tests          │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   INTEGRATION TESTS                          │
│  Cross-crate functionality, protocol bridges                 │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                      UNIT TESTS                              │
│  Per-module tests in crates/*/tests/                         │
└─────────────────────────────────────────────────────────────┘
```

### Test Categories

| Category | Location | Purpose |
|----------|----------|---------|
| Unit | `crates/*/tests/*.rs` | Per-module functionality |
| Integration | `tests/integration/*.rs` | Cross-crate behavior |
| E2E Protocol | `clasp-e2e/src/bin/*.rs` | Full protocol validation |
| Compliance | `clasp-e2e/src/compliance/*.rs` | Spec conformance |
| Load | `clasp-e2e/src/bin/load_tests.rs` | Throughput testing |
| Stress | `clasp-e2e/src/bin/*_stress*.rs` | Resilience testing |
| Benchmarks | `crates/*/benches/*.rs` | Performance measurement |

## Unit Test Coverage

### clasp-core Tests

| Module | Test File | Coverage |
|--------|-----------|----------|
| address | `address_tests.rs` | Parsing, wildcards, matching |
| codec | `codec_tests.rs` | Encode/decode all types |
| error | `error_tests.rs` | Error types, codes |
| frame | `frame_tests.rs` | Frame format, flags |
| protocol | `protocol_tests.rs` | Message types, QoS |
| state | `state_tests.rs` | ParamState, conflicts |
| time | `time_tests.rs` | Clock sync, jitter |
| timeline | (inline) | Player, interpolation |
| security | (inline) | Scopes, tokens |
| p2p | (inline) | Signals, addresses |

**Total: ~62 unit tests, ~196 integration tests**

### clasp-router Tests

| Test File | Coverage |
|-----------|----------|
| `session_tests.rs` | Session lifecycle, auth |
| `subscription_tests.rs` | Pattern matching |
| `router_tests.rs` | Message routing |
| `gesture_tests.rs` | Move coalescing |
| `bundle_tests.rs` | Atomic operations |
| `timeline_tests.rs` | Timeline playback |
| `lock_tests.rs` | Parameter locking |

### clasp-transport Tests

| Test File | Coverage |
|-----------|----------|
| `transport_tests.rs` | All transport types |
| `udp_tests.rs` | UDP specifics |
| `quic_tests.rs` | QUIC specifics |

## E2E Test Suite (clasp-e2e)

### Test Binaries (40 total)

#### Core Protocol Tests
1. `e2e_protocol_tests.rs` - Multi-protocol E2E
2. `clasp_to_clasp.rs` - Inter-device communication
3. `relay_e2e.rs` - Relay server functionality
4. `public_relay_tests.rs` - Public relay testing
5. `p2p_connection_tests.rs` - WebRTC P2P

#### Bridge Tests
6. `broker_tests.rs` - Message broker
7. `bridge_tests.rs` - Protocol bridges

#### Security Tests
8. `security_tests.rs` - Auth, tokens, scopes
9. `security_pentest.rs` - Penetration testing

#### Load & Stress Tests
10. `load_tests.rs` - Throughput testing
11. `soak_tests.rs` - Long-running stability
12. `chaos_tests.rs` - Failure injection
13. `relay_stress_tests.rs` - Production stress
14. `network_simulation_tests.rs` - Network impairment

#### Benchmarks
15. `real_benchmarks.rs` - E2E latency
16. `latency_benchmarks.rs` - Percentile analysis
17. `gesture_coalescing_benchmarks.rs` - Touch input
18. `clock_sync_benchmark.rs` - Time sync
19. `resilience_benchmark.rs` - Recovery
20. `cold_start_benchmarks.rs` - Startup time
21. `sustained_load_benchmarks.rs` - 30s load
22. `memory_benchmarks.rs` - Memory usage
23. `rendezvous_benchmarks.rs` - Discovery

#### Hardware/Embedded
24. `embedded_tests.rs` - Embedded router
25. `hardware_tests.rs` - Hardware-specific

#### Protocol Comparison
26. `protocol_comparison.rs` - CLASP vs others
27. `conformance_report.rs` - Full conformance

#### Debug Tools
28. `debug_benchmark.rs` - Ad-hoc benchmarking
29. `debug_subscription.rs` - Subscription debugging
30. `debug_snapshot.rs` - State debugging
31. `debug_late_joiner.rs` - Late-joiner testing
32. `verify_patterns.rs` - Pattern verification
33. `proof_tests.rs` - Protocol proofs
34-40. Various bug reproduction tests

### Compliance Modules

| Module | Tests | Spec Reference |
|--------|-------|----------------|
| `handshake.rs` | HELLO/WELCOME exchange | CLASP 4.1 |
| `messages.rs` | All 12 message types | CLASP 4.x |
| `subscription.rs` | Wildcards, snapshots | CLASP 4.3 |
| `state.rs` | LWW, locks, revisions | CLASP 5.x |
| `security.rs` | Tokens, scopes | CLASP 6.x |
| `encoding.rs` | Binary frame format | CLASP 3.x |

## Performance Targets

### Encoding/Decoding
```
Target: 50,000+ messages/second

Test: load_tests::test_encoding_throughput
  - 10,000 messages
  - Measure: msg/s, bytes/s
```

### Roundtrip
```
Target: 20,000+ messages/second

Test: load_tests::test_roundtrip_throughput
  - 5,000 messages
  - Encode + decode cycle
```

### Latency (Local)
```
Target: p99 < 1ms

Test: latency_benchmarks
  - 1,000 samples
  - HdrHistogram analysis
  - Percentiles: p50, p95, p99, max
```

### Fanout
```
Target: 100% delivery

Test: real_benchmarks::fanout_curve
  - 1-1000 subscribers
  - Single publisher
  - Measure delivery ratio
```

## Test Utilities

### TestRouter (clasp-test-utils)
```rust
let router = TestRouter::start().await;
let client = router.connect_client().await?;
// Test...
router.stop();
```

### ValueCollector
```rust
let collector = ValueCollector::new();
client.subscribe("/test/*", collector.callback()).await?;
client.set("/test/value", 42).await?;
collector.wait_for_count(1, Duration::from_secs(5)).await;
assert_eq!(collector.values().len(), 1);
```

### Port Allocation
```rust
let port = find_available_port().await;
let udp_port = find_available_udp_port();
```

### Condition Waiting
```rust
wait_for_count(&counter, 10, Duration::from_secs(5)).await;
wait_for_flag(&ready, Duration::from_secs(5)).await;
```

## Test Patterns

### 1. Async/Await with Tokio
```rust
#[tokio::test]
async fn test_basic_connection() {
    let router = TestRouter::start().await;
    let client = router.connect_client().await.unwrap();
    assert!(client.is_connected());
}
```

### 2. Timeout Boundaries
```rust
tokio::time::timeout(Duration::from_secs(5), async {
    // Test logic that must complete in 5 seconds
}).await.expect("Test timed out");
```

### 3. Atomic Counters
```rust
let counter = Arc::new(AtomicU32::new(0));
let counter_clone = counter.clone();
client.subscribe("/test/*", move |_, _| {
    counter_clone.fetch_add(1, Ordering::SeqCst);
}).await?;
```

### 4. Notification/Sync
```rust
let notify = Arc::new(Notify::new());
let notify_clone = notify.clone();
client.subscribe("/test", move |_, _| {
    notify_clone.notify_one();
}).await?;
notify.notified().await;
```

### 5. Latency Measurement
```rust
use hdrhistogram::Histogram;
let mut hist = Histogram::<u64>::new(3).unwrap();
for _ in 0..1000 {
    let start = Instant::now();
    // Operation
    let latency = start.elapsed().as_micros() as u64;
    hist.record(latency).unwrap();
}
println!("p50: {}µs, p99: {}µs", hist.value_at_quantile(0.5), hist.value_at_quantile(0.99));
```

## Docker Test Infrastructure

### docker-compose.load-test.yml
```yaml
services:
  router:
    build: .
    ports:
      - "7330:7330"

  load-generator:
    build:
      dockerfile: load-generator.dockerfile
    environment:
      ROUTER_URL: ws://router:7330
      NUM_CLIENTS: 100
      MESSAGES_PER_CLIENT: 1000
      DURATION_SECS: 60
```

### docker-compose.chaos-test.yml
```yaml
services:
  router:
    build: .
    cap_add:
      - NET_ADMIN  # For tc/netem

  network-simulator:
    build:
      dockerfile: network-sim.dockerfile
    cap_add:
      - NET_ADMIN
    command: |
      tc qdisc add dev eth0 root netem delay 100ms 50ms
```

### Network Simulation
- **Linux**: tc/netem for latency, jitter, packet loss
- **macOS**: pfctl/dnctl
- **Docker**: Built-in with NET_ADMIN capability

## Running Tests

### Unit Tests
```bash
cargo test -p clasp-core
cargo test -p clasp-router
cargo test -p clasp-transport
```

### Integration Tests
```bash
cargo test --test '*'
```

### E2E Tests
```bash
cd clasp-e2e
cargo run --bin run-all-tests
cargo run --bin conformance_report
cargo run --bin relay_stress_tests
```

### Benchmarks
```bash
cargo bench -p clasp-core
cargo bench -p clasp-e2e
```

### Load Tests with Docker
```bash
cd clasp-e2e/docker
docker-compose -f docker-compose.load-test.yml up
```

## Coverage Analysis

### Known Coverage Gaps
1. P2P WebRTC integration (requires manual testing)
2. BLE transport (requires hardware)
3. Serial/DMX (requires hardware)
4. MQTT with real broker (conditional)
5. Long-running soak tests (time-limited in CI)

### CI Pipeline
```yaml
jobs:
  test:
    - cargo test --all
    - cargo clippy --all
    - cargo fmt --check

  e2e:
    - cargo run -p clasp-e2e --bin conformance_report
    - cargo run -p clasp-e2e --bin load_tests

  benchmarks:
    - cargo bench -p clasp-core
    - cargo bench -p clasp-e2e
```

## Test Metrics

### Current Stats
- Unit tests: ~500 assertions
- Integration tests: ~200 assertions
- E2E tests: ~1000 assertions
- Compliance tests: ~100 assertions
- Load tests: ~50 scenarios
- Stress tests: ~30 scenarios

### Performance Baseline (from stress test report)
- SET→ACK latency: p50=96ms, p95=117ms, p99=129ms (WAN)
- Single client throughput: 407 msg/s
- Fanout (10 subs): 100% delivery
- Concurrent (100 clients): 100% success
- Connection churn (50 cycles): 100% success
- Sustained (30s): 99.6% delivery, 275 msg/s

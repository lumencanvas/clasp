# E2E Test Suite Analysis

## Overview

**Location:** `clasp-e2e/`
**Total Test Binaries:** 39
**Total Compliance Tests:** 47

---

## Test Binary Inventory

### Protocol & Core (8)

| Binary | Purpose |
|--------|---------|
| `e2e_protocol_tests.rs` | End-to-end protocol compliance |
| `clasp_to_clasp.rs` | Inter-device communication |
| `relay_e2e.rs` | Relay server functionality |
| `p2p_connection_tests.rs` | P2P and signaling |
| `public_relay_tests.rs` | Public relay infrastructure |
| `protocol_comparison.rs` | Protocol validation |
| `network_tests.rs` | Network behavior |
| `conformance_report.rs` | Conformance report generation |

### Integration (2)

| Binary | Purpose |
|--------|---------|
| `broker_tests.rs` | MQTT/OSC broker integration |
| `bridge_tests.rs` | Protocol bridges |

### Load & Stress (4)

| Binary | Purpose |
|--------|---------|
| `load_tests.rs` | High-throughput (10k+ msg/s) |
| `relay_stress_tests.rs` | Latency analysis (p50/p95/p99) |
| `soak_tests.rs` | Long-duration stability |
| `chaos_tests.rs` | Failure injection |

### Benchmarks (10)

| Binary | Purpose |
|--------|---------|
| `latency_benchmarks.rs` | P50/P95/P99 latencies |
| `cold_start_benchmarks.rs` | Connection establishment |
| `memory_benchmarks.rs` | Memory/leak detection |
| `sustained_load_benchmarks.rs` | CPU/memory under load |
| `resilience_benchmark.rs` | Recovery metrics |
| `rendezvous_benchmarks.rs` | P2P rendezvous |
| `clock_sync_benchmark.rs` | Clock sync latency |
| `gesture_coalescing_benchmarks.rs` | Touch coalescing |
| `real_benchmarks.rs` | Production harness |
| `debug_benchmark.rs` | Quick debugging |

### Security (2)

| Binary | Purpose |
|--------|---------|
| `security_tests.rs` | JWT, scopes, auth |
| `security_pentest.rs` | Vulnerability scanning |

### Hardware/Embedded (2)

| Binary | Purpose |
|--------|---------|
| `embedded_tests.rs` | Embedded runtime |
| `hardware_tests.rs` | Hardware-specific |

### Debug Tools (8)

| Binary | Purpose |
|--------|---------|
| `debug_subscription.rs` | Subscription debugging |
| `debug_snapshot.rs` | State debugging |
| `debug_late_joiner.rs` | Late-joiner sync |
| `verify_patterns.rs` | Pattern verification |
| `proof_tests.rs` | Protocol proofs |
| `test_prefix_bug.rs` | Prefix matching bugs |
| `test_globstar_bug.rs` | Globstar pattern bugs |
| `test_segments_debug.rs` | Address segments |

---

## Compliance Suite

### Structure

```
src/compliance/
├── mod.rs          # Test framework
├── handshake.rs    # CLASP 4.1
├── messages.rs     # CLASP 3.x
├── state.rs        # CLASP 5.x
├── subscription.rs # CLASP 4.3
├── security.rs     # CLASP 6.x
└── encoding.rs     # CLASP 2.x
```

### Handshake Compliance (6 tests)

| Test | Spec |
|------|------|
| `test_hello_must_be_first` | CLASP 4.1 |
| `test_welcome_contains_session_id` | CLASP 4.1 |
| `test_version_negotiation` | CLASP 4.1 |
| `test_feature_negotiation` | CLASP 4.1 |
| `test_duplicate_hello_rejected` | CLASP 4.1 |
| `test_handshake_timeout` | CLASP 4.1 |

### Messages Compliance (6 tests)

| Test | Spec |
|------|------|
| `test_set_message` | CLASP 3.x |
| `test_get_message` | CLASP 3.x |
| `test_subscribe_message` | CLASP 3.x |
| `test_publish_message` | CLASP 3.x |
| `test_ack_message` | CLASP 3.x |
| `test_error_message` | CLASP 3.x |

### State Management (7 tests)

| Test | Spec |
|------|------|
| `test_lww_resolution` | CLASP 5.x |
| `test_max_merge_strategy` | CLASP 5.x |
| `test_min_merge_strategy` | CLASP 5.x |
| `test_lock_acquisition` | CLASP 5.x |
| `test_lock_release` | CLASP 5.x |
| `test_lock_prevents_writes` | CLASP 5.x |
| `test_revision_tracking` | CLASP 5.x |

### Subscription Patterns (7 tests)

| Test | Spec |
|------|------|
| `test_exact_subscription` | CLASP 4.3 |
| `test_single_wildcard` | CLASP 4.3 |
| `test_multi_wildcard` | CLASP 4.3 |
| `test_unsubscribe` | CLASP 4.3 |
| `test_subscription_snapshot` | CLASP 4.3 |
| `test_wildcard_no_match` | CLASP 4.3 |
| `test_multiple_subscriptions` | CLASP 4.3 |

### Security (6 tests)

| Test | Spec |
|------|------|
| `test_connection_without_token` | CLASP 6.x |
| `test_connection_with_token` | CLASP 6.x |
| `test_invalid_token_rejected` | CLASP 6.x |
| `test_token_scope_read` | CLASP 6.x |
| `test_token_scope_write` | CLASP 6.x |
| `test_expired_token_rejected` | CLASP 6.x |

### Encoding (8 tests)

| Test | Spec |
|------|------|
| `test_hello_encoding` | CLASP 2.x |
| `test_set_encoding` | CLASP 2.x |
| `test_value_int_encoding` | CLASP 2.x |
| `test_value_float_encoding` | CLASP 2.x |
| `test_value_string_encoding` | CLASP 2.x |
| `test_value_bool_encoding` | CLASP 2.x |
| `test_value_bytes_encoding` | CLASP 2.x |
| `test_roundtrip_encoding` | CLASP 2.x |

---

## Performance Targets

| Metric | Target |
|--------|--------|
| Encoding | 50,000+ msg/s |
| Decoding | 50,000+ msg/s |
| Roundtrip | 20,000+ msg/s |
| Latency (local) | p99 < 1ms |
| Memory per connection | < 10KB |
| Max payload | 65,535 bytes |

---

## Load Testing

### Scenarios

1. **Encoding Throughput** (10K messages)
2. **Decoding Throughput** (10K messages)
3. **Roundtrip Throughput** (5K messages)
4. **Large Payload** (~54KB arrays)
5. **Many Small Messages**
6. **Concurrent Encoding**
7. **Memory Stability**
8. **Latency Distribution**

### Measurement

```rust
// HDR Histogram for percentiles
let histogram = Histogram::new(3)?;
histogram.record(latency_us)?;
println!("p50: {}, p99: {}",
    histogram.value_at_quantile(0.5),
    histogram.value_at_quantile(0.99));
```

---

## Stress Testing

### Chaos Scenarios

1. **Disconnect Storm** - 50+ simultaneous disconnects
2. **Memory Pressure** - State saturation
3. **Connection Churn** - Rapid connect/disconnect
4. **Rapid Subscribe/Unsubscribe**
5. **Message Flood** - Backpressure handling

### Relay Stress Scenarios

1. High-concurrency (100+ clients)
2. Race condition detection
3. Subscription edge cases
4. Message ordering
5. Throughput limits
6. Connection churn (50 cycles)
7. Large payloads (1KB-256KB)
8. Protocol edge cases
9. State consistency
10. Memory leak detection
11. P2P signaling under load

---

## Benchmark Infrastructure

### Latency Benchmarks

- Single-hop delivery
- Fanout to multiple subscribers
- Wildcard pattern matching
- Jitter calculation

### Memory Benchmarks

```rust
// Platform-specific measurement
#[cfg(target_os = "linux")]
fn get_memory_kb() -> Option<u64> {
    // /proc/self/statm → RSS pages × 4KB
}

#[cfg(target_os = "macos")]
fn get_memory_kb() -> Option<u64> {
    // ps -o rss=
}
```

### Cold Start Benchmarks

- Connection establishment
- First message delivery
- Handshake completion
- Warmup: 100 iterations excluded

---

## Docker Infrastructure

### Integration Tests

```yaml
services:
  mqtt:
    image: eclipse-mosquitto:2
    ports: ["1883:1883", "9883:9883"]

  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]

  toxiproxy:
    image: shopify/toxiproxy:2.7.0
    ports: ["8474:8474"]

  osc-echo:
    build: Dockerfile.osc-echo
    ports: ["8000:8000/udp", "9000:9000/udp"]
```

### Load Tests

```yaml
services:
  clasp-router:
    build: router.dockerfile
    resources:
      limits: { cpus: "4", memory: 4G }

  load-generator:
    environment:
      ROUTER_URL: ws://clasp-router:7330
      NUM_CLIENTS: 100
      MESSAGES_PER_CLIENT: 1000
      DURATION_SECS: 60
```

### Chaos Tests

```yaml
services:
  network-sim-runner:
    cap_add: [NET_ADMIN]
    command: |
      tc qdisc add dev eth0 root netem \
        delay 50ms 10ms \
        loss 10% \
        corrupt 1%
```

---

## Test Patterns

### Timeout Wrapper

```rust
let response = timeout(config.timeout, async {
    // Test operations
}).await;
```

### Atomic Counter

```rust
let received = Arc::new(AtomicUsize::new(0));
subscriber.subscribe(address, move |_, _| {
    received_clone.fetch_add(1, Ordering::SeqCst);
}).await?;

timeout(config.timeout, async {
    while received.load(Ordering::SeqCst) == 0 {
        sleep(Duration::from_millis(10)).await;
    }
}).await
```

### Pre-warmed Benchmark

```rust
// Warmup (excluded)
for _ in 0..100 {
    client.set("/bench/set", value).await.ok();
}

// Measurement
let mut latencies = Vec::new();
for _ in 0..count {
    let start = Instant::now();
    client.set("/bench/set", value).await?;
    latencies.push(start.elapsed().as_micros() as u64);
}
```

---

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
cargo run --bin conformance_report
cargo run --bin load_tests
cargo run --bin relay_stress_tests
```

### Benchmarks

```bash
cargo bench -p clasp-core
cargo bench -p clasp-e2e
```

### Docker Tests

```bash
cd clasp-e2e/docker
docker-compose up                           # Integration
docker-compose -f docker-compose.load-test.yml up  # Load
docker-compose -f docker-compose.chaos-test.yml up # Chaos
```

---

## Performance Baseline

From stress test reports:

| Metric | Result |
|--------|--------|
| SET→ACK latency (WAN) | p50=96ms, p95=117ms, p99=129ms |
| Single client throughput | 407 msg/s |
| Fanout (10 subs) | 100% delivery |
| Concurrent (100 clients) | 100% success |
| Connection churn (50 cycles) | 100% success |
| Sustained (30s) | 99.6% delivery, 275 msg/s |

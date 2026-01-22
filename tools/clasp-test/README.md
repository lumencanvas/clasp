# clasp-test

CLASP Test Utility - Integration tests and stress testing for CLASP servers.

## Overview

`clasp-test` is a command-line tool for testing CLASP server implementations. It provides various test modes for verifying connectivity, parameter handling, pub/sub functionality, and performance characteristics.

## Building

```bash
cargo build -p clasp-test --release
```

## Usage

```bash
# Run all tests against default server
clasp-test all

# Run all tests against a specific server
clasp-test -u ws://192.168.1.100:7330 all

# Run specific test
clasp-test connect
clasp-test params --count 1000
clasp-test pubsub --count 5000
clasp-test latency --count 500
clasp-test stress --rate 10000 --duration 30
clasp-test multi-client --clients 50 --messages 100
```

## Commands

### all
Run all integration tests in sequence.

```bash
clasp-test all
```

### connect
Test basic connectivity to the server.

```bash
clasp-test connect
```

Verifies:
- WebSocket connection establishment
- HELLO/WELCOME handshake
- Session ID assignment

### params
Test parameter get/set operations.

```bash
clasp-test params --count 100
```

Options:
- `--count, -c`: Number of parameters to test (default: 100)

Verifies:
- Parameter creation
- Value persistence
- Get operations

### pubsub
Test publish/subscribe functionality.

```bash
clasp-test pubsub --count 1000
```

Options:
- `--count, -c`: Number of messages to publish (default: 1000)

Verifies:
- Subscription registration
- Message delivery
- Pattern matching

### stress
High-rate stress test.

```bash
clasp-test stress --rate 1000 --duration 10
```

Options:
- `--rate, -r`: Target messages per second (default: 1000)
- `--duration, -d`: Test duration in seconds (default: 10)

Measures:
- Sustained throughput
- Message delivery under load

### latency
Round-trip latency measurement.

```bash
clasp-test latency --count 100
```

Options:
- `--count, -c`: Number of round trips to measure (default: 100)

Reports:
- Average latency
- Minimum latency
- Maximum latency

### multi-client
Concurrent client test.

```bash
clasp-test multi-client --clients 10 --messages 100
```

Options:
- `--clients, -c`: Number of concurrent clients (default: 10)
- `--messages, -m`: Messages per client (default: 100)

Verifies:
- Concurrent connection handling
- Cross-client isolation
- Aggregate throughput

## Global Options

- `--url, -u`: Server URL (default: `ws://localhost:7330`)
- `--verbose, -v`: Enable debug logging

## Example Output

```
CLASP Test Utility

Testing connection... ✓ PASS
Testing 100 parameters... ✓ PASS (45.23ms)
Testing pub/sub with 1000 messages... ✓ PASS (892 received, 234.56ms)
Testing latency (100 round trips)... ✓ PASS (avg: 1234µs, min: 890µs, max: 2345µs)

✓ All tests passed!
```

## Exit Codes

- `0`: All tests passed
- `1`: One or more tests failed
- `2`: Connection error

## License

MIT or Apache-2.0

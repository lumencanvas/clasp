# clasp-test-utils

Common test helpers and utilities for CLASP (Creative Low-Latency Application Streaming Protocol) test suites.

## Features

- **TestRouter** - RAII-wrapped test router with automatic cleanup
- **ValueCollector** - Thread-safe value collection for subscription testing
- **Condition-based waiting** - Avoid flaky tests with proper async waiting
- **Port allocation** - Find available ports for test servers
- **Assertion helpers** - Enhanced assertions for test readability

## Usage

```rust
use clasp_test_utils::{TestRouter, ValueCollector};
use std::time::Duration;

#[tokio::test]
async fn test_subscription() {
    // Start a test router (auto-cleanup on drop)
    let router = TestRouter::start().await;

    // Connect clients
    let client = router.connect_client().await.unwrap();

    // Collect subscription values
    let collector = ValueCollector::new();
    client.subscribe("/test/**", collector.callback_ref()).await.unwrap();

    // Wait for values (condition-based, not time-based)
    assert!(collector.wait_for_count(1, Duration::from_secs(2)).await);
}
```

## Key Components

### TestRouter

```rust
// Start with default config
let router = TestRouter::start().await;

// Start with custom config
let router = TestRouter::start_with_config(RouterConfig { ... }).await;

// Get connection URL
let url = router.url(); // "ws://127.0.0.1:XXXX"

// Connect clients
let client = router.connect_client().await?;
let client = router.connect_client_named("MyClient").await?;
```

### ValueCollector

```rust
let collector = ValueCollector::new();

// Use as subscription callback
client.subscribe("/path", collector.callback_ref()).await?;

// Wait and verify
collector.wait_for_count(5, Duration::from_secs(2)).await;
assert_eq!(collector.count(), 5);
assert!(collector.has_address("/path/value"));
```

### Condition-Based Waiting

```rust
use clasp_test_utils::{wait_for, wait_for_count, wait_for_flag};

// Wait for a custom condition
let success = wait_for(
    || async { some_condition() },
    Duration::from_millis(10),  // check interval
    Duration::from_secs(5),     // timeout
).await;

// Wait for atomic counter
wait_for_count(&counter, 10, Duration::from_secs(2)).await;

// Wait for flag
wait_for_flag(&flag, Duration::from_secs(2)).await;
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

---

Maintained by [LumenCanvas](https://lumencanvas.studio) | 2026

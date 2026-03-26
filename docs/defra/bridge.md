---
title: Signal Bridge
description: Real-time DefraDB change notifications via CLASP signals
order: 5
---

# Signal Bridge

The `clasp-defra-bridge` crate creates a bidirectional bridge between DefraDB document mutations and CLASP real-time signals. Database changes become subscribable signals; CLASP writes update documents.

## Address Convention

DefraDB documents map to CLASP addresses:

```
/defra/{collection}/{docID}          -- whole document (EMIT on create)
/defra/{collection}/{docID}/{field}  -- specific field (SET on update)
```

CLASP clients subscribe with wildcards:

```rust
// All temperature changes across all sensors
client.on("/defra/SensorData/*/temperature", |value, addr| { ... });

// Any change to user profiles
client.on("/defra/User/**", |value, addr| { ... });
```

## Architecture

```
DefraDB                          CLASP Router
  |                                   |
  |  DefraWatcher (polls)             |
  |  ---------------------->          |
  |  EMIT /defra/User/abc {name: "alice"}
  |                                   |
  |                                   |  clients subscribe
  |                                   |  to /defra/User/**
  |                                   |
  |  DefraWriter (subscribes)         |
  |  <----------------------          |
  |  SET /defra/User/abc/name "bob"   |
  |                                   |
```

## Usage

```rust
use clasp_defra_bridge::{DefraBridge, SignalSender, SignalReceiver};
use std::time::Duration;

let bridge = DefraBridge::new("http://localhost:9181", vec!["User".into(), "SensorData".into()])
    .with_echo_ttl(Duration::from_secs(10));

bridge.run(sender, receiver, shutdown).await?;
```

## Echo Prevention

The bridge uses an `OriginTracker` to prevent feedback loops. When the bridge writes a change to DefraDB (from a CLASP signal), it records the address with a TTL. If the watcher detects that same change on the next poll, it suppresses the signal emission.

Default TTL: 5 seconds. Configurable via `with_echo_ttl()`.

## Trait Injection

The bridge depends on `SignalSender` and `SignalReceiver` traits, not `clasp-client` directly:

```rust
#[async_trait]
pub trait SignalSender: Send + Sync {
    async fn set(&self, address: &str, value: Value) -> Result<()>;
    async fn emit(&self, address: &str, value: Value) -> Result<()>;
}
```

This enables mock-based unit testing without a running CLASP router.

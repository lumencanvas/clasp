# CLASP Codebase Analysis Rules & Self-Prompting Guide

**Purpose:** Instructions for AI assistants analyzing this codebase

## Core Understanding Requirements

### Before Making Changes
1. **Read the target file first** - Never propose changes without reading
2. **Understand the crate's role** - Check dependency map
3. **Check for related tests** - Look in `tests/` and `clasp-e2e/`
4. **Verify patterns used** - Match existing code style

### Crate Hierarchy
```
clasp-core → Foundation (types, codec, frame)
clasp-transport → Network abstractions
clasp-router → Central message hub
clasp-bridge → Protocol adapters
clasp-client → User-facing API
clasp-discovery → Device finding
clasp-wasm → Browser bindings
clasp-embedded → Microcontroller support
```

## Key Patterns to Recognize

### 1. Binary Encoding (v3)
```rust
// Frame header: magic(1) + flags(1) + length(2) [+ timestamp(8)]
const MAGIC_BYTE: u8 = 0x53;  // 'S'
// Message codes: 0x01-0x61
// Value codes: 0x00-0x0B
```

### 2. Subscription Patterns
```
*  → One segment exactly
** → Zero or more segments
/a/*/c → /a/b/c matches
/a/** → /a/b/c/d matches
```

### 3. Error Codes
```
100-199: Protocol errors
200-299: Address errors
300-399: Permission errors
400-499: State errors
500-599: Server errors
```

### 4. QoS Levels
```rust
QoS::Fire   = 0  // Best effort, high-rate streams
QoS::Confirm = 1 // At-least-once, parameters
QoS::Commit  = 2 // Exactly-once, bundles
```

### 5. Signal Types
```rust
SignalType::Param    // Stateful, revision-tracked
SignalType::Event    // Ephemeral, one-shot
SignalType::Stream   // High-rate, Fire QoS
SignalType::Gesture  // Multi-touch phases
SignalType::Timeline // Keyframe automation
```

## Common Tasks

### Adding a New Message Type
1. Add variant to `MessageType` enum in `clasp-core/src/types.rs`
2. Add struct with fields in `clasp-core/src/types.rs`
3. Add to `Message` enum
4. Implement encoding in `clasp-core/src/codec.rs`
5. Implement decoding in `clasp-core/src/codec.rs`
6. Add tests for roundtrip
7. Update JavaScript bindings (`bindings/js/packages/clasp-core/src/`)
8. Update Python bindings (`bindings/python/python/clasp/`)

### Adding a New Transport
1. Create module in `clasp-transport/src/`
2. Implement `TransportSender`, `TransportReceiver` traits
3. Optionally implement `TransportServer` for accept()
4. Add feature flag in `Cargo.toml`
5. Add conditional compilation in `lib.rs`
6. Add tests in `tests/`

### Adding a New Bridge
1. Create module in `clasp-bridge/src/`
2. Implement `Bridge` trait
3. Define `*BridgeConfig` struct
4. Add feature flag in `Cargo.toml`
5. Add to `lib.rs` exports
6. Add integration tests

### Adding a Test
1. Unit test: `crates/<crate>/tests/<module>_tests.rs`
2. E2E test: `clasp-e2e/src/bin/<test_name>.rs`
3. Compliance test: `clasp-e2e/src/compliance/<category>.rs`
4. Use `TestRouter` from `clasp-test-utils`
5. Use `ValueCollector` for subscription testing

## Code Style Conventions

### Rust
- Edition 2021
- Max line width: 100
- Use `thiserror` for error types
- Use `async-trait` for async traits
- Use `DashMap` for concurrent maps
- Use `parking_lot` locks over std

### TypeScript
- Strict mode enabled
- Use classes for main structures
- Use interfaces for message types
- camelCase for methods/properties

### Python
- Python 3.8+ compatible
- Use dataclasses for DTOs
- Use async/await consistently
- snake_case for methods/properties

## Testing Patterns

### Unit Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_<what>_<scenario>() {
        // Arrange
        // Act
        // Assert
    }
}
```

### Async Test Structure
```rust
#[tokio::test]
async fn test_<what>_<scenario>() {
    let router = TestRouter::start().await;
    let client = router.connect_client().await.unwrap();

    // Test logic

    router.stop();
}
```

### Using ValueCollector
```rust
let collector = ValueCollector::new();
client.subscribe(pattern, collector.callback()).await?;
// Trigger values...
collector.wait_for_count(n, timeout).await;
assert_eq!(collector.values().len(), n);
```

## Performance Considerations

### Hot Paths
1. **Message encoding** - Called for every send
2. **Pattern matching** - Called for every message
3. **Subscription lookup** - O(n) prefix scan
4. **Gesture coalescing** - Every 16ms

### Optimization Targets
- Encoding: 50k+ msg/s
- Decoding: 50k+ msg/s
- Roundtrip: 20k+ msg/s
- Latency: p99 < 1ms local

### Memory Budget (Embedded)
- Client: ~3KB
- MiniRouter: ~4KB
- StateCache: ~2KB

## Security Checklist

### Before Adding Features
- [ ] Does it require authentication?
- [ ] Are scopes correctly checked?
- [ ] Are patterns validated?
- [ ] Are sizes bounded?
- [ ] Is input sanitized?

### Scope Checking Points
- SUBSCRIBE: Check Action::Read
- GET: Check Action::Read
- SET: Check Action::Write
- PUBLISH: Check Action::Write

## Documentation Requirements

### Public APIs
- Doc comments on all public items
- Examples where non-obvious
- Error conditions documented

### Internal APIs
- Brief comments on complex logic
- No redundant comments
- Self-documenting names preferred

## File Locations Quick Reference

| Need | Location |
|------|----------|
| Protocol types | `crates/clasp-core/src/types.rs` |
| Binary codec | `crates/clasp-core/src/codec.rs` |
| Message routing | `crates/clasp-router/src/router.rs` |
| Session management | `crates/clasp-router/src/session.rs` |
| Subscription matching | `crates/clasp-router/src/subscription.rs` |
| Gesture coalescing | `crates/clasp-router/src/gesture.rs` |
| Transport traits | `crates/clasp-transport/src/traits.rs` |
| WebSocket transport | `crates/clasp-transport/src/websocket.rs` |
| Bridge trait | `crates/clasp-bridge/src/traits.rs` |
| OSC bridge | `crates/clasp-bridge/src/osc.rs` |
| Client API | `crates/clasp-client/src/client.rs` |
| Test utilities | `crates/clasp-test-utils/src/lib.rs` |
| E2E tests | `clasp-e2e/src/bin/` |
| JS bindings | `bindings/js/packages/clasp-core/src/` |
| Python bindings | `bindings/python/python/clasp/` |

## Known Issues & Gotchas

### Globstar Bug
Pattern `/**` has matching issues - check `clasp-router/src/subscription.rs`

### Payload Size Limit
Messages > 16KB may fail silently - enforce at application level

### WASM P2P
Requires signaling through main connection - not standalone

### Embedded Constraints
- No heap allocation
- Fixed buffer sizes
- 32-entry state cache max

### Rate Limiting
Per-session, not per-address - can be bypassed with multiple sessions

## Self-Prompting Questions

When analyzing code, ask:

1. **What crate is this?** → Check purpose in dependency map
2. **What's the entry point?** → main.rs or lib.rs
3. **What traits does it implement?** → Look for impl blocks
4. **What features gate this?** → Check `#[cfg(feature = "...")]`
5. **How is it tested?** → Check `tests/` directory
6. **Is there a JS/Python equivalent?** → Check bindings
7. **What errors can occur?** → Look for Result returns
8. **Is it concurrent?** → Look for Arc, Mutex, DashMap
9. **What's the message flow?** → Trace send → receive
10. **Are there performance implications?** → Check hot paths

## Analysis Checklist

When producing analysis:

- [ ] All structs documented with fields
- [ ] All enums documented with variants
- [ ] Key functions with signatures
- [ ] Trait implementations listed
- [ ] Feature flags identified
- [ ] Dependencies mapped
- [ ] Error types catalogued
- [ ] Tests referenced
- [ ] Related files linked
- [ ] Performance notes included

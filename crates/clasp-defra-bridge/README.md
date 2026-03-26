# clasp-defra-bridge

Bidirectional bridge between DefraDB document mutations and CLASP real-time signals.

## What it does

- **DefraDB -> CLASP**: Watches DefraDB collections for changes, emits CLASP signals
- **CLASP -> DefraDB**: Listens for CLASP SET on `/defra/**`, updates DefraDB documents
- **Echo prevention**: OriginTracker with TTL suppresses feedback loops

## Address convention

```
/defra/{collection}/{docID}          -- whole document (EMIT on create)
/defra/{collection}/{docID}/{field}  -- specific field (SET on update)
```

CLASP clients subscribe with wildcards: `/defra/User/*/status`

## Usage

```rust
use clasp_defra_bridge::{DefraBridge, SignalSender, SignalReceiver};

let bridge = DefraBridge::new("http://localhost:9181", vec!["User".into()]);
bridge.run(sender, receiver, shutdown).await?;
```

## Design

Uses trait injection (`SignalSender`, `SignalReceiver`) instead of depending on `clasp-client` directly. This enables mock-based unit testing without a running CLASP router.

## License

MIT OR Apache-2.0

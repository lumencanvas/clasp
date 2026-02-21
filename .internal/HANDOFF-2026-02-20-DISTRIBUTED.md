# Handoff - 2026-02-20: CLASP Distributed Evolution

## Overview

Three sessions (spanning 2026-02-19 to 2026-02-20) implemented a 5-phase plan to evolve CLASP from a single-router star topology into a distributed creative network. The plan is documented at `.claude/plans/swirling-squishing-gem.md`.

The motivation came from studying Meshblu (Octoblu/Citrix, now defunct) — the "everything is a device with a UUID" model was powerful for IoT but failed under massive operational complexity (7 microservices, Redis, MongoDB, CoffeeScript). The goal was to take the best ideas from Meshblu, UCAN, NATS leaf nodes, CRDTs, and Matrix federation and layer them onto CLASP as opt-in crates that don't break anything.

**Result: 5 new crates, 5,672 lines of Rust, ~80 new tests, all passing. 552 lines changed in existing files (all non-breaking). Workspace compiles clean. Frontend still builds.**

## What Was Built

### Phase 1: `clasp-registry` — Entity Registry (1,487 lines, 24 tests)

Persistent Ed25519 identity for devices, users, services, and routers.

**Files:**
- `entity.rs` — `Entity`, `EntityId` (`clasp:<base58-pubkey-prefix>`), `EntityType`, `EntityKeypair`, `EntityStatus`
- `store.rs` — `EntityStore` async trait with `MemoryEntityStore` impl
- `sqlite.rs` — `SqliteEntityStore` behind `#[cfg(feature = "sqlite")]`
- `token.rs` — `ent_<base64(entity_id + timestamp + signature)>` token format, encode/decode
- `validator.rs` — `EntityValidator` implementing `TokenValidator` from clasp-core
- `error.rs` — `RegistryError` enum

**Key design:** `EntityValidator` implements the same `TokenValidator` trait used by the existing `CpskValidator`. This means it plugs into `ValidatorChain` (clasp-core/src/security.rs:354) without any router changes. Tokens prefixed `ent_` are dispatched to `EntityValidator`; existing `cpsk_` tokens still go to `CpskValidator`.

### Phase 2: `clasp-journal` — Event Log & State Persistence (1,007 lines, 10 tests)

Append-only journal recording every SET and PUBLISH with sequence numbers.

**Files:**
- `entry.rs` — `JournalEntry` with seq, timestamp, author, address, signal_type, value, revision, msg_type
- `journal.rs` — `Journal` async trait (append, query, latest_seq, snapshot, load_snapshot, compact)
- `memory.rs` — `MemoryJournal` ring-buffer impl with snapshot support
- `sqlite.rs` — `SqliteJournal` behind `#[cfg(feature = "sqlite")]`
- `error.rs` — `JournalError` enum

**THIS IS THE ONLY CRATE ACTUALLY INTEGRATED INTO THE ROUTER.** The integration is behind `#[cfg(feature = "journal")]` in clasp-router:

- `RouterState` has `journal: Option<Arc<dyn Journal>>` (state.rs:73)
- `apply_set()` appends to journal after state mutation (state.rs:224-237)
- `journal_publish()` records PUBLISH events (state.rs:242-261)
- `Router::with_journal()` builder method (router.rs:362-368)
- `Message::Replay` handler streams journal entries back as SET/PUBLISH messages (router.rs:1793-1882)
- `Replay = 0x24` message type added to codec (types.rs, codec.rs)

### Phase 3: `clasp-federation` — Router Federation (1,230 lines, 12 tests)

Router-to-router connections for multi-site state synchronization.

**Files:**
- `config.rs` — `FederationMode` (Hub/Leaf/Mesh), `FederationConfig`, `PeerInfo`, `PeerState`
- `namespace.rs` — `NamespaceManager` tracking which peer owns which address patterns, conflict detection, glob matching
- `link.rs` — `FederationLink` manages a single peer connection using standard CLASP client protocol (HELLO/WELCOME handshake, SUBSCRIBE to peer namespaces, relay SET/PUBLISH with origin-based loop prevention)
- `manager.rs` — `FederationManager` orchestrates multiple links, maintains namespace registry, routes messages to correct peers
- `error.rs` — `FederationError` enum

**Core changes for federation:**
- `FederationSync = 0x04` message type in codec (types.rs, codec.rs)
- `FederationOp` enum: DeclareNamespaces, RequestSync, RevisionVector, SyncComplete
- `FederationSyncMessage` with patterns, revisions map, origin
- `ParamState.origin: Option<String>` added (state.rs) for loop prevention

**NOT wired into the router yet.** Needs glue code (see "Remaining Work" below).

### Phase 4: `clasp-caps` — Capability Tokens (952 lines, 14 tests)

UCAN-inspired delegatable tokens with Ed25519 signatures.

**Files:**
- `token.rs` — `CapabilityToken` with create_root(), delegate(), verify_signature(), encode/decode (`cap_<base64url(msgpack)>`), `pattern_is_subset()` for scope attenuation checking
- `validator.rs` — `CapabilityValidator` implementing `TokenValidator`, validates chain depth, signature, trust anchors, scope attenuation through entire delegation chain
- `error.rs` — `CapError` enum

**Key design:** Scope attenuation means each token in a delegation chain can only narrow, never widen:
```
Root:   admin:/**
  -> Child:   write:/lights/**          (valid: admin > write, /** > /lights/**)
    -> Grand: write:/lights/room1/**    (valid: narrower pattern)
      -> Bad:  write:/audio/**          (REJECTED: /audio/** not subset of /lights/**)
```

Uses the same `action:pattern` format as existing CLASP scopes. `CapabilityValidator` converts to CLASP `Scope` objects so existing `session.has_scope()` authorization works unchanged.

**NOT wired into the router yet.** Just needs adding to `ValidatorChain` (~10 lines).

### Phase 5: `clasp-rules` — Rules Engine (996 lines, 20 tests)

Server-side reactive automation.

**Files:**
- `rule.rs` — `Rule`, `Trigger` (OnChange/OnThreshold/OnEvent/OnInterval), `Condition`/`CompareOp`, `RuleAction` (Set/Publish/SetFromTrigger/Delay), `Transform` (Identity/Scale/Clamp/Threshold/Invert)
- `engine.rs` — `RulesEngine` with evaluate(), cooldown tracking, loop prevention via origin tagging
- `error.rs` — `RulesError` enum

**NOT wired into the router yet.** Needs evaluate() called after SET/PUBLISH (~30 lines).

## Changes to Existing Files (All Non-Breaking)

| File | What Changed |
|------|-------------|
| `Cargo.toml` (workspace) | +18 lines: 5 new crate members, workspace deps for serde_bytes, uuid |
| `clasp-core/src/types.rs` | +66 lines: `Replay` (0x24), `FederationSync` (0x04) message types, `ReplayMessage`, `FederationOp`, `FederationSyncMessage` |
| `clasp-core/src/codec.rs` | +185 lines: encode/decode for Replay and FederationSync messages |
| `clasp-core/src/state.rs` | +3 lines: `origin: Option<String>` on `ParamState` |
| `clasp-discovery/src/device.rs` | +4 lines: `entity_id: None` in Default impl (field added in Phase 1) |
| `clasp-discovery/src/lib.rs` | +2 lines: `entity_id: None` in two DeviceInfo constructors |
| `clasp-client/src/client.rs` | +4 lines: handle Replay/FederationSync in message match |
| `clasp-router/Cargo.toml` | +3 lines: optional clasp-journal dependency, `journal` feature |
| `clasp-router/src/state.rs` | +154 lines: journal field, set_journal(), journal_publish(), journal append in apply_set() |
| `clasp-router/src/router.rs` | +116 lines: with_journal() builder, REPLAY message handler |

All new fields use `Option<>` with `#[serde(skip_serializing_if)]`. Wire format is unchanged. Existing clients unaffected.

## What Was Done in Integration Session

All 5 crates were wired into the relay server in a subsequent session:

1. **Validators wired** -- `ValidatorChain` in relay main.rs composes CPSK, Entity, and Capability validators. CLI flags: `--trust-anchor`, `--cap-max-depth`, `--registry-db`.
2. **Rules integrated** -- `RulesEngine` wired into router with `with_rules()`. CLI flag: `--rules <path.json>`. OnInterval timer tasks spawned. `evaluate_interval()` added for timer-based rule firing.
3. **Federation leaf mode** -- `run_federation_leaf()` connects to hub, applies RemoteSet/RemotePublish to local state and broadcasts to local subscribers. CLI flags: `--federation-hub`, `--federation-id`, `--federation-namespace`, `--federation-token`.
4. **CLI flags** -- All new features have corresponding CLI flags (journal, caps, registry, rules, federation).
5. **Registry REST API secured** -- AdminToken extractor validates Bearer token with `admin:/**` scope on all `/api/entities` endpoints.
6. **Journal recovery fixed** -- Replaced `Handle::block_on()` with direct `.await` to prevent potential runtime panic.
7. **SetFromTrigger broadcast** -- Now notifies subscribers (was silently updating state).
8. **Federation broadcast** -- RemoteSet and RemotePublish now forward to local subscribers.

## Remaining Work

### 1. Hub/mesh federation mode

Only leaf mode is implemented. Hub mode (accept incoming federation connections) and mesh mode (peer-to-peer) need:
- Accept inbound `FederationSync` messages in router `handle_message`
- Outbound forwarding: after local SET/PUBLISH, check `manager.should_forward(&address)` and relay to peers
- Store-and-forward via journal on reconnect

### 2. Frontend (JS) support for new token types

The `@clasp-to/core` npm package currently only knows about `cpsk_` tokens. Capability tokens (`cap_`) work as opaque bearer strings with the existing SDK, but client-side delegation support (WebCrypto Ed25519) is not yet implemented.

### 3. Integration tests

Each crate has unit tests, but there are no cross-crate integration tests:
- Two routers federating and syncing state
- Capability token delegation chain validated by a router
- Rules engine triggering SET that reaches subscribers
- Journal replay after simulated crash
- Entity registration + token creation + authentication flow

## Gotchas and Lessons Learned

### Codec patterns
- New message types go in `MessageType` enum (types.rs) with specific byte codes: 0x04-0x0F for setup, 0x20-0x2F for data
- Must add encode AND decode functions in codec.rs, plus the match arms in `encode()` and `decode()`
- The Error enum uses `DecodeError(String)`, NOT `InvalidPayload`

### rmp_serde encoding
- `rmp_serde::to_vec()` = compact/positional format
- `rmp_serde::to_vec_named()` = named fields format
- `rmp_serde::from_slice()` expects named format when deserializing structs
- Use `to_vec_named()` for anything that round-trips through serde

### serde_bytes
- Use the `serde_bytes` crate as a dependency and `#[serde(with = "serde_bytes")]` attribute
- Do NOT write a custom `mod serde_bytes` — it shadows the crate and causes confusion

### base64
- `base64::engine::general_purpose::URL_SAFE_NO_PAD.encode()` requires `use base64::Engine;` in scope
- Import it inside the function if only used locally

### Transport/codec API
- `codec::encode(msg)` returns `Bytes` (already framed)
- `codec::decode(data)` returns `(Message, Frame)` tuple
- Don't manually construct `Frame` objects — the codec handles framing

### CLASP message struct fields (easy to get wrong)
- `SubscribeMessage`: fields are `types: Vec<SignalType>` and `options: Option<SubscribeOptions>` — NOT `max_rate`, `epsilon`, `history`
- `WelcomeMessage`: field is `session` not `session_id`
- `HelloMessage`: has `capabilities: Option<...>` field
- `SnapshotMessage.params[].revision`: is `u64` not `Option<u64>`

### Token validator integration
- `TokenValidator` trait is at clasp-core/src/security.rs:250
- `ValidatorChain` at security.rs:354 dispatches based on token prefix
- Each validator returns `NotMyToken` if it doesn't recognize the prefix
- Router calls `set_token_validator()` (router.rs:338) — takes any `impl TokenValidator`
- Relay server wraps in `SharedValidator` for Arc sharing (main.rs:44-57)

### ParamState.origin field
- Added `origin: Option<String>` to `ParamState` (state.rs)
- Used for federation loop prevention (messages carry origin router ID, never re-forwarded to origin)
- Also used by rules engine (`"rule:{id}"` origin prevents rule loops)
- Must be `None` in `ParamState::new()` constructor

## Test Counts (as of 2026-02-20)

| Crate | Unit Tests | Doc Tests | Total |
|-------|-----------|-----------|-------|
| clasp-registry | 23 | 1 | 24 |
| clasp-journal | 10 | 0 | 10 |
| clasp-federation | 10 | 1 (+1 async) | 12 |
| clasp-caps | 13 | 1 | 14 |
| clasp-rules | 19 | 1 | 20 |
| **New crate total** | **75** | **4+** | **~80** |

Existing crate tests (clasp-core, clasp-router) all still pass.

## File Tree of New Crates

```
crates/
  clasp-registry/     # Phase 1: Entity identity
    src/
      entity.rs       # Entity, EntityId, EntityType, EntityKeypair (276 lines)
      store.rs        # EntityStore trait, MemoryEntityStore (249 lines)
      sqlite.rs       # SqliteEntityStore, feature-gated (422 lines)
      token.rs        # ent_ token encode/decode (179 lines)
      validator.rs    # EntityValidator -> TokenValidator (276 lines)
      error.rs        # RegistryError (35 lines)
      lib.rs          # Re-exports (50 lines)
    Cargo.toml

  clasp-journal/      # Phase 2: Event log (INTEGRATED)
    src/
      entry.rs        # JournalEntry (77 lines)
      journal.rs      # Journal async trait (46 lines)
      memory.rs       # MemoryJournal ring buffer (399 lines)
      sqlite.rs       # SqliteJournal, feature-gated (433 lines)
      error.rs        # JournalError (26 lines)
      lib.rs          # Re-exports (26 lines)
    Cargo.toml

  clasp-federation/   # Phase 3: Router-to-router
    src/
      config.rs       # FederationMode, FederationConfig, PeerInfo (108 lines)
      namespace.rs    # NamespaceManager, conflict detection (238 lines)
      link.rs         # FederationLink, CLASP client protocol (491 lines)
      manager.rs      # FederationManager, multi-peer orchestration (279 lines)
      error.rs        # FederationError (62 lines)
      lib.rs          # Re-exports (52 lines)
    Cargo.toml

  clasp-caps/         # Phase 4: Delegatable tokens
    src/
      token.rs        # CapabilityToken, delegation, Ed25519 (536 lines)
      validator.rs    # CapabilityValidator -> TokenValidator (332 lines)
      error.rs        # CapError (42 lines)
      lib.rs          # Re-exports (42 lines)
    Cargo.toml

  clasp-rules/        # Phase 5: Rules engine
    src/
      rule.rs         # Rule, Trigger, Condition, Transform (312 lines)
      engine.rs       # RulesEngine, evaluate(), cooldowns (598 lines)
      error.rs        # RulesError (30 lines)
      lib.rs          # Re-exports (56 lines)
    Cargo.toml
```

## Git Status

All changes are uncommitted on the `main` branch:
- 10 modified files (552 insertions, 3 deletions in existing code)
- 5 untracked directories (the new crates)
- No breaking changes to wire format or public API

# CLASP Current Status
**Last Updated:** February 21, 2026
**Audited By:** Claude Opus 4.6

---

## What Actually Works (Verified)

### Core Protocol
- Binary encoding/decoding: **WORKING** (65+ unit tests pass)
- All message types (SET, PUBLISH, SUBSCRIBE, REPLAY, FEDERATION_SYNC, etc.): **WORKING**
- All value types: **WORKING**
- Address parsing & wildcards: **WORKING**
- State management (LWW, locks): **WORKING**
- Federation feature flag in binary codec: **WORKING** (bit 0x04, added session 4)

### Router
- Message routing: **WORKING**
- Subscription matching: **WORKING**
- WebSocket transport: **WORKING**
- Session management: **WORKING**
- WriteValidator trait: **WORKING**
- SnapshotFilter trait: **WORKING**
- `has_strict_read_scope()`: **WORKING**
- Federation hub (accept inbound peers): **WORKING** (feature-gated)
- Federation namespace restriction: **WORKING** (session 3 -- peers can only sync declared namespaces)
- Federation resource limits: **WORKING** (session 3 -- MAX_FEDERATION_PATTERNS=1000, MAX_REVISION_ENTRIES=10000)
- Federation scope enforcement: **WORKING** (session 3 -- authenticated mode checks scopes)
- Federation pattern matcher: **WORKING** (session 4 -- glob_match bypass fixed, 15 edge case tests)

### Auth & Security
- CPSK tokens (`cpsk_`): **FULLY FUNCTIONAL** -- register/login/guest -> token -> HELLO -> scopes enforced
- Capability tokens (`cap_`): **FULLY FUNCTIONAL** -- CLI create/delegate/inspect/verify, ValidatorChain integration
- Entity tokens (`ent_`): **FULLY FUNCTIONAL** -- CLI keygen/mint/inspect, REST API minting, ValidatorChain integration
- Admin bootstrap (`--admin-token`): **WORKING** -- generates or loads admin token file with admin:/** scope
- ValidatorChain: **WORKING** -- CPSK + Cap + Entity validators chained, dispatches by token prefix
- Secret file writes: **WORKING** -- atomic 0o600 permissions via `write_secret_file()` (no TOCTOU)
- Startup error handling: **WORKING** -- no panics on bad config, clean error propagation

### Distributed Infrastructure
- Journal persistence: **WORKING** -- SQLite or in-memory, recovery on restart, replay support
- Rules engine: **WORKING** -- OnChange, OnThreshold, OnEvent, OnInterval triggers, SetFromTrigger broadcast
- Entity registry: **WORKING** -- REST CRUD (admin-only), SQLite storage, entity validation
- Federation leaf mode: **WORKING** -- connects to hub, forwards RemoteSet/RemotePublish to local subscribers
- Federation hub mode: **WORKING** -- accepts inbound peers, DeclareNamespaces, RequestSync, RevisionVector

### CLI Admin Tooling
- Key management: **WORKING** -- `clasp key generate`, `clasp key show` (hex/did:key formats)
- Cap token commands: **WORKING** -- `clasp token cap create/delegate/inspect/verify`
- Entity token commands: **WORKING** -- `clasp token entity keygen/mint/inspect`
- Trust anchor loading: **WORKING** -- hex-encoded key files correctly decoded

### Bridges
- OSC: **WORKING**
- MIDI: **WORKING**
- Art-Net: **WORKING**
- HTTP: **WORKING**
- WebSocket: **WORKING**
- MQTT: **IMPLEMENTED** (needs more test coverage)

### Advanced Features
- BUNDLE messages: **WORKING**
- Scheduled bundles: **WORKING**
- Lock/unlock: **WORKING**
- Clock sync: **IMPLEMENTED**

### Signal Types
- Param: **WORKING**
- Event: **WORKING**
- Stream: **WORKING** (basic)
- Gesture: **IMPLEMENTED**
- Timeline: **IMPLEMENTED**

---

## What Needs Verification

### P2P WebRTC
- Full implementation with automatic relay fallback
- Tests require `--features p2p` flag

### Transports Needing Testing
- QUIC: Implemented, tests exist but coverage unclear
- UDP: Implemented, tests exist
- TCP: Implemented, needs verification
- Serial: Implemented, hardware-dependent
- BLE: Implemented, hardware-dependent

---

## What's NOT Done

### Mesh Federation
- Only hub and leaf modes exist, no peer-to-peer mesh

### Frontend Cap Delegation
- JS SDK passes cap tokens as opaque strings (works for auth)
- No client-side Ed25519 delegation via WebCrypto

### Cap Token HTTP API
- Cap tokens are CLI-only by design (offline-first)
- No HTTP endpoint for creating/delegating

---

## Test Results Summary (Verified Feb 21, 2026)

```
clasp-core:              166 tests PASS
clasp-caps:               20 tests PASS (14 original + 6 negative)
clasp-registry:           14 tests PASS (10 original + 4 negative)
clasp-rules:              23 tests PASS
clasp-router (lib):       60 tests PASS (32 original + 28 federation unit)
clasp-router (integ):      8 tests PASS (federation integration, end-to-end)
clasp-relay:              35 tests PASS (24 auth + 7 registry + 4 persist)
apps/chat (frontend):     vite build PASS
CLI e2e:                  all commands functional
```

### Relay Feature Flags

| Feature | Flag | What |
|---------|------|------|
| WebSocket | `websocket` (default) | WebSocket transport |
| QUIC | `quic` | QUIC transport (needs --cert/--key) |
| MQTT | `mqtt-server` | Accept MQTT clients |
| OSC | `osc-server` | Accept OSC clients via UDP |
| Journal | `journal` | SQLite state persistence |
| Caps | `caps` | Capability token validation |
| Registry | `registry` | Entity registry + REST API |
| Rules | `rules` | Server-side automation |
| Federation | `federation` | Router-to-router sync |
| Rendezvous | `rendezvous` (default) | WAN discovery |
| Full | `full` | All of the above |

---

## Code Quality

### Compiler Warnings (pre-existing, not from new code)
- clasp-core: 1 warning (dead code - Pattern.regex)
- clasp-transport: 7 warnings (unused imports, dead code)
- clasp-router: 1 warning (unused Broadcast variant)
- clasp-bridge: 11 warnings (unused imports, dead code)

### No New Warnings
All new code (admin tooling, security hardening, tests) compiles without warnings.

---

## Security Fixes Applied (Sessions 2-4)

| # | Severity | Issue | Status |
|---|----------|-------|--------|
| 1 | BUG | Trust anchor file format mismatch | FIXED |
| 2 | HIGH | Admin token permissions silently dropped | FIXED |
| 3 | MOD | TLS warning on entity token minting | FIXED |
| 4 | MOD | Federation subscription cleanup on re-declare | FIXED |
| 5 | MOD | Entity `create` renamed to `keygen` | FIXED |
| 6 | CRIT | Federation namespace restriction missing | FIXED |
| 7 | CRIT | Federation resource limits missing | FIXED |
| 8 | CRIT | Federation scope enforcement missing | FIXED |
| 9 | HIGH | Panics on bad config crash server | FIXED |
| 10 | HIGH | TOCTOU on secret file writes | FIXED |
| 11 | CRIT | Federation feature missing from binary codec | FIXED |
| 12 | CRIT | Pattern matcher bypass via glob_match | FIXED |

---

## Recommended Next Steps

1. **Commit & PR** -- All work is on `feat/distributed-infrastructure`, uncommitted
2. **Mesh federation** -- Peer-to-peer mode (hub+leaf covers most use cases)
3. **Frontend cap support** -- WebCrypto Ed25519 for client-side delegation

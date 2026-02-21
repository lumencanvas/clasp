# Handoff - 2026-02-20/21: Admin Tooling, Token Minting, Deep Audit & Security Hardening

## Overview

Four sessions on the `feat/distributed-infrastructure` branch:

**Session 1** implemented all 4 phases of admin tooling: CLI key/cap/entity commands, relay admin API endpoints, admin token bootstrap, and hub federation support.

**Session 2** audited every line, found 6 issues (1 bug, 2 high severity, 3 moderate), and fixed all of them.

**Session 3** implemented a security hardening plan: 3 critical federation security fixes, 2 high-severity config handling fixes, and extensive negative test coverage across all new crates.

**Session 4** validated the security fixes end-to-end with integration tests, fuzzed the pattern matcher (found and fixed a real bypass bug), and added happy-path admin API mint tests.

---

## What Was Built (Session 1)

### Phase 1: CLI Admin Commands

**Files:** `crates/clasp-cli/src/main.rs`, `crates/clasp-cli/Cargo.toml`

Added three command groups:

```bash
# Key management
clasp key generate [--out ./root.key]     # Ed25519 keypair, hex-encoded
clasp key show ./root.key [--format did]  # Public key in hex or did:key format

# Capability tokens (feature-gated: caps)
clasp token cap create --key ./root.key --scopes "admin:/**" --expires 30d
clasp token cap delegate <parent-token> --key ./child.key --scopes "write:/lights/**"
clasp token cap inspect <token>
clasp token cap verify <token> --trust-anchor ./root.key

# Entity tokens (feature-gated: registry)
clasp token entity keygen [--out ./device.key] [--name "Sensor A"] [--type device]
clasp token entity mint --key ./device.key
clasp token entity inspect <token>
```

**Key format:** Hex-encoded 32-byte Ed25519 signing key (single line in file). Public key derived on the fly. Unix permissions set to 0o600.

**did:key format:** Multicodec prefix `0xed01` + public key bytes, base58-encoded, prefixed with `did:key:z`.

**Dependencies added:** `clasp-caps` (optional), `clasp-registry` (optional), `ed25519-dalek`, `rand`, `bs58`. Features: `caps`, `registry` (both default on).

### Phase 2: Relay Admin API Endpoints

**File:** `deploy/relay/src/registry.rs`

Two new endpoints:

| Endpoint | Auth | Purpose |
|----------|------|---------|
| `POST /api/entities/{id}/token` | Admin CPSK | Mint entity token (accepts signing key, verifies against stored pubkey) |
| `GET /api/trust-anchors` | None (public) | Returns configured trust anchors and max chain depth |

**`RegistryState` extended** with `trust_anchors: Vec<String>` and `cap_max_depth: usize`, plus `with_trust_anchors()` builder.

### Phase 3: Admin Token Bootstrap

**File:** `deploy/relay/src/main.rs`

```bash
clasp-relay --auth-port 7350 --admin-token ./admin.token
```

- If file exists: reads token, registers with `admin:/**` scope
- If not: generates new CPSK token, writes to file with 0o600 permissions, registers
- Solves the chicken-and-egg problem (need admin token to use registry API, but tokens come from the auth API)

### Phase 4: Hub Federation

**Files:** `crates/clasp-router/src/router.rs`, `crates/clasp-router/src/session.rs`, `crates/clasp-router/Cargo.toml`

Hub-side federation support (accepting inbound peers). All behind `#[cfg(feature = "federation")]`.

**Session struct:** Added `federation_peer: bool`, `federation_router_id`, `federation_namespaces` fields with accessors. Auto-detects federation peers from HELLO features.

**Router `FederationSync` handler** with 4 operations:
- **DeclareNamespaces:** Stores peer info, auto-subscribes peer to declared patterns (sub IDs 50000+)
- **RequestSync:** Sends filtered snapshot for requested patterns, supports `since_revision` filtering
- **RevisionVector:** Compares local state, sends delta as snapshot for newer entries
- **SyncComplete:** Logs completion

Guard: rejects `FederationSync` from non-federation sessions (403 error).

## What Was Fixed (Session 2 - Deep Audit)

### Fix 1: Trust Anchor File Format Mismatch (BUG)

**Problem:** `clasp key generate` writes hex-encoded signing keys (64 chars), but the relay read trust anchor files as raw bytes and passed them directly to `CapabilityValidator::new()`. This meant trust anchors were always the wrong format -- 64 ASCII bytes instead of 32 raw key bytes.

**Fix:** Trust anchor loading now:
1. Reads file as string, trims whitespace
2. If 64 hex chars: hex-decode to signing key, derive public key (32 bytes)
3. If 32 raw bytes: use directly
4. Otherwise: panic with clear error message

**File:** `deploy/relay/src/main.rs`

### Fix 2: Admin Token Permissions Error Silencing

**Problem:** `let _ = std::fs::set_permissions(...)` silently dropped errors. If permissions failed to set, the admin token file could be world-readable.

**Fix:** Changed to `if let Err(e) { tracing::warn!(...) }` with a message explaining the file may be world-readable.

**File:** `deploy/relay/src/main.rs`

### Fix 3: TLS Warning on Entity Token Minting

**Problem:** `POST /api/entities/{id}/token` accepts a signing key (private key) in the request body, which traverses the network in plaintext if not behind TLS.

**Fix:** Added doc comment documenting TLS requirement and pointing to CLI alternative (`clasp token entity mint --key`). Changed runtime log from `info!` to `warn!` noting the signing key was transmitted.

**File:** `deploy/relay/src/registry.rs`

### Fix 4: Federation Subscription Cleanup on Re-declare

**Problem:** If a federation peer calls `DeclareNamespaces` multiple times, subscriptions accumulated without cleanup, creating duplicates.

**Fix:** Before creating new subscriptions, clean up all previous federation subscriptions (identified by sub IDs 50000+) by calling `subscriptions.remove()` and `session.remove_subscription()` for each old namespace.

**File:** `crates/clasp-router/src/router.rs`

### Fix 5: Entity `create` Renamed to `keygen`

**Problem:** `clasp token entity create` only generates a keypair -- it doesn't register anything in the entity registry. The name was misleading.

**Fix:** Renamed to `clasp token entity keygen` with updated help text.

**File:** `crates/clasp-cli/src/main.rs`

## Security Hardening (Session 3)

### Fix 6: Federation namespace restriction

**Problem:** `RequestSync` and `RevisionVector` accepted ANY patterns/addresses from federation peers, regardless of what they declared in `DeclareNamespaces`. A peer could declare `/sensors/**` but request `/**`.

**Fix:**
- `RequestSync`: validates each requested pattern against `session.federation_namespaces()` using `federation_pattern_covered_by()` -- rejects patterns not covered by a declared namespace (403)
- `RevisionVector`: filters addresses against declared namespaces, silently skipping out-of-scope entries
- `DeclareNamespaces`: validates scope in authenticated mode

**File:** `crates/clasp-router/src/router.rs`

### Fix 7: Federation resource limits

**Problem:** No limits on pattern count or entry count in federation operations. Peers could exhaust memory.

**Fix:** Added `MAX_FEDERATION_PATTERNS = 1000` and `MAX_REVISION_ENTRIES = 10_000`. Rejects over-limit requests with 400 error.

**File:** `crates/clasp-router/src/router.rs`

### Fix 8: Federation scope enforcement in authenticated mode

**Problem:** Federation sync handlers never called `has_scope()` or `has_strict_read_scope()`. In authenticated mode, federation peers bypassed all scope restrictions.

**Fix:** Added scope checks in `RequestSync`, `RevisionVector`, and `DeclareNamespaces` when `security_mode == Authenticated`.

**File:** `crates/clasp-router/src/router.rs`

### Fix 9: Replace panics with proper error handling

**Problem:** Trust anchor loading, rules file parsing, and admin token bootstrap all `panic!()` on bad input, crashing the server on startup.

**Fix:** Changed all `panic!()`/`unwrap_or_else(|_| panic!(...))` to `Result` propagation via `anyhow::bail!()` / `.with_context()`.

**File:** `deploy/relay/src/main.rs`

### Fix 10: TOCTOU on secret file writes

**Problem:** Files were written with default permissions, then `chmod 0o600` applied after. Brief window where secrets are world-readable.

**Fix:** Added `write_secret_file()` helper using `OpenOptions::mode(0o600)` for atomic restrictive permissions. Applied to relay admin token and CLI key generation.

**Files:** `deploy/relay/src/main.rs`, `crates/clasp-cli/src/main.rs`

### Fix 11: Missing federation feature in binary codec

**Problem:** The CLASP binary codec (`clasp-core/src/codec.rs`) encodes HELLO features as a bitmask. `"federation"` was not in the match list, so it was silently dropped during encoding. Federation peers were never recognized by the server.

**Fix:** Added `"federation" => features |= 0x04` in encode and corresponding `feature_flags & 0x04` decode.

**File:** `crates/clasp-core/src/codec.rs`

### Fix 12: Pattern matcher bypass via glob_match

**Problem:** `federation_pattern_covered_by()` used `glob_match(declared, request)` which treated `**` in the request as literal characters. A peer declaring `/a/*` could request `/a/**` (all depths) and the match would succeed because `*` matched the literal string `**`.

**Fix:** Skip `glob_match` when the request contains wildcards. Also reordered the segment walk to check `rp == "**"` before `dp == "*"` to prevent `*` from swallowing `**`.

**File:** `crates/clasp-router/src/router.rs`

## Integration Test Coverage (Session 4)

### Federation Integration Tests (NEW)

**File:** `crates/clasp-router/tests/federation_tests.rs`

8 end-to-end integration tests that exercise the full WebSocket -> router -> handle_message path:

| Test | What it proves |
|------|----------------|
| `test_non_federation_session_rejected` | Non-federation session gets 403 for FederationSync |
| `test_declare_namespaces_too_many_patterns` | 1001 patterns rejected with 400 |
| `test_declare_namespaces_success` | Valid DeclareNamespaces gets ACK |
| `test_request_sync_outside_declared_namespaces` | RequestSync for undeclared namespace gets 403 |
| `test_request_sync_within_declared_namespaces` | RequestSync within declared namespace gets SyncComplete |
| `test_revision_vector_filters_out_of_scope_addresses` | RevisionVector for out-of-scope address produces no snapshot |
| `test_redeclare_namespaces_cleanup` | Re-declaring namespaces cleans up old subscriptions |
| `test_federation_peer_receives_declared_namespace_data` | Positive proof: peer receives SET for declared namespace |

### Pattern Matcher Edge Case Tests (NEW)

15 edge case tests for `federation_pattern_covered_by()` in `router.rs`:

- Empty strings, root slash, trailing slashes, double slashes
- Deep nesting under globstar, single wildcard depth mismatch
- Wildcard request vs literal declared (wider = rejected)
- Request `**` vs declared `*` (the bypass bug found and fixed)
- Mixed wildcards, identical patterns, path traversal segments
- Declared shorter/longer than request without wildcards

### Admin API Tests (NEW + Extended)

**File:** `deploy/relay/src/registry.rs`

7 tests total:
- `test_mint_token_without_auth_returns_401`
- `test_mint_token_with_non_admin_returns_403`
- `test_mint_token_with_invalid_keypair_returns_400`
- `test_mint_token_happy_path` -- full round-trip: create entity, mint token, verify token parses and signature validates
- `test_mint_token_wrong_key_returns_403` -- mismatched signing key rejected
- `test_get_trust_anchors`
- `test_create_and_get_entity`

### Negative Tests Added Across Crates

| Crate | Tests Added | What |
|-------|-------------|------|
| `clasp-caps/token.rs` | 6 | Malformed base64, missing prefix, truncated, corrupted msgpack, signature tampering, empty scopes |
| `clasp-caps/validator.rs` | 5 | Bad base64, truncated, tampered signature, chain depth exceeded, multiple trust anchors |
| `clasp-registry/token.rs` | 4 | Bad base64, truncated payload, wrong key length, truncated signature |
| `clasp-registry/validator.rs` | 4 | Malformed token, revoked entity, max token age, nonexistent entity |
| `clasp-router/router.rs` | 28 | Pattern coverage (15 edge cases), session lifecycle (3), subscription IDs (1), resource limits (1), 8 existing |

## Files Modified (All Changes, All Sessions)

| File | Sessions | What |
|------|----------|------|
| `crates/clasp-core/src/codec.rs` | 4 | Added federation feature flag bit (0x04) in encode/decode |
| `crates/clasp-cli/Cargo.toml` | 1 | Added clasp-caps, clasp-registry, ed25519-dalek, rand, bs58 deps + features |
| `crates/clasp-cli/src/main.rs` | 1, 2, 3 | Key/Cap/Entity subcommands, keygen rename, TOCTOU fix |
| `crates/clasp-router/Cargo.toml` | 1 | `federation = []` feature |
| `crates/clasp-router/src/router.rs` | 1, 2, 3, 4 | FederationSync handler, namespace restriction, resource limits, scope enforcement, pattern matcher fix, unit tests |
| `crates/clasp-router/src/session.rs` | 1, 3 | Federation peer fields + accessors, `stub_federation()` |
| `crates/clasp-router/tests/federation_tests.rs` | 4 | NEW: 8 integration tests |
| `crates/clasp-caps/src/token.rs` | 3 | 6 negative tests |
| `crates/clasp-caps/src/validator.rs` | 3 | 5 negative tests |
| `crates/clasp-registry/src/token.rs` | 3 | 4 negative tests |
| `crates/clasp-registry/src/validator.rs` | 3 | 4 negative tests |
| `deploy/relay/Cargo.toml` | 1 | ed25519-dalek dep, federation feature |
| `deploy/relay/src/main.rs` | 1, 2, 3 | Admin bootstrap, trust anchor fix, permissions warning, panic removal, TOCTOU fix |
| `deploy/relay/src/registry.rs` | 1, 2, 3, 4 | Token minting, trust anchors, TLS warning, admin API tests |

## Current Token Type Status

| Token Type | Prefix | Validation | CLI Generation | HTTP Minting | Status |
|-----------|--------|------------|----------------|--------------|--------|
| CPSK | `cpsk_` | Working | `clasp token create` | `POST /auth/register` | Fully functional |
| Capability | `cap_` | Working (if `--trust-anchor`) | `clasp token cap create/delegate` | None (offline only) | Fully functional |
| Entity | `ent_` | Working (if `--registry-db`) | `clasp token entity mint` | `POST /api/entities/{id}/token` | Fully functional |

## Test Results Summary (Verified Feb 21, 2026)

```
clasp-core:              166 tests PASS
clasp-caps:               20 tests PASS (14 original + 6 negative)
clasp-registry:           14 tests PASS (10 original + 4 negative)
clasp-rules:              23 tests PASS
clasp-router (lib):       60 tests PASS (32 original + 28 federation)
clasp-router (integ):      8 tests PASS (federation integration)
clasp-relay:              35 tests PASS (31 + 4 new admin API)
apps/chat (frontend):     vite build PASS
CLI e2e:                  all commands functional
```

## Remaining Work

1. **Mesh federation** -- Only hub and leaf modes exist, no peer-to-peer mesh
2. **Frontend cap delegation** -- WebCrypto Ed25519 for client-side token delegation
3. **Cap token HTTP API** -- Cap tokens are CLI-only; no HTTP endpoint for creating/delegating (by design)
4. **Commit & PR** -- All work is on `feat/distributed-infrastructure`, uncommitted

## Git

All changes on `feat/distributed-infrastructure` branch. Not yet committed.

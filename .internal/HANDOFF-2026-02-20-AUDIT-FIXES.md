# Handoff - 2026-02-20: Audit Fixes & Integration Hardening

## Overview

After the 5-phase distributed evolution (see `HANDOFF-2026-02-20-DISTRIBUTED.md`), a post-implementation audit identified 5 actionable issues — 2 critical (security hole, potential panic) and 3 moderate (incomplete features). All 5 were fixed in this session, plus documentation updates.

## What Was Fixed

### Fix 1: Registry REST API Auth Middleware (CRITICAL)

**Problem:** `/api/entities` endpoints were completely unprotected. Anyone reaching the auth port could create/delete entities without authentication.

**Fix:** Added `AdminToken` extractor to `deploy/relay/src/registry.rs` that validates a `Bearer` token from the `Authorization` header against `CpskValidator`, requiring `admin:/**` scope. Added to all 5 handlers (create, list, get, delete, update_status). `RegistryState::new()` now takes `Arc<CpskValidator>`. The `cpsk_validator` Arc is cloned before passing to both `AuthState` and `RegistryState`.

**Files:** `deploy/relay/src/registry.rs`, `deploy/relay/src/main.rs`

### Fix 2: Journal Recovery block_on -> .await (CRITICAL)

**Problem:** `Handle::block_on()` was used inside a `#[tokio::main]` async context to call `recover_from_journal().await`. This can panic in some Tokio runtime configurations and is an anti-pattern.

**Fix:** Replaced with direct `.await` since we're already in an async main function. Removed the temporary `state` binding and `Handle::current()` call.

**File:** `deploy/relay/src/main.rs` (lines 322-337 became 322-333)

### Fix 3: OnInterval Rule Timer Tasks (MODERATE)

**Problem:** Interval rules were collected but never spawned. `OnInterval` triggers could never fire because `Trigger::matches()` returns false for them (they don't match on address/signal_type).

**Fix (3 parts):**
1. Added `evaluate_interval()` method to `RulesEngine` (68 lines) — fires a rule by ID, checking enabled/cooldown/conditions but skipping address matching. Uses `Value::Null` for `SetFromTrigger` transforms since there's no trigger value. Origin tagged as `"interval:{rule_id}"`.
2. Made `execute_rule_actions` public in `clasp-router/src/router.rs` and exported from `clasp-router/src/lib.rs` (behind `#[cfg(feature = "rules")]`).
3. In `main.rs`, hoisted `interval_rules` Vec outside the cfg block, then after `shared_state()` spawns a `tokio::interval` task per interval rule that calls `evaluate_interval` + `execute_rule_actions` in a loop.

**Files:** `crates/clasp-rules/src/engine.rs`, `crates/clasp-router/src/router.rs`, `crates/clasp-router/src/lib.rs`, `deploy/relay/src/main.rs`

**Tests added:** 4 new tests — `test_evaluate_interval`, `test_evaluate_interval_with_condition`, `test_evaluate_interval_disabled`, `test_evaluate_interval_nonexistent`. Total: 23 passing.

### Fix 4: SetFromTrigger Subscriber Broadcast (MODERATE)

**Problem:** The `SetFromTrigger` handler in `execute_rule_actions` updated state silently — it called `state.set()` but never encoded a SET message or broadcast to subscribers, unlike the `Set` action handler right above it.

**Fix:** Replaced the minimal handler with full broadcast logic matching the `Set` pattern: `state.set()` -> encode `Message::Set` -> `find_subscribers` -> `try_send_with_drop_tracking_sync` to each.

**File:** `crates/clasp-router/src/router.rs` (lines 2252-2260)

### Fix 5: Federation RemotePublish/RemoteSet Broadcast (MODERATE)

**Problem:** `run_federation_leaf()` applied `RemoteSet` to local state but never broadcast to local subscribers. `RemotePublish` was logged and ignored entirely.

**Fix:** Updated `run_federation_leaf()` signature to accept `sessions: Arc<DashMap<SessionId, Arc<Session>>>` and `subscriptions: Arc<SubscriptionManager>`. `RemoteSet` now encodes and broadcasts after applying. `RemotePublish` extracts the address and signal type, finds subscribers, and forwards. Added `dashmap = "5.5"` to relay Cargo.toml.

**Files:** `deploy/relay/src/federation.rs`, `deploy/relay/src/main.rs`, `deploy/relay/Cargo.toml`

### Fix 6: Documentation Updates

**README.md** — Full rewrite of `deploy/relay/README.md`. Added all CLI flags grouped by feature, features table with cargo feature flags, security section documenting CPSK/capability/entity auth, registry REST API section with endpoints and curl examples, usage examples for common feature combos.

**HANDOFF doc** — Updated `HANDOFF-2026-02-20-DISTRIBUTED.md` "What's NOT Done" section to reflect completed integration work. Reduced remaining items to: hub/mesh federation, frontend cap support, integration tests.

## Practical Status Assessment

### What's Fully Functional

| Feature | Status | Notes |
|---------|--------|-------|
| Open relay (no auth) | Working | Default mode, unchanged |
| CPSK auth | Working | register/login/guest -> token -> WebSocket HELLO |
| Token scoping | Working | `action:pattern` enforcement on every message |
| Journal persistence | Working | SQLite or in-memory, recovery on restart |
| Rules engine (OnChange) | Working | Triggers on param changes, broadcasts to subscribers |
| Rules engine (OnThreshold) | Working | Numeric threshold crossing detection |
| Rules engine (OnEvent) | Working | Triggers on published events |
| Rules engine (OnInterval) | Working | Timer tasks now spawned (Fix 3) |
| SetFromTrigger broadcast | Working | Subscribers now notified (Fix 4) |
| Registry REST CRUD | Working | Admin-only, secured with Bearer token (Fix 1) |
| Federation leaf (broadcast) | Working | RemoteSet + RemotePublish forwarded to local subscribers (Fix 5) |

### What Validates But Lacks Admin Tooling

| Token Type | Prefix | Validation | Generation | Gap |
|-----------|--------|------------|------------|-----|
| CPSK | `cpsk_` | Working | Working (auth HTTP API) | None |
| Capability | `cap_` | Working (if `--trust-anchor`) | **No API/CLI** | Need admin tool or endpoint |
| Entity | `ent_` | Working (if `--registry-db`) | **No API/CLI** | Need `POST /api/entities/{id}/token` |

### What's Not Implemented

1. **Hub/mesh federation** — Only leaf mode exists. No inbound federation handling.
2. **Cap token admin tooling** — No way to create root cap tokens or delegate without writing Rust.
3. **Entity token generation API** — Registry CRUD exists but no token minting endpoint.
4. **Frontend cap delegation** — JS SDK passes tokens as opaque strings (works), but no client-side Ed25519 delegation.
5. **Integration tests** — No cross-crate tests (federation sync, cap chain validation through router, rules -> subscriber, journal replay after crash).

### Auth Flow (How Clients Work Today)

```
1. Client -> POST /auth/login {username, password}
2. Server -> {token: "cpsk_...", user_id: "...", username: "..."}
3. Client stores token in localStorage
4. Client -> WebSocket connect
5. Client -> HELLO {token: "cpsk_...", name: "...", features: [...]}
6. Router validates token via CpskValidator
7. Router -> WELCOME {session: "...", features: [...]}
8. Session enforces scopes on every subsequent SET/SUBSCRIBE/PUBLISH
```

Cap tokens would follow the same flow (steps 5-7) but the token would be `cap_...` instead and validated by `CapabilityValidator` in the chain. Entity tokens same with `ent_...`.

## Verification Results

- `cargo check --workspace` — clean
- `cargo test -p clasp-rules` — 23/23 pass (4 new)
- `cargo test -p clasp-core` — all pass
- `cd deploy/relay && cargo build --features full` — compiles
- `cd deploy/relay && cargo build` — default features compile
- `cd apps/chat && npx vite build` — builds

## Files Modified

| File | Fixes |
|------|-------|
| `deploy/relay/src/registry.rs` | 1 (AdminToken extractor + all handlers) |
| `deploy/relay/src/main.rs` | 1, 2, 3, 5 (validator pass, .await fix, interval tasks, federation refs) |
| `deploy/relay/src/federation.rs` | 5 (sessions + subscriptions params, broadcast) |
| `deploy/relay/Cargo.toml` | 5 (dashmap dependency) |
| `crates/clasp-rules/src/engine.rs` | 3 (evaluate_interval + 4 tests) |
| `crates/clasp-router/src/router.rs` | 3, 4 (pub execute_rule_actions, SetFromTrigger broadcast) |
| `crates/clasp-router/src/lib.rs` | 3 (export execute_rule_actions) |
| `deploy/relay/README.md` | 6 (full rewrite) |
| `.internal/HANDOFF-2026-02-20-DISTRIBUTED.md` | 6 (updated remaining work) |

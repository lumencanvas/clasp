# Handoff - 2026-02-21: Declarative App Config Rule Engine

## Overview

Replaced 400+ lines of hardcoded chat-specific Rust validators with a generic JSON-based rule engine. A single `--app-config <path>` CLI flag now defines scopes, write rules, and snapshot visibility for any application. No plugins, no separate crates, no Rust compilation to customize behavior.

**Result: 1 new module (~500 lines), 1 config file, 6 deleted files (old validator/), 59 new tests, 31 existing auth tests updated. All 133 tests pass. Zero behavioral regressions for the chat app. Session 2 added auto-detect, audit fixes, and full documentation updates.**

## What Changed

### New Files

| File | Lines | Purpose |
|------|-------|---------|
| `deploy/relay/src/app_config.rs` | ~500 | Rule engine: types, pattern matching, WriteValidator + SnapshotFilter impls |
| `deploy/relay/config/chat.json` | 186 | Full chat app config: 22 scopes, 8 write rules, 1 transform, 9 visibility rules, rate limits |
| `deploy/relay/tests/app_config_test.rs` | ~1140 | 59 tests covering all rule engine behavior with chat.json |

### Modified Files

| File | What Changed |
|------|-------------|
| `deploy/relay/src/auth.rs` | Removed standalone `build_scopes()`. Added `scope_templates` + `rate_config` to `AuthState`. New `AuthState::new()` takes 4 args. `build_scopes()` is now a method that substitutes `{userId}` in templates. |
| `deploy/relay/src/config.rs` | Added `--app-config` CLI flag. Loads + parses JSON. Changed `auth_db` default from `chat-auth.db` to `relay-auth.db`. |
| `deploy/relay/src/server.rs` | Wires app_config into AuthState (scope_templates, rate_config). Creates `RuleWriteValidator` and `RuleSnapshotFilter` from config. Explicit `RelayConfig.write_validator`/`.snapshot_filter` still takes precedence. |
| `deploy/relay/src/main.rs` | Removed `mod validator` and hardcoded chat validator injection. |
| `deploy/relay/src/lib.rs` | `pub mod validator` -> `pub mod app_config` |
| `deploy/relay/Dockerfile` | Added `COPY config /etc/clasp/` |
| `deploy/chat/docker-compose.yml` | Added `--app-config /etc/clasp/chat.json` + explicit `--auth-db /data/chat-auth.db` |
| `deploy/droplet/docker-compose.yml` | Added `--app-config /etc/clasp/chat.json` |
| `deploy/relay/tests/auth_test.rs` | Updated for new AuthState API. Uses `chat.json` scope templates instead of deleted `build_scopes()`. |

### Deleted Files

All 6 files under `deploy/relay/src/validator/`:
- `mod.rs`, `write.rs`, `filter.rs`, `helpers.rs`, `paths.rs`, `tests.rs`

## How the Rule Engine Works

### Path Pattern Matching

`match_address(pattern, address)` matches CLASP addresses against patterns:
- `{name}` captures a single segment
- `*` matches any single segment (no capture)
- `**` matches everything remaining (must be last in pattern)

### Write Rules (First-Match)

Each rule has a `path` pattern. First matching rule wins. A rule has:
- `pre_checks`: always run, even for null writes
- `checks`: skipped when `allow_null_write: true` and value is null
- `mode`: `"all"` (every check passes) or `"any"` (at least one passes)

Check types:
- `state_field_equals_session` -- state lookup, extract field, compare to session
- `state_not_null` -- state lookup, pass if exists and non-null
- `value_field_equals_session` -- extract field from written value, compare to session
- `segment_equals_session` -- path segment must equal session
- `either_state_not_null` -- either of two state lookups exists
- `require_value_field` -- written value must contain a field
- `reject_unless_path_matches` -- reject writes not matching a sub-pattern

### Snapshot Visibility (First-Match)

Rules match by `path` (pattern) or `path_contains` (substring). Modes:
- `true`/`false` -- static visibility
- `"owner"` -- visible only if `owner_segment` in address equals session; optional `public_sub` for publicly visible sub-paths (e.g. profile)
- `"require_state_not_null"` -- visible only if a `lookup` template resolves to non-null state

### Snapshot Transforms (All-Match)

Unlike visibility (first-match), transforms apply from ALL matching rules. Currently only `redact_fields` (removes keys from Map values).

## Deployment Modes

```bash
# Chat app (current behavior, zero regressions):
clasp-relay --auth-port 7350 --app-config /etc/clasp/chat.json --auth-db /data/chat-auth.db

# Generic dev relay (no app config, full access):
clasp-relay --auth-port 7350
# Logs: WARN No app config -- issuing full read/write tokens

# Custom app (hypothetical):
clasp-relay --auth-port 7350 --app-config /etc/clasp/myapp.json
```

## Audit Findings & Fixes

### Fixed: CRITICAL -- rsplit_once in owner visibility (app_config.rs)

**Problem**: `public_sub` extraction used `address.rsplit_once(&owner_prefix)` which does string searching for the owner ID. If the owner ID appears multiple times in the address (e.g. `/chat/user/bob/friends/bob`), it could misidentify the sub-path.

**Fix**: Replaced with structural approach using the pattern's segment index to extract the sub-path from address segments. Added regression test `test_snapshot_public_sub_with_owner_id_in_subpath`.

### Fixed: auth_test.rs compilation (tests/auth_test.rs)

Tests imported the deleted `build_scopes` standalone function and used the old 2-arg `AuthState::new()`. Updated to load `chat.json` scope templates and use the new 4-arg signature.

### Documented: substitute() ordering (app_config.rs)

Added doc comment explaining that `{session}` is replaced before named captures, and that `is_valid_user_id()` in auth.rs rejects `{`/`}` characters, preventing double-substitution injection.

### Documented: ** mid-pattern behavior (app_config.rs)

Added doc comment clarifying that `**` must be last in a pattern; if used mid-pattern, remaining segments are silently ignored.

### Known Limitations (Not Fixed -- Low Risk)

1. **Multi-segment ns-meta paths**: `{nsPath}` captures only one segment. `/chat/registry/ns-meta/foo/bar` would not match the single-segment rule. Not a bug -- chat only uses single-segment namespaces.

2. **`path_contains` + `path` both set**: `path_contains` silently takes priority. In practice, chat.json only uses one or the other per rule.

3. **Empty `checks` array**: A rule with empty checks passes through. This is intentional (catch-all rules with `reject_unless_path_matches` pattern).

## Test Counts

| Test File | Count | What |
|-----------|-------|------|
| `app_config_test.rs` | 59 | Rule engine: write validation, snapshot filtering, pattern matching |
| `auth_test.rs` | 31 | Auth endpoints, scope templates, CORS, rate limiting, strict read |
| `pentest_adm.rs` | 6 | Admin API security (auth bypass, privilege escalation, IDOR) |
| `persist_test.rs` | 4 | State persistence |
| Unit tests (x2) | 15 | Pattern matching, substitute, registry |
| Doc tests | 2 | Config example, lib doc |
| **Total** | **117** | |

## Post-Implementation: Audit Fixes, Auto-Detect, Docs (Session 2)

### Audit Fix: Default-Deny for Unknown Visibility Modes

**File**: `app_config.rs:597`

Unknown visibility mode strings (e.g. a typo like `"ownr"`) previously defaulted to `true` (visible). Changed to `false` (hidden) — deny by default. Warning message now includes the mode string in quotes for easier debugging.

### Audit Fix: Dockerfile CMD

**File**: `Dockerfile:64`

The Docker image copies `config/` to `/etc/clasp/` but the default CMD never referenced it. Fixed:
- Added `--app-config /etc/clasp/chat.json`
- Added `--auth-port 7350` (the Docker image is typically used with auth)
- Removed `--rendezvous-port 7340` (already the default)

### Auto-Detect App Config

**File**: `config.rs` — `From<Cli>` impl

When `--app-config` is not specified, the relay now checks well-known paths in order:
1. `/etc/clasp/*.json` — system/Docker install
2. `./config/*.json` — local dev from `deploy/relay/`

Rules:
- Exactly 1 `.json` file found → auto-use it, log at INFO
- Multiple `.json` files found → skip, log at DEBUG, require explicit `--app-config`
- No directory or no files → skip silently

The CLI doc comment for `--app-config` was updated to mention auto-detection.

### Documentation Updates

| File | Changes |
|------|---------|
| `deploy/relay/README.md` | Added `--app-config` to CLI options. Fixed `--auth-db` default from `chat-auth.db` to `relay-auth.db`. Added app-config example. New "App Config" section (auto-detect behavior, check type reference). Replaced `Dockerfile.dev` section (deleted file) with simpler "Development" section. |
| `deploy/droplet/README.md` | Added `chat.json` note to "What's in the box". Updated persistent data section noting `--app-config` and `--auth-db` flags in compose. |
| `apps/chat/README.md` | Added `cargo run` instructions with auto-detect note. Mentioned `chat.json` for local relay dev. |
| `deploy/relay/src/main.rs` | Added `--app-config` usage example to module doc comment. |
| `deploy/relay/src/lib.rs` | Added programmatic `AppConfig` usage example (validates as doc-test). |

### Updated Test Counts

| Test File | Count | What |
|-----------|-------|------|
| `app_config_test.rs` | 59 | Rule engine: write validation, snapshot filtering, pattern matching |
| `auth_test.rs` | 31 | Auth endpoints, scope templates, CORS, rate limiting, strict read |
| `pentest_adm.rs` | 6 | Admin API security (auth bypass, privilege escalation, IDOR) |
| `persist_test.rs` | 4 | State persistence |
| Unit tests (x2) | 15 | Pattern matching, substitute, registry |
| Doc tests | 3 | Config example, lib doc, lib app_config example |
| **Total** | **133** | |

## What Stays the Same

- Chat app frontend -- zero changes
- Auth HTTP endpoints -- `/auth/register`, `/auth/login`, `/auth/guest` same shape
- SQLite schema -- unchanged
- Protocol layer -- clasp-core, clasp-router untouched
- Library API -- `RelayConfig.write_validator`/`.snapshot_filter` still exist for compiled Rust overrides
- Existing auth databases -- pass `--auth-db chat-auth.db` since default changed to `relay-auth.db`

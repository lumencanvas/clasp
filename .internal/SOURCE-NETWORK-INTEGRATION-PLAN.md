# Source Network Integration Plan for CLASP

> **Status**: Revised draft -- verified against real APIs, cross-referenced with existing codebase
> **Date**: 2026-03-28
> **Scope**: Beyond DefraDB -- integrating LensVM, OpenPubKey, ACP/Zanzi, Orbis, SourceHub into the CLASP Bridge

---

## Verification Summary

Each technology was verified against package registries, GitHub repos, and official docs before planning. Findings that contradict or refine the original plan are marked with `[VERIFIED]`, `[CORRECTED]`, or `[FLAGGED]` throughout.

### Source Network npm Packages (Verified 2026-03-28)

Only **3 packages** exist under the `@sourcenetwork` scope on npm:

| Package | Version | Published | Purpose |
|---------|---------|-----------|---------|
| `@sourcenetwork/acp-js` | 1.0.5 | 2025-12-09 | ACP client for SourceHub (policy CRUD, relationships, access verification) |
| `@sourcenetwork/wasm-bridge` | 1.0.0 | 2025-12-09 | Go-compiled WASM loader (bundles `wasm_exec.js`; **NOT for Rust WASM**) |
| `@sourcenetwork/hublet` | 1.0.2 | 2025-08-01 | SourceHub wallet (secp256r1/P-256, Cosmos tx signing, bech32 addresses) |

**NOT on npm** (despite references in various docs):
- `@sourcenetwork/openpubkey-js` -- public GitHub repo exists (v0.3.3), never published to npm
- `@sourcenetwork/sourcehub-js` -- does not exist anywhere

**Key architectural detail**: `acp-js` depends on `hublet`, which depends on `@cosmjs/encoding`, `@noble/curves`, `ts-proto`, `grpc-web`. Despite appearances, `acp-js` has only ONE mode:

**SourceHub chain mode only** (`SourceHubACP` class in `client.ts`): ALL operations (addPolicy, verifyAccess, setRelationship, etc.) talk to a running SourceHub node via REST/gRPC. The `bridge.ts` file sets up `window.acp_*` convenience globals, but these are thin wrappers around `client-helpers.ts` which creates a `SourceHubACP` client pointing at `localhost:1317/26657/9090` by default. There is no local-only evaluation path in this package.

**CORRECTION from earlier analysis**: The `wasm-bridge` re-export in `wasm.ts` is a utility for loading Go-compiled WASM modules, but `bridge.ts` does NOT use it. The `window.acp_*` functions all make HTTP/gRPC calls to SourceHub. The separate `acp_core` repo has a Go WASM playground target (`cmd/playground_js/main.go`) that CAN run locally, but it exposes a different sandbox-based API (`AcpPlayground.new()` with `NewSandbox`, `Simulate`, `VerifyTheorems`, etc.) -- not the same as the `acp_AddPolicy`/`acp_VerifyAccessRequest` interface. And the playground WASM binary is not distributed; it must be compiled from Go source (`make playground:wasm_js`).

**What this means for CLASP**: The only local-only ACP path is **DefraDB's built-in ACP** via `@policy` directives. Using `acp-js` requires a running SourceHub node. For CLASP, the DefraDB path is clearly correct.

**Crypto curve mismatch**: `hublet` uses secp256r1 (P-256/NIST), while CLASP uses Ed25519 (via `clasp-identity`). These are different curves. SourceHub addresses are Cosmos-style bech32 (`source1...`), not CLASP DIDs (`did:key:z6Mk...`). Integration would need an identity bridge or dual-key approach. This mismatch does NOT affect the DefraDB path (DefraDB uses its own identity model).

### Integration Status Table

| # | Integration | Status | Package Available | API Verified |
|---|------------|--------|-------------------|-------------|
| 1 | LensVM WASM Transforms | Real, stable | `lens_sdk` v0.8.1 on crates.io | Yes |
| 2 | ACP/Zanzi Authorization | Real, documented | DefraDB built-in (no npm pkg needed); `@sourcenetwork/acp-js` v1.0.5 for SourceHub only | Yes |
| 3 | OpenPubKey Identity | Real, unpublished | **NOT on npm** -- GitHub only (public repo, v0.3.3) | Yes (from repo) |
| 4 | Orbis Encrypted Routing | Real, early stage | Go-only, no JS/Rust package | Partially |
| 5 | SourceHub Audit Trail | Real, testnet only | `@sourcenetwork/hublet` v1.0.2 on npm (wallet only, no audit client) | Partially |

---

## Existing Infrastructure (What CLASP Already Has)

**The original plan overlooked significant existing infrastructure.** Before adding new systems, understand what is already built:

| Crate | What It Does | Relevance |
|-------|-------------|-----------|
| `clasp-identity` | Ed25519 keypair generating EntityId + DID (`did:key:z6Mk...`) + libp2p PeerID | Already has DID support |
| `clasp-crypto` | AES-256-GCM, ECDH P-256, HKDF, ECDSA, E2E session protocol | Already has E2E encryption |
| `clasp-caps` | UCAN-style delegatable capability tokens (`cap_<base64url>`) | Already has fine-grained auth |
| `clasp-bridge/transform.rs` | Expression engine, curves, aggregation, conditionals, JSON path (~280 LOC) | Already has Rust transforms |
| `clasp-journal` + `clasp-journal-defra` | Append-only journaling with DefraDB persistence | Already has audit trail |
| `clasp-config-defra/schema.rs` | GraphQL SDL schemas with `owner` field on every document | Already has ownership model |

Each integration below is evaluated against this existing foundation. Where existing infrastructure already covers the use case, the integration is downgraded or redesigned as a complement rather than a replacement.

### Extension Architecture

All Source Network integrations follow CLASP's existing optional feature pattern:

**Rust side**: Cargo feature flags with `#[cfg(feature = "...")]` conditional compilation
```toml
# In clasp-bridge/Cargo.toml
[features]
lens = ["clasp-lens"]  # Same pattern as osc = ["rosc"], midi = ["midir"], etc.

[dependencies]
clasp-lens = { workspace = true, optional = true }
```

**Electron side**: Optional IPC handlers registered only when the feature is active
```javascript
// electron/preload.js -- same pattern as defraHealthCheck, defraConfigExport
lensLoad: (wasmBytes) => ipcRenderer.invoke('lens-load', wasmBytes),
lensTransform: (id, value) => ipcRenderer.invoke('lens-transform', id, value),
```

**Vue side**: Optional chaining on `window.clasp?.featureApi` in composables
```typescript
// Same pattern as useDefra.ts -- graceful when API not available
const api = (window as any).clasp
if (api?.lensLoad) {
    await api.lensLoad(wasmBytes)
}
```

This ensures each integration is fully optional, compilable in or out, and never affects core functionality.

---

## Priority Ranking (Revised)

| # | Integration | Replaces/Augments | Effort | Impact | Confidence |
|---|------------|-------------------|--------|--------|------------|
| 1 | **LensVM WASM Transforms** | Built-in transform types | 3-4 weeks | High -- unlimited extensibility | High |
| 2 | **ACP/Zanzi Authorization** | SecurityPanel for DefraDB users | 2-3 weeks | High -- federated access control | High |
| 3 | **OpenPubKey Identity** | None (complements clasp-identity) | 2-3 weeks | Medium -- OIDC identity binding | Medium |
| 4 | **SourceHub Audit Trail** | clasp-journal for federated deployments | 2 weeks | Low -- niche (federated + compliance) | Low |
| 5 | **Orbis Encrypted Routing** | clasp-crypto E2E for PRE use cases | 1 week prototype | Low -- speculative, niche | Low |

**Changes from original plan:**
- ACP moved up to #2 (DefraDB already integrated, lower risk than OpenPubKey)
- OpenPubKey moved to #3 (npm availability issue, existing identity system covers most needs)
- Orbis and SourceHub swapped (SourceHub is more concrete)
- Confidence column added to reflect verification results

---

## 1. LensVM WASM Transforms

### Verification Status

- [VERIFIED] `lens_sdk` v0.8.1 on crates.io (57k downloads, published by Source Devs)
- [VERIFIED] GitHub: `github.com/sourcenetwork/lens` (v0.10.0, 62 commits)
- [VERIFIED] WASM exports: `alloc(u64)`, `transform()`, `inverse()`, `set_param()` -- all confirmed
- [VERIFIED] WASM import: `next()` from `"lens"` module -- confirmed
- [VERIFIED] Transport buffer format: `[TypeId i8][Length u32 LE][Payload]` with types -1/0/1/127 -- confirmed
- [VERIFIED] Go host exists in `host-go/` directory -- confirmed, not usable in our stack
- [CLARIFICATION] `@sourcenetwork/wasm-bridge` on npm is for **Go-compiled WASM** (bundles `wasm_exec.js`). It is NOT usable for Rust-compiled lens WASM modules. LensVM lenses target `wasm32-unknown-unknown` and need a standard WebAssembly host (wasmtime in Rust, `WebAssembly.instantiate()` in JS).

### What It Is

LensVM is a bidirectional WASM data transformation engine. Lenses are WASM modules with `transform()` and optional `inverse()` exports. The Rust SDK (`lens_sdk` on crates.io) provides macros to define lenses that compile to `wasm32-unknown-unknown`.

### Why It Matters

CLASP has two parallel transform systems:
- **Electron-side** (`apps/bridge/src/lib/transforms.ts`): 18 hardcoded types for UI preview and client-side transforms
- **Rust-side** (`crates/clasp-bridge/src/transform.rs`): Rich `Transform` enum with expression engine, curves, aggregation, conditionals, JSON path

Adding a new transform type currently requires touching 4+ files in the bridge app (types.ts, transforms.ts, constants.ts, SignalRouteModal.vue) plus the Rust enum if it should work server-side.

LensVM lets users author custom transforms as WASM modules and hot-load them. Benefits:
- Infinite extensibility without touching CLASP source
- Sandboxed execution (untrusted WASM can't crash the bridge or access the filesystem)
- Bidirectional transforms (`transform` + `inverse`) map to signal encoding/decoding
- Same WASM module runs in both Rust (via wasmtime) and browser (via WebAssembly API)

### Technical Details

#### WASM Module Interface

Every lens WASM module exports:
```
alloc(size: u64) -> *mut u8        # Host allocates memory for input
transform() -> *mut u8             # Execute forward transform
inverse() -> *mut u8               # Execute reverse transform (optional)
set_param(ptr: u8) -> *mut u8      # Receive static config (optional)
```

And imports from `"lens"` module:
```
next() -> *mut u8                  # Pull next input item from host
```

#### Transport Buffer Format (crossing WASM boundary)
```
[TypeId: i8] [Length: u32 LE] [Payload: bytes]

TypeId values:
  -1 = error (payload is error string)
   0 = nil (no length/payload)
   1 = JSON item (payload is JSON bytes)
 127 = end of stream (no length/payload)
```

#### Writing a Signal Transform in Rust
```rust
use lens_sdk::StreamOption;
use serde::{Serialize, Deserialize};

lens_sdk::define!(PARAMS: TransformParams, try_transform, try_inverse);

#[derive(Serialize, Deserialize)]
struct TransformParams {
    scale_factor: f64,
    offset: f64,
}

#[derive(Serialize, Deserialize)]
struct SignalValue {
    value: f64,
}

fn try_transform(
    iter: &mut dyn Iterator<Item = lens_sdk::Result<Option<SignalValue>>>,
) -> Result<StreamOption<SignalValue>, Box<dyn std::error::Error>> {
    let params = PARAMS.read().unwrap();
    let p = params.as_ref().unwrap();

    match iter.next() {
        Some(Ok(Some(input))) => {
            Ok(StreamOption::Some(SignalValue {
                value: input.value * p.scale_factor + p.offset,
            }))
        }
        _ => Ok(StreamOption::EndOfStream),
    }
}
```

Compile: `cargo build --target wasm32-unknown-unknown --release`

### Architecture

```
                    Signal Route Pipeline
                    ---------------------

  Signal -->  matchesSource() --> extractValue()
                  |                    |
                  v                    v
              Transform Dispatcher
              |
              type === 'wasm'?
                YES -> WasmTransformHost
                NO  -> applyTransform() (existing built-in types)
              |
              v
          api.sendSignal(target)

  WasmTransformHost:
    1. set_param(config JSON)
    2. alloc(size) + write input
    3. Implement next() import
    4. Call transform()
    5. Read transport buffer
    6. Deserialize output JSON

  Integration points:
    Rust:  crates/clasp-bridge/src/transform.rs (add Transform::Wasm variant)
    JS:    apps/bridge/src/lib/transforms.ts (add 'wasm' case in applyTransform)
```

### Implementation Plan

#### Phase 1: Rust-side WASM Host (crate: `clasp-lens`)
- [ ] New crate `clasp-lens` depending on `wasmtime` and `lens_sdk` (for test fixtures)
- [ ] Implement LensVM host protocol: alloc, next, transform, set_param
- [ ] Transport buffer serializer/deserializer
- [ ] `WasmTransform` struct: load .wasm bytes, configure params, execute transform
- [ ] Add `Transform::Wasm { module: Vec<u8>, params: serde_json::Value }` variant to `clasp-bridge/src/transform.rs`
- [ ] Wire into existing transform dispatch in `clasp-bridge`

#### Phase 2: Electron-side WASM Host (JS)
- [ ] `apps/bridge/src/lib/wasm-transform.ts` -- loads WASM via `WebAssembly.instantiate()`
- [ ] Implements same host protocol as Rust side (alloc, next, set_param, transform)
- [ ] `WasmTransformPool` -- caches compiled modules, manages instances
- [ ] Wire into `applyTransform()` in `transforms.ts` for `type: 'wasm'`

#### Phase 3: UI Integration
- [ ] Add `'wasm'` to `TransformType` union in `types.ts`
- [ ] Add `wasmModule?: ArrayBuffer` and `wasmParams?: Record<string, unknown>` to `TransformConfig`
- [ ] Upload UI in SignalRouteModal.vue -- file picker for .wasm, JSON editor for params
- [ ] Preview still works (runs transform with test value via browser WASM)
- [ ] Badge: `formatTransformBadge` returns `"WASM"` or module name

#### Phase 4: Bundled Lens Library
- [ ] Ship 5-10 pre-built lenses as .wasm files bundled with the app:
  - Butterworth low-pass filter
  - Exponential decay envelope
  - Hysteresis / Schmitt trigger
  - PID controller
  - Bezier curve interpolator
  - Moving average (configurable window)
  - Signal gate with attack/release
  - Frequency detector (zero-crossing)
- [ ] `clasp-lens-template` Rust crate with CI that builds .wasm
- [ ] Store user lenses in DefraDB (binary blob in collection)

### Testing Strategy

#### Unit Tests (`crates/clasp-lens/tests/`)
```rust
// test_basic_transform.rs -- compile a sample lens, load, verify output
#[test]
fn test_scale_lens_produces_correct_output() {
    // 1. Load pre-compiled test lens .wasm from fixtures/
    let wasm_bytes = include_bytes!("fixtures/scale_lens.wasm");
    let host = WasmTransformHost::new(wasm_bytes).unwrap();

    // 2. Configure params
    host.set_params(json!({ "scale_factor": 2.0, "offset": 0.5 })).unwrap();

    // 3. Transform a value
    let result = host.transform(json!({ "value": 0.25 })).unwrap();
    assert_eq!(result["value"], 1.0); // 0.25 * 2.0 + 0.5
}

#[test]
fn test_inverse_transform_round_trips() {
    let wasm_bytes = include_bytes!("fixtures/scale_lens.wasm");
    let host = WasmTransformHost::new(wasm_bytes).unwrap();
    host.set_params(json!({ "scale_factor": 2.0, "offset": 0.5 })).unwrap();

    let original = json!({ "value": 0.7 });
    let transformed = host.transform(original.clone()).unwrap();
    let restored = host.inverse(transformed).unwrap();
    assert!((restored["value"].as_f64().unwrap() - 0.7).abs() < 1e-10);
}

#[test]
fn test_malformed_wasm_returns_error_not_panic() {
    let bad_bytes = b"not a wasm module";
    let result = WasmTransformHost::new(bad_bytes);
    assert!(result.is_err());
}

#[test]
fn test_wasm_trap_does_not_crash_host() {
    let wasm_bytes = include_bytes!("fixtures/panicking_lens.wasm");
    let host = WasmTransformHost::new(wasm_bytes).unwrap();
    let result = host.transform(json!({ "value": 0.5 }));
    assert!(result.is_err()); // Error, not panic
}
```

#### Integration Tests (`crates/clasp-lens/tests/integration/`)
```rust
// test_wasm_in_signal_pipeline.rs -- real router + WASM transform
#[tokio::test]
async fn test_wasm_transform_in_live_pipeline() {
    // 1. Start a real clasp-router
    let router = clasp_test_utils::start_router(RouterConfig::default()).await;

    // 2. Create a bridge with a WASM transform route
    let wasm_bytes = include_bytes!("fixtures/scale_lens.wasm");
    let bridge = clasp_bridge::BridgeBuilder::new()
        .source("osc", "/test/input")
        .target("osc", "/test/output")
        .transform(Transform::Wasm {
            module: wasm_bytes.to_vec(),
            params: json!({ "scale_factor": 3.0, "offset": 0.0 }),
        })
        .build()
        .await;

    // 3. Send a signal and verify transformed output
    let client = clasp_client::Clasp::connect(&router.address()).await.unwrap();
    client.publish("/test/input", 0.5).await.unwrap();

    // 4. Receive and verify
    let received = client.subscribe("/test/output").next().await.unwrap();
    assert_eq!(received.value(), 1.5); // 0.5 * 3.0
}
```

#### Benchmark Tests (`crates/clasp-lens/benches/`)
```rust
// bench_wasm_transform.rs -- latency comparison
fn bench_wasm_vs_builtin(c: &mut Criterion) {
    let wasm_host = WasmTransformHost::new(include_bytes!("fixtures/scale_lens.wasm")).unwrap();
    wasm_host.set_params(json!({ "scale_factor": 2.0, "offset": 0.5 })).unwrap();

    let builtin = Transform::Scale {
        from_min: 0.0, from_max: 1.0,
        to_min: 0.5, to_max: 2.5,
    };

    c.bench_function("wasm_scale", |b| {
        b.iter(|| wasm_host.transform(json!({ "value": 0.5 })))
    });

    c.bench_function("builtin_scale", |b| {
        b.iter(|| builtin.apply(0.5))
    });
}
```

#### JS Tests (`apps/bridge/src/lib/__tests__/wasm-transform.test.ts`)
```typescript
import { describe, it, expect } from 'vitest'
import { WasmTransformHost } from '../wasm-transform'
import scaleWasm from '../../../fixtures/scale_lens.wasm?arraybuffer'

describe('WasmTransformHost', () => {
  it('transforms value through WASM module', async () => {
    const host = await WasmTransformHost.load(scaleWasm)
    host.setParams({ scale_factor: 2.0, offset: 0.5 })
    const result = host.transform({ value: 0.25 })
    expect(result.value).toBeCloseTo(1.0)
  })

  it('caches compiled modules for reuse', async () => {
    const pool = new WasmTransformPool()
    const h1 = await pool.get(scaleWasm)
    const h2 = await pool.get(scaleWasm)
    // Same compiled module, different instances
    expect(h1).not.toBe(h2)
  })
})
```

### Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| WASM overhead per signal | Cache compiled modules; instantiate once per route, not per signal. Benchmark to establish overhead budget. |
| No official non-Go host | Transport buffer format is trivial; we implement our own host (~200 LOC). The 5-function ABI is stable. |
| Large .wasm files | Most lenses are <50KB; gzip in DefraDB storage. |
| User-authored WASM could panic | WASM traps are caught by wasmtime; fall back to passthrough on error. |
| lens_sdk API changes | Pin to specific version (0.8.x); the ABI is stable. |

### Documentation

New docs to create:
- `docs/transforms/wasm-transforms.md` -- User guide: what WASM transforms are, how to use them in the UI
- `docs/transforms/authoring-lenses.md` -- Developer guide: how to write a custom lens in Rust using lens_sdk
- `docs/transforms/lens-api-reference.md` -- Reference: WASM host protocol, transport buffer format, param schema
- Update `docs/core-features/transforms.md` to mention WASM as an option

---

## 2. ACP/Zanzi Authorization (via DefraDB)

### Verification Status

- [VERIFIED] DefraDB supports `@policy(id: "...", resource: "...")` directives on schema types
- [VERIFIED] Zanzi is a real project: `github.com/sourcenetwork/zanzi` (107 commits, Go)
- [VERIFIED] `@sourcenetwork/acp-js` v1.0.5 on npm -- full API surface inspected from package tarball
- [VERIFIED] `@sourcenetwork/wasm-bridge` v1.0.0 on npm -- Go-compiled WASM loader (utility, NOT used by acp-js bridge)
- [VERIFIED] Policy format uses YAML with resources/relations/permissions -- confirmed in docs

### Actual acp-js API (from npm package inspection)

The `@sourcenetwork/acp-js` package (212 files, 1.5MB) provides a **SourceHub chain client only**:

```typescript
import { SourceHubACP } from '@sourcenetwork/acp-js'

// Requires a running SourceHub node
const acp = await SourceHubACP.create(apiUrl, rpcUrl, grpcUrl, chainId)

await acp.addPolicy(policyYaml, marshalType)
await acp.registerObject(policyId, 'router_config', objectId)
await acp.addActorRelationship(policyId, 'router_config', objectId, 'operator', actorDid)
const allowed = await acp.verifyAccessRequest('write', actorId, policyId, 'router_config', objectId)
```

Default endpoints (from `client-helpers.ts`):
```typescript
{
  apiUrl: 'http://localhost:1317',    // Cosmos REST
  rpcUrl: 'http://localhost:26657',   // CometBFT RPC
  grpcUrl: 'http://localhost:9090',   // gRPC
  chainId: 'sourcehub-dev',
  denom: 'uopen',
}
```

**Dependency chain**: `acp-js` -> `@sourcenetwork/hublet` -> `@cosmjs/encoding`, `@noble/curves`, `@improbable-eng/grpc-web`, `ts-proto`, `bech32`, `bitcoinjs-lib`, `multiformats`. This is a heavy dependency tree for an Electron app.

**Proto definitions included**: Full protobuf types for `sourcehub.acp` and `sourcenetwork.acp_core` (policy, relationship, access decision, registration, etc.)

**Key takeaway**: `acp-js` is only useful if you run a SourceHub node. For CLASP, the DefraDB `@policy` path is the correct local-only approach.

### What It Is

DefraDB embeds Zanzi (a Zanzibar-style RelBAC engine) as its authorization layer. Schemas can be annotated with `@policy` directives that enforce relationship-based access control per-document, per-query. This is the primary integration path for CLASP.

For reference, there is also a Go WASM build of ACP Core (in the `acp_core` repo, `cmd/playground_js/`) that can evaluate policies locally in a browser sandbox. However, it uses a different API (`AcpPlayground.new()` with sandbox-based operations), must be compiled from Go source, and is not distributed as a package. It is relevant only as a future option, not for initial integration.

### Why This Moved Up to #2

- DefraDB is already integrated into CLASP (7 DefraDB crates, CI tests passing)
- `@policy` is a schema-level annotation -- adding it is low-risk
- `clasp-config-defra/src/schema.rs` already has `owner` fields on every document type
- No new npm dependencies needed for the DefraDB path
- `acp-js` is NOT needed for the DefraDB path (only needed for direct SourceHub interaction)

### Relationship to Existing Security

CLASP already has:
- **clasp-caps**: UCAN-style capability tokens for token-based auth (action:pattern scopes)
- **SecurityPanel**: Hand-rolled scopes, write rules, visibility rules
- **clasp-identity**: Ed25519 identity with DID support (ownership model)

ACP does NOT replace these. It adds document-level access control:
- clasp-caps handles token-based authorization (bearer tokens, delegation chains)
- ACP handles relationship-based authorization (who can read/write which documents)
- SecurityPanel stays as local-mode fallback when DefraDB is not enabled

### DefraDB Schema Changes

Current schemas in `clasp-config-defra/src/schema.rs` need `@policy` annotations:

```graphql
type ClaspRouterConfig @policy(
  id: "%local-policy-id%",
  resource: "router_config"
) {
    configId: String @index
    name: String
    host: String
    port: Int
    transports: [String]
    securityMode: String
    maxSessions: Int
    paramTtlSecs: Int
    features: [String]
    owner: String @index
    updatedAt: Int
    version: Int
}
```

### Policy Definition

```yaml
name: clasp-bridge-policy
description: Access control for CLASP Bridge resources

resources:
  - name: router_config
    relations:
      - name: owner
        types: []
      - name: operator
        types: ["actor"]
      - name: viewer
        types: ["actor", "group->member"]
    permissions:
      - name: read
        expr: "owner + operator + viewer"
      - name: write
        expr: "owner + operator"
      - name: delete
        expr: "owner"

  - name: signal_route
    relations:
      - name: owner
        types: []
      - name: editor
        types: ["actor"]
      - name: user
        types: ["actor", "group->member"]
    permissions:
      - name: read
        expr: "owner + editor + user"
      - name: write
        expr: "owner + editor"
      - name: execute
        expr: "owner + editor + user"

  - name: bridge
    relations:
      - name: owner
        types: []
      - name: operator
        types: ["actor"]
    permissions:
      - name: read
        expr: "owner + operator"
      - name: write
        expr: "owner + operator"
      - name: start_stop
        expr: "owner + operator"

  - name: lens_module
    relations:
      - name: author
        types: []
      - name: user
        types: ["actor", "group->member"]
    permissions:
      - name: read
        expr: "author + user"
      - name: execute
        expr: "author + user"
      - name: delete
        expr: "author"
```

### Implementation Plan

#### Phase 1: Schema and Policy Setup
- [ ] Register CLASP ACP policy with DefraDB (local Zanzi engine, no SourceHub needed)
- [ ] Add `@policy` directives to all schemas in `clasp-config-defra/src/schema.rs`
- [ ] Set `owner` field from `clasp-identity` DID on document creation
- [ ] Update GraphQL mutations in `clasp-config-defra/src/store.rs` to include identity context

#### Phase 2: Composable and UI
- [ ] New composable: `apps/bridge/src/composables/useAccessControl.ts`
  - `relationships: Ref<Relationship[]>` -- loaded from DefraDB
  - `grantAccess(resource, relation, actor)`, `revokeAccess(...)`
  - `checkAccess(resource, permission, actor) -> boolean`
- [ ] Extend SecurityPanel with relationship management:
  - "Share this router with..." dialog (adds relationship tuple)
  - Permission viewer: who can access what resources
  - Active only when DefraDB is connected (hidden otherwise)

#### Phase 3: Federated Policy Sync
- [ ] Policies stored in DefraDB replicate via P2P automatically
- [ ] Relationship tuples stored in DefraDB replicate via P2P
- [ ] Policy version tracking via DefraDB Merkle DAG commit history

### Testing Strategy

#### Integration Tests (Rust, requires DefraDB)
```rust
// tests/defra/crates/clasp-config-defra/tests/test_acp_policy.rs
#[tokio::test]
async fn test_policy_enforces_owner_only_write() {
    // 1. Start DefraDB with ACP enabled
    let defra = start_defra_with_acp().await;

    // 2. Provision schema with @policy directive
    defra.add_schema(ROUTER_CONFIG_SCHEMA_WITH_POLICY).await.unwrap();

    // 3. Create document as alice
    let alice = Identity::generate();
    defra.create_with_identity(&alice, router_config_doc()).await.unwrap();

    // 4. Attempt write as bob -- should fail
    let bob = Identity::generate();
    let result = defra.update_with_identity(&bob, router_config_doc()).await;
    assert!(result.is_err());

    // 5. Grant bob operator access
    defra.add_relationship("router_config", "operator", bob.did()).await.unwrap();

    // 6. Retry write as bob -- should succeed
    let result = defra.update_with_identity(&bob, router_config_doc()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_policy_replicates_across_peers() {
    // 1. Start two DefraDB nodes with P2P
    let (node_a, node_b) = start_defra_pair().await;

    // 2. Provision schema on node_a
    node_a.add_schema(ROUTER_CONFIG_SCHEMA_WITH_POLICY).await.unwrap();

    // 3. Create document with policy on node_a
    let alice = Identity::generate();
    node_a.create_with_identity(&alice, router_config_doc()).await.unwrap();

    // 4. Wait for replication
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 5. Verify document exists on node_b with same policy
    let doc = node_b.get_router_config(config_id).await.unwrap();
    assert!(doc.is_some());
}
```

### Migration Path

| Current Feature | ACP Equivalent | Migration |
|----------------|---------------|-----------|
| Scopes (path + action) | Policy permissions with path-based resources | Parallel operation |
| Write rules (require_auth) | Implicit -- ACP requires identity | Keep as local fallback |
| Write rules (owner_only) | Relationship check: `owner` relation | Replace when DefraDB active |
| Visibility rules | Permission check: `read` permission | Replace when DefraDB active |
| Rate limits | Keep as-is (orthogonal to authorization) | No change |
| Capability tokens (clasp-caps) | Complementary -- caps for bearer auth, ACP for document auth | Both active |

### Documentation

- `docs/auth/acp-authorization.md` -- User guide: what ACP is, how relationship-based access works
- `docs/auth/sharing-resources.md` -- User guide: sharing routers/routes with other users
- `docs/defra/acp-setup.md` -- Setup guide: enabling ACP in DefraDB, policy configuration
- Update `docs/auth/` index to reference ACP alongside existing CPSK and capability token docs

---

## 3. OpenPubKey Identity

### Verification Status

- [FLAGGED] `@sourcenetwork/openpubkey-js` is **NOT published on npm**. `npm view` returns 404. `npm search @sourcenetwork` lists only 3 packages (acp-js, wasm-bridge, hublet) -- openpubkey-js is not among them.
- [VERIFIED] Public GitHub repo: `github.com/sourcenetwork/openpubkey-js` (16 commits)
- [VERIFIED] `package.json` declares v0.3.3, ESM-only (`"type": "module"`), deps: `jose` + `@noble/hashes`
- [VERIFIED] Exports confirmed from `src/index.ts`:
  - Core: `OpkClient`, `OpkClientBrowser`, `PKToken`, `Verifier`, `Claims`
  - Providers: `GoogleOp`, `GoogleBrowserOp`, `GitHubOp`, `GitLabOp`, `GitLabCIOp`, `AzureOp`
  - GQ signatures: `gq256SignJWT`, `gq256VerifyJWT`
  - Token serialization: `compactPKToken`, `splitCompactPKToken`
  - Browser-specific: `newSignedMessageBrowser`, `verifySignedMessageBrowser`
- [VERIFIED] Node.js requires `>= 18.0.0`; browsers need Web Crypto API
- [VERIFIED] Supports Google, GitHub Actions, GitLab CI, Azure, generic OIDC

### Risk Assessment

The npm availability issue is the primary risk. Options:
1. **Install from GitHub**: `npm install github:sourcenetwork/openpubkey-js` -- works but pins to a git ref, no semver guarantees, ESM-only may conflict with Electron CJS requirements
2. **Vendor the package**: Copy source into `apps/bridge/lib/openpubkey/` -- full control but maintenance burden
3. **Wait for npm publish**: If Source Network publishes it, standard npm install -- unknown timeline

**Additional risk**: The package is ESM-only (`"type": "module"`). The bridge app uses `vite.config.mjs` (not .ts) specifically because Electron CJS does not support `"type": "module"`. The renderer (Vite) handles ESM fine, but importing in `electron/ipc/identity.js` (CJS) would require dynamic `import()` or a build step.

**Recommendation**: Start with GitHub install pinned to a commit hash. If the library proves stable, vendor it. If Source Network publishes to npm, switch to that.

### Why This Is #3 (Not #2)

- CLASP already has a working identity system (`clasp-identity` with Ed25519 + DID + PeerID)
- CLASP already has working auth (CPSK tokens + capability tokens)
- OpenPubKey adds *convenience* (Google/GitHub login) not *capability* (the crypto identity already exists)
- The npm availability issue adds integration risk
- ACP (#2) delivers more value per unit of effort because it builds on an already-integrated foundation (DefraDB)

### What It Adds (That CLASP Does Not Have)

OpenPubKey binds cryptographic keys to OIDC identities (Google, GitHub) without modifying the identity provider. A user logs in with Google, gets a PK Token containing their ephemeral public key signed by both Google and themselves.

What this enables that clasp-identity alone cannot:
- "Log in with Google" for bridge operators (no manual key management)
- Human-readable identity in federated deployments ("alice@gmail.com" vs "did:key:z6Mk...")
- Third-party verifiable identity (Google vouches for the user, not just a self-signed key)

### Architecture (Complement, Not Replace)

```
  OpenPubKey adds an OIDC binding layer ON TOP of clasp-identity:

  Google/GitHub OIDC --> OpkClient --> PK Token --> derive Ed25519 key
                                                        |
                                                        v
                                                  clasp-identity
                                                  (same DID, EntityId, PeerID)

  Users who don't want OIDC continue using locally generated identities.
  Users who log in with Google get the same identity types, backed by OIDC proof.
```

### Implementation Plan

#### Phase 1: Core Identity Layer
- [ ] Install: `npm install github:sourcenetwork/openpubkey-js#<commit-hash>`
- [ ] New IPC module: `electron/ipc/identity.js`
  - `opk-login` handler: instantiate GoogleOp + OpkClient, call auth(), return PK Token
  - `opk-logout` handler: clear stored token
  - `opk-sign` handler: sign arbitrary payload with PK Token
  - `opk-verify` handler: verify signed message
  - Store PK Token in `safeStorage` (Electron encrypted storage)
- [ ] Expose in `preload.js`: `clasp.login()`, `clasp.logout()`, `clasp.sign(payload)`, `clasp.verify(signed)`

#### Phase 2: Vue Integration
- [ ] New composable: `useIdentity.ts`
  - `identity: Ref<{ sub, email, name, picture, pkToken } | null>`
  - `login()`, `logout()`, `isAuthenticated`
  - Auto-restore identity on app launch from safeStorage
  - Derive `clasp-identity` compatible DID from PK Token's ephemeral key
- [ ] Identity UI: Login button in Settings panel
  - Shows Google avatar + email when logged in
  - Logout button
  - "Using local identity" indicator when not logged in via OIDC

#### Phase 3: Signed Config Changes
- [ ] Wrap mutating composable calls with signed messages:
  - `useRoutes.add()` signs the route config
  - `useBridges.add()` signs the bridge config
  - `useRouters.add()` signs the router config
- [ ] Store signatures alongside configs (new `signature` field)
- [ ] Display signature verification status in UI (verified badge / unverified indicator)

#### Phase 4: Bridge-to-Bridge Identity
- [ ] Include PK Token in CLASP HELLO message (alongside, not replacing, CPSK token)
- [ ] Router validates PK Token on connection
- [ ] DID derivation from PK Token ephemeral key feeds into existing clasp-identity system

### Testing Strategy

#### Unit Tests (mocked OIDC)
```typescript
// apps/bridge/src/composables/__tests__/useIdentity.test.ts
import { describe, it, expect, vi } from 'vitest'

describe('useIdentity', () => {
  it('derives DID from PK Token ephemeral key', async () => {
    // Mock the IPC layer to return a fixture PK Token
    vi.mock('../../lib/ipc', () => ({
      invoke: vi.fn().mockResolvedValue(FIXTURE_PK_TOKEN)
    }))

    const { login, identity } = useIdentity()
    await login()

    expect(identity.value).not.toBeNull()
    expect(identity.value.email).toBe('test@example.com')
    expect(identity.value.did).toMatch(/^did:key:z6Mk/)
  })

  it('falls back to local identity when OIDC unavailable', async () => {
    vi.mock('../../lib/ipc', () => ({
      invoke: vi.fn().mockRejectedValue(new Error('no network'))
    }))

    const { identity, isAuthenticated } = useIdentity()
    expect(isAuthenticated.value).toBe(false)
    // Local identity still works
    expect(identity.value).toBeNull() // No OIDC identity
  })
})
```

#### Manual Integration Test (requires Google account)
```
Test procedure (manual, not CI):
1. Build bridge app: npm run build
2. Launch app
3. Navigate to Settings > Identity
4. Click "Log in with Google"
5. Complete Google OAuth flow in browser
6. Verify: avatar and email appear in the UI
7. Create a new router config
8. Verify: config has a signature field
9. Verify: signature verification badge shows "verified"
10. Restart app
11. Verify: identity persists (restored from safeStorage)
```

### Documentation

- `docs/auth/openpubkey-identity.md` -- User guide: logging in with Google/GitHub, what it means for security
- `docs/auth/signed-config-changes.md` -- User guide: how config signing works, verifying signatures
- Update `docs/auth/` index to reference OpenPubKey alongside CPSK and capability tokens

---

## 4. SourceHub Audit Trail

### Verification Status

- [VERIFIED] SourceHub is a real Cosmos SDK chain: `github.com/sourcenetwork/sourcehub`
- [VERIFIED] ACP module exists (on-chain Zanzibar-style authorization)
- [VERIFIED] Bulletin module exists (trust-minimized broadcast)
- [FLAGGED] "Epochs" module NOT confirmed in docs -- docs mention "Developer-Lock Tier" instead
- [CORRECTED] Current version is v0.3.2, not v0.2.0 as originally stated
- [CORRECTED] `@sourcenetwork/sourcehub-js` does NOT exist on npm. The JS client is `@sourcenetwork/hublet` v1.0.2 (wallet library only) plus `@sourcenetwork/acp-js` v1.0.5 (ACP client that talks to SourceHub). There is no general-purpose SourceHub JS client for bulletin/audit operations.
- [VERIFIED] `@sourcenetwork/hublet` v1.0.2 on npm: lightweight wallet using Web Crypto API + secp256r1 (P-256). Dependencies: `@cosmjs/encoding`, `@noble/curves`, `@noble/hashes`, `@improbable-eng/grpc-web`, `ts-proto`, `bech32`, `multiformats`, `varint`. Addresses are Cosmos-style bech32 (`source1...`).

### Available JS Tooling for SourceHub

| What You Need | Package | Status |
|---------------|---------|--------|
| Wallet / key management | `@sourcenetwork/hublet` v1.0.2 | On npm, works |
| ACP policy CRUD | `@sourcenetwork/acp-js` v1.0.5 | On npm, works |
| Bulletin / audit submission | None | Would need custom gRPC/REST client |
| General chain queries | None | Would need cosmjs or custom client |

To submit audit entries to the Bulletin module, we would need to build a thin client against the SourceHub gRPC/REST API. The existing `hublet` package handles key management and transaction signing, but there are no pre-built message types for Bulletin operations.

### Why This Is #4 (Moved Down)

- `clasp-journal` and `clasp-journal-defra` already provide audit trail functionality
- SourceHub is testnet-only with no mainnet date
- No Bulletin JS client exists -- would need custom implementation
- On-chain audit is only valuable for multi-org federated deployments with compliance requirements
- Most CLASP users do not need blockchain-backed audit
- The crypto curve mismatch (CLASP uses Ed25519, SourceHub uses secp256r1) adds complexity

### What It Adds (That CLASP Does Not Have)

For federated deployments across organizations:
- Immutable, third-party-verifiable audit trail (blockchain consensus, not just local journal)
- Cross-org policy transparency (all participants can verify access control rules)
- Dispute resolution via cryptographic proof of who authorized what

### Implementation Plan (Abbreviated -- Low Priority)

#### Phase 1: Local Devnet + Client
- [ ] Set up SourceHub local devnet for development (Docker)
- [ ] Use `@sourcenetwork/hublet` for wallet/key management + `@sourcenetwork/acp-js` for ACP operations
- [ ] Build thin Bulletin client against SourceHub gRPC API (no existing package for this)
- [ ] Handle curve mismatch: generate secp256r1 key via hublet alongside Ed25519 identity, or use a bridge
- [ ] New IPC module: `electron/ipc/sourcehub.js`
- [ ] New composable: `useSourceHub.ts` with `connected`, `submitAuditEntry()`, `queryAuditLog()`

#### Phase 2: Audit Integration
- [ ] Hook into config-mutating operations: route/bridge/policy CRUD generates audit entries
- [ ] Audit log viewer panel in the UI
- [ ] Entries signed with OpenPubKey or clasp-identity, submitted as SourceHub transactions

### Testing Strategy

```rust
// Integration test requires SourceHub devnet running in Docker
#[tokio::test]
#[ignore] // Requires: docker run sourcenetwork/sourcehub:v0.3.2
async fn test_audit_entry_round_trips_through_sourcehub() {
    let client = SourceHubClient::connect("http://localhost:26657").await.unwrap();

    let identity = Identity::generate();
    let entry = AuditEntry {
        action: "create_route",
        resource_id: "route-123",
        actor: identity.did(),
        timestamp: chrono::Utc::now(),
        payload: json!({ "source": "/osc/fader1", "target": "/midi/cc/7" }),
    };

    let tx_hash = client.submit_audit_entry(&entry, &identity).await.unwrap();
    assert!(!tx_hash.is_empty());

    // Query back
    let entries = client.query_audit_log("route-123").await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].actor, identity.did());
}
```

### Documentation

- `docs/federation/audit-trail.md` -- Guide: setting up SourceHub audit for federated deployments
- `docs/federation/sourcehub-setup.md` -- Setup: running SourceHub devnet, connecting CLASP

---

## 5. Orbis Encrypted Routing

### Verification Status

- [VERIFIED] Orbis exists: `github.com/sourcenetwork/orbis-go` (599 stars, 388 commits)
- [VERIFIED] Implements DKG, PRE, threshold cryptography
- [FLAGGED] Repo README states: "This project is still early in its architectural journey. Although the overall high-level design and goals are defined, the concrete implementation details are in flux."
- [FLAGGED] Go-only -- no JS or Rust bindings
- [FLAGGED] No published client library on any package registry

### Why This Is #5 (Lowest Priority)

- `clasp-crypto` already has AES-256-GCM, ECDH P-256, and E2E session protocol
- The existing E2E encryption covers the vast majority of use cases
- PRE (Proxy Re-Encryption) latency is likely too high for signal-rate data (>1000 Hz)
- Orbis is self-described as early/unstable
- Go-only means either gRPC sidecar or full reimplementation
- The use case (zero-knowledge signal routing) is extremely niche

### Honest Assessment

The existing `clasp-crypto` E2E system already provides:
- End-to-end encryption between CLASP nodes (ECDH key exchange + AES-256-GCM)
- Key storage (memory and filesystem backends)
- Session management (E2ESession state machine)

What Orbis would add beyond this:
- **PRE**: Route encrypted signals without decrypting (zero-knowledge routing)
- **Threshold keys**: No single point of key compromise
- **PSS**: Proactive secret sharing for long-term security

For 99% of CLASP deployments, the existing E2E encryption is sufficient. PRE is relevant only for high-security environments where even the bridge operator should not see plaintext signals.

### Recommendation

**Prototype only.** If anyone needs this, the prototype should measure PRE latency at signal rates before committing to a full integration.

#### Phase 1: Latency Prototype (1 week)
- [ ] Set up Orbis Go node locally
- [ ] Write a benchmark: encrypt signal value, PRE re-encrypt, decrypt at destination
- [ ] Measure latency per operation at 100, 1000, 10000 signals/sec
- [ ] Decision gate: if PRE latency > 500us per operation, stop here

If PRE is too slow (likely), the pragmatic path is:
- Use existing `clasp-crypto` E2E for endpoint encryption
- Use envelope encryption (AES per-session key, encrypted with destination's public key) for high-throughput paths
- Both of these already exist in the codebase

### Documentation

- `docs/auth/encryption.md` -- Update existing E2E encryption docs to mention Orbis as a future option for PRE use cases

---

## Critical Gaps and Unresolved Issues

These issues surfaced during deep analysis and must be addressed before or during implementation.

### 1. WASM Transform Config Serialization

`applyTransform()` in `transforms.ts` is synchronous. WASM instantiation (`WebAssembly.instantiate()`) is async. Options:
- **Pre-instantiation**: When a route is created/loaded, pre-compile and cache the WASM module. `applyTransform()` stays synchronous, using the pre-compiled instance.
- **Async transform pipeline**: Make the entire transform path async. This is a larger refactor.

**Recommendation**: Pre-instantiation via `WasmTransformPool`. Initialize on route load, fail fast if WASM is invalid.

### 2. WASM Module Storage and Config References

Raw WASM bytes (typically 10-100KB) cannot be embedded in JSON config. The `TransformConfig` needs a reference, not inline bytes:
```typescript
interface TransformConfig {
  type: 'wasm'
  wasmModuleId?: string          // Reference to stored module (hash or name)
  wasmParams?: Record<string, unknown>
}
```

Storage options:
- **Electron filesystem**: `~/.clasp/lenses/<hash>.wasm`
- **DefraDB blob**: Store in a dedicated collection with P2P sync
- **Bundled**: Ship pre-built lenses alongside the app

Config export/import must handle WASM modules separately (like attachments, not inline JSON).

### 3. lens_sdk Version Mismatch

crates.io has `lens_sdk` v0.8.1, but the lens repo is at v0.10.0. The repo may have breaking changes not yet published. Pin to v0.8.1 (the published crate) for initial implementation. Test against v0.10.0 if needed.

### 4. acp-js Does NOT Have Local WASM Mode

This was initially misidentified. Reading the actual `bridge.js` source confirms that ALL `window.acp_*` functions call `client-helpers.ts`, which creates a `SourceHubACP` client talking to SourceHub over HTTP/gRPC. The `wasm-bridge` re-export is a utility, not used by the bridge. The `acp_core` playground WASM is a separate thing with a different (sandbox) API.

**Consequence**: For local-only ACP, use DefraDB `@policy` directives exclusively. `acp-js` is only relevant if running a SourceHub node.

### 5. OpenPubKey ESM vs Electron CJS

`openpubkey-js` is ESM-only (`"type": "module"`). Electron's main process uses CJS. The IPC handler (`electron/ipc/identity.js`) cannot `require()` an ESM package directly. Options:
- Dynamic `import()` in the CJS handler (works in Node.js >= 14)
- Bundle openpubkey-js into a CJS wrapper as a build step
- Use openpubkey-js only in the renderer (Vite handles ESM), not in main process

### 6. OpenPubKey Ephemeral Key Curve

The PK Token's CIC (Client Instance Claims) contains an ephemeral public key. The key algorithm depends on what the client generates. The `jose` library (used by openpubkey-js) supports Ed25519 via EdDSA, but it is unclear whether openpubkey-js allows choosing the key algorithm. If it forces P-256 or RSA, deriving an Ed25519 key for clasp-identity is not straightforward. This must be verified before assuming interoperability.

---

## Dependency Graph

```
                LensVM WASM            ACP/Zanzi
                Transforms             Authorization
                (independent)          (independent -- uses existing clasp-identity)
                    |                      |
                    v                      v
                  [done]               [done]
                                          |
                                    OpenPubKey Identity
                                    (optional -- adds OIDC binding
                                     to existing identity system)
                                          |
                           +--------------+--------------+
                           |                             |
                     Orbis PRE                    SourceHub
                     Encrypted                    Audit Trail
                     Routing                      (needs identity + ACP)
                     (needs identity + ACP)
```

**LensVM and ACP are both independent and can start immediately.**
ACP via DefraDB uses existing `clasp-identity` (already done) -- no dependency on OpenPubKey.
OpenPubKey enhances the identity system but is not a prerequisite for anything.
Tracks 4 and 5 depend on having identity + ACP in place.

---

## Recommended Execution Order

### Sprint 1 (Weeks 1-4): Foundations (Parallel Tracks)

**Track A: LensVM WASM Transforms**
- Week 1-2: `clasp-lens` Rust crate with wasmtime host, unit tests with compiled fixture lenses
- Week 3: Browser-side WASM host in `lib/wasm-transform.ts`, Vitest tests
- Week 4: UI integration (upload, configure, preview), integration tests with real router

**Track B: ACP/Zanzi via DefraDB**
- Week 1: Add `@policy` directives to `clasp-config-defra/src/schema.rs`, update store.rs
- Week 2: Integration tests with real DefraDB (extend existing CI test infrastructure)
- Week 3: `useAccessControl.ts` composable + SecurityPanel extensions
- Week 4: Relationship management UI, documentation

### Sprint 2 (Weeks 5-7): Identity

**OpenPubKey Identity**
- Week 5: `electron/ipc/identity.js` + preload exposure, install from GitHub
- Week 6: `useIdentity.ts` composable + Settings panel UI
- Week 7: Signed config changes (wrap mutating composables), manual testing with real Google OAuth

### Sprint 3 (Weeks 8-10): Optional/Advanced

**Only if justified by user demand:**
- Week 8: Orbis PRE latency prototype -- go/no-go decision
- Week 9-10: SourceHub audit client + audit log viewer (if federated deployment demand exists)

### Sprint 4 (Week 11-12): Polish

- Pre-built WASM lens library (5-10 modules)
- `clasp-lens-template` Rust crate for community authoring
- End-to-end integration tests across all active integrations
- Documentation for all new features
- Update README and docs site

---

## What Each Integration Unlocks

| Feature | Requires |
|---------|----------|
| Custom signal transforms in Rust/AssemblyScript | LensVM |
| Document-level access control in DefraDB | ACP |
| "Share this router with alice@company.com" | ACP + clasp-identity |
| Per-route read/write/execute permissions | ACP |
| Federated policy consistency across nodes | ACP + DefraDB P2P |
| "Log in with Google" for bridge operators | OpenPubKey |
| Signed, auditable config changes | OpenPubKey (or clasp-identity alone) |
| Community transform marketplace | LensVM + DefraDB P2P |
| Immutable audit trail for compliance | SourceHub + OpenPubKey |
| Zero-knowledge encrypted signal routing | Orbis (speculative) |

---

## Files That Will Be Created/Modified

### New Files
| File | Purpose |
|------|---------|
| `crates/clasp-lens/` | Rust WASM host crate (wasmtime) |
| `crates/clasp-lens/src/host.rs` | LensVM host protocol implementation |
| `crates/clasp-lens/src/transform.rs` | WasmTransform struct |
| `crates/clasp-lens/tests/` | Unit and integration tests |
| `crates/clasp-lens/benches/` | Latency benchmarks |
| `apps/bridge/src/lib/wasm-transform.ts` | Browser-side WASM transform host |
| `apps/bridge/src/lib/__tests__/wasm-transform.test.ts` | Vitest tests |
| `apps/bridge/electron/ipc/identity.js` | OpenPubKey IPC handlers |
| `apps/bridge/src/composables/useIdentity.ts` | OIDC identity composable |
| `apps/bridge/src/composables/useAccessControl.ts` | ACP relationship management |
| `apps/bridge/src/composables/useSourceHub.ts` | SourceHub audit composable |
| `apps/bridge/electron/ipc/sourcehub.js` | SourceHub client IPC |
| `lenses/` | Pre-built WASM lens modules |
| `clasp-lens-template/` | Template crate for community lens authoring |
| `docs/transforms/wasm-transforms.md` | WASM transform user guide |
| `docs/transforms/authoring-lenses.md` | Lens authoring developer guide |
| `docs/auth/acp-authorization.md` | ACP user guide |
| `docs/auth/openpubkey-identity.md` | OpenPubKey user guide |
| `docs/federation/audit-trail.md` | SourceHub audit guide |

### Modified Files
| File | Change |
|------|--------|
| `Cargo.toml` (workspace) | Add `clasp-lens` crate |
| `crates/clasp-bridge/src/transform.rs` | Add `Transform::Wasm` variant |
| `crates/clasp-config-defra/src/schema.rs` | Add `@policy` directives |
| `crates/clasp-config-defra/src/store.rs` | Identity context in mutations |
| `apps/bridge/src/lib/types.ts` | Add `'wasm'` TransformType, identity types |
| `apps/bridge/src/lib/transforms.ts` | Dispatch to WasmTransformHost for `type: 'wasm'` |
| `apps/bridge/src/lib/constants.ts` | Add WASM transform to type list |
| `apps/bridge/src/components/modals/SignalRouteModal.vue` | WASM transform config |
| `apps/bridge/src/components/panels/SecurityPanel.vue` | ACP relationship management |
| `apps/bridge/electron/preload.js` | Expose identity + audit APIs |
| `apps/bridge/package.json` | Add openpubkey-js dep (from GitHub) |
| `.github/workflows/ci.yml` | Add clasp-lens tests, DefraDB ACP tests |

---

## Open Questions

1. **LensVM host in Rust vs JS**: Both. Rust host for production pipeline, JS host for UI preview. This is not optional -- users need preview before deploying a transform.

2. **OpenPubKey provider priority**: Google first (most common). GitHub second (developer audience). Both use the same library.

3. **ACP granularity**: Per-resource-type with per-instance ownership. The `owner` field already exists on every DefraDB document.

4. **Orbis: worth the complexity?** Almost certainly not for the general case. Prototype to validate latency, expect to fall back to existing clasp-crypto E2E.

5. **SourceHub: mainnet or devnet?** Local devnet only. SourceHub is testnet (v0.3.2). Do not build production features against it until it reaches v1.0.

6. **WASM lens distribution**: Bundle 5-10 common lenses in the app. User-authored lenses stored in DefraDB with P2P sync.

7. **OpenPubKey npm availability**: Install from GitHub pinned to commit hash. Re-evaluate when/if published to npm.

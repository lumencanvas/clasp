# CLASP + Source Network / DefraDB: Deep Research

## Overview

CLASP (Creative Low-Latency Application Streaming Protocol) is a real-time signal router for creative applications — sub-100us latency, hardware protocol bridges (OSC, MIDI, DMX, Art-Net), binary wire format, 17 Rust crates.

DefraDB is a peer-to-peer document database built on Merkle CRDTs — edge-first, offline-capable, GraphQL query engine, libp2p networking. It's the data layer of Source Network.

Source Network is a decentralized data infrastructure stack: DefraDB (storage), SourceHub (trust/access control blockchain), Orbis (secrets), LensVM (schema migrations), Zk-KMS (zero-knowledge auth).

---

## Where They Overlap

| Dimension | CLASP | DefraDB |
|-----------|-------|---------|
| Philosophy | Edge-first, real-time signal routing | Edge-first, local-first document storage |
| Identity | Ed25519 keypairs (clasp-registry) | Ed25519 peer IDs + DIDs |
| Auth | Capability tokens (CPSK + delegatable Ed25519 caps) | Google Zanzibar-style ACP with DID policies |
| Conflict resolution | LWW, Max, Min, Lock, Merge per-parameter | LWW-Register, PN-Counter per-field via Merkle CRDTs |
| Pub/Sub | Pattern-based subscriptions (trie, wildcards) | GossipSub per-collection topics |
| Transports | WS, QUIC, UDP, TCP, BLE, Serial, WebRTC | libp2p (TCP, QUIC, WS, WebTransport) |
| Federation | Hub-leaf router federation with namespace ownership | P2P mesh via libp2p replicator + pubsub peering |
| Persistence | SQLite append-only journal (optional) | BadgerDB with Merkle DAG (mandatory) |

---

## Where They Diverge

### CLASP fills gaps DefraDB can't:
- Sub-100us latency (DefraDB is ms-to-seconds for DB ops)
- Hardware protocol bridges (OSC, MIDI, DMX, Art-Net, sACN)
- 5 semantic signal types (Param, Event, Stream, Gesture, Timeline)
- Binary wire format (31-byte SET, 8M msg/s codec)
- Clock synchronization (NTP-style)
- Gesture coalescing, bundle scheduling, real-time automation

### DefraDB fills gaps CLASP can't:
- Structured document storage with schemas and migrations
- Full query engine (GraphQL with filters, joins, aggregates)
- Complete version history as Merkle DAG (content-addressed commits)
- Time-travel queries (query any historical state)
- Schema versioning with lazy WASM-based migrations (LensVM)
- Content-addressable data (CIDs) for cryptographic integrity proofs
- Decentralized access control with on-chain audit trails (SourceHub)
- Vector embeddings for edge AI inference

### Key insight: they're complementary, not competitive.
CLASP is the nervous system (real-time signals). DefraDB is the memory (persistent, queryable, syncable state).

---

## Integration Ideas

### 1. `clasp-defra` crate -- DefraDB-backed journal (LOW EFFORT)

CLASP already has a `Journal` trait with a SQLite implementation. A `clasp-defra` crate implements that same trait against DefraDB:

- P2P state sync for free (Merkle CRDTs sync journal entries between routers)
- GraphQL queries over signal history
- Content-addressable audit trail (every entry gets a CID)
- Schema-versioned state (LensVM migrates stored data lazily)

Drop-in: implement the trait, swap the backend. CLASP's real-time path stays untouched.

### 2. Unified identity package (LOW EFFORT)

Both use Ed25519. A shared identity layer produces keypairs that simultaneously serve as:
- A CLASP entity ID (for caps and router auth)
- A DID (for DefraDB ACP policies)
- A libp2p peer ID (for DefraDB P2P)

One key, three systems. The crypto is identical -- just encoding/wrapping.

### 3. Real-time DefraDB change notifications via CLASP (MEDIUM EFFORT)

DefraDB's GossipSub is collection-level. CLASP's subscription system is address-level with wildcards. A bridge/sidecar maps:

```
DefraDB mutation on User doc "abc123" field "status"
  -> CLASP EMIT /defra/User/abc123/status { value: "online" }
```

Any CLASP client subscribes to `/defra/User/*/status` for sub-ms push notifications. DefraDB gets a real-time notification layer it currently lacks.

### 4. Config/scene persistence in DefraDB (MEDIUM EFFORT)

CLASP configs stored as DefraDB documents:
- P2P config sync (team of designers shares configs without a server)
- Version history (roll back to "last Friday's show config")
- Access control (only the LD edits fixtures, operator reads)

```graphql
type ClaspRouter {
  name: String
  host: String
  port: Int
  transports: [String]
  owner: String  # DID
}

type ClaspBridge {
  protocol: String
  config: String
  router: ClaspRouter
}
```

### 5. CLASP as DefraDB's browser transport (LARGE EFFORT)

DefraDB's libp2p has limited browser support. CLASP has production WebSocket + WebRTC + WASM clients. A `defra-transport-clasp` tunnels DefraDB sync over CLASP's transport layer:
- Browser-native P2P via CLASP's WebRTC signaling
- BLE transport for mobile/IoT
- QUIC with connection migration

### 6. DefraDB-backed parameter state store (LARGE EFFORT)

Replace CLASP's in-memory `RouterState` with DefraDB documents. Each address becomes a document:

```graphql
type Signal {
  address: String @index
  value: String
  revision: Int
  writer: String
  signal_type: String
  ttl_mode: String
  ttl_seconds: Int
}
```

Hot path stays in-memory with async write-through to DefraDB. Gets: persistent state, queryable via DQL, P2P sync via CRDTs instead of custom federation.

### 7. Timeline automation in DefraDB (LARGE EFFORT)

Timeline signals as DefraDB documents with full version history:
- Fork/branch automation sequences
- Compare versions, merge edits from multiple designers
- Like Git for show programming

---

## Low-Hanging Fruit Summary

| Effort | Integration | What You Get |
|--------|-------------|--------------|
| Small | Unified Ed25519 identity | One keypair for CLASP auth + DefraDB DID + P2P peer ID |
| Small | `clasp-defra` journal backend | P2P state sync, GraphQL history queries, content-addressable audit |
| Medium | Change notification bridge | Real-time DefraDB subscriptions via CLASP pub/sub |
| Medium | Config/scene storage in DefraDB | P2P config sync, version history, access control |
| Large | DefraDB-backed state store | Full persistent, queryable, P2P-synced parameter state |
| Large | CLASP as DefraDB browser transport | Browser-native P2P for DefraDB via WebRTC/WebSocket |

---

## Source Network Technical Details

### DefraDB
- Language: Go (1.25.5)
- Version: v1.0.0-rc1 (March 5, 2026)
- License: BSL -> Apache 2.0
- Storage: BadgerDB (default), LevelDB (WASM), in-memory (testing)
- Networking: libp2p (gossipsub + replicator peering)
- CRDTs: LWW-Register, PN-Counter, DocComposite, CollectionSet
- Query: DQL (GraphQL-compatible with extensions)
- Schema: GraphQL SDL, versioned, LensVM migrations (WASM)
- Identity: DIDs, Ed25519, secp256k1
- ACP: Google Zanzibar-based, local/node/decentralized modes
- Key deps: badger/v4, go-libp2p, go-ipld-prime, cosmos-sdk, acp_core

### SourceHub
- Language: Go (Cosmos SDK + CometBFT)
- Status: Testnet phase
- Purpose: Trust anchor -- consensus on access policies, identity, audit trails
- Not required for DefraDB data operations (optional trust layer)

### Other Components
- Orbis: Threshold proxy re-encryption for secrets management
- LensVM: WASM schema transformation engine (any language -> WASM)
- Zk-KMS: Zero-knowledge proof authentication

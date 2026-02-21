# Federation Message Sequence

Detailed protocol sequences for CLASP router-to-router federation, covering the handshake, initial sync, steady-state forwarding, and revision vector reconciliation.

## Handshake and Initial Sync

```mermaid
sequenceDiagram
    participant Leaf as Leaf Router
    participant Hub as Hub Router

    Note over Leaf, Hub: Phase 1: CLASP Handshake
    Leaf->>Hub: HELLO (name, features: ["param","event","stream","federation"])
    Hub->>Leaf: WELCOME (name, features: ["param","event","federation"], sessionId)

    Note over Leaf, Hub: Phase 2: Namespace Declaration
    Leaf->>Hub: FederationSync::DeclareNamespaces(["/site-a/**"])
    Hub->>Hub: Store peer namespaces on session
    Hub->>Hub: Auto-subscribe peer to /site-a/** (sub IDs 50000+)
    Hub->>Leaf: ACK

    Note over Leaf, Hub: Phase 3: Initial State Sync
    Leaf->>Hub: FederationSync::RequestSync(pattern: "/site-a/**", since: None)
    Hub->>Hub: Query state matching /site-a/**
    Hub->>Leaf: Snapshot (current state for /site-a/**)
    Hub->>Leaf: FederationSync::SyncComplete(pattern: "/site-a/**")

    Note over Leaf, Hub: Phase 4: Steady State
    Leaf->>Hub: SET /site-a/lights/1 0.75 (origin: "site-a-router")
    Hub->>Hub: Apply state, broadcast to local subscribers
    Hub--xLeaf: (not forwarded back -- origin matches)
```

## Namespace Declaration

When a federation peer sends `DeclareNamespaces`:

```mermaid
flowchart TD
    DN[DeclareNamespaces<br/>patterns: list of String] --> Check{Pattern count<br/><= 1000?}
    Check -->|No| Reject[400: Too many patterns]
    Check -->|Yes| Auth{Authenticated<br/>mode?}
    Auth -->|Yes| Scope{Session has scope<br/>for each pattern?}
    Auth -->|No| Store
    Scope -->|No| Reject403[403: Insufficient scope]
    Scope -->|Yes| Cleanup[Remove old federation subs<br/>IDs >= 50000]
    Cleanup --> Store[Store namespaces on session]
    Store --> Sub[Auto-subscribe peer<br/>to each pattern]
    Sub --> Ack[Send ACK]
```

If a peer calls `DeclareNamespaces` again, old federation subscriptions (identified by sub IDs >= 50000) are cleaned up before creating new ones.

## Request Sync

When a federation peer requests state sync:

```mermaid
flowchart TD
    RS[RequestSync<br/>pattern, since_revision] --> NCheck{Pattern covered by<br/>declared namespaces?}
    NCheck -->|No| Reject[403: Outside declared scope]
    NCheck -->|Yes| Query[Query state matching pattern]
    Query --> Filter{since_revision<br/>specified?}
    Filter -->|Yes| Delta[Filter: only entries with<br/>revision > since_revision]
    Filter -->|No| Full[Send full snapshot]
    Delta --> Send[Send Snapshot message]
    Full --> Send
    Send --> Complete[Send SyncComplete]
```

**Namespace restriction:** `RequestSync` validates that each requested pattern is covered by the peer's declared namespaces using `federation_pattern_covered_by()`. A peer that declared `/site-a/**` cannot request sync for `/site-b/**` or `/**`.

## Revision Vector Exchange

Periodic revision vector exchange detects state drift:

```mermaid
sequenceDiagram
    participant Leaf as Leaf Router
    participant Hub as Hub Router

    Note over Leaf, Hub: Every sync_interval (default 30s)
    Leaf->>Hub: FederationSync::RevisionVector({<br/>  "/site-a/lights/1": 42,<br/>  "/site-a/lights/2": 17<br/>})

    Hub->>Hub: Compare with local state
    Hub->>Hub: Filter: only addresses within<br/>peer's declared namespaces
    Hub->>Hub: Find addresses where<br/>local revision > peer revision

    alt Local state is newer
        Hub->>Leaf: Snapshot (delta for stale addresses)
    else Peer is up to date
        Note over Hub: No action needed
    end
```

**Resource limit:** Revision vectors are capped at 10,000 entries. Vectors exceeding this limit are rejected with a 400 error.

## Steady-State Forwarding

During active operation, state changes are forwarded based on namespace ownership:

```mermaid
sequenceDiagram
    participant ClientA as Client (Site A)
    participant LeafA as Leaf A<br/>/site-a/**
    participant Hub as Hub Router
    participant LeafB as Leaf B<br/>/site-b/**
    participant ClientB as Client (Site B)

    ClientA->>LeafA: SET /site-a/sensor/temp 23.5
    LeafA->>Hub: SET /site-a/sensor/temp 23.5<br/>(origin: "leaf-a")
    Hub->>Hub: Apply state
    Hub->>LeafB: SET /site-a/sensor/temp 23.5<br/>(origin: "leaf-a")
    Hub--xLeafA: Not forwarded (origin = sender)
    LeafB->>ClientB: Broadcast to subscribers of /site-a/**
```

## Loop Prevention

Every forwarded message carries an `origin` field set to the source router's `router_id`. The forwarding logic checks:

1. **FederationLink**: `forward_set()` and `forward_publish()` skip sending if `origin == peer.router_id`
2. **NamespaceManager**: `peers_for_address()` accepts `exclude_origin` to filter out the originating peer
3. **Rules Engine**: Actions with origin starting with `"rule:"` are not re-evaluated by the rules engine

This prevents messages from bouncing back to their source and prevents rule-triggered cascades.

## Namespace Restriction Enforcement

Federation peers can only access data within their declared namespaces:

| Operation | Enforcement |
|-----------|-------------|
| `DeclareNamespaces` | Pattern count <= 1,000; scope check in authenticated mode |
| `RequestSync` | Requested pattern must be covered by a declared namespace |
| `RevisionVector` | Addresses outside declared namespaces are silently skipped |
| `SET/PUBLISH forwarding` | Only messages matching peer's namespace subscriptions are sent |

## See Also

- [Federation State Machine](federation-state-machine.md) -- PeerState transitions
- [Distributed Architecture](distributed-architecture.md) -- Overall system architecture
- [Security Model](security-model.md) -- Authentication and access control

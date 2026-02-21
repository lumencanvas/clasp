# Distributed Architecture

How CLASP's distributed infrastructure crates extend the core router with authentication, persistence, automation, and multi-site federation.

## Overview

The distributed infrastructure is a set of five opt-in crates that plug into the CLASP router via feature flags. When disabled, they add zero overhead. When enabled, they compose into a layered system where each crate handles a distinct concern:

```mermaid
graph TB
    subgraph "CLASP Router"
        Router[Router Core<br/>sessions, subscriptions, state]

        Router --> Caps[clasp-caps<br/>Capability Tokens]
        Router --> Registry[clasp-registry<br/>Entity Registry]
        Router --> Journal[clasp-journal<br/>Event Journal]
        Router --> Rules[clasp-rules<br/>Rules Engine]
        Router --> Federation[clasp-federation<br/>Federation Manager]
    end

    Caps -.->|validates| Auth[Auth Layer<br/>ValidatorChain]
    Registry -.->|validates| Auth
    Auth -->|scopes| Router

    Journal -.->|persists| Storage[(SQLite / Memory)]
    Registry -.->|persists| Storage

    Federation -.->|connects| PeerA[Peer Router A]
    Federation -.->|connects| PeerB[Peer Router B]
```

## Feature Flag Matrix

Each crate maps to a feature flag on `clasp-router`:

| Feature | Crate | What It Adds |
|---------|-------|-------------|
| (none) | `clasp-caps` | Standalone capability token library (no router integration needed) |
| (none) | `clasp-registry` | Standalone entity registry (no router integration needed) |
| `journal` | `clasp-journal` | State persistence, crash recovery, replay |
| `rules` | `clasp-rules` | Reactive automation after state changes |
| `federation` | (built-in) | Hub-side federation peer handling |

The `clasp-caps` and `clasp-registry` crates integrate with the router through `clasp-core`'s `TokenValidator` trait and the `ValidatorChain`, without needing a dedicated router feature flag.

## Message Flow

When a client sends a SET message, it passes through the following stages:

```mermaid
sequenceDiagram
    participant Client
    participant Router
    participant Auth as ValidatorChain
    participant State as State Store
    participant Journal as Journal
    participant Rules as Rules Engine
    participant Fed as Federation
    participant Subs as Subscribers

    Client->>Router: SET /lights/room1 0.75
    Router->>Auth: Check session scopes
    Auth-->>Router: Allowed (write:/lights/**)
    Router->>State: Apply state change (LWW)
    State-->>Router: Revision 42
    Router->>Journal: Append entry (seq, addr, value, rev)
    Router->>Rules: Evaluate matching rules
    Rules-->>Router: [PendingAction: Set /dmx/1 192]
    Router->>State: Apply rule action
    Router->>Fed: Forward to peers owning /lights/**
    Router->>Subs: Broadcast to matching subscribers
```

Each stage is optional:
- **Auth**: Only in `Authenticated` security mode
- **Journal**: Only with `journal` feature
- **Rules**: Only with `rules` feature
- **Federation**: Only with `federation` feature and active peers

## Authentication Architecture

CLASP supports three token types, each handled by a dedicated validator:

| Token Prefix | Validator | Crate | Use Case |
|-------------|-----------|-------|----------|
| `cpsk_` | `CpskValidator` | `clasp-core` | Pre-shared keys (register/login flow) |
| `cap_` | `CapabilityValidator` | `clasp-caps` | Delegatable capability tokens |
| `ent_` | `EntityValidator` | `clasp-registry` | Device/service identity tokens |

These are composed into a `ValidatorChain` that dispatches by prefix:

```mermaid
flowchart LR
    Token[Incoming Token] --> Chain[ValidatorChain]
    Chain -->|cpsk_| CPSK[CpskValidator]
    Chain -->|cap_| Cap[CapabilityValidator]
    Chain -->|ent_| Ent[EntityValidator]
    CPSK -->|Valid/Invalid| Result[ValidationResult]
    Cap -->|Valid/Invalid| Result
    Ent -->|Valid/Invalid| Result
    Result -->|Valid| Session[Session with Scopes]
```

Each validator returns `NotMyToken` for unrecognized prefixes, allowing the chain to try the next validator.

## Persistence Architecture

The journal and registry both use a pluggable storage pattern:

```mermaid
graph LR
    subgraph "Journal"
        JT[Journal Trait] --> MJ[MemoryJournal<br/>Ring buffer]
        JT --> SJ[SqliteJournal<br/>Persistent]
    end

    subgraph "Registry"
        ES[EntityStore Trait] --> ME[MemoryEntityStore<br/>HashMap]
        ES --> SE[SqliteEntityStore<br/>Persistent]
    end
```

- **Memory backends**: Fast, no dependencies, suitable for development and testing
- **SQLite backends**: WAL mode, persistent across restarts, suitable for production

## Federation Architecture

Federation uses a hub/leaf topology where leaf routers connect to a central hub:

```mermaid
graph TB
    subgraph "Site A"
        LA[Leaf Router<br/>owns /site-a/**] --> CA[Local Clients]
    end

    subgraph "Hub"
        HUB[Hub Router]
    end

    subgraph "Site B"
        LB[Leaf Router<br/>owns /site-b/**] --> CB[Local Clients]
    end

    LA <-->|WebSocket| HUB
    LB <-->|WebSocket| HUB
```

Each leaf declares which namespace patterns it owns. The hub auto-subscribes to those patterns and forwards matching state changes. Loop prevention is handled via the `origin` field on forwarded messages.

See [Federation State Machine](federation-state-machine.md) and [Federation Message Sequence](federation-message-sequence.md) for protocol details.

## See Also

- [Token Validation Flow](token-validation-flow.md) -- ValidatorChain dispatch details
- [Capability Delegation](capability-delegation.md) -- Delegation chains and scope attenuation
- [Federation State Machine](federation-state-machine.md) -- PeerState transitions
- [Federation Message Sequence](federation-message-sequence.md) -- Handshake and sync protocol
- [Security Model](security-model.md) -- Encryption, tokens, and threat model

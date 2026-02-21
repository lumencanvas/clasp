# Explanation

Understanding-oriented documentation that explains concepts, design decisions, and background knowledge.

## Core Concepts

### [Why CLASP?](why-clasp.md)
The problems CLASP solves and why existing protocols aren't enough.

### [Architecture](architecture.md)
How CLASP is structured: routers, clients, bridges, and transports.

### [Router vs Client](router-vs-client.md)
Understanding the roles of routers and clients in CLASP systems.

## Protocol Design

### [Signals Not Messages](signals-not-messages.md)
Why CLASP uses semantic signal types instead of generic messages.

### [State Management](state-management.md)
How CLASP handles state, revisions, and late-joiner synchronization.

### [Conflict Resolution](conflict-resolution.md)
Strategies for handling concurrent writes from multiple clients.

### [Timing Model](timing-model.md)
Clock synchronization, scheduled bundles, and timing guarantees.

## Infrastructure

### [Transport Agnosticism](transport-agnosticism.md)
Why CLASP works over any transport and how to choose the right one.

### [Bridge Architecture](bridge-architecture.md)
How bridges translate between CLASP and legacy protocols.

### [Security Model](security-model.md)
Encryption, token types (CPSK, Capability, Entity), and security best practices.

## Distributed Infrastructure

### [Distributed Architecture](distributed-architecture.md)
How the distributed infrastructure crates extend the core router: authentication, persistence, automation, and federation.

### [Token Validation Flow](token-validation-flow.md)
ValidatorChain dispatch: how CPSK, Capability, and Entity tokens are validated.

### [Capability Delegation](capability-delegation.md)
Ed25519 delegation chains with scope attenuation, expiration clamping, and chain depth limits.

### [Federation State Machine](federation-state-machine.md)
PeerState transitions: Connecting, Handshaking, Syncing, Active, Disconnected, Failed.

### [Federation Message Sequence](federation-message-sequence.md)
Hub/Leaf handshake, namespace declaration, state sync, steady-state forwarding, and loop prevention.

## Reading Order

For a complete understanding, read in this order:

1. **[Why CLASP?](why-clasp.md)** — Understand the problem
2. **[Architecture](architecture.md)** — See the big picture
3. **[Signals Not Messages](signals-not-messages.md)** — Core protocol concepts
4. **[State Management](state-management.md)** — How state works
5. **[Bridge Architecture](bridge-architecture.md)** — Protocol integration
6. **[Security Model](security-model.md)** — Authentication and access control
7. **[Distributed Architecture](distributed-architecture.md)** — Distributed infrastructure overview
8. **[Token Validation Flow](token-validation-flow.md)** — Token dispatch
9. **[Capability Delegation](capability-delegation.md)** — Delegation chains
10. **[Federation State Machine](federation-state-machine.md)** — Federation lifecycle
11. **[Federation Message Sequence](federation-message-sequence.md)** — Federation protocol

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
Encryption, capability tokens, and security best practices.

## Reading Order

For a complete understanding, read in this order:

1. **[Why CLASP?](why-clasp.md)** — Understand the problem
2. **[Architecture](architecture.md)** — See the big picture
3. **[Signals Not Messages](signals-not-messages.md)** — Core protocol concepts
4. **[State Management](state-management.md)** — How state works
5. **[Bridge Architecture](bridge-architecture.md)** — Protocol integration

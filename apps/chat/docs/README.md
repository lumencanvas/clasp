# CLASP Chat Documentation

Technical documentation for CLASP Chat, a real-time encrypted chat application built on the CLASP protocol.

## Contents

| Document | Description |
|----------|-------------|
| [Architecture Overview](./architecture.md) | System topology, component tree, composable layer, data flow |
| [CLASP Protocol](./clasp-protocol.md) | How CLASP is used: operations, addressing, subscriptions, binary codec |
| [Security Model](./security.md) | Authentication, authorization scopes, E2E encryption, key exchange, threat model |
| [State & Lifecycle Diagrams](./state-diagrams.md) | Mermaid diagrams for auth flow, room lifecycle, key exchange, presence |
| [Deployment](./deployment.md) | Docker, Caddy, production infrastructure, environment variables |

## Quick Reference

```
apps/chat/            Vue 3 SPA (Vite)
  src/
    composables/      State management (useChat, useCrypto, useAdmin, ...)
    components/       UI components (MessageList, VideoGrid, AdminPanel, ...)
    pages/            Route pages (AuthPage, JoinPage, ChatPage)
    lib/              Utilities (crypto.js, constants.js, plugins.js, storage.js)
    router.js         Vue Router config

deploy/relay/         Rust relay server (standalone binary)
  src/
    main.rs           WebSocket server, CLASP router, state persistence
    auth.rs           HTTP auth API (register, login, guest, rate limiting)

deploy/chat/          Docker Compose for local dev
deploy/droplet/       Production deployment scripts (DigitalOcean)
```

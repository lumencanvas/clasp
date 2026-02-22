---
title: "Security"
description: "Documentation page for the CLASP protocol."
section: api
order: 4
---
## Security

CLASP is designed to be **secure by default** while still being approachable in development environments.

### Encryption

- Production deployments SHOULD use **TLS 1.3** (for WebSocket/QUIC) or **DTLS** (for UDP/DataChannel).
- Local development MAY use plain `ws://` to reduce friction.
- The protocol’s frame format includes an **“encrypted” flag**, but actual key exchange is delegated to the underlying transport.

### Capability Tokens

CLASP uses capability‑style tokens (often JWTs) to describe what a client may do:

- Allowed **read** patterns (`/lumen/**`, `/dmx/1/*`, etc.).
- Allowed **write** patterns.
- Optional **constraints** (e.g. value ranges, max update rate).

Typical flow:

1. A user or provisioning tool issues a token for a specific device/app.
2. The client presents the token when connecting.
3. The router enforces permissions on every message.

Language‑specific docs show how to generate, attach, and validate these tokens in each runtime.


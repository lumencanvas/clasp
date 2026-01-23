## Discovery

CLASP favors **zero‑config discovery** when possible, with explicit configuration as a reliable fallback.

### LAN Discovery

Typical discovery mechanisms include:

- **mDNS/Bonjour**: `_clasp._tcp.local` service for routers on the local network.
- **UDP broadcast**: A simple discovery frame on a well‑known port (e.g. `7331`).

Routers and desktop tools can advertise themselves over mDNS and/or UDP; embedded and browser clients then:

1. Discover available routers.
2. Present choices to the user or auto‑select based on policy.

### Browser Considerations

Browsers cannot do raw mDNS or arbitrary UDP:

- Browser clients usually connect to a known WebSocket endpoint (`wss://host:7330/clasp`).
- A separate discovery UI or rendezvous service can provide that endpoint.

### Manual Configuration

For constrained or locked‑down environments (corporate networks, WAN scenarios), discovery may be disabled entirely. In those cases:

- Users configure router addresses explicitly.
- P2P setups typically use a **rendezvous/router** for signaling but not for data.

Language‑specific docs show how to opt into discovery helpers where they exist, and how to fall back to manual configuration when they do not.


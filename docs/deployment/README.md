---
title: Deployment
description: From local development to production deployment
order: 1
---

# Deployment

CLASP offers two server options: a lightweight development router and a full-featured production relay. This section covers both, from running locally to deploying in the cloud.

## Router vs Relay

|                   | `clasp server`              | `clasp-relay`                          |
| ----------------- | --------------------------- | -------------------------------------- |
| **Use case**      | Local dev, prototyping      | Production                             |
| **Auth**          | None                        | CPSK + Capability + Entity             |
| **Persistence**   | None                        | Journal (SQLite) + snapshots           |
| **Rules Engine**  | No                          | Yes (`--features rules`)              |
| **Federation**    | No                          | Yes (`--features federation`)         |
| **App Config**    | No                          | Yes (declarative write rules)          |
| **Protocol Servers** | No                       | MQTT + OSC embedded                    |
| **Install**       | `cargo install clasp-cli`   | `cargo install clasp-relay` or Docker  |

The development router (`clasp server`) is a single-command WebSocket router with no configuration. The relay (`clasp-relay`) is a standalone Rust binary with authentication, persistence, federation, embedded protocol servers, and a rules engine -- all controlled by feature flags and CLI options.

## Feature Flags

The relay uses Cargo feature flags to enable optional subsystems. Only compile what you need, or use `full` for everything.

| Feature        | Description                                      |
| -------------- | ------------------------------------------------ |
| `journal`      | SQLite event journal for full signal history      |
| `caps`         | Capability token delegation and validation        |
| `registry`     | Entity registry with persistent identity          |
| `rules`        | JSON rules engine for signal filtering            |
| `federation`   | Inter-relay federation via hub-and-spoke          |
| `full`         | All of the above                                  |

Build with specific features:

```bash
cargo build --release --features journal,rules
```

Or everything:

```bash
cargo build --release --features full
```

The default build includes `websocket` and `rendezvous` support.

## Guides

- [Local Development](local-dev.md) -- `clasp server`, Docker Compose, quick iteration
- [Relay Server](relay.md) -- what the relay is, how to configure it
- [Docker Deployment](docker.md) -- building and running containers
- [Cloud Deployment](cloud.md) -- DigitalOcean walkthrough
- [Production Checklist](production-checklist.md) -- TLS, auth, persistence, monitoring

## Next Steps

Start with [Local Development](local-dev.md) to get a router running in seconds, or jump to [Relay Server](relay.md) if you are ready to configure a production deployment.

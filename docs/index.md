---
title: CLASP
description: Real-time signal router and protocol bridge
order: 0
---

# CLASP

CLASP is a real-time signal router that bridges protocols like OSC, MIDI, MQTT, DMX, Art-Net, sACN, HTTP, and WebSocket into a single unified API. One API for state, events, and streams across JavaScript, Python, and Rust -- with built-in auth, persistence, federation, and 8 protocol bridges.

## Quick Install

```bash
npm install @clasp-to/core
```

```javascript
import { ClaspBuilder } from '@clasp-to/core';

const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('My App')
  .connect();

client.set('/lights/brightness', 0.8);
client.on('/lights/*', (value, address) => console.log(address, value));
```

## Features

| Category | Details |
|---|---|
| **Protocol Bridges** | OSC, MIDI, MQTT, DMX, Art-Net, sACN, HTTP, WebSocket |
| **Signal Types** | Param, Event, Stream, Gesture, Timeline |
| **State Management** | Revisions, conflict resolution, late-joiner sync |
| **Auth** | CPSK tokens, Ed25519 capability delegation, entity registry |
| **Server Features** | Rules engine, journal persistence, app config |
| **Deployment** | Federation, discovery, Docker, cloud |

## Start Here

| Goal | Link |
|---|---|
| New to CLASP? | [First Connection](getting-started/first-connection.md) |
| Building a client? | [JavaScript SDK](sdk/javascript.md) / [Python SDK](sdk/python.md) / [Rust SDK](sdk/rust.md) |
| Deploying to production? | [Relay Server](deployment/relay.md) |
| Understanding the protocol? | [Architecture](concepts/architecture.md) |

## Choose Your Path

**Web / Desktop Developer**
SDK guide -> State management -> Auth tokens. Start with the [JavaScript SDK](sdk/javascript.md), learn [state operations](core/state.md), then configure [auth](auth/README.md).

**Creative / Live Performance**
Connect your tools, bridge protocols, deploy. Start with [First Connection](getting-started/first-connection.md), set up [Protocol Bridges](protocols/README.md), then [deploy](deployment/relay.md).

**IoT / Embedded**
Lightweight clients, automatic discovery, multi-site federation. Start with the [Embedded SDK](sdk/rust.md), configure [Discovery](server/discovery.md), then set up [Federation](server/federation.md).

**DevOps / Deployment**
Run, secure, and scale CLASP infrastructure. Start with the [Relay Server](deployment/relay.md), containerize with [Docker](deployment/docker.md), then follow the [Production Checklist](deployment/production-checklist.md).

---
title: "How-To Guides"
description: "Task-oriented guides that solve specific problems. Each guide assumes you have basic CLASP knowledge (complete a tutorial first)."
section: how-to
order: 0
---
# How-To Guides

Task-oriented guides that solve specific problems. Each guide assumes you have basic CLASP knowledge (complete a [tutorial](../tutorials/README.md) first).

## Installation

- [Install CLI](installation/cli.md) — Install the CLASP command-line tool
- [Install Desktop App](installation/desktop-app.md) — Install the visual interface
- [Rust Library](installation/rust-library.md) — Add CLASP to a Rust project
- [JavaScript Library](installation/javascript-library.md) — Add CLASP to JS/TS projects
- [Python Library](installation/python-library.md) — Add CLASP to Python projects
- [Docker](installation/docker.md) — Run CLASP in containers

## Connections

- [Start a Router](connections/start-router.md) — Start the CLASP router
- [Connect a Client](connections/connect-client.md) — Connect to a router
- [Add OSC](connections/add-osc.md) — Connect OSC devices
- [Add MIDI](connections/add-midi.md) — Connect MIDI devices
- [Add Art-Net](connections/add-artnet.md) — Connect Art-Net lighting
- [Add DMX](connections/add-dmx.md) — Connect DMX fixtures
- [Add MQTT](connections/add-mqtt.md) — Connect MQTT brokers
- [Add HTTP](connections/add-http.md) — Add REST API access
- [Add WebSocket](connections/add-websocket.md) — Add WebSocket bridge

## State

- [Subscribe to Changes](state/subscribe-to-changes.md) — Listen for value updates
- [Get and Set Values](state/get-set-values.md) — Read and write parameters
- [Handle Conflicts](state/handle-conflicts.md) — Resolve concurrent writes
- [Use Locks](state/use-locks.md) — Exclusive parameter control
- [Late Joiner Sync](state/late-joiner-sync.md) — Sync state on connect

## Timing

- [Clock Sync](timing/clock-sync.md) — Synchronize clocks
- [Scheduled Bundles](timing/scheduled-bundles.md) — Schedule future execution
- [Atomic Bundles](timing/bundle-atomic.md) — Execute messages atomically

## Discovery

- [mDNS Discovery](discovery/mdns-discovery.md) — Auto-discover on LAN
- [UDP Broadcast](discovery/udp-broadcast.md) — Fallback discovery
- [Manual Connection](discovery/manual-connection.md) — Configure manually

## Security

- [Enable TLS](security/enable-tls.md) — Encrypt connections
- [Capability Tokens](security/capability-tokens.md) — Access control
- [Pairing](security/pairing.md) — Zero-config security

## Advanced

- [P2P with WebRTC](advanced/p2p-webrtc.md) — Peer-to-peer connections
- [Build a Custom Bridge](advanced/custom-bridge.md) — Create protocol bridges
- [Embed a Router](advanced/embed-router.md) — Router in your application
- [Performance Tuning](advanced/performance-tuning.md) — Optimize throughput

## Troubleshooting

- [Troubleshooting Guide](troubleshooting.md) — Common issues and solutions

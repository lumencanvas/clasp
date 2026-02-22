---
title: MQTT Bridge
description: Bridge MQTT to CLASP
order: 4
---

# MQTT Bridge

Bridge MQTT messages to CLASP. MQTT topics map to CLASP paths under the `/mqtt` namespace. The bridge supports both standalone mode (connecting to an external MQTT broker) and embedded server mode (the relay acts as the MQTT broker).

## Quick Start

**Standalone bridge** -- connects to an existing MQTT broker and forwards messages to a CLASP router:

```bash
clasp mqtt --broker mqtt://localhost:1883 --target ws://localhost:7330
```

**Embedded in relay** -- the relay itself becomes an MQTT broker:

```bash
clasp-relay --mqtt-port 1883
```

MQTT clients connect directly to the relay on port 1883. No external broker needed.

## Address Mapping

MQTT topic separators (`/`) map directly to CLASP path separators. The namespace prefix (default `/mqtt`) is prepended:

| MQTT Topic | CLASP Address |
|---|---|
| `sensors/temp` | `/mqtt/sensors/temp` |
| `home/living-room/light` | `/mqtt/home/living-room/light` |
| `device/status` | `/mqtt/device/status` |

The mapping is bidirectional:

- **Inbound**: MQTT publishes are translated to CLASP signals and forwarded to the router.
- **Outbound**: CLASP signals published to addresses under `/mqtt/**` are translated back to MQTT publishes on the corresponding topic.

MQTT wildcard subscriptions (`+` and `#`) are translated to CLASP glob patterns (`*` and `**`):

| MQTT Subscription | CLASP Subscription |
|---|---|
| `sensors/+/temp` | `/mqtt/sensors/*/temp` |
| `sensors/#` | `/mqtt/sensors/**` |

To customize the namespace:

```bash
clasp mqtt --broker mqtt://localhost:1883 --namespace /iot --target ws://localhost:7330
```

## Signal Type Mapping

MQTT's retained flag determines the CLASP signal type:

| MQTT Message | CLASP Signal Type | Behavior |
|---|---|---|
| Retained message | Param | Stored in state, delivered to late joiners |
| Non-retained message | Event | Fire-and-forget, not stored |

MQTT QoS levels map to CLASP delivery guarantees:

| MQTT QoS | CLASP QoS | Guarantee |
|---|---|---|
| QoS 0 | Fire | At most once, no acknowledgment |
| QoS 1 | Confirm | At least once, acknowledged |
| QoS 2 | Commit | Exactly once, two-phase |

## Embedded Server Mode

The relay can act as a full MQTT broker, eliminating the need for an external broker like Mosquitto:

```bash
clasp-relay --mqtt-port 1883 --mqtt-namespace /mqtt
```

In this mode:

- MQTT clients connect directly to the relay on the specified port
- MQTT messages are translated to CLASP signals in-process (no network hop)
- CLASP WebSocket clients and MQTT clients share the same state
- Retained MQTT messages are backed by the CLASP state store

This is the recommended setup for new deployments. Use standalone mode only when you need to bridge an existing MQTT broker that other non-CLASP services depend on.

## JSON Payloads

MQTT payloads are interpreted based on content:

**JSON payloads** are parsed into CLASP Map values:

```
MQTT: sensors/temp  {"value": 22.5, "unit": "C"}
CLASP: /mqtt/sensors/temp  Map{"value": 22.5, "unit": "C"}
```

**Numeric strings** are parsed as numbers:

```
MQTT: sensors/temp  "22.5"
CLASP: /mqtt/sensors/temp  22.5
```

**Plain strings** remain as strings:

```
MQTT: device/status  "online"
CLASP: /mqtt/device/status  "online"
```

**Binary payloads** are stored as bytes:

```
MQTT: firmware/chunk  <binary>
CLASP: /mqtt/firmware/chunk  bytes(...)
```

## Configuration

| Option | CLI Flag | Default | Description |
|---|---|---|---|
| Broker URL | `--broker` | -- | MQTT broker to connect to (standalone mode) |
| Target router | `--target` | `ws://localhost:7330` | WebSocket URL of the CLASP router |
| Namespace | `--namespace` | `/mqtt` | CLASP path prefix for bridged topics |
| Client ID | `--client-id` | (auto) | MQTT client ID |
| Username | `--username` | -- | MQTT broker username |
| Password | `--password` | -- | MQTT broker password |
| Subscribe | `--subscribe` | `#` | MQTT topic filter to subscribe to |

## Troubleshooting

**Connection refused to broker**
- Verify the broker URL is correct and the broker is running: `mosquitto_sub -h localhost -p 1883 -t '#'`
- Check that the broker allows connections from the bridge's IP address

**Messages not appearing in CLASP**
- Confirm the bridge's `--subscribe` filter matches the topics you expect
- Check the namespace: messages appear under `/mqtt/...` by default, not at the root

**Retained messages not synced**
- In standalone mode, the bridge receives retained messages on initial subscription. If the broker has no retained messages, CLASP state will be empty.
- In embedded mode, retained messages are stored in the CLASP state store and survive restarts if journal persistence is enabled.

**Duplicate messages**
- MQTT QoS 1 guarantees at-least-once delivery, which may produce duplicates. CLASP deduplicates by revision number, so state remains consistent.

## Next Steps

- [Art-Net Bridge](artnet.md) -- bridge DMX over Ethernet
- [Protocol Bridges Overview](../protocols/README.md) -- all 8 protocol bridges
- Bridge Configuration -- custom topic mappings and value transforms

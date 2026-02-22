---
title: OSC Bridge
description: Bridge OpenSound Control to CLASP
order: 2
---

# OSC Bridge

Bridge OpenSound Control (OSC) messages to CLASP. OSC addresses map directly to CLASP paths under the `/osc` namespace, making existing OSC-based tools (TouchOSC, Max/MSP, SuperCollider, Ableton Live) work with CLASP without reconfiguration.

## Quick Start

**Standalone bridge** -- runs as a separate process, connects to a router:

```bash
clasp osc --listen 8000 --target ws://localhost:7330
```

**Embedded in relay** -- runs inside the relay process:

```bash
clasp-relay --osc-port 8000
```

Both accept OSC messages on UDP port 8000 and route them into the CLASP address space.

## Address Mapping

OSC addresses map one-to-one to CLASP paths by prepending the namespace prefix (default `/osc`):

| OSC Address | CLASP Address |
|---|---|
| `/synth/volume` | `/osc/synth/volume` |
| `/mixer/ch/1/fader` | `/osc/mixer/ch/1/fader` |
| `/cue/go` | `/osc/cue/go` |

The mapping is bidirectional:

- **Inbound**: OSC messages arriving on the listen port are translated to CLASP signals and forwarded to the router.
- **Outbound**: CLASP signals published to addresses under `/osc/**` are translated back to OSC messages and sent to the originating OSC client (or a configured target).

To customize the namespace:

```bash
clasp osc --listen 8000 --namespace /stage/osc --target ws://localhost:7330
```

## Value Mapping

OSC argument types map to CLASP value types:

| OSC Type | Tag | CLASP Type |
|---|---|---|
| float32 | `f` | f64 |
| int32 | `i` | i64 |
| string | `s` | string |
| blob | `b` | bytes |
| True / False | `T` / `F` | bool |
| double | `d` | f64 |
| int64 | `h` | i64 |

**Multi-argument messages**: OSC messages with multiple arguments are mapped to a CLASP array value. For example, `/color 255 0 128` becomes `[255, 0, 128]` at `/osc/color`.

**OSC bundles**: An OSC bundle is translated to a CLASP bundle, preserving the timetag as a scheduled execution time. Nested bundles are flattened.

## Configuration

| Option | CLI Flag | Default | Description |
|---|---|---|---|
| Listen port | `--listen` | `8000` | UDP port for incoming OSC messages |
| Target router | `--target` | `ws://localhost:7330` | WebSocket URL of the CLASP router |
| Namespace | `--namespace` | `/osc` | CLASP path prefix for bridged addresses |
| Send port | `--send-port` | (auto) | UDP port for outbound OSC messages |
| Transport | `--transport` | `udp` | `udp` or `tcp` |

## Embedded Mode

Run the OSC bridge inside the relay process for lower latency and simpler deployment:

```bash
clasp-relay --osc-port 8000 --osc-namespace /osc
```

In embedded mode, the bridge shares the relay's state store directly. There is no network hop between the bridge and the router. This is the recommended setup when the relay and OSC devices are on the same network.

Embedded mode supports the same address and value mappings as standalone mode.

## Troubleshooting

**No messages received**
- Verify the listen port matches what your OSC sender is targeting: `clasp osc --listen 8000`
- Check firewall rules -- UDP port must be open for inbound traffic
- Confirm the sender is using UDP (not TCP) if the bridge is in default UDP mode

**Namespace mismatch**
- If you set a custom namespace with `--namespace`, CLASP clients must subscribe to that namespace (e.g., `/stage/osc/**` instead of `/osc/**`)

**Messages received but not forwarded**
- Check that the target router is reachable: `curl -s ws://localhost:7330` or verify the relay is running
- Look at bridge logs for connection errors: `clasp osc --listen 8000 --target ws://localhost:7330 --log-level debug`

**OSC bundles not arriving atomically**
- Ensure the CLASP client subscribes with bundle awareness. Bundles are delivered as atomic units only to subscribers that request bundle delivery.

## Next Steps

- [MIDI Bridge](midi.md) -- bridge MIDI controllers and instruments
- [Protocol Bridges Overview](../protocols/README.md) -- all 8 protocol bridges
- Bridge Configuration -- address mapping and value transforms

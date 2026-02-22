---
title: sACN Bridge
description: Bridge sACN/E1.31 streaming to CLASP
order: 7
---

# sACN Bridge

Bridge sACN (E1.31 Streaming ACN) to CLASP. sACN is a standard for streaming DMX data over Ethernet using multicast, commonly used in large-scale lighting installations where Art-Net's broadcast model becomes inefficient.

## Quick Start

```bash
clasp bridge sacn --target ws://localhost:7330
```

The bridge joins multicast groups for the configured universes and begins receiving sACN packets.

## Address Mapping

sACN universe/channel pairs map to CLASP addresses under the `/sacn` namespace, following the same pattern as Art-Net:

| sACN | CLASP Address | Value Range |
|---|---|---|
| Universe 1, Channel 1 | `/sacn/1/1` | 0-255 |
| Universe 1, Channel 512 | `/sacn/1/512` | 0-255 |
| Universe 10, Channel 100 | `/sacn/10/100` | 0-255 |

- Universes range from 1 to 63999
- Channels range from 1 to 512 per universe

The mapping is bidirectional. Setting `/sacn/1/100` to `128` in CLASP causes the bridge to transmit an sACN packet updating universe 1, channel 100.

To customize the namespace:

```bash
clasp bridge sacn --namespace /venue/sacn --target ws://localhost:7330
```

## Multicast

sACN uses IP multicast for efficient one-to-many distribution. Each universe has a dedicated multicast group address (`239.255.{universe_high}.{universe_low}`). The bridge automatically:

- Joins multicast groups for all configured universes on startup
- Leaves multicast groups on shutdown
- Handles IGMP membership so network switches can optimize traffic delivery

This is more network-efficient than Art-Net's broadcast approach, especially in installations with many universes spread across multiple network segments.

## Configuration

| Option | CLI Flag | Default | Description |
|---|---|---|---|
| Target router | `--target` | `ws://localhost:7330` | WebSocket URL of the CLASP router |
| Namespace | `--namespace` | `/sacn` | CLASP path prefix for bridged addresses |
| Interface | `--interface` | `0.0.0.0` | Network interface to bind for multicast |
| Universes | `--universes` | `1-10` | sACN universes to subscribe to |
| Priority | `--priority` | `100` | sACN source priority (0-200) for outbound packets |
| Mode | `--mode` | `channel` | `channel` (individual values) or `universe` (full frame) |
| Source name | `--source-name` | `clasp-sacn` | sACN source name for outbound packets |

## sACN vs Art-Net

Both protocols carry DMX data over Ethernet. Choose based on your needs:

| Feature | sACN | Art-Net |
|---|---|---|
| Transport | Multicast (efficient) | Broadcast (simple) |
| Universe limit | 63999 | 32768 |
| Priority system | Yes (per-source) | No |
| Network efficiency | Better at scale | Better for small setups |
| Discovery | Via protocol | Via ArtPoll |

If you are already using Art-Net, see the [Art-Net Bridge](artnet.md). Both bridges can run simultaneously with different namespaces.

## Troubleshooting

**No sACN data received**
- Verify multicast is working on your network: `tcpdump -i en0 multicast and udp port 5568`
- Ensure the `--interface` flag points to the correct network interface
- Check that network switches support IGMP snooping (or disable it for testing)

**Missing universes**
- Confirm the `--universes` range includes the universes your sACN source is transmitting
- Each universe requires joining a separate multicast group -- some switches limit IGMP group membership

**Priority conflicts**
- sACN supports per-source priority (0-200, higher wins). If another source has higher priority, its values take precedence. Set `--priority` to match or exceed the other source.

## Next Steps

- [HTTP Bridge](http.md) -- REST API access to CLASP state
- [Art-Net Bridge](artnet.md) -- alternative DMX-over-Ethernet protocol
- [Protocol Bridges Overview](../protocols/README.md) -- all 8 protocol bridges

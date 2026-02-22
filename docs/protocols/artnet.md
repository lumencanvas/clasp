---
title: Art-Net Bridge
description: Bridge Art-Net DMX over Ethernet to CLASP
order: 5
---

# Art-Net Bridge

Bridge Art-Net (DMX over Ethernet) to CLASP. Art-Net universe and channel pairs map to CLASP addresses, allowing lighting fixtures, media servers, and console software to be controlled through the CLASP signal graph.

## Quick Start

```bash
clasp bridge artnet --target ws://localhost:7330
```

The bridge listens for Art-Net packets on the default Art-Net port (6454/UDP) and forwards channel values to the CLASP router.

## Address Mapping

Art-Net universe/channel pairs map to CLASP addresses under the `/artnet` namespace:

| Art-Net | CLASP Address |
|---|---|
| Universe 0, Channel 1 | `/artnet/0/1` |
| Universe 0, Channel 512 | `/artnet/0/512` |
| Universe 1, Channel 1 | `/artnet/1/1` |
| Universe 15, Channel 256 | `/artnet/15/256` |

- Universes range from 0 to 32767
- Channels range from 1 to 512 per universe
- Values are integers 0-255 (raw DMX) or normalized floats 0.0-1.0 depending on configuration

The mapping is bidirectional. Setting `/artnet/0/100` to `200` in CLASP causes the bridge to transmit an Art-Net packet updating universe 0, channel 100 to value 200.

To customize the namespace:

```bash
clasp bridge artnet --namespace /lights/artnet --target ws://localhost:7330
```

## Configuration

| Option | CLI Flag | Default | Description |
|---|---|---|---|
| Target router | `--target` | `ws://localhost:7330` | WebSocket URL of the CLASP router |
| Namespace | `--namespace` | `/artnet` | CLASP path prefix for bridged addresses |
| Interface | `--interface` | `0.0.0.0` | Network interface to listen on |
| Universe range | `--universes` | `0-15` | Art-Net universes to bridge |
| Normalize | `--normalize` | `false` | Map 0-255 to 0.0-1.0 float values |
| Mode | `--mode` | `channel` | `channel` (individual values) or `universe` (full frame) |

## Performance

Art-Net transmits full 512-channel DMX frames at up to 44Hz per universe. The bridge handles this in two ways:

**Channel mode** (default): the bridge compares each incoming frame to the previous frame and emits CLASP signals only for channels that changed. This reduces traffic when only a few channels are active.

**Universe mode**: the bridge emits the entire 512-channel frame as a single CLASP Stream signal at `/artnet/{universe}`. This is efficient when most channels change every frame (e.g., pixel mapping) and preserves frame atomicity.

```bash
# Channel mode (default) -- individual channel updates
clasp bridge artnet --mode channel --target ws://localhost:7330

# Universe mode -- full 512-channel frames as Stream signals
clasp bridge artnet --mode universe --target ws://localhost:7330
```

For high-universe-count installations, limit the bridged universe range to reduce load:

```bash
clasp bridge artnet --universes 0-3 --target ws://localhost:7330
```

## Troubleshooting

**No Art-Net data received**
- Art-Net uses UDP port 6454. Ensure no other Art-Net node is binding the same port on the same machine.
- Check that the bridge's `--interface` matches the network where Art-Net traffic is present.
- Use a packet sniffer to confirm Art-Net packets are arriving: `tcpdump -i en0 udp port 6454`

**High CPU usage**
- Receiving many universes at 44Hz generates significant traffic. Limit `--universes` to only those you need.
- Use universe mode instead of channel mode if most channels change every frame.

**Value range mismatch**
- By default, values are raw integers 0-255. Use `--normalize` to get 0.0-1.0 floats, which are compatible with other CLASP sources like OSC faders.

## Next Steps

- [DMX Bridge](dmx.md) -- bridge DMX-512 via USB serial
- [sACN Bridge](sacn.md) -- multicast alternative to Art-Net
- [Protocol Bridges Overview](../protocols/README.md) -- all 8 protocol bridges

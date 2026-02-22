---
title: "Art-Net Bridge"
description: "Mapping between Art-Net and CLASP."
section: reference
order: 1
---
# Art-Net Bridge

Mapping between Art-Net and CLASP.

## Overview

The Art-Net bridge translates between Art-Net DMX data and CLASP signals, enabling control of professional lighting equipment.

## Address Format

```
/artnet/{net}/{subnet}/{universe}/{channel}
```

- **net**: 0-127 (Art-Net network)
- **subnet**: 0-15 (Art-Net subnet)
- **universe**: 0-15 (Universe within subnet)
- **channel**: 1-512 (DMX channel)

### Common Addressing

Most setups use net=0, subnet=0:

```
/artnet/0/0/0/1    # Universe 0, Channel 1
/artnet/0/0/1/1    # Universe 1, Channel 1
/artnet/0/0/2/100  # Universe 2, Channel 100
```

## Value Mapping

### CLASP to Art-Net

| CLASP Value | DMX Value |
|-------------|-----------|
| Int 0-255 | Direct mapping |
| Float 0.0-1.0 | Scaled to 0-255 |
| Bool true | 255 |
| Bool false | 0 |

```javascript
// All equivalent for full brightness:
await client.set('/artnet/0/0/0/1', 255);
await client.set('/artnet/0/0/0/1', 1.0);
await client.set('/artnet/0/0/0/1', true);
```

### Art-Net to CLASP

Incoming DMX values (0-255) become Int values:

```javascript
client.on('/artnet/0/0/0/1', (value) => {
  console.log(value);  // 0-255
});
```

## Signal Types

### Param (Default)

Individual channel control:

```javascript
await client.set('/artnet/0/0/0/1', 255);
```

### Stream

High-rate dimmer control:

```javascript
const stream = client.stream('/artnet/0/0/0/1');
stream.send(fadeValue);
```

## Bulk Updates

### Universe Snapshot

Set entire universe at once:

```javascript
const channels = new Array(512).fill(0);
channels[0] = 255;  // Channel 1
channels[1] = 128;  // Channel 2

await client.set('/artnet/0/0/0', channels);
```

### Bundles

Atomic multi-channel updates:

```javascript
await client.bundle()
  .set('/artnet/0/0/0/1', 255)  // R
  .set('/artnet/0/0/0/2', 128)  // G
  .set('/artnet/0/0/0/3', 64)   // B
  .execute();
```

## Universe Subscription

Subscribe to entire universe:

```javascript
// Any channel in universe 0
client.on('/artnet/0/0/0/*', (value, address) => {
  const channel = parseInt(address.split('/').pop());
  console.log(`Channel ${channel}: ${value}`);
});

// All universes
client.on('/artnet/**', (value, address) => {
  console.log(`${address}: ${value}`);
});
```

## Refresh Rate

Art-Net typically operates at 44 fps (22.7ms per frame). The bridge:

- Batches CLASP updates into Art-Net frames
- Sends minimum of 1 frame per second (even if no changes)
- Respects Art-Net timing requirements

## Configuration

### CLI

```bash
clasp artnet --bind 0.0.0.0:6454
```

### Configuration File

```yaml
artnet:
  bind: "0.0.0.0:6454"

  # Broadcast address for output
  broadcast: "255.255.255.255:6454"

  # Or specific targets
  targets:
    - "192.168.1.50:6454"
    - "192.168.1.51:6454"

  # Universes to handle
  universes: [0, 1, 2, 3]

  # Direction
  input_enabled: true
  output_enabled: true

  # Refresh rate (fps)
  refresh_rate: 44
```

### Rust API

```rust
use clasp_bridge::artnet::{ArtNetBridge, ArtNetConfig};

let config = ArtNetConfig {
    bind_addr: "0.0.0.0:6454".parse()?,
    broadcast_addr: "255.255.255.255:6454".parse()?,
    universes: vec![0, 1, 2, 3],
    input_enabled: true,
    output_enabled: true,
    refresh_rate: 44,
};

let bridge = ArtNetBridge::new(client, config).await?;
```

## Art-Net Port Address

Art-Net uses a 15-bit port address:

```
Port Address = (Net << 8) | (Subnet << 4) | Universe
```

Examples:
- Net 0, Subnet 0, Universe 0 → Port 0x0000
- Net 0, Subnet 0, Universe 1 → Port 0x0001
- Net 0, Subnet 1, Universe 0 → Port 0x0010

## Art-Net OpCodes

Supported:
- **OpDmx** (0x5000) - DMX data
- **OpPoll** (0x2000) - Device discovery
- **OpPollReply** (0x2100) - Discovery response

## Network Configuration

### Broadcast Mode

Default for most setups:

```yaml
artnet:
  broadcast: "255.255.255.255:6454"
```

### Unicast Mode

For specific nodes:

```yaml
artnet:
  targets:
    - "192.168.1.50:6454"
```

### Multicast Mode

For larger installations:

```yaml
artnet:
  multicast: "239.255.0.1:6454"
```

## Fixture Addresses

Common fixture addressing:

```javascript
// 4-channel RGBW fixture starting at channel 1
const fixture = {
  r: '/artnet/0/0/0/1',
  g: '/artnet/0/0/0/2',
  b: '/artnet/0/0/0/3',
  w: '/artnet/0/0/0/4'
};

await client.bundle()
  .set(fixture.r, 255)
  .set(fixture.g, 128)
  .set(fixture.b, 64)
  .set(fixture.w, 0)
  .execute();
```

## See Also

- [Add Art-Net](../../how-to/connections/add-artnet.md)
- [DMX Bridge](dmx.md)
- [Live Performance](../../use-cases/live-performance.md)

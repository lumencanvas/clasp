# sACN Bridge

Mapping between sACN (E1.31) and CLASP.

## Overview

The sACN (Streaming ACN) bridge translates between sACN/E1.31 DMX over IP and CLASP signals. sACN is commonly used in professional entertainment and architectural lighting.

## Address Format

```
/sacn/{universe}/{channel}
```

- **universe**: 1-63999 (sACN universe)
- **channel**: 1-512 (DMX channel)

```
/sacn/1/1     # Universe 1, Channel 1
/sacn/1/512   # Universe 1, Channel 512
/sacn/10/100  # Universe 10, Channel 100
```

## Value Mapping

| CLASP Value | DMX Value |
|-------------|-----------|
| Int 0-255 | Direct mapping |
| Float 0.0-1.0 | Scaled to 0-255 |
| Bool true | 255 |
| Bool false | 0 |

## Configuration

### CLI

```bash
clasp sacn --universes 1,2,3,4
```

### Configuration File

```yaml
sacn:
  # Source settings
  source_name: "CLASP Bridge"
  cid: "00000000-0000-0000-0000-000000000001"  # Optional UUID
  priority: 100  # 0-200, default 100

  # Universes
  universes: [1, 2, 3, 4]

  # Multicast (default)
  multicast: true

  # Or unicast targets
  # multicast: false
  # targets:
  #   - "192.168.1.50"
  #   - "192.168.1.51"

  # Timing
  refresh_rate: 44  # fps

clasp:
  router: "ws://localhost:7330"
```

### Rust API

```rust
use clasp_bridge::sacn::{SacnBridge, SacnConfig};

let config = SacnConfig {
    source_name: "CLASP Bridge".into(),
    universes: vec![1, 2, 3, 4],
    priority: 100,
    multicast: true,
    refresh_rate: 44,
};

let bridge = SacnBridge::new(client, config).await?;
```

## sACN vs Art-Net

| Feature | sACN | Art-Net |
|---------|------|---------|
| Standard | ANSI E1.31 | Artistic License |
| Universes | 1-63999 | 0-32767 |
| Multicast | Native | Extension |
| Sync | E1.31-2018 | ArtSync |
| Discovery | E1.31-2016 | ArtPoll |
| Per-address priority | Yes | No |

## Priority

sACN supports per-source priority (0-200):

```yaml
sacn:
  priority: 100  # Default
```

Higher priority sources take precedence when multiple sources control the same universe.

## Synchronization

E1.31-2018 synchronization:

```yaml
sacn:
  sync:
    enabled: true
    universe: 65535  # Sync universe
```

## Universe Discovery

sACN receivers can discover available universes:

```yaml
sacn:
  discovery:
    enabled: true
    interval: 10  # seconds
```

## Multicast Addresses

sACN uses multicast addresses:

```
239.255.{universe_high}.{universe_low}
```

Examples:
- Universe 1: 239.255.0.1
- Universe 256: 239.255.1.0
- Universe 1000: 239.255.3.232

## Unicast Mode

For point-to-point communication:

```yaml
sacn:
  multicast: false
  targets:
    - "192.168.1.50"
    - "192.168.1.51"
```

## Per-Universe Settings

Configure different settings per universe:

```yaml
sacn:
  universes:
    - number: 1
      priority: 100
    - number: 2
      priority: 150
    - number: 10
      priority: 100
      unicast_targets: ["192.168.1.100"]
```

## Examples

### Set Single Channel

```javascript
await client.set('/sacn/1/1', 255);
```

### Set Fixture

```javascript
// 6-channel fixture at address 1
await client.bundle()
  .set('/sacn/1/1', 255)   // Intensity
  .set('/sacn/1/2', 128)   // Red
  .set('/sacn/1/3', 64)    // Green
  .set('/sacn/1/4', 32)    // Blue
  .set('/sacn/1/5', 0)     // White
  .set('/sacn/1/6', 0)     // Strobe
  .execute();
```

### Subscribe to Universe

```javascript
client.on('/sacn/1/*', (value, address) => {
  const channel = parseInt(address.split('/').pop());
  console.log(`Channel ${channel}: ${value}`);
});
```

## See Also

- [Art-Net Bridge](artnet.md)
- [DMX Bridge](dmx.md)
- [Live Performance](../../use-cases/live-performance.md)

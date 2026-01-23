# DMX Bridge

Direct DMX512 output for CLASP.

## Overview

The DMX bridge sends DMX512 data via USB or serial interfaces, enabling direct control of lighting fixtures without an Art-Net network.

## Address Format

```
/dmx/{universe}/{channel}
```

- **universe**: 0-based universe index
- **channel**: 1-512 (DMX channel)

```
/dmx/0/1    # Universe 0, Channel 1
/dmx/0/512  # Universe 0, Channel 512
/dmx/1/1    # Universe 1, Channel 1
```

## Value Mapping

| CLASP Value | DMX Value |
|-------------|-----------|
| Int 0-255 | Direct mapping |
| Float 0.0-1.0 | Scaled to 0-255 |
| Bool true | 255 |
| Bool false | 0 |

```javascript
// All equivalent:
await client.set('/dmx/0/1', 255);
await client.set('/dmx/0/1', 1.0);
await client.set('/dmx/0/1', true);
```

## Interface Types

### FTDI USB-DMX

Most common USB-DMX interfaces:

```yaml
dmx:
  interface: ftdi
  device_index: 0  # First FTDI device
```

### Serial

Generic serial DMX:

```yaml
dmx:
  interface: serial
  port: "/dev/ttyUSB0"
  baud: 250000
```

### Enttec Pro

Enttec DMX USB Pro:

```yaml
dmx:
  interface: enttec
  port: "/dev/ttyUSB0"
```

### Open DMX

Enttec Open DMX USB:

```yaml
dmx:
  interface: open_dmx
  port: "/dev/ttyUSB0"
```

## Configuration

### CLI

```bash
# FTDI interface
clasp dmx --interface ftdi --device 0

# Serial interface
clasp dmx --interface serial --port /dev/ttyUSB0

# Enttec Pro
clasp dmx --interface enttec --port /dev/ttyUSB0
```

### Configuration File

```yaml
dmx:
  interface: ftdi
  device_index: 0

  # Or serial:
  # interface: serial
  # port: "/dev/ttyUSB0"
  # baud: 250000

  universe: 0
  refresh_rate: 44  # fps

clasp:
  router: "ws://localhost:7330"
```

### Rust API

```rust
use clasp_bridge::dmx::{DmxBridge, DmxConfig, DmxInterface};

let config = DmxConfig {
    interface: DmxInterface::Ftdi { device_index: 0 },
    universe: 0,
    refresh_rate: 44,
};

let bridge = DmxBridge::new(client, config).await?;
```

## DMX Timing

DMX512 specifications:
- Baud rate: 250,000
- Break: 88µs minimum
- Mark After Break (MAB): 8µs minimum
- Frame rate: Up to 44 fps

The bridge handles timing automatically.

## Bulk Updates

### Set Multiple Channels

```javascript
// Set channels 1-4 (RGBW fixture)
await client.bundle()
  .set('/dmx/0/1', 255)  // R
  .set('/dmx/0/2', 128)  // G
  .set('/dmx/0/3', 64)   // B
  .set('/dmx/0/4', 200)  // W
  .execute();
```

### Set Universe

```javascript
// Set all 512 channels at once
const channels = new Array(512).fill(0);
channels[0] = 255;
channels[1] = 128;

await client.set('/dmx/0', channels);
```

## Multiple Universes

Connect multiple interfaces:

```yaml
dmx:
  universes:
    - interface: ftdi
      device_index: 0
      universe: 0
    - interface: ftdi
      device_index: 1
      universe: 1
```

## Comparison with Art-Net

| Feature | DMX Bridge | Art-Net Bridge |
|---------|------------|----------------|
| Connection | USB/Serial | Ethernet |
| Universes | 1-2 typical | 256+ |
| Latency | Very low | Low |
| Cost | Low | Medium |
| Distance | 300m max | Unlimited |

Use DMX for:
- Single universe installations
- Direct fixture control
- Budget setups

Use Art-Net for:
- Multi-universe
- Long distances
- Network flexibility

## Troubleshooting

### Device Not Found

1. Check USB connection
2. Verify device permissions:
   ```bash
   sudo chmod 666 /dev/ttyUSB0
   # Or add user to dialout group:
   sudo usermod -a -G dialout $USER
   ```
3. List devices:
   ```bash
   ls /dev/ttyUSB*
   ls /dev/serial/by-id/
   ```

### No Output

1. Check cable polarity (pins 2, 3)
2. Verify DMX termination (120Ω)
3. Test with DMX tester
4. Check refresh rate isn't too high

## See Also

- [Add DMX](../../how-to/connections/add-dmx.md)
- [Art-Net Bridge](artnet.md)
- [Live Performance](../../use-cases/live-performance.md)

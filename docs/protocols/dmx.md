---
title: DMX Bridge
description: Bridge DMX-512 via USB to CLASP
order: 6
---

# DMX Bridge

Bridge DMX-512 via USB interfaces to CLASP. Channel values from a DMX universe are mapped to CLASP addresses, allowing physical lighting fixtures to be read from and written to through the CLASP signal graph.

## Quick Start

```bash
clasp bridge dmx --port /dev/ttyUSB0 --target ws://localhost:7330
```

On macOS, the port is typically `/dev/tty.usbserial-*`. On Windows, use `COM3` or similar.

## Address Mapping

DMX channels map to CLASP addresses under the `/dmx` namespace:

| DMX Channel | CLASP Address | Value Range |
|---|---|---|
| Channel 1 | `/dmx/1` | 0-255 |
| Channel 100 | `/dmx/100` | 0-255 |
| Channel 512 | `/dmx/512` | 0-255 |

The mapping is bidirectional:

- **Inbound**: DMX channel values read from the USB interface are published as CLASP Param signals.
- **Outbound**: Setting `/dmx/{channel}` in CLASP writes the value to the DMX output on the USB interface.

To customize the namespace:

```bash
clasp bridge dmx --port /dev/ttyUSB0 --namespace /stage/dmx --target ws://localhost:7330
```

## Supported Hardware

The bridge supports FTDI-based DMX USB interfaces:

| Device | Support | Notes |
|---|---|---|
| ENTTEC Open DMX USB | Full | Output only, no RDM |
| ENTTEC DMX USB Pro | Full | Input and output, RDM capable |
| ENTTEC DMX USB Pro Mk2 | Full | Two universes |
| Generic FTDI USB-to-serial | Basic | Output only, may require driver configuration |

The ENTTEC DMX USB Pro is recommended for bidirectional operation (reading and writing DMX values).

## Configuration

| Option | CLI Flag | Default | Description |
|---|---|---|---|
| Serial port | `--port` | -- | Path to USB serial device (required) |
| Target router | `--target` | `ws://localhost:7330` | WebSocket URL of the CLASP router |
| Namespace | `--namespace` | `/dmx` | CLASP path prefix for bridged channels |
| Baud rate | `--baud` | `250000` | Serial baud rate (standard DMX is 250000) |
| Direction | `--direction` | `both` | `input`, `output`, or `both` |
| Channels | `--channels` | `1-512` | Channel range to bridge |
| Refresh rate | `--refresh` | `40` | DMX output refresh rate in Hz |

## Troubleshooting

**Device not found**
- List available serial ports: `ls /dev/tty.usb*` (macOS) or `ls /dev/ttyUSB*` (Linux)
- On Linux, ensure the user has permission: `sudo usermod -aG dialout $USER` (requires logout/login)
- On macOS, install FTDI VCP drivers if the device does not appear

**No DMX output**
- Verify the baud rate is 250000 (standard DMX-512)
- Check cable connections -- DMX uses 5-pin XLR (or 3-pin in some fixtures)
- Confirm the interface supports output (ENTTEC Open DMX USB is output-only)

**Flickering fixtures**
- Reduce the channel range with `--channels` to avoid sending data for unused channels
- Ensure only one bridge instance is writing to the same USB interface

## Next Steps

- [sACN Bridge](sacn.md) -- bridge sACN/E1.31 streaming over multicast
- [Art-Net Bridge](artnet.md) -- bridge DMX over Ethernet
- [Protocol Bridges Overview](../protocols/README.md) -- all 8 protocol bridges

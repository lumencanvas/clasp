---
title: "Add DMX"
description: "Control DMX lighting fixtures through CLASP."
section: how-to
order: 2
---
# Add DMX

Control DMX lighting fixtures through CLASP.

## Prerequisites

- Running CLASP router
- USB DMX interface (ENTTEC, FTDI-based, etc.)

## List DMX Devices

```bash
clasp dmx --list
```

Output:
```
DMX interfaces:
  /dev/ttyUSB0: ENTTEC DMX USB Pro
  /dev/ttyUSB1: Generic FTDI
```

## Start DMX Bridge

### CLI

```bash
clasp dmx --device /dev/ttyUSB0
```

On Windows:
```bash
clasp dmx --device COM3
```

### Desktop App

1. Click **Add Protocol**
2. Select **DMX**
3. Select USB device
4. Click **Start**

## Address Format

DMX channels are addressed as:

```
/dmx/{universe}/{channel}
```

- Universe: 0-based (usually 0)
- Channel: 1-512

## Control DMX Channels

```javascript
// Set channel 1 to full (255)
await client.set('/dmx/0/1', 255);

// Set channel 47 to 50%
await client.set('/dmx/0/47', 128);

// Set multiple channels atomically
await client.bundle([
  { set: ['/dmx/0/1', 255] },   // Dimmer
  { set: ['/dmx/0/2', 255] },   // Red
  { set: ['/dmx/0/3', 0] },     // Green
  { set: ['/dmx/0/4', 0] }      // Blue
]);
```

## RGB Fixture Example

For an RGB fixture starting at channel 1:

```javascript
// Set color to purple (R=255, G=0, B=255)
await client.bundle([
  { set: ['/dmx/0/1', 255] },   // Red
  { set: ['/dmx/0/2', 0] },     // Green
  { set: ['/dmx/0/3', 255] }    // Blue
]);

// Helper function
async function setRGB(startChannel, r, g, b) {
  await client.bundle([
    { set: [`/dmx/0/${startChannel}`, r] },
    { set: [`/dmx/0/${startChannel + 1}`, g] },
    { set: [`/dmx/0/${startChannel + 2}`, b] }
  ]);
}

await setRGB(1, 255, 128, 0);  // Orange
```

## Multiple Universes

```bash
# Universe 0 on first interface
clasp dmx --device /dev/ttyUSB0 --universe 0

# Universe 1 on second interface
clasp dmx --device /dev/ttyUSB1 --universe 1
```

```javascript
await client.set('/dmx/0/1', 255);  // Universe 0
await client.set('/dmx/1/1', 255);  // Universe 1
```

## Frame Rate

DMX updates at approximately 44 Hz by default. Adjust if needed:

```bash
clasp dmx --device /dev/ttyUSB0 --fps 30
```

## Troubleshooting

### Device not found

```bash
# Linux: Check device exists
ls -la /dev/ttyUSB*

# Add user to dialout group
sudo usermod -a -G dialout $USER
# Log out and back in

# Windows: Check COM port in Device Manager
```

### Lights flickering

- Ensure proper DMX termination (120Ω resistor at end of chain)
- Check cable quality
- Reduce frame rate: `--fps 25`

### No output

- Verify DMX addresses on fixtures
- Check DMX chain continuity
- Try direct connection (no splitters)

## Fixture Profiles

For complex fixtures, create a profile:

```javascript
const fixture = {
  name: 'RGB Par',
  startChannel: 1,
  channels: {
    dimmer: 0,
    red: 1,
    green: 2,
    blue: 3,
    strobe: 4
  }
};

async function setFixture(fixture, values) {
  const messages = Object.entries(values).map(([name, value]) => ({
    set: [`/dmx/0/${fixture.startChannel + fixture.channels[name]}`, value]
  }));
  await client.bundle(messages);
}

await setFixture(fixture, { dimmer: 255, red: 255, green: 0, blue: 128 });
```

## Art-Net Alternative

For DMX over Ethernet, use Art-Net instead:

```bash
clasp artnet --bind 0.0.0.0:6454
```

See [Add Art-Net](add-artnet.md).

## Next Steps

- [Add Art-Net](add-artnet.md)
- [DMX Bridge Reference](../../reference/bridges/dmx.md)
- [Live Performance Guide](../../use-cases/live-performance.md)

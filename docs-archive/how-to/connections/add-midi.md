---
title: "Add MIDI"
description: "Connect MIDI devices and applications to CLASP."
section: how-to
order: 4
---
# Add MIDI

Connect MIDI devices and applications to CLASP.

## Prerequisites

- Running CLASP router
- MIDI device connected to your computer

## List MIDI Devices

```bash
clasp midi --list
```

Output:
```
Input devices:
  0: Launchpad X
  1: USB MIDI Interface

Output devices:
  0: Launchpad X
  1: USB MIDI Interface
```

## Start MIDI Bridge

### CLI

```bash
clasp midi --device "Launchpad X"
```

Or by index:
```bash
clasp midi --device-index 0
```

### Desktop App

1. Click **Add Protocol**
2. Select **MIDI**
3. Select input/output devices
4. Click **Start**

## Address Mapping

MIDI messages map to CLASP addresses:

| MIDI | CLASP Address | Value |
|------|---------------|-------|
| Note On | `/midi/{device}/note` | `{note, velocity, channel}` |
| Note Off | `/midi/{device}/note` | `{note, velocity: 0, channel}` |
| CC | `/midi/{device}/cc/{channel}/{cc}` | 0-127 |
| Pitch Bend | `/midi/{device}/bend/{channel}` | -8192 to 8191 |
| Program Change | `/midi/{device}/program` | `{program, channel}` |

## Receive MIDI

```javascript
// All MIDI from device
client.on('/midi/launchpad/**', (value, address) => {
  console.log(address, value);
});

// Just CC messages
client.on('/midi/launchpad/cc/**', (value, address) => {
  const parts = address.split('/');
  const channel = parts[4];
  const cc = parts[5];
  console.log(`CC ${cc} on ch ${channel}: ${value}`);
});

// Just notes
client.on('/midi/launchpad/note', (data) => {
  console.log(`Note ${data.note} velocity ${data.velocity}`);
});
```

## Send MIDI

```javascript
// Send CC
await client.set('/midi/launchpad/cc/1/7', 100);  // CC 7 (volume) = 100

// Send Note On
await client.emit('/midi/launchpad/note', {
  note: 60,       // Middle C
  velocity: 100,
  channel: 1
});

// Send Note Off
await client.emit('/midi/launchpad/note', {
  note: 60,
  velocity: 0,
  channel: 1
});
```

## Multiple MIDI Devices

```bash
# First device
clasp midi --device "Launchpad" --prefix /midi/launchpad

# Second device
clasp midi --device "Keyboard" --prefix /midi/keyboard
```

## MIDI Clock

Receive MIDI clock:

```javascript
client.on('/midi/*/clock', () => {
  // 24 pulses per quarter note
});

client.on('/midi/*/transport', (state) => {
  // state: "start", "stop", or "continue"
});
```

## Virtual MIDI (macOS)

Create a virtual MIDI port:

```bash
clasp midi --virtual "CLASP Virtual"
```

Other apps can then send/receive MIDI through "CLASP Virtual".

## Troubleshooting

### Device not found

- Check device is connected: `clasp midi --list`
- Try unplugging and reconnecting
- On Linux, check permissions: `sudo usermod -a -G audio $USER`

### Notes stuck

Send all notes off:
```javascript
for (let ch = 1; ch <= 16; ch++) {
  await client.set(`/midi/device/cc/${ch}/123`, 0);  // All Notes Off
}
```

### Latency

MIDI should be sub-millisecond on USB. If laggy:
- Check USB hub (direct connection is better)
- Reduce CLASP message rate

## Example: MIDI Controller to Lights

```javascript
// Map MIDI CC to light brightness
client.on('/midi/controller/cc/1/1', async (value) => {
  // CC 1 controls light brightness (scale 0-127 to 0-1)
  const brightness = value / 127;
  await client.set('/dmx/0/1', Math.round(brightness * 255));
});
```

## Next Steps

- [Ableton Integration](../../integrations/ableton.md)
- [MIDI Bridge Reference](../../reference/bridges/midi.md)

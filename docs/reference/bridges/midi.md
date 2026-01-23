# MIDI Bridge

Mapping between MIDI and CLASP.

## Overview

The MIDI bridge translates MIDI messages to CLASP signals and vice versa, enabling integration with MIDI controllers, instruments, and software.

## Address Format

### General Pattern

```
/midi/{device}/{message_type}[/{channel}][/{param}]
```

### Note Messages

```
# Combined format
/midi/{device}/note
Value: { note: 0-127, velocity: 0-127, channel: 0-15 }

# Separate addresses
/midi/{device}/ch/{channel}/note/{note}
Value: velocity (0-127)
```

### Control Change (CC)

```
/midi/{device}/cc/{channel}/{cc_number}
Value: 0-127
```

### Pitch Bend

```
/midi/{device}/pb/{channel}
Value: -8192 to 8191
```

### Program Change

```
/midi/{device}/pc/{channel}
Value: 0-127
```

### Channel Pressure (Aftertouch)

```
/midi/{device}/pressure/{channel}
Value: 0-127
```

### Poly Pressure

```
/midi/{device}/poly/{channel}/{note}
Value: 0-127
```

## Value Mapping

### MIDI to CLASP

| MIDI Message | CLASP Address | CLASP Value |
|--------------|---------------|-------------|
| Note On | `/midi/dev/note` | `{note, velocity, channel}` |
| Note Off | `/midi/dev/note` | `{note, velocity: 0, channel}` |
| CC | `/midi/dev/cc/0/7` | `100` (int 0-127) |
| Pitch Bend | `/midi/dev/pb/0` | `0` (int -8192 to 8191) |
| Program | `/midi/dev/pc/0` | `5` (int 0-127) |
| Pressure | `/midi/dev/pressure/0` | `64` (int 0-127) |

### CLASP to MIDI

Set values on MIDI addresses to send MIDI:

```javascript
// Send Note On
await client.emit('/midi/device/note', { note: 60, velocity: 100, channel: 0 });

// Send CC
await client.set('/midi/device/cc/0/7', 100);

// Send Pitch Bend
await client.set('/midi/device/pb/0', 4096);
```

## Signal Types

### Note Messages

Notes typically use Event signal type (ephemeral):

```javascript
// Note On
client.emit('/midi/device/note', { note: 60, velocity: 100, channel: 0 });

// Note Off
client.emit('/midi/device/note', { note: 60, velocity: 0, channel: 0 });
```

### CC Messages

CCs use Param signal type (stateful):

```javascript
// Sets and retains CC value
client.set('/midi/device/cc/0/7', 100);
```

### High-Rate Control

Use Stream for continuous controller data:

```javascript
const stream = client.stream('/midi/device/cc/0/1');
// Send frequent updates without individual set() calls
```

## MIDI Clock

When enabled with `--include-clock`:

```
/midi/{device}/clock/tick     - Clock tick (24 per quarter note)
/midi/{device}/clock/start    - Start playback
/midi/{device}/clock/stop     - Stop playback
/midi/{device}/clock/continue - Continue playback
```

## SysEx Messages

When enabled with `--include-sysex`:

```
/midi/{device}/sysex
Value: [0xF0, ...data..., 0xF7] (Blob)
```

## Channel Filtering

Configure which MIDI channels to bridge:

```yaml
midi:
  channels: [1, 2, 10]  # Only channels 1, 2, and 10 (drums)
```

Or via CLI:

```bash
clasp midi --device "Controller" --channels 1,2,10
```

## Device Names

Device names in addresses are normalized:
- Spaces → underscores
- Lowercase
- Special characters removed

```
"Launchpad X" → launchpad_x
"MIDI Controller 1" → midi_controller_1
```

## Configuration

### CLI

```bash
clasp midi --device "Launchpad X"
```

### Configuration File

```yaml
midi:
  device: "Launchpad X"
  # Or separate:
  # input: "Controller In"
  # output: "Synth Out"

  channels: null  # All channels, or [1, 2, 3]
  include_clock: false
  include_sysex: false

clasp:
  router: "ws://localhost:7330"

mapping:
  prefix: "/midi"
  device_name: "launchpad"  # Override auto-detected name
```

### Rust API

```rust
use clasp_bridge::midi::{MidiBridge, MidiConfig};

let config = MidiConfig {
    device_name: "Launchpad X".into(),
    input_enabled: true,
    output_enabled: true,
    channels: None,  // All channels
    include_clock: false,
    include_sysex: false,
};

let bridge = MidiBridge::new(client, config).await?;
```

## Common CC Numbers

| CC | Name | Typical Use |
|----|------|-------------|
| 0 | Bank Select MSB | Bank switching |
| 1 | Modulation | Mod wheel |
| 7 | Volume | Channel volume |
| 10 | Pan | Stereo position |
| 11 | Expression | Dynamic control |
| 64 | Sustain | Pedal on/off |
| 74 | Brightness | Filter cutoff |
| 120 | All Sound Off | Emergency stop |
| 121 | Reset All Controllers | Reset |

## Virtual MIDI Ports

Create virtual MIDI ports for software:

```bash
# macOS/Linux
clasp midi --virtual "CLASP MIDI"
```

Other applications can then connect to "CLASP MIDI".

## See Also

- [Add MIDI](../../how-to/connections/add-midi.md)
- [clasp midi CLI](../cli/clasp-midi.md)
- [Ableton Integration](../../integrations/ableton.md)

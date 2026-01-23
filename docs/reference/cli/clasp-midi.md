# clasp midi

Start a MIDI bridge.

## Synopsis

```
clasp midi [OPTIONS]
```

## Description

Creates a bridge between MIDI and CLASP. Translates MIDI messages to CLASP signals and vice versa.

## Options

### Device Selection

```
--device <NAME>
    MIDI device name (partial match)

--input <NAME>
    MIDI input device (if different from output)

--output <NAME>
    MIDI output device (if different from input)

--list
    List available MIDI devices and exit

--virtual <NAME>
    Create virtual MIDI port (macOS/Linux)
```

### CLASP Connection

```
--router <URL>
    CLASP router URL [default: ws://localhost:7330]

--token <TOKEN>
    Authentication token

--name <NAME>
    Client name [default: midi-bridge]
```

### Address Mapping

```
--prefix <PREFIX>
    CLASP address prefix [default: /midi]

--device-name <NAME>
    Device name in addresses [default: auto-detected]
```

### Filtering

```
--channels <LIST>
    MIDI channels to bridge (1-16, comma-separated)

--include-clock
    Include MIDI clock messages

--include-sysex
    Include SysEx messages
```

### Other

```
-c, --config <PATH>
    Configuration file

-v, --verbose
    Verbose output

-h, --help
    Print help
```

## Examples

### List Devices

```bash
clasp midi --list
```

Output:
```
Input Devices:
  0: Launchpad X
  1: IAC Driver Bus 1

Output Devices:
  0: Launchpad X
  1: IAC Driver Bus 1
```

### Connect to Device

```bash
clasp midi --device "Launchpad"
```

Connects to first device matching "Launchpad".

### Separate Input/Output

```bash
clasp midi --input "Controller" --output "Synth"
```

### Virtual Port

```bash
clasp midi --virtual "CLASP MIDI"
```

Creates a virtual MIDI port other applications can connect to.

### Filter Channels

```bash
clasp midi --device "Controller" --channels 1,2,10
```

Only bridges channels 1, 2, and 10 (drum channel).

## Address Format

### Note Messages

```
/midi/{device}/note
Value: { note: 60, velocity: 100, channel: 0 }

# Or individual addresses:
/midi/{device}/ch/{channel}/note/{note}
Value: velocity (0-127)
```

### Control Change

```
/midi/{device}/cc/{channel}/{cc}
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

### Channel Pressure

```
/midi/{device}/pressure/{channel}
Value: 0-127
```

### Clock

```
/midi/{device}/clock/tick
/midi/{device}/clock/start
/midi/{device}/clock/stop
/midi/{device}/clock/continue
```

## Configuration File

```yaml
# midi-bridge.yaml
midi:
  device: "Launchpad X"
  # Or separate:
  # input: "Controller"
  # output: "Synth"

  channels: [1, 2, 3, 4]
  include_clock: false
  include_sysex: false

clasp:
  router: "ws://localhost:7330"
  name: "midi-bridge"

mapping:
  prefix: "/midi"
  device_name: "launchpad"
```

## MIDI to CLASP Examples

### Note On

```
MIDI: Note On, channel 1, note 60, velocity 100
→ CLASP: /midi/launchpad/note = { note: 60, velocity: 100, channel: 0 }
→ CLASP: /midi/launchpad/ch/0/note/60 = 100
```

### Control Change

```
MIDI: CC 7 (volume), channel 1, value 100
→ CLASP: /midi/launchpad/cc/0/7 = 100
```

### Pitch Bend

```
MIDI: Pitch Bend, channel 1, value 8192
→ CLASP: /midi/launchpad/pb/0 = 8192
```

## CLASP to MIDI Examples

### Send Note

```javascript
// Note on
await client.emit('/midi/launchpad/note', { note: 60, velocity: 100, channel: 0 });

// Note off (velocity 0)
await client.emit('/midi/launchpad/note', { note: 60, velocity: 0, channel: 0 });
```

### Send CC

```javascript
await client.set('/midi/launchpad/cc/0/7', 100);
```

## See Also

- [Add MIDI](../../how-to/connections/add-midi.md)
- [Ableton Integration](../../integrations/ableton.md)
- [MIDI Bridge Reference](../bridges/midi.md)

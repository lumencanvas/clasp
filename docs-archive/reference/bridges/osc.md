---
title: "OSC Bridge"
description: "Mapping between OSC (Open Sound Control) and CLASP."
section: reference
order: 6
---
# OSC Bridge

Mapping between OSC (Open Sound Control) and CLASP.

## Overview

The OSC bridge translates between OSC messages and CLASP signals, enabling communication with OSC-compatible software like TouchOSC, Resolume, QLab, and many others.

## Address Mapping

### OSC to CLASP

OSC addresses are prefixed with `/osc`:

| OSC Address | CLASP Address |
|-------------|---------------|
| `/1/fader1` | `/osc/1/fader1` |
| `/composition/master` | `/osc/composition/master` |
| `/live/tempo` | `/osc/live/tempo` |

### CLASP to OSC

CLASP addresses under `/osc/` are forwarded to OSC:

| CLASP Address | OSC Address |
|---------------|-------------|
| `/osc/1/fader1` | `/1/fader1` |
| `/osc/composition/master` | `/composition/master` |

## Value Mapping

### OSC to CLASP

| OSC Type | CLASP Type | Notes |
|----------|------------|-------|
| int32 (i) | Int | |
| int64 (h) | Int | |
| float32 (f) | Float | |
| float64 (d) | Float | |
| string (s) | String | |
| blob (b) | Blob | |
| True (T) | Bool(true) | |
| False (F) | Bool(false) | |
| Nil (N) | Null | |
| Impulse (I) | Null | Used for triggers |
| timetag (t) | Int | NTP timestamp |
| RGBA (r) | Array | [r, g, b, a] |
| MIDI (m) | Map | {port, status, data1, data2} |

### Multi-Argument Messages

OSC messages with multiple arguments become arrays:

```
OSC: /xy [0.5, 0.8]
→ CLASP: /osc/xy = [0.5, 0.8]

OSC: /color [255, 128, 64, 255]
→ CLASP: /osc/color = [255, 128, 64, 255]
```

### CLASP to OSC

| CLASP Type | OSC Type |
|------------|----------|
| Null | Nil |
| Bool(true) | True |
| Bool(false) | False |
| Int | int32 or int64 |
| Float | float32 |
| String | string |
| Blob | blob |
| Array | multiple arguments |
| Map | *not directly supported* |

## Signal Types

### Param (Default)

State values map to OSC normally:

```javascript
client.set('/osc/1/fader1', 0.5);
// → OSC: /1/fader1 [0.5]
```

### Event

Events send OSC messages but don't retain state:

```javascript
client.emit('/osc/cue/go', null);
// → OSC: /cue/go []  (Impulse/Nil)
```

### Stream

High-rate data sends OSC at the message rate:

```javascript
const stream = client.stream('/osc/audio/level');
stream.send(0.8);
// → OSC: /audio/level [0.8]
```

## Bundle Support

OSC bundles map to CLASP bundles:

```
OSC Bundle [timetag: 1234567890] {
  /1/fader1 [0.5]
  /1/fader2 [0.8]
}
→ CLASP Bundle [timestamp: 1234567890] {
  SET /osc/1/fader1 = 0.5
  SET /osc/1/fader2 = 0.8
}
```

CLASP bundles sent to `/osc/**` addresses become OSC bundles:

```javascript
client.bundle()
  .set('/osc/1/fader1', 0.5)
  .set('/osc/1/fader2', 0.8)
  .execute();
// → OSC Bundle { /1/fader1 [0.5], /1/fader2 [0.8] }
```

## Custom Address Mapping

Configure custom mappings:

```yaml
mapping:
  custom:
    "/touchosc/fader*": "/control/fader*"
    "/resolume/layer/*/opacity": "/video/layer/*/opacity"
```

```rust
let mapper = AddressMapper::new()
    .map("/touchosc/fader*", "/control/fader*")
    .map("/resolume/layer/*/opacity", "/video/layer/*/opacity");
```

## Configuration

### CLI

```bash
clasp osc --port 8000 --target 192.168.1.100:9000
```

### Configuration File

```yaml
osc:
  # Receive settings
  port: 8000
  bind: "0.0.0.0"

  # Send settings
  targets:
    - "192.168.1.100:9000"
    - "192.168.1.101:9000"

  # Address prefix
  prefix: "/osc"

  # Strip prefix when sending
  strip_prefix: true
```

### Rust API

```rust
use clasp_bridge::osc::{OscBridge, OscConfig};

let config = OscConfig {
    bind_addr: "0.0.0.0:8000".parse()?,
    targets: vec!["192.168.1.100:9000".parse()?],
    prefix: "/osc".into(),
    strip_prefix: true,
};

let bridge = OscBridge::new(client, config).await?;
```

## Common Software Ports

| Software | Default Send | Default Receive |
|----------|--------------|-----------------|
| TouchOSC | 8000 | 9000 |
| Resolume | 7000 | 7001 |
| QLab | 53000 | 53001 |
| Ableton (M4L) | 9000 | 9001 |
| MadMapper | 8010 | 8011 |
| TouchDesigner | 9000 | 10000 |

## See Also

- [Add OSC](../../how-to/connections/add-osc.md)
- [clasp osc CLI](../cli/clasp-osc.md)
- [TouchOSC Integration](../../integrations/touchosc.md)

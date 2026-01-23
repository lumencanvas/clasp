# clasp osc

Start an OSC bridge.

## Synopsis

```
clasp osc [OPTIONS]
```

## Description

Creates a bridge between OSC (Open Sound Control) and CLASP. Translates OSC messages to CLASP signals and vice versa.

## Options

### Network

```
--port <PORT>
    OSC receive port [default: 8000]

--bind <ADDRESS>
    Address to bind to [default: 0.0.0.0]

--target <HOST:PORT>
    OSC send target address
    Can be specified multiple times for multiple targets
```

### CLASP Connection

```
--router <URL>
    CLASP router URL [default: ws://localhost:7330]

--token <TOKEN>
    Authentication token

--name <NAME>
    Client name [default: osc-bridge]
```

### Address Mapping

```
--prefix <PREFIX>
    CLASP address prefix for OSC messages [default: /osc]

--strip-prefix
    Strip prefix when sending to OSC

--map <OSC=CLASP>
    Custom address mapping (can be repeated)
```

### Filtering

```
--include <PATTERN>
    Only bridge addresses matching pattern (can be repeated)

--exclude <PATTERN>
    Exclude addresses matching pattern (can be repeated)
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

### Basic Bridge

```bash
clasp osc --port 8000
```

Receives OSC on port 8000, forwards to CLASP as `/osc/...`.

### Bidirectional

```bash
clasp osc --port 8000 --target 192.168.1.100:9000
```

OSC on port 8000 → CLASP, CLASP `/osc/**` → 192.168.1.100:9000.

### Multiple Targets

```bash
clasp osc --port 8000 \
  --target 192.168.1.100:9000 \
  --target 192.168.1.101:9000
```

### Custom Mapping

```bash
clasp osc --port 8000 \
  --map "/1/fader*=/control/fader*" \
  --map "/1/xy*=/control/xy*"
```

### TouchOSC Setup

```bash
clasp osc --port 8000 --target 192.168.1.50:9000 --prefix /touchosc
```

### Resolume Setup

```bash
clasp osc --port 7001 --target 127.0.0.1:7000 --prefix /resolume
```

## Address Translation

### OSC to CLASP

```
OSC: /1/fader1 [0.5]
→ CLASP: /osc/1/fader1 = 0.5

OSC: /live/tempo [120.0]
→ CLASP: /osc/live/tempo = 120.0
```

### CLASP to OSC

```
CLASP: /osc/1/fader1 = 0.75
→ OSC: /1/fader1 [0.75]
```

### Value Conversion

| OSC Type | CLASP Type |
|----------|------------|
| int32 | Int |
| float32 | Float |
| string | String |
| blob | Blob |
| True/False | Bool |
| Nil | Null |
| array | Array |

## Configuration File

```yaml
# osc-bridge.yaml
osc:
  port: 8000
  bind: "0.0.0.0"
  targets:
    - "192.168.1.100:9000"
    - "192.168.1.101:9000"

clasp:
  router: "ws://localhost:7330"
  name: "osc-bridge"

mapping:
  prefix: "/osc"
  strip_prefix: true
  custom:
    "/1/fader*": "/control/fader*"
    "/1/xy*": "/control/xy*"

filter:
  include:
    - "/1/**"
    - "/2/**"
  exclude:
    - "/**/debug"
```

## See Also

- [Add OSC](../../how-to/connections/add-osc.md)
- [TouchOSC Integration](../../integrations/touchosc.md)
- [Resolume Integration](../../integrations/resolume.md)
- [OSC Bridge Reference](../bridges/osc.md)

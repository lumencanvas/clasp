---
title: "Addressing"
description: "CLASP uses a hierarchical address space with path-style addresses and wildcard pattern matching."
section: reference
order: 3
---
# Addressing

CLASP uses a hierarchical address space with path-style addresses and wildcard pattern matching.

## Address Format

Addresses are UTF-8 strings with path-style segments separated by `/`:

```
/namespace/category/item/property
```

### Rules

- Must start with `/`
- Segments separated by `/`
- No trailing `/` (except root `/`)
- Case-sensitive
- UTF-8 encoded
- Maximum length: 256 bytes

### Examples

```
/app/scene/0/layer/3/opacity
/lights/front/brightness
/sensors/room1/temperature
/midi/launchpad/note
/dmx/1/47
```

### Conventions

While not enforced, these conventions are recommended:

| Pattern | Description | Example |
|---------|-------------|---------|
| `/app/...` | Application-specific | `/app/scene/0/opacity` |
| `/midi/...` | MIDI bridge | `/midi/device/note` |
| `/osc/...` | OSC bridge | `/osc/synth/cutoff` |
| `/dmx/...` | DMX bridge | `/dmx/1/47` |
| `/artnet/...` | Art-Net bridge | `/artnet/0/1/2/47` |
| `/mqtt/...` | MQTT bridge | `/mqtt/sensors/temp` |

## Wildcard Patterns

CLASP supports two wildcard types for subscriptions and queries:

### Single-Segment Wildcard (`*`)

Matches exactly one path segment.

```
Pattern: /lights/*/brightness
Matches:
  /lights/front/brightness    ✓
  /lights/back/brightness     ✓
  /lights/brightness          ✗ (no segment to match)
  /lights/front/back/brightness ✗ (too many segments)
```

### Multi-Segment Wildcard (`**`)

Matches zero or more path segments.

```
Pattern: /lights/**
Matches:
  /lights                     ✓
  /lights/front               ✓
  /lights/front/brightness    ✓
  /lights/front/rgb/red       ✓
  /other/lights               ✗ (wrong prefix)
```

### Combined Patterns

Wildcards can be combined:

```
Pattern: /app/*/layer/**/opacity
Matches:
  /app/scene/layer/opacity           ✓
  /app/scene/layer/0/opacity         ✓
  /app/scene/layer/0/mask/opacity    ✓
  /app/scene/0/layer/3/opacity       ✗ (0 should match *, not segment)
```

### Pattern Placement

| Position | Example | Valid |
|----------|---------|-------|
| Start | `/**/foo` | Yes |
| Middle | `/foo/**/bar` | Yes |
| End | `/foo/**` | Yes |
| Consecutive | `/foo/*/*/bar` | Yes |
| Adjacent `**` | `/foo/**/**/bar` | No (redundant) |

## Subscription Examples

### Exact Match

```javascript
client.on('/lights/front/brightness', callback);
// Matches only: /lights/front/brightness
```

### Single Wildcard

```javascript
client.on('/lights/*/brightness', callback);
// Matches: /lights/front/brightness, /lights/back/brightness
// Not: /lights/front/rgb/brightness
```

### Multi-Segment Wildcard

```javascript
client.on('/lights/**', callback);
// Matches all addresses starting with /lights/
```

### Combined

```javascript
client.on('/app/scene/*/layer/**/opacity', callback);
// Matches: /app/scene/0/layer/1/opacity
// Matches: /app/scene/main/layer/group/0/opacity
```

## Address Resolution

When a message is published, the router matches it against all subscriptions:

```
Message: SET /lights/front/brightness = 0.8

Subscriptions:
  /lights/front/brightness    → MATCH (exact)
  /lights/*/brightness        → MATCH (wildcard)
  /lights/**                  → MATCH (multi-wildcard)
  /lights/back/brightness     → NO MATCH
  /other/**                   → NO MATCH
```

## Namespaces

Clients can register namespaces via ANNOUNCE:

```javascript
{
  type: "ANNOUNCE",
  namespace: "/app",
  signals: [
    { address: "/app/scene/*/opacity", type: "param" }
  ]
}
```

Namespaces provide:
- Signal discovery
- Access control boundaries
- Documentation of available addresses

## Reserved Addresses

These address prefixes are reserved for protocol use:

| Prefix | Purpose |
|--------|---------|
| `/_clasp/` | Protocol internals |
| `/_meta/` | Metadata queries |
| `/_admin/` | Administrative functions |

## Performance Considerations

- Exact match subscriptions are fastest
- `*` wildcards have moderate overhead
- `**` wildcards are slowest (must check all segments)
- Keep address depth reasonable (< 10 segments)
- Avoid subscribing to `/**` in production (matches everything)

## Cross-Protocol Addressing

When bridging to other protocols, addresses are mapped:

### OSC → CLASP
```
OSC: /synth/osc1/cutoff
CLASP: /osc/synth/osc1/cutoff
```

### MIDI → CLASP
```
MIDI: Note On, Channel 1, Note 60
CLASP: /midi/{device}/note (with payload)
```

### MQTT → CLASP
```
MQTT: sensors/room1/temperature
CLASP: /mqtt/sensors/room1/temperature
```

See the [Bridge Reference](../bridges/) for complete mapping details.

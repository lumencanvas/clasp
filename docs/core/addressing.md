---
title: Addressing
description: Path hierarchy, wildcards, and namespace conventions
order: 3
---

# Addressing

Every signal in CLASP has an address -- a hierarchical path that identifies what it refers to. Addresses use forward slashes, like file system paths. They are the primary mechanism for routing signals from publishers to subscribers.

## Path Format

Addresses follow the pattern `/namespace/category/instance/property`. There is no fixed depth -- use as many segments as needed to organize your signal hierarchy.

```
/lights/room1/brightness
/sensors/temperature
/chat/room/lobby/messages
/lumen/scene/0/layer/3/opacity
/midi/launchpad/cc/74
```

Rules:

- Addresses must start with `/`.
- Segments are separated by `/`.
- Empty segments between slashes are not allowed (e.g., `/lights//brightness` is invalid).
- Segments can contain any characters except `/`.

The first segment is the **namespace**. The last segment is the **property**. These are accessible programmatically via `address.namespace()` and `address.property()` in the Rust API.

## Wildcards

Wildcards are used in subscriptions and auth scopes to match multiple addresses. There are two wildcard types.

### Single-segment wildcard: `*`

`*` matches exactly one segment. It cannot span multiple segments.

| Pattern | Matches | Does Not Match |
|---------|---------|----------------|
| `/lights/*/brightness` | `/lights/room1/brightness`, `/lights/room2/brightness` | `/lights/room1/zone/brightness` |
| `/sensors/*/temperature` | `/sensors/outdoor/temperature` | `/sensors/floor/2/temperature` |
| `/*/room1/brightness` | `/lights/room1/brightness`, `/audio/room1/brightness` | `/room1/brightness` |

### Multi-segment wildcard: `**`

`**` matches zero or more segments. It is typically used at the end of a pattern.

| Pattern | Matches | Does Not Match |
|---------|---------|----------------|
| `/lights/**` | `/lights`, `/lights/room1`, `/lights/room1/brightness`, `/lights/room1/zone/a/brightness` | `/audio/room1` |
| `/lumen/**/opacity` | `/lumen/opacity`, `/lumen/scene/0/opacity`, `/lumen/scene/0/layer/3/opacity` | `/lumen/scene/0/color` |

### Combining wildcards

You can combine `*` and `**` in the same pattern:

```
/lights/*/zone/**    matches /lights/room1/zone/a/brightness
                     matches /lights/room2/zone
                     does not match /lights/room1/brightness
```

## Subscribing with Patterns

Pass a pattern to `on()` or `subscribe()` to receive signals matching that pattern:

**JavaScript:**

```javascript
// Exact address
client.on('/lights/room1/brightness', (value) => { /* ... */ });

// All params under a namespace
client.on('/lights/**', (value, meta) => {
  console.log(`${meta.address}: ${value}`);
});

// One level of rooms, brightness only
client.on('/lights/*/brightness', (value, meta) => {
  console.log(`${meta.address}: ${value}`);
});
```

**Python:**

```python
# All params under a namespace
@client.on('/lights/**')
async def on_lights(value, meta):
    print(f"{meta.address}: {value}")
```

When subscribing to a pattern that includes params, the router sends a SNAPSHOT of all currently matching param values before delivering live updates.

## Namespace Conventions

There are no enforced namespacing rules, but the following conventions help avoid collisions:

**Protocol bridges** use reserved prefixes that match the bridge name:

| Bridge | Prefix | Example |
|--------|--------|---------|
| MQTT | `/mqtt/` | `/mqtt/sensors/temperature` |
| OSC | `/osc/` | `/osc/1/fader1` |
| MIDI | `/midi/` | `/midi/launchpad/note/60` |
| Art-Net | `/artnet/` | `/artnet/universe/0/channel/1` |

**Application data** uses any namespace that does not collide with bridge prefixes. Common patterns:

```
/app/entity/property          -- generic application
/chat/room/lobby/messages     -- chat application
/lumen/scene/0/layer/3/opacity -- media server
/game/player/1/position       -- game state
```

## Addresses in Auth

Auth scopes use the same pattern syntax as subscriptions. A scope like `write:/lights/**` grants write access to all addresses starting with `/lights/`. Scopes are specified as `action:pattern` pairs:

| Scope | Grants |
|-------|--------|
| `read:/lights/**` | Read and subscribe to all light addresses |
| `write:/lights/room1/*` | Write to any property of room1 |
| `read:/sensors/**` | Read-only access to all sensors |
| `write:/**` | Write access to everything |

See the auth documentation for full details on capability tokens and scope enforcement.

## Next Steps

- [Signal Types](./signals.md) -- the 5 signal types and how they interact with addresses.
- [Bundles](./bundles.md) -- sending multiple signals to different addresses atomically.
- [State Management](./state.md) -- how param state is tracked per-address.

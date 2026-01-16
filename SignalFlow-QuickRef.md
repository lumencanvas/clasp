# SignalFlow Quick Reference

## Connection
```javascript
const sf = new SignalFlow('wss://localhost:7330');
await sf.connect();
```

## Reading
```javascript
// Get current value
const value = await sf.get('/lumen/layer/0/opacity');

// Subscribe to changes
sf.on('/lumen/layer/*/opacity', (value, address) => {
  console.log(`${address} = ${value}`);
});

// With options
sf.on('/controller/*', callback, { maxRate: 30, epsilon: 0.01 });
```

## Writing
```javascript
// Set param (stateful)
sf.set('/lumen/layer/0/opacity', 0.75);

// Emit event (ephemeral)
sf.emit('/cue/fire', { id: 'intro' });

// Stream (high-rate)
sf.stream('/fader/1', 0.5);
```

## Bundles (Atomic)
```javascript
sf.bundle([
  { set: ['/light/1', 1.0] },
  { set: ['/light/2', 0.0] }
]);

// Scheduled
sf.bundle([...], { at: sf.time() + 100000 });  // 100ms later
```

## Signal Types

| Type | Use | QoS | State? |
|------|-----|-----|--------|
| Param | Values with history | Confirm | ✓ |
| Event | Triggers | Confirm | ✗ |
| Stream | High-rate data | Fire | ✗ |
| Gesture | Touch/motion | Fire | Phase |
| Timeline | Automation | Commit | ✓ |

## Discovery
```javascript
// mDNS service: _signalflow._tcp.local
// UDP broadcast: port 7331

// In browser (no native discovery):
const sf = new SignalFlow('wss://192.168.1.42:7330');
```

## Frame Format (4 bytes minimum)
```
[0]    Magic 'S' (0x53)
[1]    Flags (QoS, timestamp, encrypted, compressed)
[2-3]  Payload length (uint16 BE)
[4+]   MessagePack payload
```

## Bridge Mappings

### MIDI → SignalFlow
```
Note On/Off  → /midi/{dev}/note     Event
CC           → /midi/{dev}/cc/{n}   Param u8
Pitch Bend   → /midi/{dev}/bend     Param i16
```

### OSC → SignalFlow
```
/synth/cutoff ,f 0.5  →  SET /osc/synth/cutoff 0.5
```

### DMX → SignalFlow
```
Universe 1, Ch 47  →  /dmx/1/47  Param u8
```

## Addresses
```
/namespace/category/instance/property
/lumen/scene/0/layer/3/opacity
/midi/launchpad/cc/74

Wildcards (subscribe only):
*   = one segment
**  = any segments

/lumen/scene/*/layer/**/opacity
```

## Common Ports
- WebSocket: 7330
- UDP Discovery: 7331
- mDNS: 5353 (standard)

## Error Codes
- 1xx: Protocol errors
- 2xx: Address errors  
- 3xx: Permission errors
- 4xx: State errors
- 5xx: Server errors

## Security
```javascript
// Capability token (JWT)
{
  "sf": {
    "read": ["/lumen/**"],
    "write": ["/lumen/layer/*/opacity"],
    "constraints": {
      "/lumen/layer/*/opacity": { "range": [0, 1], "maxRate": 60 }
    }
  }
}
```

## Timing
```javascript
// Sync happens automatically on connect
// All timestamps: microseconds since session start

// Schedule for future
sf.bundle([...], { at: sf.time() + 500000 });  // 500ms
```

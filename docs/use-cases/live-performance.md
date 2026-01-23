# Live Performance

Connect lighting, audio, video, and control systems for live shows.

## Overview

Live performance typically involves:

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  Controller │  │   Audio     │  │   Video     │  │  Lighting   │
│  (MIDI)     │  │   (OSC)     │  │   (OSC)     │  │  (Art-Net)  │
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                │                │                │
       └────────────────┴────────────────┴────────────────┘
                                 │
                          ┌──────▼──────┐
                          │    CLASP    │
                          │   Router    │
                          └─────────────┘
```

## Setup

### 1. Start Router and Bridges

```bash
# Terminal 1: Router
clasp server --port 7330

# Terminal 2: MIDI (controller)
clasp midi --device "Launchpad X"

# Terminal 3: OSC (audio software)
clasp osc --port 9000

# Terminal 4: Art-Net (lighting)
clasp artnet --bind 0.0.0.0:6454
```

### 2. Map Controller to Lights

```javascript
// Map MIDI CC to DMX
client.on('/midi/launchpad/cc/1/*', async (value, address) => {
  const cc = parseInt(address.split('/').pop());
  // CC 1-8 controls lights 1-8
  await client.set(`/artnet/0/0/0/${cc}`, Math.round(value * 2));
});
```

### 3. Trigger Cues

```javascript
// MIDI notes trigger cues
client.on('/midi/launchpad/note', async (data) => {
  if (data.velocity > 0) {
    await client.emit('/cue/fire', { id: data.note });
  }
});

// Handle cues
client.on('/cue/fire', async (data) => {
  const cue = cues[data.id];
  if (cue) {
    await client.bundle(cue.actions);
  }
});
```

## Common Patterns

### Beat-Synced Effects

```javascript
// Receive tempo from audio software
let bpm = 120;
client.on('/osc/live/tempo', (value) => {
  bpm = value;
});

// Flash on beat
function flashOnBeat() {
  const beatMs = 60000 / bpm;
  setInterval(async () => {
    await client.set('/artnet/0/0/0/1', 255);
    setTimeout(() => client.set('/artnet/0/0/0/1', 0), 50);
  }, beatMs);
}
```

### Master Intensity

```javascript
// Master fader controls all lights
let master = 1.0;
client.on('/midi/controller/cc/1/7', (value) => {
  master = value / 127;
});

// Apply master to all light updates
function setLight(channel, intensity) {
  const final = Math.round(intensity * master * 255);
  return client.set(`/artnet/0/0/0/${channel}`, final);
}
```

### Scene Presets

```javascript
const scenes = {
  blackout: [0, 0, 0, 0, 0, 0, 0, 0],
  full: [255, 255, 255, 255, 255, 255, 255, 255],
  warm: [255, 180, 100, 0, 255, 180, 100, 0],
};

async function setScene(name) {
  const values = scenes[name];
  const ops = values.map((v, i) => ({
    set: [`/artnet/0/0/0/${i + 1}`, v]
  }));
  await client.bundle(ops);
}
```

## Integration Examples

### Ableton Live → Lighting

Ableton sends OSC on transport and tempo changes:

```javascript
client.on('/osc/live/play', async (playing) => {
  if (playing) {
    await setScene('show');
  } else {
    await setScene('standby');
  }
});
```

### TouchOSC → Everything

TouchOSC as universal remote:

```javascript
// XY pad controls two lights
client.on('/osc/touchosc/1/xy1', async (data) => {
  await client.bundle([
    { set: ['/artnet/0/0/0/1', Math.round(data.x * 255)] },
    { set: ['/artnet/0/0/0/2', Math.round(data.y * 255)] }
  ]);
});
```

### Resolume → Lighting

Resolume sends layer info via OSC:

```javascript
client.on('/osc/resolume/composition/layers/*/video/opacity', (value, addr) => {
  const layer = addr.split('/')[5];
  // Mirror video layer opacity to light fixture
  client.set(`/artnet/0/0/0/${layer}`, Math.round(value * 255));
});
```

## Performance Tips

1. **Use streams for continuous data** (faders, sensors)
2. **Use bundles for coordinated changes** (scenes, cues)
3. **Rate-limit subscriptions** (`{ maxRate: 30 }`)
4. **Pre-compute cue actions** (don't calculate during show)

## Troubleshooting

### Latency

- Use wired Ethernet for Art-Net
- Keep router on same machine as time-critical bridges
- Profile with `RUST_LOG=debug`

### Dropped Messages

- Check network congestion
- Reduce message rate
- Use QoS Confirm for critical cues

## Next Steps

- [TouchOSC Integration](../integrations/touchosc.md)
- [Resolume Integration](../integrations/resolume.md)
- [Ableton Integration](../integrations/ableton.md)

# Resolume Integration

This guide explains how to connect Resolume Arena/Avenue to CLASP for VJ control.

## Overview

Resolume is a powerful VJ software that supports OSC control. CLASP enables you to:

- Control Resolume parameters from any CLASP client
- Receive Resolume state changes in your applications
- Bridge Resolume with MIDI controllers, TouchOSC, and other systems

## Setup

### 1. Enable OSC in Resolume

1. Open Resolume Arena or Avenue
2. Go to **Preferences** > **OSC**
3. Configure:
   - **OSC Input Enabled**: On
   - **Input Port**: 7000
   - **OSC Output Enabled**: On
   - **Output Port**: 7001
   - **Output Host**: Your computer's IP (or `127.0.0.1` for local)

### 2. Configure CLASP Bridge

Create an OSC bridge to communicate with Resolume:

```bash
# Receive from Resolume
clasp bridge create \
  --source osc:0.0.0.0:7001 \
  --target internal

# Send to Resolume
clasp bridge create \
  --source internal \
  --target osc:127.0.0.1:7000
```

Or for bidirectional in one bridge:

```bash
clasp bridge create \
  --source osc:0.0.0.0:7001 \
  --target osc:127.0.0.1:7000
```

## Resolume OSC Address Reference

### Composition Level

| Address | Type | Description |
|---------|------|-------------|
| `/composition/master` | float | Master opacity (0-1) |
| `/composition/speed` | float | Master speed |
| `/composition/bpm` | float | BPM |
| `/composition/tempocontroller/tempo` | float | Tempo |

### Layer Control

| Address | Type | Description |
|---------|------|-------------|
| `/composition/layers/N/video/opacity` | float | Layer N opacity |
| `/composition/layers/N/video/effects/E/opacity` | float | Effect E opacity |
| `/composition/layers/N/clear` | trigger | Clear layer N |
| `/composition/layers/N/bypassed` | int | Bypass layer (0/1) |
| `/composition/layers/N/solo` | int | Solo layer (0/1) |

### Clip Control

| Address | Type | Description |
|---------|------|-------------|
| `/composition/layers/N/clips/M/connect` | trigger | Trigger clip M on layer N |
| `/composition/layers/N/clips/M/video/position/values` | float,float | Clip position X,Y |
| `/composition/layers/N/clips/M/video/scale/values` | float,float | Clip scale X,Y |
| `/composition/layers/N/clips/M/video/opacity/values` | float | Clip opacity |

### Column/Deck Control

| Address | Type | Description |
|---------|------|-------------|
| `/composition/columns/N/connect` | trigger | Trigger column N |
| `/composition/decks/N/select` | trigger | Select deck N |

## Example: Layer Controller

Control Resolume layers from CLASP:

```javascript
const { Clasp } = require('@clasp-to/core');

async function main() {
  const clasp = await Clasp.builder('ws://localhost:7330')
    .withName('resolume-control')
    .connect();

  // Set layer 1 opacity
  clasp.set('/osc/composition/layers/1/video/opacity', 0.8);

  // Trigger clip 3 on layer 2
  clasp.emit('/osc/composition/layers/2/clips/3/connect', 1);

  // Trigger column 5 (all layers)
  clasp.emit('/osc/composition/columns/5/connect', 1);

  // Set master BPM
  clasp.set('/osc/composition/bpm', 120);
}

main();
```

## Example: MIDI to Resolume

Bridge a MIDI controller to Resolume:

```javascript
const { Clasp } = require('@clasp-to/core');

async function main() {
  const clasp = await Clasp.builder('ws://localhost:7330')
    .withName('midi-resolume-bridge')
    .connect();

  // Map MIDI faders to layer opacities
  for (let i = 0; i < 8; i++) {
    clasp.on(`/midi/controller/cc/${i}`, (value) => {
      // MIDI CC is 0-127, Resolume wants 0-1
      const opacity = value / 127;
      clasp.set(`/osc/composition/layers/${i + 1}/video/opacity`, opacity);
    });
  }

  // Map MIDI pads to clip triggers
  clasp.on('/midi/controller/note', (value, _, meta) => {
    if (meta.velocity > 0) {
      // Note on - map note number to clip
      const layer = Math.floor(meta.note / 8) + 1;
      const clip = (meta.note % 8) + 1;
      clasp.emit(`/osc/composition/layers/${layer}/clips/${clip}/connect`, 1);
    }
  });

  // Map transport buttons
  clasp.on('/midi/controller/cc/116', (value) => {
    if (value > 64) {
      clasp.set('/osc/composition/tempocontroller/resync', 1);
    }
  });
}

main();
```

## Example: TouchOSC to Resolume

Create a TouchOSC interface for Resolume:

```javascript
const { Clasp } = require('@clasp-to/core');

async function main() {
  const clasp = await Clasp.builder('ws://localhost:7330')
    .withName('touchosc-resolume')
    .connect();

  // Layer faders from TouchOSC to Resolume
  for (let i = 1; i <= 4; i++) {
    clasp.on(`/osc/resolume/layer${i}`, (value) => {
      clasp.set(`/osc/composition/layers/${i}/video/opacity`, value);
    });
  }

  // Clip trigger grid (4x4)
  for (let layer = 1; layer <= 4; layer++) {
    for (let clip = 1; clip <= 4; clip++) {
      clasp.on(`/osc/resolume/grid/${layer}/${clip}`, (value) => {
        if (value === 1) {
          clasp.emit(`/osc/composition/layers/${layer}/clips/${clip}/connect`, 1);
        }
      });
    }
  }

  // Master controls
  clasp.on('/osc/resolume/master', (value) => {
    clasp.set('/osc/composition/master', value);
  });

  clasp.on('/osc/resolume/bpm', (value) => {
    // Map 0-1 to 60-180 BPM
    const bpm = 60 + (value * 120);
    clasp.set('/osc/composition/bpm', bpm);
  });

  // Feedback: Listen to Resolume output and send to TouchOSC
  clasp.on('/osc/composition/layers/*/video/opacity', (value, address) => {
    const layer = address.match(/layers\/(\d+)/)[1];
    clasp.set(`/osc/resolume/layer${layer}`, value);
  });
}

main();
```

## Example: Automated Show

Create timed visual sequences:

```javascript
const { Clasp } = require('@clasp-to/core');

async function main() {
  const clasp = await Clasp.builder('ws://localhost:7330')
    .withName('resolume-automation')
    .connect();

  // Crossfade between layers
  async function crossfade(fromLayer, toLayer, duration = 2000) {
    const steps = 60;
    const interval = duration / steps;

    for (let i = 0; i <= steps; i++) {
      const progress = i / steps;
      clasp.set(`/osc/composition/layers/${fromLayer}/video/opacity`, 1 - progress);
      clasp.set(`/osc/composition/layers/${toLayer}/video/opacity`, progress);
      await sleep(interval);
    }
  }

  // Sequence of clips
  async function runSequence() {
    // Trigger clip 1 on layer 1
    clasp.emit('/osc/composition/layers/1/clips/1/connect', 1);
    await sleep(5000);

    // Crossfade to layer 2
    clasp.emit('/osc/composition/layers/2/clips/1/connect', 1);
    await crossfade(1, 2, 2000);

    // Trigger effect burst
    clasp.set('/osc/composition/layers/2/video/effects/1/opacity', 1);
    await sleep(500);
    clasp.set('/osc/composition/layers/2/video/effects/1/opacity', 0);
  }

  // Run on cue trigger
  clasp.on('/show/start', async () => {
    await runSequence();
  });
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

main();
```

## Tips

### Use Effect Mapping

Resolume's OSC output can be mapped in the OSC Map. This allows you to:

1. Right-click any parameter in Resolume
2. Select "Edit OSC Map"
3. Set custom output addresses

### Optimize Performance

For high-frequency updates (like audio-reactive visuals):

```javascript
// Use stream instead of set for high-rate data
clasp.stream('/osc/composition/layers/1/video/opacity', audioLevel);

// Rate limit feedback from Resolume
clasp.on('/osc/composition/**', callback, { maxRate: 30 });
```

### Group Controls with Bundles

For synchronized changes:

```javascript
clasp.bundle([
  { set: ['/osc/composition/layers/1/video/opacity', 0] },
  { set: ['/osc/composition/layers/2/video/opacity', 1] },
  { emit: ['/osc/composition/layers/2/clips/1/connect', 1] }
]);
```

## Troubleshooting

### No Response from Resolume

1. Verify OSC is enabled in Resolume Preferences
2. Check port numbers match CLASP bridge config
3. Test with OSC monitor app first

### Clips Not Triggering

1. Ensure clip address is correct (layers and clips are 1-indexed)
2. Send value of 1 (not just any value)
3. Check clip is loaded and not empty

### Feedback Loop

If you're sending and receiving on the same addresses:
1. Use separate namespaces for input/output
2. Add a debounce to prevent loops
3. Check "only send changed values" in Resolume OSC settings

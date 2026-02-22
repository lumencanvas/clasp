---
title: "MadMapper Integration"
description: "Connect MadMapper to CLASP for projection mapping control."
section: integrations
order: 3
---
# MadMapper Integration

Connect MadMapper to CLASP for projection mapping control.

## Overview

MadMapper supports OSC for remote control. CLASP enables you to:

- Control MadMapper from MIDI controllers, web interfaces, or other software
- Sync MadMapper with lighting (DMX/Art-Net) systems
- Trigger scenes and cues from external events

## Setup

### 1. Enable OSC in MadMapper

1. Open MadMapper
2. Go to **Preferences** > **OSC**
3. Configure:
   - **Enable OSC Input**: On
   - **Input Port**: 8010
   - **Enable OSC Output**: On
   - **Output Port**: 8011
   - **Output Address**: 127.0.0.1 (or CLASP machine IP)

### 2. Configure CLASP Bridge

```bash
# Bidirectional OSC bridge
clasp osc --port 8011 --target 127.0.0.1:8010
```

Or separate bridges:

```bash
# Receive from MadMapper
clasp bridge create --source osc:0.0.0.0:8011 --target internal

# Send to MadMapper
clasp bridge create --source internal --target osc:127.0.0.1:8010
```

## MadMapper OSC Reference

### Media Control

| Address | Arguments | Description |
|---------|-----------|-------------|
| `/medias/{name}/opacity` | float | Media opacity (0-1) |
| `/medias/{name}/play` | - | Play media |
| `/medias/{name}/pause` | - | Pause media |
| `/medias/{name}/stop` | - | Stop media |
| `/medias/{name}/restart` | - | Restart from beginning |
| `/medias/{name}/speed` | float | Playback speed |

### Surface Control

| Address | Arguments | Description |
|---------|-----------|-------------|
| `/surfaces/{name}/opacity` | float | Surface opacity (0-1) |
| `/surfaces/{name}/visible` | int | Show/hide (0/1) |
| `/surfaces/{name}/solo` | int | Solo surface (0/1) |

### Cue Control

| Address | Arguments | Description |
|---------|-----------|-------------|
| `/cues/go` | - | Trigger next cue |
| `/cues/goBack` | - | Go to previous cue |
| `/cues/{index}/go` | - | Trigger specific cue |
| `/cues/stop` | - | Stop current cue |

### Scene Control

| Address | Arguments | Description |
|---------|-----------|-------------|
| `/scenes/{name}/select` | - | Select scene |
| `/scenes/next` | - | Next scene |
| `/scenes/previous` | - | Previous scene |

### Master Controls

| Address | Arguments | Description |
|---------|-----------|-------------|
| `/master/opacity` | float | Master opacity (0-1) |
| `/master/intensity` | float | Master intensity |
| `/master/blackout` | int | Blackout on/off |

## Example: Basic Control

Control MadMapper from CLASP:

```javascript
const { ClaspBuilder } = require('@clasp-to/core');

async function main() {
  const client = await new ClaspBuilder('ws://localhost:7330')
    .withName('madmapper-control')
    .connect();

  // Set media opacity
  await client.set('/osc/medias/video1/opacity', 0.8);

  // Trigger cue
  await client.emit('/osc/cues/go', null);

  // Switch scene
  await client.emit('/osc/scenes/intro/select', null);

  // Master blackout
  await client.set('/osc/master/blackout', 1);
}

main();
```

## Example: MIDI Controller Integration

Map MIDI controller to MadMapper:

```javascript
async function main() {
  const client = await new ClaspBuilder('ws://localhost:7330')
    .withName('midi-madmapper')
    .connect();

  // Faders control media opacities
  const mediaMap = ['video1', 'video2', 'video3', 'video4'];

  for (let i = 0; i < mediaMap.length; i++) {
    client.on(`/midi/controller/cc/0/${i}`, async (value) => {
      const opacity = value / 127;
      await client.set(`/osc/medias/${mediaMap[i]}/opacity`, opacity);
    });
  }

  // Master fader
  client.on('/midi/controller/cc/0/7', async (value) => {
    await client.set('/osc/master/opacity', value / 127);
  });

  // Buttons trigger cues
  client.on('/midi/controller/note', async (data) => {
    if (data.velocity > 0 && data.note >= 36 && data.note <= 43) {
      const cueIndex = data.note - 36;
      await client.emit(`/osc/cues/${cueIndex}/go`, null);
    }
  });

  // Blackout button
  client.on('/midi/controller/cc/0/119', async (value) => {
    await client.set('/osc/master/blackout', value > 64 ? 1 : 0);
  });

  console.log('MIDI-MadMapper bridge running...');
}

main();
```

## Example: TouchOSC Interface

Control MadMapper from TouchOSC:

```javascript
async function main() {
  const client = await new ClaspBuilder('ws://localhost:7330')
    .withName('touchosc-madmapper')
    .connect();

  // Layer faders
  for (let i = 1; i <= 4; i++) {
    client.on(`/osc/touchosc/layer${i}`, async (value) => {
      await client.set(`/osc/medias/layer${i}/opacity`, value);
    });
  }

  // Scene buttons
  const scenes = ['intro', 'main', 'climax', 'outro'];
  scenes.forEach((scene, index) => {
    client.on(`/osc/touchosc/scene/${index + 1}`, async (value) => {
      if (value === 1) {
        await client.emit(`/osc/scenes/${scene}/select`, null);
      }
    });
  });

  // XY pad for position control
  client.on('/osc/touchosc/position', async (data) => {
    const [x, y] = data;
    await client.bundle([
      { set: ['/osc/medias/main/position/x', x] },
      { set: ['/osc/medias/main/position/y', y] }
    ]);
  });

  // Crossfader between two layers
  client.on('/osc/touchosc/crossfade', async (value) => {
    await client.bundle([
      { set: ['/osc/medias/layerA/opacity', 1 - value] },
      { set: ['/osc/medias/layerB/opacity', value] }
    ]);
  });
}

main();
```

## Example: Sync with Lighting

Coordinate MadMapper with DMX lighting:

```javascript
async function main() {
  const client = await new ClaspBuilder('ws://localhost:7330')
    .withName('madmapper-lighting-sync')
    .connect();

  // Master control affects both
  client.on('/control/master', async (value) => {
    await client.bundle([
      // MadMapper master
      { set: ['/osc/master/opacity', value] },
      // DMX dimmer
      { set: ['/artnet/0/0/0/1', Math.round(value * 255)] }
    ]);
  });

  // Scene changes trigger lighting cues
  const lightingCues = {
    intro: [128, 0, 0, 0],      // Dim red
    main: [255, 200, 150, 100], // Bright warm
    climax: [255, 255, 255, 255], // Full
    outro: [50, 50, 100, 0]    // Dim blue
  };

  client.on('/osc/madmapper/scene/*', async (value, address) => {
    const scene = address.split('/').pop();
    const cue = lightingCues[scene];

    if (cue) {
      const ops = cue.map((v, i) => ({
        set: [`/artnet/0/0/0/${i + 1}`, v]
      }));
      await client.bundle(ops);
    }
  });

  // Audio-reactive: MadMapper sends levels, we apply to both
  client.on('/osc/madmapper/audio/level', async (level) => {
    // Strobe effect on beat
    if (level > 0.9) {
      await client.set('/artnet/0/0/0/5', 255);
      setTimeout(() => client.set('/artnet/0/0/0/5', 0), 50);
    }
  });
}

main();
```

## Example: Show Sequencer

Automated show control:

```javascript
async function main() {
  const client = await new ClaspBuilder('ws://localhost:7330')
    .withName('madmapper-sequencer')
    .connect();

  const showSequence = [
    { time: 0, scene: 'intro', lighting: 'dim' },
    { time: 30000, scene: 'buildup', lighting: 'warm' },
    { time: 60000, scene: 'main', lighting: 'bright' },
    { time: 120000, scene: 'climax', lighting: 'strobe' },
    { time: 150000, scene: 'outro', lighting: 'fade' },
    { time: 180000, action: 'blackout' }
  ];

  async function runShow() {
    for (const cue of showSequence) {
      await sleep(cue.time);

      if (cue.scene) {
        await client.emit(`/osc/scenes/${cue.scene}/select`, null);
      }

      if (cue.lighting) {
        await client.emit(`/lighting/preset/${cue.lighting}`, null);
      }

      if (cue.action === 'blackout') {
        await client.set('/osc/master/blackout', 1);
      }

      console.log(`Cue: ${cue.scene || cue.action}`);
    }
  }

  // Start show on trigger
  client.on('/show/start', () => {
    client.set('/osc/master/blackout', 0);
    runShow();
  });
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

main();
```

## Tips

### Smooth Transitions

```javascript
// Crossfade between scenes
async function crossfadeScene(from, to, duration = 2000) {
  const steps = 60;
  const interval = duration / steps;

  for (let i = 0; i <= steps; i++) {
    const progress = i / steps;
    await client.bundle([
      { set: [`/osc/medias/${from}/opacity`, 1 - progress] },
      { set: [`/osc/medias/${to}/opacity`, progress] }
    ]);
    await sleep(interval);
  }
}
```

### Surface Groups

```javascript
// Control multiple surfaces together
const surfaceGroups = {
  left: ['surface1', 'surface2'],
  right: ['surface3', 'surface4'],
  all: ['surface1', 'surface2', 'surface3', 'surface4']
};

async function setGroupOpacity(group, opacity) {
  const surfaces = surfaceGroups[group];
  if (!surfaces) return;

  const ops = surfaces.map(s => ({
    set: [`/osc/surfaces/${s}/opacity`, opacity]
  }));
  await client.bundle(ops);
}
```

### Error Recovery

```javascript
// Handle MadMapper restart
client.on('disconnect', async () => {
  console.log('Connection lost, MadMapper may have restarted');
});

client.on('connect', async () => {
  // Restore last known state
  await restoreState();
});
```

## Troubleshooting

### OSC Not Responding

1. Check MadMapper OSC preferences are enabled
2. Verify port numbers match CLASP bridge
3. Test with OSC monitoring tool
4. Check firewall settings

### Timing Issues

1. Use bundles for synchronized changes
2. Consider network latency
3. Use MadMapper's built-in transition timing

### Media Not Playing

1. Check media file path is correct
2. Verify media is loaded in MadMapper
3. Check opacity is not zero
4. Verify surface is visible

## Next Steps

- [Live Performance Guide](../use-cases/live-performance.md)
- [Add OSC Connection](../how-to/connections/add-osc.md)
- [Resolume Integration](resolume.md)

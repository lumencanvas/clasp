---
title: "Ableton Live Integration"
description: "Connect Ableton Live to CLASP for music production and live performance control."
section: integrations
order: 1
---
# Ableton Live Integration

Connect Ableton Live to CLASP for music production and live performance control.

## Overview

Ableton Live can communicate via MIDI and OSC. CLASP enables you to:

- Send MIDI from controllers through CLASP to Ableton
- Receive tempo, transport, and track data from Ableton
- Bridge Ableton with lighting, visuals, and other systems

## Setup Options

### Option 1: MIDI (Native)

Ableton has built-in MIDI support:

```bash
# Start CLASP MIDI bridge
clasp midi --device "IAC Driver Bus 1"  # macOS
clasp midi --device "loopMIDI"          # Windows
```

Configure Ableton:
1. Go to **Preferences** > **Link/Tempo/MIDI**
2. Enable Track/Sync/Remote for your virtual MIDI device
3. Map controls using MIDI Learn (Cmd/Ctrl+M)

### Option 2: OSC (via Max for Live)

For OSC, use a Max for Live device:

1. Install "OSC Send/Receive" Max for Live device
2. Configure to send to CLASP OSC bridge port

```bash
# Start OSC bridge
clasp osc --port 9000
```

### Option 3: Ableton Link

For tempo synchronization:

```bash
# CLASP can receive Link tempo
clasp link --enable
```

## MIDI Control

### Receive from Controllers

```javascript
// MIDI controller → CLASP → Ableton
client.on('/midi/controller/cc/*/*', async (value, address) => {
  // Forward to Ableton's MIDI input
  await client.set('/midi/ableton' + address.replace('/midi/controller', ''), value);
});

// Note messages
client.on('/midi/controller/note', async (data) => {
  await client.set('/midi/ableton/note', data);
});
```

### Send to Controllers (Feedback)

```javascript
// Ableton → CLASP → Controller LEDs
client.on('/midi/ableton/feedback/*', async (value, address) => {
  await client.set('/midi/controller' + address.replace('/midi/ableton/feedback', ''), value);
});
```

## Track Control

### Volume and Pan

```javascript
// Control track volumes
async function setTrackVolume(track, volume) {
  // MIDI CC mapping (example: CC 7 for volume)
  await client.set(`/midi/ableton/cc/0/${track}`, Math.round(volume * 127));
}

// Map CLASP fader to Ableton track
client.on('/control/track/*/volume', async (value, address) => {
  const track = parseInt(address.split('/')[3]);
  await setTrackVolume(track, value);
});
```

### Clip Launching

```javascript
// Launch clips via MIDI notes
// Note number = (track - 1) + (scene - 1) * 8 (Push layout)
async function launchClip(track, scene) {
  const note = (track - 1) + (scene - 1) * 8;
  await client.emit('/midi/ableton/note', {
    note,
    velocity: 127,
    channel: 0
  });
}

// Map grid controller
client.on('/midi/controller/note', (data) => {
  if (data.velocity > 0) {
    const track = (data.note % 8) + 1;
    const scene = Math.floor(data.note / 8) + 1;
    launchClip(track, scene);
  }
});
```

### Transport Control

```javascript
// Play/Stop
client.on('/control/transport/play', () => {
  client.emit('/midi/ableton/cc/0/118', 127);  // MIDI mapped to play
});

client.on('/control/transport/stop', () => {
  client.emit('/midi/ableton/cc/0/119', 127);  // MIDI mapped to stop
});

// Tempo (requires Max for Live device)
client.on('/control/tempo', async (bpm) => {
  await client.set('/osc/live/tempo', bpm);
});
```

## OSC Integration (Max for Live)

### Receiving from Ableton

With a Max for Live OSC device:

```javascript
// Tempo sync
client.on('/osc/live/tempo', (bpm) => {
  console.log(`Tempo: ${bpm} BPM`);
  // Forward to other systems
  client.set('/osc/resolume/composition/bpm', bpm);
});

// Transport state
client.on('/osc/live/playing', (playing) => {
  if (playing) {
    client.emit('/show/start', {});
  } else {
    client.emit('/show/stop', {});
  }
});

// Track meters
client.on('/osc/live/track/*/meter', (level, address) => {
  const track = address.split('/')[4];
  // Audio-reactive visuals
  client.set(`/visuals/track/${track}/level`, level);
});
```

### Sending to Ableton

```javascript
// Parameter control
client.on('/control/synth/*', async (value, address) => {
  const param = address.split('/').pop();
  await client.set(`/osc/live/device/1/${param}`, value);
});
```

## Example: Visual-Audio Sync

Sync lighting to Ableton:

```javascript
// Beat detection
let lastBeat = Date.now();
let beatInterval = 500; // 120 BPM default

client.on('/osc/live/tempo', (bpm) => {
  beatInterval = 60000 / bpm;
});

client.on('/osc/live/beat', async () => {
  // Flash light on beat
  await client.set('/artnet/0/0/0/1', 255);
  setTimeout(() => client.set('/artnet/0/0/0/1', 0), 50);
});

// Track-based lighting
client.on('/osc/live/track/*/volume', async (level, address) => {
  const track = parseInt(address.split('/')[4]);
  // Map track volumes to light fixtures
  await client.set(`/artnet/0/0/0/${track}`, Math.round(level * 255));
});
```

## Example: Push Controller Bridge

Use a Push controller via CLASP:

```javascript
// Push sends notes for clip grid
client.on('/midi/push/note', async (data) => {
  if (data.velocity > 0) {
    // Calculate clip position
    const track = (data.note % 8) + 1;
    const scene = Math.floor(data.note / 8) + 1;

    // Launch clip
    await launchClip(track, scene);

    // Visual feedback
    await client.set(`/visuals/grid/${track}/${scene}`, 1);
  }
});

// Push encoders for device control
client.on('/midi/push/cc/0/*', async (value, address) => {
  const encoder = parseInt(address.split('/').pop());
  await client.set(`/osc/live/device/current/param/${encoder}`, value / 127);
});
```

## Example: Multi-Room Audio

Route Ableton output to different zones:

```javascript
// Receive track output assignments
const zoneRouting = {
  1: ['main', 'booth'],
  2: ['main'],
  3: ['booth'],
  4: ['lounge']
};

// When track volume changes, update zone
client.on('/midi/ableton/cc/*/volume', async (value, address) => {
  const track = parseInt(address.split('/')[4]);
  const zones = zoneRouting[track] || [];

  for (const zone of zones) {
    await client.set(`/audio/zone/${zone}/level/${track}`, value / 127);
  }
});
```

## Tips

### Reduce Latency

- Use MIDI over USB (not Bluetooth)
- Minimize buffer size in Ableton
- Keep CLASP router on same machine as Ableton

### Organize MIDI Mappings

```javascript
// Define mapping tables for clarity
const midiMap = {
  volume: { channel: 0, cc: 7 },
  pan: { channel: 0, cc: 10 },
  send1: { channel: 0, cc: 20 },
  send2: { channel: 0, cc: 21 }
};

async function setTrackParam(track, param, value) {
  const mapping = midiMap[param];
  if (mapping) {
    await client.set(
      `/midi/ableton/cc/${track - 1}/${mapping.cc}`,
      Math.round(value * 127)
    );
  }
}
```

### Handle Bank Switching

```javascript
// Track bank for controllers with limited tracks
let trackBank = 0;

client.on('/control/bank/up', () => {
  trackBank = Math.min(trackBank + 8, 120);
  updateBankDisplay();
});

client.on('/control/bank/down', () => {
  trackBank = Math.max(trackBank - 8, 0);
  updateBankDisplay();
});

client.on('/midi/controller/cc/0/*', async (value, address) => {
  const controllerTrack = parseInt(address.split('/').pop());
  const abletonTrack = trackBank + controllerTrack;
  await client.set(`/midi/ableton/cc/${abletonTrack}/7`, value);
});
```

## Troubleshooting

### MIDI Not Working

1. Check virtual MIDI device is running (IAC/loopMIDI)
2. Verify device is enabled in Ableton preferences
3. Test with MIDI Monitor application
4. Check CLASP MIDI bridge is connected to correct device

### Latency Issues

1. Reduce Ableton buffer size (Preferences > Audio)
2. Use direct USB connection for controller
3. Check CPU usage in Ableton

### OSC Not Receiving

1. Verify Max for Live device is active
2. Check OSC port matches CLASP bridge
3. Test with OSC monitoring tool

## Next Steps

- [Add MIDI Connection](../how-to/connections/add-midi.md)
- [Live Performance Guide](../use-cases/live-performance.md)
- [Resolume Integration](resolume.md)

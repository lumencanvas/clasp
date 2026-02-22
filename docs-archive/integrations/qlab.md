---
title: "QLab Integration"
description: "This guide explains how to connect QLab to CLASP for show control."
section: integrations
order: 4
---
# QLab Integration

This guide explains how to connect QLab to CLASP for show control.

## Overview

QLab is professional show control software for macOS. CLASP enables you to:

- Trigger QLab cues from any CLASP client
- Receive cue status updates in your applications
- Bridge QLab with lighting (DMX), audio (MIDI), and custom systems

## Setup

### 1. Enable OSC in QLab

1. Open your QLab workspace
2. Go to **Settings** (gear icon) > **Network**
3. In the OSC tab:
   - Check **Use OSC Controls**
   - Note the **OSC Input Port** (default: 53000)
   - Set **Passcode** if desired (recommended for production)

### 2. Configure CLASP Bridge

Create an OSC bridge to communicate with QLab:

```bash
# Send commands to QLab
clasp bridge create \
  --source internal \
  --target osc:127.0.0.1:53000
```

For bidirectional communication (receiving cue updates):

```bash
# Configure QLab to send OSC to CLASP
# In QLab: Settings > Network > OSC Output
# Add destination: your-ip:8000

clasp bridge create \
  --source osc:0.0.0.0:8000 \
  --target osc:127.0.0.1:53000
```

## QLab OSC Command Reference

### Cue Control

| Address | Arguments | Description |
|---------|-----------|-------------|
| `/go` | - | GO (fire next cue) |
| `/cue/{cue_number}/start` | - | Start specific cue |
| `/cue/{cue_number}/stop` | - | Stop specific cue |
| `/cue/{cue_number}/pause` | - | Pause specific cue |
| `/cue/{cue_number}/resume` | - | Resume specific cue |
| `/cue/{cue_number}/load` | - | Load specific cue |
| `/cue/{cue_number}/reset` | - | Reset specific cue |

### Cue Parameters

| Address | Arguments | Description |
|---------|-----------|-------------|
| `/cue/{id}/level/{row}/{col}` | float | Set audio level |
| `/cue/{id}/rate` | float | Set playback rate |
| `/cue/{id}/preWait` | float | Set pre-wait time |
| `/cue/{id}/postWait` | float | Set post-wait time |
| `/cue/{id}/duration` | float | Set duration |
| `/cue/{id}/opacity` | float | Set opacity (video) |

### Workspace Control

| Address | Arguments | Description |
|---------|-----------|-------------|
| `/panic` | - | Panic (stop all cues) |
| `/reset` | - | Reset all cues |
| `/pause` | - | Pause all cues |
| `/resume` | - | Resume all cues |
| `/toggleFullScreen` | - | Toggle fullscreen |

### Playhead Control

| Address | Arguments | Description |
|---------|-----------|-------------|
| `/select/{cue_number}` | - | Select cue |
| `/playhead/{cue_number}` | - | Move playhead to cue |
| `/playheadNext` | - | Move playhead to next |
| `/playheadPrevious` | - | Move playhead to previous |

## Example: Basic Show Control

Control QLab from CLASP:

```javascript
const { ClaspBuilder } = require('@clasp-to/core');

async function main() {
  const clasp = await new ClaspBuilder('ws://localhost:7330')
    .withName('qlab-control')
    .connect();

  // Fire GO
  function go() {
    clasp.emit('/osc/go', null);
  }

  // Start specific cue
  function startCue(cueNumber) {
    clasp.emit(`/osc/cue/${cueNumber}/start`, null);
  }

  // Stop specific cue
  function stopCue(cueNumber) {
    clasp.emit(`/osc/cue/${cueNumber}/stop`, null);
  }

  // Panic (stop everything)
  function panic() {
    clasp.emit('/osc/panic', null);
  }

  // Example: Start show sequence
  await startCue('1');    // Start cue 1
  await sleep(5000);
  await go();             // Fire next cue
  await sleep(10000);
  await startCue('BLACKOUT');
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

main();
```

## Example: MIDI Cue Controller

Trigger QLab cues from a MIDI controller:

```javascript
const { ClaspBuilder } = require('@clasp-to/core');

async function main() {
  const clasp = await new ClaspBuilder('ws://localhost:7330')
    .withName('midi-qlab-bridge')
    .connect();

  // Map MIDI notes to cue numbers
  const cueMap = {
    36: '1',      // Kick drum -> Cue 1
    37: '2',      // Snare -> Cue 2
    38: '3',      // etc.
    39: '4',
    40: '5',
    41: 'BLACKOUT',
    42: 'RESTORE',
  };

  // Listen for MIDI notes
  clasp.on('/midi/*/note', (value, address, meta) => {
    if (meta.velocity > 0) {  // Note on
      const cue = cueMap[meta.note];
      if (cue) {
        clasp.emit(`/osc/cue/${cue}/start`, null);
        console.log(`Triggered cue: ${cue}`);
      }
    }
  });

  // Panic button (MIDI CC 120)
  clasp.on('/midi/*/cc/120', (value) => {
    if (value > 64) {
      clasp.emit('/osc/panic', null);
      console.log('PANIC!');
    }
  });

  // GO button (MIDI CC 121)
  clasp.on('/midi/*/cc/121', (value) => {
    if (value > 64) {
      clasp.emit('/osc/go', null);
      console.log('GO');
    }
  });

  console.log('MIDI-QLab bridge running...');
}

main();
```

## Example: Web-Based Show Control

Create a web interface for QLab:

```javascript
// server.js
const { ClaspBuilder } = require('@clasp-to/core');
const express = require('express');
const http = require('http');
const { Server } = require('socket.io');

const app = express();
const server = http.createServer(app);
const io = new Server(server);

async function main() {
  const clasp = await new ClaspBuilder('ws://localhost:7330')
    .withName('qlab-web-controller')
    .connect();

  // Serve static files
  app.use(express.static('public'));

  // WebSocket connection from browser
  io.on('connection', (socket) => {
    console.log('Web client connected');

    socket.on('go', () => {
      clasp.emit('/osc/go', null);
    });

    socket.on('cue', (cueNumber) => {
      clasp.emit(`/osc/cue/${cueNumber}/start`, null);
    });

    socket.on('panic', () => {
      clasp.emit('/osc/panic', null);
    });

    socket.on('level', ({ cue, row, col, level }) => {
      clasp.set(`/osc/cue/${cue}/level/${row}/${col}`, level);
    });
  });

  // Forward cue updates from QLab to web clients
  clasp.on('/osc/reply/**', (value, address) => {
    io.emit('qlab-update', { address, value });
  });

  server.listen(3000, () => {
    console.log('Web controller at http://localhost:3000');
  });
}

main();
```

## Example: Timecode Sync

Sync QLab with external timecode:

```javascript
const { ClaspBuilder } = require('@clasp-to/core');

async function main() {
  const clasp = await new ClaspBuilder('ws://localhost:7330')
    .withName('timecode-qlab-sync')
    .connect();

  // Cue list with timecode triggers
  const cueList = [
    { time: '00:00:00:00', cue: 'INTRO' },
    { time: '00:00:30:00', cue: 'SCENE1' },
    { time: '00:01:00:00', cue: 'SCENE2' },
    { time: '00:01:45:00', cue: 'CLIMAX' },
    { time: '00:02:30:00', cue: 'BLACKOUT' },
  ];

  // Track which cues have been fired
  const firedCues = new Set();

  // Listen for timecode from external source
  clasp.on('/timecode/current', (tc) => {
    const tcStr = formatTimecode(tc);

    for (const item of cueList) {
      if (tcStr >= item.time && !firedCues.has(item.cue)) {
        clasp.emit(`/osc/cue/${item.cue}/start`, null);
        firedCues.add(item.cue);
        console.log(`${tcStr}: Triggered ${item.cue}`);
      }
    }
  });

  // Reset on timecode restart
  clasp.on('/timecode/reset', () => {
    firedCues.clear();
    clasp.emit('/osc/reset', null);
    console.log('Timecode reset - cues cleared');
  });
}

function formatTimecode(frames, fps = 30) {
  const totalSeconds = Math.floor(frames / fps);
  const f = frames % fps;
  const s = totalSeconds % 60;
  const m = Math.floor(totalSeconds / 60) % 60;
  const h = Math.floor(totalSeconds / 3600);
  return `${pad(h)}:${pad(m)}:${pad(s)}:${pad(f)}`;
}

function pad(n) {
  return n.toString().padStart(2, '0');
}

main();
```

## Tips

### Use Cue Names Instead of Numbers

QLab allows using cue names in OSC addresses:

```javascript
// Using cue name (more readable, won't break if cues renumbered)
clasp.emit('/osc/cue/BLACKOUT/start', null);

// Using cue number
clasp.emit('/osc/cue/99/start', null);
```

### Query Cue State

Request information about cues:

```javascript
// This requires setting up QLab to respond to queries
// Configure reply address in QLab OSC settings

clasp.emit('/osc/cue/1/isRunning', null);
// QLab will reply to configured address with true/false
```

### Handle Network Passcode

If QLab has a passcode configured:

```javascript
// Include passcode in command (QLab 4+)
clasp.emit('/osc/workspace/PASSCODE/go', null);
```

## Troubleshooting

### Commands Not Working

1. Verify OSC is enabled in QLab Settings > Network
2. Check port number matches CLASP bridge config
3. Try sending commands with QLab's built-in OSC monitor
4. Check if workspace passcode is required

### Cue Not Found

1. Verify cue number or name exists
2. Check for typos in cue ID
3. Use QLab's OSC log to see incoming messages

### Timing Issues

1. Use bundles for synchronized commands
2. Consider network latency in timecode applications
3. Use QLab's pre-wait for precise timing

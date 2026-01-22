# TouchOSC Integration

This guide explains how to connect TouchOSC to CLASP for bidirectional control.

## Overview

TouchOSC is a modular OSC control surface for iOS and Android. CLASP's OSC bridge allows you to:

- Receive OSC messages from TouchOSC as CLASP signals
- Send CLASP values back to TouchOSC for feedback
- Create bidirectional control interfaces

## Setup

### 1. Configure TouchOSC

1. Open TouchOSC on your device
2. Go to **Connections** > **OSC**
3. Configure the connection:
   - **Host**: Your computer's IP address
   - **Send Port**: 8000 (or your chosen CLASP OSC port)
   - **Receive Port**: 9000 (for feedback)
   - **Enabled**: On

### 2. Configure CLASP Bridge

Create an OSC bridge in CLASP:

```bash
# Using CLI
clasp bridge create --source osc:0.0.0.0:8000 --target internal

# Or in the desktop app:
# 1. Click "Add Bridge"
# 2. Select "OSC" as source
# 3. Enter bind address: 0.0.0.0:8000
```

For bidirectional control (sending feedback to TouchOSC):

```bash
clasp bridge create \
  --source osc:0.0.0.0:8000 \
  --target osc:192.168.1.100:9000  # TouchOSC device IP
```

## Address Mapping

### TouchOSC to CLASP

OSC addresses are converted to CLASP addresses with the `/osc` namespace prefix:

| TouchOSC | CLASP |
|----------|-------|
| `/1/fader1` | `/osc/1/fader1` |
| `/main/volume` | `/osc/main/volume` |
| `/xy/1` | `/osc/xy/1` |

### CLASP to TouchOSC

To send values back to TouchOSC, set values on the corresponding addresses:

```javascript
// JavaScript
clasp.set('/osc/1/fader1', 0.75);  // Updates fader in TouchOSC
clasp.set('/osc/1/led1', 1);       // Turns on LED
```

```rust
// Rust
client.set("/osc/1/fader1", 0.75).await?;
```

## Control Types

### Faders

TouchOSC faders send float values 0.0-1.0:

```javascript
clasp.on('/osc/1/fader*', (value, address) => {
  console.log(`${address} = ${value}`);  // value is 0.0-1.0
});
```

### Buttons (Momentary)

Momentary buttons send 1 on press, 0 on release:

```javascript
clasp.on('/osc/1/push*', (value, address) => {
  if (value === 1) {
    console.log(`${address} pressed`);
  } else {
    console.log(`${address} released`);
  }
});
```

### Toggles

Toggle buttons send 0 or 1:

```javascript
clasp.on('/osc/1/toggle*', (value, address) => {
  console.log(`${address} is ${value ? 'on' : 'off'}`);
});
```

### XY Pads

XY pads send two floats [x, y]:

```javascript
clasp.on('/osc/1/xy*', (value, address) => {
  const [x, y] = value;
  console.log(`${address}: x=${x}, y=${y}`);
});
```

### Rotary Encoders

Rotary controls send float values 0.0-1.0:

```javascript
clasp.on('/osc/1/rotary*', (value, address) => {
  const degrees = value * 360;
  console.log(`${address} = ${degrees}°`);
});
```

## Example: Lighting Controller

This example creates a simple lighting controller with faders and buttons:

```javascript
const { Clasp } = require('@clasp-to/core');

async function main() {
  const clasp = await Clasp.builder('ws://localhost:7330')
    .withName('touchosc-lights')
    .connect();

  // Map TouchOSC faders to DMX channels
  clasp.on('/osc/lights/fader*', (value, address) => {
    const channel = parseInt(address.split('fader')[1]);
    const dmxValue = Math.round(value * 255);
    clasp.set(`/dmx/1/${channel}`, dmxValue);
  });

  // Map buttons to cues
  clasp.on('/osc/lights/cue*', (value, address) => {
    if (value === 1) {
      const cueNum = address.split('cue')[1];
      clasp.emit('/lighting/cue/trigger', { cue: cueNum });
    }
  });

  // Send feedback to TouchOSC when cues complete
  clasp.on('/lighting/cue/complete', (value) => {
    clasp.set(`/osc/lights/cue${value.cue}`, 0);
  });
}

main();
```

## Example: Audio Mixer

Control an audio mixer with TouchOSC:

```javascript
const { Clasp } = require('@clasp-to/core');

async function main() {
  const clasp = await Clasp.builder('ws://localhost:7330')
    .withName('touchosc-mixer')
    .connect();

  // Channel faders
  for (let ch = 1; ch <= 8; ch++) {
    clasp.on(`/osc/mixer/ch${ch}/fader`, (value) => {
      // Convert 0-1 to dB (-inf to +10dB)
      const db = value === 0 ? -Infinity : (value * 70) - 60;
      clasp.set(`/mixer/ch${ch}/volume`, db);
    });

    // Mute buttons
    clasp.on(`/osc/mixer/ch${ch}/mute`, (value) => {
      clasp.set(`/mixer/ch${ch}/mute`, value === 1);
    });

    // Solo buttons
    clasp.on(`/osc/mixer/ch${ch}/solo`, (value) => {
      clasp.set(`/mixer/ch${ch}/solo`, value === 1);
    });
  }

  // Master fader
  clasp.on('/osc/mixer/master', (value) => {
    clasp.set('/mixer/master/volume', value);
  });
}

main();
```

## Tips

### Reduce Network Traffic

Use rate limiting for high-frequency controls like faders:

```javascript
clasp.on('/osc/1/fader*', callback, { maxRate: 30 });  // 30 updates/sec max
```

### Handle Multiple Devices

Use different OSC ports for different TouchOSC instances:

```bash
clasp bridge create --source osc:0.0.0.0:8001 --target internal  # Device 1
clasp bridge create --source osc:0.0.0.0:8002 --target internal  # Device 2
```

### Test Connection

Use Learn Mode in the CLASP desktop app to verify incoming messages:

1. Enable Learn Mode
2. Move controls on TouchOSC
3. Watch addresses appear in the signal monitor

## Troubleshooting

### No Messages Received

1. Verify TouchOSC Host IP matches your computer
2. Check that ports are not blocked by firewall
3. Ensure CLASP OSC bridge is running
4. Try binding to `0.0.0.0` instead of `127.0.0.1`

### Feedback Not Working

1. Verify TouchOSC Receive Port matches CLASP target port
2. Check TouchOSC device IP in CLASP bridge config
3. Ensure bidirectional bridge is configured

### High Latency

1. Use wired network if possible
2. Enable rate limiting on high-frequency controls
3. Check for network congestion

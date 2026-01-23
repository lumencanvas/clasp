# Installation Art

Create interactive installations with CLASP connecting sensors, visuals, and audio.

## Overview

Installation art typically involves:

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   Sensors   │  │   Visuals   │  │    Audio    │  │  Lighting   │
│ (HTTP/MQTT) │  │   (OSC)     │  │   (OSC)     │  │  (Art-Net)  │
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                │                │                │
       └────────────────┴────────────────┴────────────────┘
                                │
                         ┌──────▼──────┐
                         │    CLASP    │
                         │   Router    │
                         └─────────────┘
```

## Common Sensor Types

### Motion/Presence

```javascript
// PIR sensor data (from ESP32 or similar)
client.on('/sensors/motion/*', async (detected, address) => {
  const zone = address.split('/').pop();
  if (detected) {
    await activateZone(zone);
  }
});
```

### Distance/Proximity

```javascript
// Ultrasonic or LIDAR sensor
client.on('/sensors/distance/*', async (cm, address) => {
  const zone = address.split('/').pop();
  // Closer = more intense
  const intensity = Math.max(0, Math.min(1, (200 - cm) / 200));
  await client.set(`/zones/${zone}/intensity`, intensity);
});
```

### Touch/Capacitive

```javascript
// Capacitive touch panels
client.on('/sensors/touch/*', async (touched, address) => {
  const panel = address.split('/').pop();
  if (touched) {
    await client.emit('/interaction/touch', { panel });
  }
});
```

### Environmental

```javascript
// Temperature, humidity, light level
client.on('/sensors/environment/#', async (value, address) => {
  const parts = address.split('/');
  const sensorType = parts[3];
  const location = parts[4];

  // Use environment data to influence artwork
  await adjustArtwork(sensorType, location, value);
});
```

## Architecture Patterns

### Zone-Based Control

Divide installation into zones, each responding independently:

```javascript
const zones = {
  entrance: { sensors: [1, 2], lights: [1, 2, 3], visuals: 'layer1' },
  main: { sensors: [3, 4, 5], lights: [4, 5, 6, 7], visuals: 'layer2' },
  exit: { sensors: [6, 7], lights: [8, 9], visuals: 'layer3' }
};

// When sensor triggers, activate its zone
client.on('/sensors/motion/*', async (value, address) => {
  const sensorId = parseInt(address.split('/').pop());

  for (const [zoneName, zone] of Object.entries(zones)) {
    if (zone.sensors.includes(sensorId) && value) {
      await activateZone(zoneName, zone);
    }
  }
});

async function activateZone(name, zone) {
  const ops = [];

  // Lights
  for (const light of zone.lights) {
    ops.push({ set: [`/artnet/0/0/0/${light}`, 255] });
  }

  // Visuals
  ops.push({ set: [`/osc/composition/layers/${zone.visuals}/video/opacity`, 1] });

  await client.bundle(ops);
}
```

### State Machine

Track installation state for complex behaviors:

```javascript
let state = 'idle';
let lastActivity = Date.now();

const states = {
  idle: {
    onEnter: async () => {
      await client.set('/visuals/mode', 'ambient');
      await client.set('/lights/master', 0.1);
    },
    onActivity: () => transition('active')
  },
  active: {
    onEnter: async () => {
      await client.set('/visuals/mode', 'interactive');
      await client.set('/lights/master', 0.8);
    },
    onIdle: () => transition('cooldown')
  },
  cooldown: {
    onEnter: async () => {
      await client.set('/visuals/mode', 'fadeout');
      // Fade lights over 10 seconds
      fadeOut('/lights/master', 10000);
    },
    onActivity: () => transition('active'),
    onComplete: () => transition('idle')
  }
};

async function transition(newState) {
  console.log(`${state} -> ${newState}`);
  state = newState;
  await states[state].onEnter?.();
}

// Activity detection
client.on('/sensors/**', () => {
  lastActivity = Date.now();
  states[state].onActivity?.();
});

// Idle detection
setInterval(() => {
  if (Date.now() - lastActivity > 30000) {
    states[state].onIdle?.();
  }
}, 1000);
```

### Multi-User Tracking

Track multiple visitors simultaneously:

```javascript
const visitors = new Map();

client.on('/sensors/entry/*', (value, address) => {
  const entryId = address.split('/').pop();
  if (value) {
    const id = generateId();
    visitors.set(id, { zone: 'entrance', enteredAt: Date.now() });
    client.emit('/visitors/entered', { id, zone: 'entrance' });
  }
});

client.on('/sensors/zone/*/presence', (count, address) => {
  const zone = address.split('/')[3];
  client.set(`/zones/${zone}/occupancy`, count);
});

client.on('/sensors/exit/*', (value) => {
  if (value && visitors.size > 0) {
    // Remove oldest visitor
    const [[id]] = visitors.entries();
    visitors.delete(id);
    client.emit('/visitors/exited', { id });
  }
});
```

## Example: Interactive Light Wall

Wall that responds to proximity:

```javascript
const WALL_WIDTH = 10;
const WALL_HEIGHT = 5;

// Proximity sensors at bottom of wall
client.on('/sensors/proximity/*', async (distance, address) => {
  const sensorIndex = parseInt(address.split('/').pop());
  const column = Math.floor(sensorIndex * WALL_WIDTH / NUM_SENSORS);

  // Height based on distance (closer = higher)
  const height = Math.floor((1 - distance / MAX_DISTANCE) * WALL_HEIGHT);

  // Update column of lights
  const ops = [];
  for (let y = 0; y < WALL_HEIGHT; y++) {
    const channel = column * WALL_HEIGHT + y + 1;
    const on = y < height;
    ops.push({ set: [`/artnet/0/0/0/${channel}`, on ? 255 : 0] });
  }

  await client.bundle(ops);
});
```

## Example: Sound-Reactive Installation

Audio input drives visuals and lights:

```javascript
// Receive audio analysis from audio software
client.on('/osc/audio/fft/*', async (value, address) => {
  const band = address.split('/').pop();

  // Map frequency bands to visual elements
  switch (band) {
    case 'bass':
      await client.set('/artnet/0/0/0/1', Math.round(value * 255));
      await client.set('/osc/visuals/bass', value);
      break;
    case 'mid':
      await client.set('/artnet/0/0/0/2', Math.round(value * 255));
      await client.set('/osc/visuals/mid', value);
      break;
    case 'high':
      await client.set('/artnet/0/0/0/3', Math.round(value * 255));
      await client.set('/osc/visuals/high', value);
      break;
  }
});

// Beat detection
client.on('/osc/audio/beat', async () => {
  // Flash all lights on beat
  await client.set('/artnet/0/0/0/10', 255);
  setTimeout(() => client.set('/artnet/0/0/0/10', 0), 50);
});
```

## Reliability Considerations

### Graceful Degradation

```javascript
// Heartbeat monitoring for critical components
const components = new Map();

client.on('/heartbeat/*', (timestamp, address) => {
  const component = address.split('/').pop();
  components.set(component, Date.now());
});

setInterval(() => {
  const now = Date.now();
  for (const [component, lastSeen] of components) {
    if (now - lastSeen > 5000) {
      console.warn(`Component ${component} offline`);
      handleComponentFailure(component);
    }
  }
}, 1000);

function handleComponentFailure(component) {
  // Fall back to safe state
  if (component === 'sensors') {
    // Use ambient mode without interactivity
    client.set('/visuals/mode', 'ambient');
  }
}
```

### Auto-Recovery

```javascript
// Reconnection handling
client.on('disconnect', () => {
  console.log('Connection lost, attempting recovery...');
});

client.on('connect', async () => {
  // Restore state after reconnection
  await client.set('/system/status', 'online');
  await restoreLastKnownState();
});
```

## Scheduling

### Time-Based Behavior

```javascript
// Different behaviors at different times
function getTimeOfDayMode() {
  const hour = new Date().getHours();
  if (hour >= 9 && hour < 18) return 'day';
  if (hour >= 18 && hour < 22) return 'evening';
  return 'night';
}

setInterval(async () => {
  const mode = getTimeOfDayMode();
  await client.set('/installation/timeMode', mode);
}, 60000);

client.on('/installation/timeMode', async (mode) => {
  switch (mode) {
    case 'day':
      await client.set('/lights/maxBrightness', 0.5);
      break;
    case 'evening':
      await client.set('/lights/maxBrightness', 1.0);
      break;
    case 'night':
      await client.set('/lights/maxBrightness', 0.3);
      break;
  }
});
```

## Next Steps

- [Embedded Systems](embedded-systems.md)
- [TouchDesigner Integration](../integrations/touchdesigner.md)
- [Add Art-Net](../how-to/connections/add-artnet.md)

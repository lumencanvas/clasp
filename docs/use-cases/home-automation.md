# Home Automation

Integrate CLASP with smart home systems for unified control.

## Overview

CLASP bridges creative tools with home automation:

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   Sensors   │  │   Lights    │  │   Climate   │
│   (MQTT)    │  │  (Art-Net)  │  │   (MQTT)    │
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                │                │
       └────────────────┼────────────────┘
                        │
                 ┌──────▼──────┐
                 │    CLASP    │
                 │   Router    │
                 └──────┬──────┘
                        │
              ┌─────────┼─────────┐
              │         │         │
       ┌──────▼──────┐  │  ┌──────▼──────┐
       │    Home     │  │  │   Control   │
       │  Assistant  │  │  │    App      │
       └─────────────┘  │  └─────────────┘
                 ┌──────▼──────┐
                 │  Creative   │
                 │   Tools     │
                 └─────────────┘
```

## Integration Options

### MQTT (Recommended)

Most home automation systems support MQTT:

```bash
# Connect to MQTT broker
clasp mqtt --host localhost --port 1883

# With authentication
clasp mqtt --host mqtt.local --port 1883 --user clasp --password secret
```

### Home Assistant via MQTT

```yaml
# Home Assistant configuration.yaml
mqtt:
  broker: localhost
  port: 1883

light:
  - platform: mqtt
    name: "Studio Light"
    command_topic: "clasp/lights/studio/set"
    state_topic: "clasp/lights/studio/state"
    brightness_command_topic: "clasp/lights/studio/brightness/set"
    brightness_state_topic: "clasp/lights/studio/brightness"
```

```javascript
// CLASP handler
client.on('/mqtt/clasp/lights/studio/set', async (value) => {
  const on = value === 'ON';
  await client.set('/artnet/0/0/0/1', on ? 255 : 0);
  await client.set('/mqtt/clasp/lights/studio/state', value);
});

client.on('/mqtt/clasp/lights/studio/brightness/set', async (value) => {
  await client.set('/artnet/0/0/0/1', parseInt(value));
  await client.set('/mqtt/clasp/lights/studio/brightness', value);
});
```

## Common Automations

### Scene Control

```javascript
const scenes = {
  work: {
    '/artnet/0/0/0/1': 255,   // Desk lamp full
    '/artnet/0/0/0/2': 128,   // Ambient half
    '/mqtt/climate/mode': 'cool',
    '/mqtt/climate/temp': 22
  },
  relax: {
    '/artnet/0/0/0/1': 50,
    '/artnet/0/0/0/2': 200,
    '/mqtt/climate/mode': 'auto',
    '/mqtt/climate/temp': 24
  },
  movie: {
    '/artnet/0/0/0/1': 0,
    '/artnet/0/0/0/2': 20,
    '/mqtt/media/projector': 'ON'
  },
  away: {
    '/artnet/0/0/0/1': 0,
    '/artnet/0/0/0/2': 0,
    '/mqtt/climate/mode': 'eco'
  }
};

async function activateScene(name) {
  const scene = scenes[name];
  if (!scene) return;

  const ops = Object.entries(scene).map(([address, value]) => ({
    set: [address, value]
  }));

  await client.bundle(ops);
  await client.set('/home/scene', name);
}

// Trigger from various sources
client.on('/osc/touchosc/scene/*', (value, address) => {
  if (value === 1) {
    const scene = address.split('/').pop();
    activateScene(scene);
  }
});

client.on('/mqtt/homeassistant/scene/*/set', (value, address) => {
  if (value === 'ON') {
    const scene = address.split('/')[4];
    activateScene(scene);
  }
});
```

### Motion-Activated Lighting

```javascript
// Motion sensor triggers
client.on('/mqtt/sensors/motion/*', async (value, address) => {
  const room = address.split('/').pop();

  if (value === 'ON') {
    // Check time of day
    const hour = new Date().getHours();
    const brightness = (hour >= 22 || hour < 6) ? 50 : 255;

    await client.set(`/artnet/0/0/${room}/1`, brightness);

    // Reset auto-off timer
    clearTimeout(timers[room]);
    timers[room] = setTimeout(() => {
      client.set(`/artnet/0/0/${room}/1`, 0);
    }, 5 * 60 * 1000); // 5 minutes
  }
});

const timers = {};
```

### Presence-Based Climate

```javascript
let occupancy = {};

client.on('/mqtt/sensors/presence/*', async (value, address) => {
  const room = address.split('/').pop();
  occupancy[room] = value === 'ON';

  const anyOccupied = Object.values(occupancy).some(v => v);

  if (anyOccupied) {
    await client.set('/mqtt/climate/mode', 'comfort');
  } else {
    await client.set('/mqtt/climate/mode', 'eco');
  }
});
```

### Sunrise Simulation

```javascript
async function sunriseAlarm(duration = 30 * 60 * 1000) {
  const steps = 100;
  const interval = duration / steps;

  for (let i = 0; i <= steps; i++) {
    const brightness = Math.round((i / steps) * 255);
    // Warm color temperature (simulate sunrise)
    const warmth = Math.min(255, brightness * 1.2);
    const cool = Math.round(brightness * 0.3);

    await client.bundle([
      { set: ['/artnet/0/0/0/1', warmth] },  // Warm channel
      { set: ['/artnet/0/0/0/2', brightness] },
      { set: ['/artnet/0/0/0/3', cool] }     // Cool channel
    ]);

    await sleep(interval);
  }
}

// Schedule alarm
client.on('/home/alarm/set', (time) => {
  const [hours, minutes] = time.split(':').map(Number);
  scheduleAt(hours, minutes, () => sunriseAlarm());
});
```

## Sensor Integration

### Temperature Monitoring

```javascript
// Aggregate temperature sensors
const temperatures = {};

client.on('/mqtt/sensors/temperature/*', (value, address) => {
  const room = address.split('/').pop();
  temperatures[room] = parseFloat(value);

  // Calculate average
  const values = Object.values(temperatures);
  const avg = values.reduce((a, b) => a + b, 0) / values.length;

  client.set('/home/temperature/average', avg.toFixed(1));
});
```

### Door/Window Sensors

```javascript
const openings = new Set();

client.on('/mqtt/sensors/door/*', async (value, address) => {
  const door = address.split('/').pop();

  if (value === 'open') {
    openings.add(door);

    // Pause HVAC if exterior door open
    if (door === 'front' || door === 'back') {
      await client.set('/mqtt/climate/pause', true);
    }
  } else {
    openings.delete(door);

    if (openings.size === 0) {
      await client.set('/mqtt/climate/pause', false);
    }
  }

  await client.set('/home/openings', Array.from(openings));
});
```

## Energy Management

### Power Monitoring

```javascript
client.on('/mqtt/energy/power', async (watts) => {
  const kw = parseFloat(watts) / 1000;
  await client.set('/home/energy/current_kw', kw.toFixed(2));

  // Alert on high usage
  if (kw > 5) {
    await client.emit('/alerts/high_power', { kw });
  }
});

// Track daily usage
let dailyKwh = 0;
let lastReading = Date.now();

client.on('/mqtt/energy/power', (watts) => {
  const now = Date.now();
  const hours = (now - lastReading) / 3600000;
  dailyKwh += (parseFloat(watts) / 1000) * hours;
  lastReading = now;

  client.set('/home/energy/today_kwh', dailyKwh.toFixed(2));
});
```

### Solar Integration

```javascript
client.on('/mqtt/solar/production', async (watts) => {
  const producing = parseFloat(watts);
  const consuming = parseFloat(await client.get('/home/energy/current_kw')) * 1000;
  const net = producing - consuming;

  await client.set('/home/energy/net', net.toFixed(0));
  await client.set('/home/energy/exporting', net > 0);
});
```

## Multi-Room Audio

```javascript
// Zone grouping
const audioZones = {
  living: ['speaker-1', 'speaker-2'],
  kitchen: ['speaker-3'],
  bedroom: ['speaker-4']
};

client.on('/audio/zone/*/volume', async (value, address) => {
  const zone = address.split('/')[3];
  const speakers = audioZones[zone];

  if (speakers) {
    const ops = speakers.map(speaker => ({
      set: [`/mqtt/audio/${speaker}/volume`, value]
    }));
    await client.bundle(ops);
  }
});

// Follow-me audio
client.on('/mqtt/sensors/presence/*', async (value, address) => {
  if (value !== 'ON') return;

  const room = address.split('/').pop();
  const currentTrack = await client.get('/audio/now_playing');

  if (currentTrack) {
    // Move audio to current room
    for (const [zone, speakers] of Object.entries(audioZones)) {
      const volume = zone === room ? 80 : 0;
      for (const speaker of speakers) {
        await client.set(`/mqtt/audio/${speaker}/volume`, volume);
      }
    }
  }
});
```

## Security Integration

### Alarm System

```javascript
let alarmState = 'disarmed';

client.on('/home/alarm/arm', async (mode) => {
  alarmState = mode; // 'away', 'home', 'disarmed'
  await client.set('/home/alarm/state', alarmState);

  if (mode === 'away') {
    await activateScene('away');
  }
});

client.on('/mqtt/sensors/motion/*', async (value) => {
  if (alarmState !== 'disarmed' && value === 'ON') {
    await client.emit('/alerts/intrusion', {
      state: alarmState,
      timestamp: Date.now()
    });

    // Flash lights
    for (let i = 0; i < 5; i++) {
      await client.set('/artnet/0/0/0/1', 255);
      await sleep(200);
      await client.set('/artnet/0/0/0/1', 0);
      await sleep(200);
    }
  }
});
```

## Dashboard Integration

Expose CLASP state to web dashboards:

```javascript
const express = require('express');
const app = express();

// REST API for dashboard
app.get('/api/state', async (req, res) => {
  const state = {
    temperature: await client.get('/home/temperature/average'),
    energy: await client.get('/home/energy/current_kw'),
    scene: await client.get('/home/scene'),
    alarm: await client.get('/home/alarm/state')
  };
  res.json(state);
});

app.post('/api/scene/:name', async (req, res) => {
  await activateScene(req.params.name);
  res.json({ success: true });
});

app.listen(3000);
```

## Next Steps

- [Home Assistant Integration](../integrations/home-assistant.md)
- [Add MQTT](../how-to/connections/add-mqtt.md)
- [Embedded Systems](embedded-systems.md)

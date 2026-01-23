# Home Assistant Integration

Connect Home Assistant to CLASP for smart home control.

## Overview

Home Assistant supports MQTT for device communication.

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│    Home     │────►│    MQTT     │────►│MQTT Bridge  │────►│   Router    │
│  Assistant  │◄────│   Broker    │◄────│             │◄────│             │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

## Setup

### 1. Enable MQTT in Home Assistant

Add to `configuration.yaml`:

```yaml
mqtt:
  broker: localhost
  port: 1883
```

### 2. Start CLASP

```bash
# Start router
clasp server --port 7330

# Connect to Home Assistant's MQTT
clasp mqtt --host localhost --port 1883 --topic "homeassistant/#"
```

## Home Assistant MQTT Topics

Home Assistant uses discovery topics:

| Topic | Purpose |
|-------|---------|
| `homeassistant/sensor/*/state` | Sensor states |
| `homeassistant/light/*/set` | Light commands |
| `homeassistant/switch/*/state` | Switch states |

## Receive Home Assistant Data

```javascript
// All Home Assistant updates
client.on('/mqtt/homeassistant/**', (value, address) => {
  console.log(address, value);
});

// Specific sensor
client.on('/mqtt/homeassistant/sensor/temperature/state', (value) => {
  console.log('Temperature:', value);
});
```

## Control Home Assistant Devices

```javascript
// Turn on a light
await client.set('/mqtt/homeassistant/light/living_room/set', {
  state: 'ON',
  brightness: 200
});

// Turn off a switch
await client.set('/mqtt/homeassistant/switch/fan/set', 'OFF');

// Set thermostat
await client.set('/mqtt/homeassistant/climate/hvac/set', {
  temperature: 22,
  hvac_mode: 'heat'
});
```

## Create Custom Sensors

Expose CLASP data as Home Assistant sensors:

```javascript
// Publish sensor data that Home Assistant can discover
async function createSensor(name, value, unit) {
  // Discovery message
  await client.set(`/mqtt/homeassistant/sensor/${name}/config`, {
    name: name,
    state_topic: `homeassistant/sensor/${name}/state`,
    unit_of_measurement: unit
  });

  // State
  await client.set(`/mqtt/homeassistant/sensor/${name}/state`, value);
}

// Example: Expose CLASP sensor
createSensor('studio_temp', 23.5, '°C');
```

## Automation Integration

### Trigger CLASP from Home Assistant

Home Assistant automation:
```yaml
automation:
  - alias: "Motion triggers CLASP"
    trigger:
      - platform: state
        entity_id: binary_sensor.motion
        to: 'on'
    action:
      - service: mqtt.publish
        data:
          topic: "clasp/events/motion"
          payload: '{"room": "living"}'
```

CLASP handler:
```javascript
client.on('/mqtt/clasp/events/motion', async (data) => {
  await client.set('/lights/living/brightness', 255);
});
```

### Trigger Home Assistant from CLASP

```javascript
// Turn on lights when entering show mode
async function startShow() {
  await client.set('/mqtt/homeassistant/scene/concert_mode/set', 'ON');
}
```

## Example: Unified Control

Control Home Assistant devices alongside creative equipment:

```javascript
// Master dimmer controls everything
client.on('/control/master', async (value) => {
  // CLASP lights (Art-Net)
  await client.set('/artnet/0/0/0/1', Math.round(value * 255));

  // Home Assistant lights
  await client.set('/mqtt/homeassistant/light/ceiling/set', {
    state: value > 0 ? 'ON' : 'OFF',
    brightness: Math.round(value * 255)
  });
});
```

## Troubleshooting

### MQTT not connecting

1. Check MQTT broker is running
2. Verify credentials if required
3. Check firewall allows port 1883

### Devices not responding

1. Verify topic format matches Home Assistant
2. Check device supports MQTT
3. Test with MQTT client: `mosquitto_sub -t '#'`

## Next Steps

- [Add MQTT](../how-to/connections/add-mqtt.md)
- [Home Automation Guide](../use-cases/home-automation.md)

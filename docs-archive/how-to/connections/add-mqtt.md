---
title: "Add MQTT"
description: "Connect MQTT brokers and IoT devices to CLASP."
section: how-to
order: 5
---
# Add MQTT

Connect MQTT brokers and IoT devices to CLASP.

## Prerequisites

- Running CLASP router
- MQTT broker (Mosquitto, HiveMQ, etc.)

## Start MQTT Bridge

### CLI

```bash
clasp mqtt --host localhost --port 1883
```

### With Topics

```bash
# Subscribe to specific topics
clasp mqtt --host localhost --port 1883 \
  --topic "sensors/#" \
  --topic "home/+"
```

### Desktop App

1. Click **Add Protocol**
2. Select **MQTT**
3. Enter broker host/port
4. Add topic subscriptions
5. Click **Start**

## Address Mapping

MQTT topics map to CLASP addresses with `/mqtt/` prefix:

```
MQTT: sensors/room1/temperature
CLASP: /mqtt/sensors/room1/temperature

MQTT: home/lights/living/brightness
CLASP: /mqtt/home/lights/living/brightness
```

## Receive MQTT Messages

```javascript
// All MQTT messages
client.on('/mqtt/**', (value, address) => {
  console.log(address, value);
});

// Specific topic pattern
client.on('/mqtt/sensors/*/temperature', (value, address) => {
  console.log(`Temperature: ${value}`);
});
```

## Publish to MQTT

```javascript
// Publish to MQTT topic
await client.set('/mqtt/home/lights/living/brightness', 80);

// Will publish to MQTT: home/lights/living/brightness = 80
```

## MQTT Wildcards

MQTT wildcards map to CLASP wildcards:

| MQTT | CLASP |
|------|-------|
| `sensors/+/temp` | `/mqtt/sensors/*/temp` |
| `sensors/#` | `/mqtt/sensors/**` |

## Retained Messages

MQTT retained messages become CLASP Params:

```bash
# Messages on these topics are treated as params (retained)
clasp mqtt --host localhost --retained-topics "config/#"
```

Non-retained messages are treated as Events.

## QoS Mapping

| MQTT QoS | CLASP QoS |
|----------|-----------|
| QoS 0 | Fire |
| QoS 1 | Confirm |
| QoS 2 | Commit |

## Authentication

```bash
clasp mqtt --host broker.example.com \
  --username myuser \
  --password mypassword
```

Or with environment variables:
```bash
MQTT_USERNAME=myuser MQTT_PASSWORD=mypassword clasp mqtt --host broker.example.com
```

## TLS

```bash
clasp mqtt --host broker.example.com --port 8883 --tls
```

With client certificate:
```bash
clasp mqtt --host broker.example.com --port 8883 \
  --tls \
  --ca-cert /path/to/ca.pem \
  --client-cert /path/to/client.pem \
  --client-key /path/to/client-key.pem
```

## JSON Payloads

MQTT messages with JSON payloads are parsed:

```javascript
// MQTT payload: {"temperature": 23.5, "humidity": 65}
client.on('/mqtt/sensors/room1', (value) => {
  console.log(value.temperature);  // 23.5
  console.log(value.humidity);     // 65
});
```

## Example: IoT Sensors

```javascript
// Subscribe to all sensors
client.on('/mqtt/sensors/**', (value, address) => {
  // Parse address to get location and type
  const parts = address.split('/');
  const location = parts[3];  // e.g., "room1"
  const sensor = parts[4];    // e.g., "temperature"

  console.log(`${location} ${sensor}: ${value}`);
});

// Control a device
await client.set('/mqtt/home/lights/living/state', 'ON');
```

## Example: Home Assistant

```bash
clasp mqtt --host homeassistant.local --port 1883 \
  --topic "homeassistant/#"
```

```javascript
// Listen to Home Assistant state changes
client.on('/mqtt/homeassistant/sensor/**', (value, address) => {
  console.log(address, value);
});

// Control Home Assistant devices
await client.set('/mqtt/homeassistant/light/living_room/set', {
  state: 'ON',
  brightness: 200
});
```

## Troubleshooting

### Can't connect to broker

- Check broker is running: `mosquitto_sub -h localhost -t '#'`
- Verify port (1883 unencrypted, 8883 TLS)
- Check firewall settings

### Messages not received

- Verify topic subscription matches publisher
- Check MQTT wildcards (`+` and `#`) are correct
- Test with `mosquitto_sub`

## Next Steps

- [Home Assistant Integration](../../integrations/home-assistant.md)
- [MQTT Bridge Reference](../../reference/bridges/mqtt.md)

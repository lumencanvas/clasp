# MQTT Bridge

Mapping between MQTT and CLASP.

## Overview

The MQTT bridge translates between MQTT messages and CLASP signals, enabling integration with IoT devices, home automation systems, and message brokers.

## Address Mapping

### MQTT to CLASP

MQTT topics become CLASP addresses with `/mqtt` prefix:

| MQTT Topic | CLASP Address |
|------------|---------------|
| `sensors/temperature` | `/mqtt/sensors/temperature` |
| `home/living/light/state` | `/mqtt/home/living/light/state` |
| `devices/sensor1/data` | `/mqtt/devices/sensor1/data` |

### CLASP to MQTT

CLASP addresses under `/mqtt/` are published to MQTT:

| CLASP Address | MQTT Topic |
|---------------|------------|
| `/mqtt/control/led` | `control/led` |
| `/mqtt/commands/restart` | `commands/restart` |

## Value Mapping

### MQTT to CLASP

| MQTT Payload | CLASP Value |
|--------------|-------------|
| JSON object | Map |
| JSON array | Array |
| JSON number | Int or Float |
| JSON string | String |
| JSON true/false | Bool |
| JSON null | Null |
| Raw bytes | Blob |
| Plain text | String |

### JSON Mode

With `--json` or `json: true`:

```
MQTT: sensors/data → {"temp": 23.5, "humidity": 65}
→ CLASP: /mqtt/sensors/data = { temp: 23.5, humidity: 65 }

CLASP: /mqtt/control/led = { state: "ON", brightness: 200 }
→ MQTT: control/led → {"state":"ON","brightness":200}
```

### Raw Mode

Without JSON mode:

```
MQTT: sensors/temp → "23.5"
→ CLASP: /mqtt/sensors/temp = "23.5" (string)
```

## Topic Wildcards

### MQTT Subscription Wildcards

- `+` - Single level: `devices/+/status` matches `devices/sensor1/status`
- `#` - Multi level: `sensors/#` matches `sensors/room1/temp`

```yaml
topics:
  - "sensors/#"           # All sensor data
  - "devices/+/status"    # Status of all devices
  - "home/+/+/state"      # State of all home devices
```

### CLASP Wildcard Subscriptions

```javascript
// Match all MQTT topics under sensors
client.on('/mqtt/sensors/**', (value, address) => {
  console.log(address, value);
});

// Match specific pattern
client.on('/mqtt/devices/*/status', (value, address) => {
  const device = address.split('/')[3];
  console.log(`Device ${device}: ${value}`);
});
```

## QoS Levels

### MQTT QoS

| Level | Description |
|-------|-------------|
| 0 | At most once (fire and forget) |
| 1 | At least once (acknowledged) |
| 2 | Exactly once (assured) |

Configure default QoS:

```yaml
mqtt:
  qos: 1  # Default QoS for subscriptions
  publish_qos: 1  # Default QoS for publishing
```

### Mapping to CLASP QoS

| MQTT QoS | CLASP QoS |
|----------|-----------|
| 0 | Fire |
| 1 | Confirm |
| 2 | Commit |

## Retained Messages

MQTT retained messages map to CLASP Param signal type:

```yaml
mqtt:
  retain: true  # Publish with retain flag
```

```javascript
// CLASP Param values publish with retain
client.set('/mqtt/state/current', 'active');

// CLASP Events publish without retain
client.emit('/mqtt/event/button', { pressed: true });
```

## Will Messages

Configure Last Will and Testament:

```yaml
mqtt:
  will:
    topic: "clasp/bridge/status"
    payload: "offline"
    qos: 1
    retain: true
```

## Configuration

### CLI

```bash
clasp mqtt --host localhost --port 1883 --topic "sensors/#"
```

### Configuration File

```yaml
mqtt:
  host: "localhost"
  port: 1883
  username: "user"
  password: "password"
  client_id: "clasp-bridge"

  tls:
    enabled: false
    ca: /path/to/ca.pem

  topics:
    - "sensors/#"
    - "devices/+/status"

  publish:
    qos: 1
    retain: false

  json: true

  will:
    topic: "clasp/status"
    payload: "offline"

clasp:
  router: "ws://localhost:7330"

mapping:
  prefix: "/mqtt"
  strip_prefix: true
```

### Rust API

```rust
use clasp_bridge::mqtt::{MqttBridge, MqttConfig};

let config = MqttConfig {
    host: "localhost".into(),
    port: 1883,
    username: Some("user".into()),
    password: Some("password".into()),
    topics: vec!["sensors/#".into()],
    json: true,
    ..Default::default()
};

let bridge = MqttBridge::new(client, config).await?;
```

## Home Assistant Integration

### Discovery

Home Assistant MQTT discovery:

```javascript
// Publish discovery config
await client.set('/mqtt/homeassistant/sensor/temp/config', {
  name: 'Temperature',
  state_topic: 'homeassistant/sensor/temp/state',
  unit_of_measurement: '°C',
  device_class: 'temperature'
});

// Publish state
await client.set('/mqtt/homeassistant/sensor/temp/state', 23.5);
```

### Topics

| Topic Pattern | Purpose |
|---------------|---------|
| `homeassistant/sensor/*/config` | Sensor discovery |
| `homeassistant/sensor/*/state` | Sensor state |
| `homeassistant/light/*/set` | Light commands |
| `homeassistant/switch/*/state` | Switch state |

## See Also

- [Add MQTT](../../how-to/connections/add-mqtt.md)
- [clasp mqtt CLI](../cli/clasp-mqtt.md)
- [Home Assistant Integration](../../integrations/home-assistant.md)
- [Home Automation](../../use-cases/home-automation.md)

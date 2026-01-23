# Migrating from MQTT

Guide for transitioning from MQTT to CLASP.

## Overview

MQTT and CLASP both use publish-subscribe patterns with hierarchical topics/addresses. This guide helps you migrate MQTT-based systems to CLASP while maintaining interoperability.

## Key Differences

| Feature | MQTT | CLASP |
|---------|------|-------|
| Topics/Addresses | Hierarchical | Hierarchical |
| Wildcards | +, # | *, ** |
| QoS Levels | 0, 1, 2 | Fire, Confirm, Commit |
| Retained | Per-message | Signal type based |
| Broker/Router | Required | Required |
| Protocol | MQTT | CLASP Binary |
| Transports | TCP, WebSocket | WS, QUIC, UDP, WebRTC |

## Topic Translation

### Wildcards

| MQTT | CLASP | Meaning |
|------|-------|---------|
| `+` | `*` | Single level |
| `#` | `**` | Multiple levels |

**MQTT:**
```
sensors/+/temperature
devices/#
```

**CLASP:**
```
/sensors/*/temperature
/devices/**
```

### Bridge Mapping

MQTT topics become CLASP addresses with `/mqtt` prefix:

```
MQTT:  sensors/room1/temp
CLASP: /mqtt/sensors/room1/temp
```

## QoS Mapping

| MQTT QoS | CLASP QoS | Behavior |
|----------|-----------|----------|
| 0 (At most once) | Fire | Best effort |
| 1 (At least once) | Confirm | Acknowledged |
| 2 (Exactly once) | Commit | Exactly once |

## Retained Messages

**MQTT**: Retained flag per message.

**CLASP**: Signal type determines retention.

| MQTT | CLASP |
|------|-------|
| `retain: true` | Param (default, retained) |
| `retain: false` | Event (ephemeral) |

```javascript
// Retained (like MQTT retain: true)
await client.set('/sensors/temp', 23.5);

// Not retained (like MQTT retain: false)
await client.emit('/events/button', { pressed: true });
```

## Migration Strategies

### Strategy 1: MQTT Bridge (Gradual)

Keep existing MQTT devices, add CLASP via bridge:

```
┌─────────────┐          ┌─────────────┐
│MQTT Devices │──MQTT───►│MQTT Broker  │
└─────────────┘          └──────┬──────┘
                                │
                         ┌──────▼──────┐
                         │ MQTT Bridge │
                         └──────┬──────┘
                                │
┌─────────────┐          ┌──────▼──────┐
│ CLASP App   │◄────────►│CLASP Router │
└─────────────┘          └─────────────┘
```

Configuration:
```yaml
mqtt:
  host: "localhost"
  port: 1883
  topics:
    - "sensors/#"
    - "devices/+/status"

clasp:
  router: "ws://localhost:7330"
```

### Strategy 2: Native Migration (Full)

Replace MQTT with native CLASP:

**Before (MQTT):**
```javascript
const mqtt = require('mqtt');

const client = mqtt.connect('mqtt://localhost:1883');

client.on('connect', () => {
  client.subscribe('sensors/#');
});

client.on('message', (topic, message) => {
  console.log(topic, message.toString());
});

client.publish('sensors/temp', '23.5', { retain: true });
```

**After (CLASP):**
```javascript
const { Clasp } = require('@clasp-to/core');

const client = await Clasp.connect('ws://localhost:7330');

client.on('/sensors/**', (value, address) => {
  console.log(address, value);
});

await client.set('/sensors/temp', 23.5);
```

## Feature Comparison

### Last Will → Connection Events

**MQTT**: Last Will and Testament (LWT).

**CLASP**: Connection events and cleanup.

```javascript
// CLASP approach
await client.set('/devices/mydevice/status', 'online');

client.on('disconnected', async () => {
  // Server-side cleanup or client reconnect handling
});
```

### Topic Filters → Pattern Subscriptions

Both support hierarchical subscriptions:

**MQTT:**
```javascript
client.subscribe('sensors/+/temperature');
client.subscribe('devices/#');
```

**CLASP:**
```javascript
client.on('/sensors/*/temperature', handler);
client.on('/devices/**', handler);
```

### Payload Format → Native Types

**MQTT**: Byte payload, format is application-defined.

**CLASP**: Native typed values.

```javascript
// MQTT: Manual serialization
client.publish('data', JSON.stringify({ temp: 23.5, humidity: 65 }));

// CLASP: Native objects
await client.set('/data', { temp: 23.5, humidity: 65 });
```

## Home Assistant Integration

### Current (MQTT)

Home Assistant native MQTT integration.

### With CLASP

CLASP bridge to MQTT broker:

```yaml
# CLASP subscribes to Home Assistant MQTT
mqtt:
  host: "localhost"
  topics:
    - "homeassistant/#"
```

CLASP clients can now access Home Assistant data:

```javascript
client.on('/mqtt/homeassistant/sensor/*/state', (value, address) => {
  const sensor = address.split('/')[4];
  console.log(`Sensor ${sensor}: ${value}`);
});

// Control devices
await client.set('/mqtt/homeassistant/light/living/set', {
  state: 'ON',
  brightness: 200
});
```

## IoT Device Migration

### ESP32/ESP8266

**Before (MQTT):**
```cpp
#include <PubSubClient.h>

void loop() {
  float temp = readTemp();
  client.publish("sensors/temp", String(temp).c_str());
}
```

**After (CLASP via HTTP):**
```cpp
#include <HTTPClient.h>

void loop() {
  float temp = readTemp();

  HTTPClient http;
  http.begin("http://clasp-server:3000/api/state/sensors/temp");
  http.addHeader("Content-Type", "application/json");
  http.PUT("{\"value\":" + String(temp) + "}");
  http.end();
}
```

**After (Native CLASP):**
```cpp
#include <CLASP.h>

void loop() {
  float temp = readTemp();
  clasp.set("/sensors/temp", temp);
}
```

## Coexistence

Run MQTT and CLASP together:

```javascript
// Access both MQTT devices and native CLASP clients
client.on('/mqtt/sensors/**', handleMqttSensors);
client.on('/sensors/**', handleNativeSensors);
```

## Checklist

- [ ] Identify all MQTT topics in use
- [ ] Set up CLASP router
- [ ] Configure MQTT bridge
- [ ] Test existing MQTT devices work through bridge
- [ ] Migrate applications to native CLASP
- [ ] Update IoT devices if needed
- [ ] Consider removing MQTT broker when complete

## Benefits After Migration

1. **Lower latency**: QUIC/UDP transport options
2. **Richer data types**: Native objects, not just bytes
3. **Better subscriptions**: Get current state on subscribe
4. **Multiple transports**: WebSocket, QUIC, UDP, WebRTC
5. **Browser support**: Native WebSocket in browsers
6. **P2P capability**: WebRTC for direct connections

## See Also

- [MQTT Bridge Reference](../../reference/bridges/mqtt.md)
- [Add MQTT Connection](../../how-to/connections/add-mqtt.md)
- [Home Assistant Integration](../../integrations/home-assistant.md)

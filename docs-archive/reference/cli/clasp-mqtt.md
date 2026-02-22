---
title: "clasp mqtt"
description: "Start an MQTT bridge."
section: reference
order: 3
---
# clasp mqtt

Start an MQTT bridge.

## Synopsis

```
clasp mqtt [OPTIONS]
```

## Description

Creates a bridge between MQTT and CLASP. Subscribes to MQTT topics and publishes them as CLASP signals, and vice versa.

## Options

### MQTT Connection

```
--host <HOST>
    MQTT broker hostname [default: localhost]

--port <PORT>
    MQTT broker port [default: 1883]

--username <USER>
    MQTT username

--password <PASS>
    MQTT password

--client-id <ID>
    MQTT client ID [default: clasp-mqtt-bridge]

--tls
    Use TLS connection

--ca <PATH>
    CA certificate file for TLS
```

### Topics

```
--topic <TOPIC>
    MQTT topic to subscribe (can be repeated)
    Supports wildcards: sensors/#, devices/+/status

--publish-topic <TOPIC>
    MQTT topic pattern for publishing
    Use {address} placeholder: clasp/{address}
```

### CLASP Connection

```
--router <URL>
    CLASP router URL [default: ws://localhost:7330]

--token <TOKEN>
    Authentication token

--name <NAME>
    Client name [default: mqtt-bridge]
```

### Address Mapping

```
--prefix <PREFIX>
    CLASP address prefix [default: /mqtt]

--strip-prefix
    Strip prefix when publishing to MQTT
```

### Options

```
--qos <LEVEL>
    MQTT QoS level: 0, 1, 2 [default: 1]

--retain
    Retain MQTT messages

--json
    Parse/encode values as JSON
```

### Other

```
-c, --config <PATH>
    Configuration file

-v, --verbose
    Verbose output

-h, --help
    Print help
```

## Examples

### Basic Bridge

```bash
clasp mqtt --host localhost --topic "sensors/#"
```

### With Authentication

```bash
clasp mqtt \
  --host mqtt.example.com \
  --username user \
  --password secret \
  --topic "home/#"
```

### TLS Connection

```bash
clasp mqtt \
  --host mqtt.example.com \
  --port 8883 \
  --tls \
  --ca /path/to/ca.pem \
  --topic "sensors/#"
```

### Multiple Topics

```bash
clasp mqtt \
  --host localhost \
  --topic "sensors/#" \
  --topic "devices/+/status" \
  --topic "commands/#"
```

### Home Assistant

```bash
clasp mqtt \
  --host localhost \
  --topic "homeassistant/#" \
  --prefix /ha
```

### Bidirectional with Publish Pattern

```bash
clasp mqtt \
  --host localhost \
  --topic "sensors/#" \
  --publish-topic "clasp/{address}"
```

## Topic to Address Translation

### MQTT to CLASP

```
MQTT: sensors/temperature → payload: 23.5
→ CLASP: /mqtt/sensors/temperature = 23.5

MQTT: home/living/light/state → payload: "ON"
→ CLASP: /mqtt/home/living/light/state = "ON"
```

### CLASP to MQTT

```
CLASP: /mqtt/control/led = 255
→ MQTT: control/led → payload: 255

With --publish-topic "clasp/{address}":
CLASP: /mqtt/control/led = 255
→ MQTT: clasp/control/led → payload: 255
```

### JSON Payloads

With `--json`:

```
MQTT: sensors/data → payload: {"temp": 23.5, "humidity": 65}
→ CLASP: /mqtt/sensors/data = { temp: 23.5, humidity: 65 }
```

## Configuration File

```yaml
# mqtt-bridge.yaml
mqtt:
  host: "mqtt.example.com"
  port: 8883
  username: "user"
  password: "secret"
  client_id: "clasp-bridge"
  tls:
    enabled: true
    ca: /path/to/ca.pem

  topics:
    - "sensors/#"
    - "devices/+/status"

  publish:
    topic_pattern: "clasp/{address}"
    qos: 1
    retain: false

  json: true

clasp:
  router: "ws://localhost:7330"
  name: "mqtt-bridge"

mapping:
  prefix: "/mqtt"
  strip_prefix: true
```

## Wildcards

### MQTT Wildcards

- `+` - Single level: `devices/+/status` matches `devices/sensor1/status`
- `#` - Multi level: `sensors/#` matches `sensors/room1/temp`

### CLASP Wildcards

- `*` - Single segment: `/mqtt/devices/*/status`
- `**` - Multiple segments: `/mqtt/sensors/**`

## See Also

- [Add MQTT](../../how-to/connections/add-mqtt.md)
- [Home Assistant Integration](../../integrations/home-assistant.md)
- [MQTT Bridge Reference](../bridges/mqtt.md)

# Bridge Configuration

Configuration reference for CLASP protocol bridges.

## Common Options

All bridges share these configuration options:

```yaml
# Connection to CLASP router
clasp:
  router: "ws://localhost:7330"
  token: null
  name: "bridge-name"
  auto_reconnect: true
  reconnect_interval: 1000

# Address mapping
mapping:
  prefix: "/bridge"
  strip_prefix: false
  custom: {}

# Filtering
filter:
  include: []
  exclude: []
```

## OSC Bridge

```yaml
osc:
  # Receive settings
  port: 8000
  bind: "0.0.0.0"

  # Send settings (optional)
  targets:
    - "192.168.1.100:9000"

  # Bidirectional mode
  bidirectional: true

clasp:
  router: "ws://localhost:7330"

mapping:
  prefix: "/osc"
  strip_prefix: true
  custom:
    "/1/fader*": "/control/fader*"
```

## MIDI Bridge

```yaml
midi:
  # Device selection
  device: "Launchpad X"
  # Or separate:
  # input: "Controller In"
  # output: "Synth Out"

  # Virtual port (macOS/Linux)
  virtual: null

  # Channel filter (1-16, null = all)
  channels: null

  # Include clock/sysex
  include_clock: false
  include_sysex: false

clasp:
  router: "ws://localhost:7330"

mapping:
  prefix: "/midi"
  device_name: null  # Auto-detect
```

## Art-Net Bridge

```yaml
artnet:
  # Network settings
  bind: "0.0.0.0:6454"
  broadcast: "255.255.255.255:6454"

  # Or unicast targets
  # targets:
  #   - "192.168.1.50:6454"

  # Universes to handle
  universes: [0, 1, 2, 3]

  # Direction
  input_enabled: true
  output_enabled: true

  # Timing
  refresh_rate: 44

clasp:
  router: "ws://localhost:7330"

mapping:
  prefix: "/artnet"
```

## DMX Bridge

```yaml
dmx:
  # Interface type
  interface: ftdi  # ftdi, serial, enttec, open_dmx

  # For FTDI
  device_index: 0

  # For serial
  # port: "/dev/ttyUSB0"
  # baud: 250000

  universe: 0
  refresh_rate: 44

clasp:
  router: "ws://localhost:7330"

mapping:
  prefix: "/dmx"
```

## MQTT Bridge

```yaml
mqtt:
  # Connection
  host: "localhost"
  port: 1883
  username: null
  password: null
  client_id: "clasp-mqtt"

  # TLS
  tls:
    enabled: false
    ca: null

  # Topics to subscribe
  topics:
    - "sensors/#"
    - "devices/+/status"

  # Publishing
  publish:
    qos: 1
    retain: false
    topic_pattern: null  # Use default

  # JSON mode
  json: true

  # Will message
  will:
    topic: "clasp/status"
    payload: "offline"
    qos: 1
    retain: true

clasp:
  router: "ws://localhost:7330"

mapping:
  prefix: "/mqtt"
  strip_prefix: true
```

## sACN Bridge

```yaml
sacn:
  # Source info
  source_name: "CLASP Bridge"
  cid: null  # Auto-generate UUID

  # Universes
  universes: [1, 2, 3, 4]

  # Priority (0-200)
  priority: 100

  # Multicast (default) or unicast
  multicast: true
  # targets: []

  # Timing
  refresh_rate: 44

  # Sync
  sync:
    enabled: false
    universe: 65535

clasp:
  router: "ws://localhost:7330"

mapping:
  prefix: "/sacn"
```

## HTTP Bridge

```yaml
http:
  # Server settings
  port: 3000
  bind: "0.0.0.0"

  # TLS
  tls:
    enabled: false
    cert: null
    key: null

  # CORS
  cors:
    enabled: true
    origins: ["*"]
    methods: ["GET", "POST", "PUT", "DELETE"]
    headers: ["Content-Type", "X-API-Key"]

  # Authentication
  auth:
    api_key: null
    basic: null  # "user:password"
    jwt_secret: null

  # Features
  features:
    sse: true
    websocket: false

  # URL prefix
  prefix: "/api"

clasp:
  router: "ws://localhost:7330"
```

## Address Mapping

### Prefix

Add prefix to incoming addresses:

```yaml
mapping:
  prefix: "/osc"

# /1/fader1 → /osc/1/fader1
```

### Strip Prefix

Remove prefix when sending outbound:

```yaml
mapping:
  prefix: "/osc"
  strip_prefix: true

# /osc/1/fader1 → /1/fader1
```

### Custom Mapping

Define specific address translations:

```yaml
mapping:
  custom:
    "/input/fader*": "/control/fader*"
    "/input/xy": "/control/position"
```

## Filtering

### Include Filter

Only process matching addresses:

```yaml
filter:
  include:
    - "/sensors/**"
    - "/control/*"
```

### Exclude Filter

Ignore matching addresses:

```yaml
filter:
  exclude:
    - "/**/debug"
    - "/internal/**"
```

## Environment Variables

```bash
# Common
CLASP_ROUTER_URL=ws://localhost:7330
CLASP_TOKEN=your-token

# OSC
CLASP_OSC_PORT=8000
CLASP_OSC_TARGET=192.168.1.100:9000

# MIDI
CLASP_MIDI_DEVICE="Launchpad X"

# MQTT
CLASP_MQTT_HOST=localhost
CLASP_MQTT_USERNAME=user
CLASP_MQTT_PASSWORD=pass
```

## See Also

- [Router Configuration](router-config.md)
- [Bridge Reference](../bridges/)
- [CLI Commands](../cli/)

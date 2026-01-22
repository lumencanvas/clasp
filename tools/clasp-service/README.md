# clasp-service

CLASP Bridge Service - A JSON-RPC service for managing protocol bridges.

## Overview

`clasp-service` is a headless service designed to be spawned by the CLASP desktop app (Electron). It communicates via stdin/stdout using JSON messages, allowing the desktop app to create and manage protocol bridges.

## Architecture

```
┌─────────────────────┐     stdin/stdout      ┌──────────────────┐
│   Electron App      │ ◄──────────────────► │  clasp-service   │
│   (Desktop UI)      │      JSON-RPC         │  (Bridge Manager)│
└─────────────────────┘                       └──────────────────┘
                                                      │
                                      ┌───────────────┼───────────────┐
                                      ▼               ▼               ▼
                                 ┌─────────┐    ┌─────────┐    ┌─────────┐
                                 │OSC Bridge│   │MIDI Bridge│  │MQTT Bridge│
                                 └─────────┘    └─────────┘    └─────────┘
```

## Features

- **Bridge Management**: Create, delete, and list protocol bridges
- **Health Monitoring**: Get diagnostics and health status for all bridges
- **Signal Routing**: Send signals through bridges programmatically
- **Event Streaming**: Receive real-time events from bridges

## Supported Protocols

Enabled via Cargo features:

| Protocol | Feature Flag | Description |
|----------|--------------|-------------|
| OSC | `osc` | Open Sound Control |
| MIDI | `midi` | Musical Instrument Digital Interface |
| Art-Net | `artnet` | DMX over Ethernet |
| DMX | `dmx` | Direct DMX output |
| MQTT | `mqtt` | Message Queue Telemetry Transport |
| WebSocket | `websocket` | WebSocket client/server |
| HTTP | `http` | HTTP REST API server |

## Building

```bash
# Build with all features
cargo build -p clasp-service --release --all-features

# Build with specific features
cargo build -p clasp-service --release --features "osc,midi,mqtt"
```

## Usage

The service is typically launched by the Electron app, but can be run manually for testing:

```bash
./clasp-service
```

Then send JSON commands via stdin:

```json
{"type": "create_bridge", "source": "osc", "source_addr": "0.0.0.0:8000", "target": "midi", "target_addr": "default"}
```

## JSON-RPC Commands

### create_bridge
Create a new protocol bridge.

```json
{
  "type": "create_bridge",
  "id": "optional-custom-id",
  "source": "osc",
  "source_addr": "0.0.0.0:8000",
  "target": "midi",
  "target_addr": "default",
  "config": {}
}
```

### delete_bridge
Remove an existing bridge.

```json
{
  "type": "delete_bridge",
  "id": "bridge-id"
}
```

### list_bridges
List all active bridges.

```json
{"type": "list_bridges"}
```

### get_diagnostics
Get detailed diagnostics for a specific bridge or all bridges.

```json
{"type": "get_diagnostics", "bridge_id": "optional-id"}
```

### health_check
Get overall service health status.

```json
{"type": "health_check"}
```

### send_signal
Send a signal through a specific bridge.

```json
{
  "type": "send_signal",
  "bridge_id": "bridge-id",
  "address": "/control/fader1",
  "value": 0.75
}
```

### ping
Health ping.

```json
{"type": "ping"}
```

### shutdown
Gracefully shutdown the service.

```json
{"type": "shutdown"}
```

## Response Format

All responses follow this format:

```json
{"type": "ok", "data": {...}}
```

Or for errors:

```json
{"type": "error", "message": "Error description"}
```

## Events

The service emits events for bridge activity:

### signal
Received signal from a bridge.

```json
{
  "type": "signal",
  "bridge_id": "bridge-id",
  "address": "/osc/fader1",
  "value": 0.5
}
```

### bridge_event
Bridge status changes.

```json
{
  "type": "bridge_event",
  "bridge_id": "bridge-id",
  "event": "connected|disconnected|error",
  "data": "optional details"
}
```

## Testing

Run the integration tests:

```bash
cargo test -p clasp-service
```

## License

MIT or Apache-2.0

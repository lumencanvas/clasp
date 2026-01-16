# CLASP Implementation Tasks

Quick-reference task list. See HANDOFF.md for detailed specifications.

## Immediate Tasks

### Rebrand (Priority: HIGH)
- [ ] Rename project folder to `clasp`
- [ ] Update all `signalflow` → `clasp` in:
  - [ ] Cargo.toml (workspace)
  - [ ] All crate Cargo.tomls
  - [ ] apps/bridge/package.json
  - [ ] All Rust source files
  - [ ] All JS source files
  - [ ] README.md
- [ ] Update app titlebar: "SIGNALFLOW BRIDGE" → "CLASP"
- [ ] Replace logo in app
- [ ] Generate app icons from logo.svg

### Protocol: WebSocket (Priority: HIGH)
- [ ] Create `crates/clasp-bridge/src/websocket.rs`
- [ ] Implement client mode (connect to WS server)
- [ ] Implement server mode (accept WS connections)
- [ ] Add JSON/MsgPack message format option
- [ ] Add to bridge service
- [ ] Add to Electron app UI

### Protocol: MQTT (Priority: HIGH)
- [ ] Add `rumqttc` dependency
- [ ] Create `crates/clasp-bridge/src/mqtt.rs`
- [ ] Implement broker connection
- [ ] Implement topic subscription
- [ ] Implement publish
- [ ] Add to bridge service
- [ ] Add to Electron app UI

### Protocol: Socket.IO (Priority: MEDIUM)
- [ ] Add `rust-socketio` dependency
- [ ] Create `crates/clasp-bridge/src/socketio.rs`
- [ ] Implement event listening
- [ ] Implement event emitting
- [ ] Add to bridge service
- [ ] Add to Electron app UI

### Protocol: HTTP/REST (Priority: HIGH)
- [ ] Add `axum` dependency
- [ ] Create `crates/clasp-bridge/src/http/mod.rs`
- [ ] Implement REST server mode
  - [ ] Dynamic endpoint registration
  - [ ] Path params (`:id`)
  - [ ] Query params
  - [ ] Request body parsing
  - [ ] Response generation
- [ ] Implement REST client mode
  - [ ] Template-based requests
  - [ ] Response mapping
- [ ] Add API Designer tab to Electron app
- [ ] OpenAPI spec generation

### Documentation Site (Priority: MEDIUM)
- [ ] Set up VitePress in `docs/`
- [ ] Write getting started guide
- [ ] Document each protocol
- [ ] Create example walkthroughs
- [ ] Deploy to GitHub Pages

## File Changes Needed

### Rust Crates to Rename
```
crates/signalflow-core/      → crates/clasp-core/
crates/signalflow-transport/ → crates/clasp-transport/
crates/signalflow-discovery/ → crates/clasp-discovery/
crates/signalflow-bridge/    → crates/clasp-bridge/
crates/signalflow-router/    → crates/clasp-router/
crates/signalflow-client/    → crates/clasp-client/
crates/signalflow-embedded/  → crates/clasp-embedded/
crates/signalflow-wasm/      → crates/clasp-wasm/
tools/sf-bridge-service/     → tools/clasp-service/
```

### New Files to Create
```
crates/clasp-bridge/src/mqtt.rs
crates/clasp-bridge/src/websocket.rs
crates/clasp-bridge/src/socketio.rs
crates/clasp-bridge/src/http/mod.rs
crates/clasp-bridge/src/http/server.rs
crates/clasp-bridge/src/http/client.rs
apps/bridge/src/api-designer.js
tools/clasp-cli/
```

### Dependencies to Add (Cargo.toml)
```toml
# MQTT
rumqttc = "0.24"

# Socket.IO
rust_socketio = "0.5"

# HTTP Server
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Templating (for REST requests)
handlebars = "5"
```

## UI Updates Needed

### Protocol Dropdowns
Add to all protocol selectors:
- MQTT
- WebSocket
- Socket.IO
- HTTP (Server)
- HTTP (Client)

### New Modal: MQTT Config
```
Broker URL: [mqtt://localhost:1883]
Client ID: [clasp-bridge-1]
Username: [optional]
Password: [optional]
Topics: [sensor/+/data, home/#]
QoS: [0/1/2]
```

### New Modal: WebSocket Config
```
Mode: [Client / Server]
URL: [ws://localhost:8080]
Path: [/ws] (server only)
Format: [JSON / MsgPack / Raw]
```

### New Modal: Socket.IO Config
```
Server URL: [http://localhost:3000]
Namespace: [/]
Events: [update, message, data]
Auth: [JSON payload]
```

### New Tab: API Designer
- Endpoint list (left panel)
- Endpoint editor (right panel)
  - Method: GET/POST/PUT/DELETE/PATCH
  - Path: /api/resource/:id
  - Parameters table
  - Request body schema
  - Response schema
  - CLASP address mapping
  - Transform config
- Test panel (bottom)
- Export OpenAPI button

## Quick Commands

```bash
# Rename directories
mv crates/signalflow-core crates/clasp-core
mv crates/signalflow-transport crates/clasp-transport
# ... etc

# Find/replace in files
find . -name "*.rs" -exec sed -i '' 's/signalflow/clasp/g' {} \;
find . -name "*.toml" -exec sed -i '' 's/signalflow/clasp/g' {} \;
find . -name "*.json" -exec sed -i '' 's/signalflow/clasp/g' {} \;
find . -name "*.js" -exec sed -i '' 's/signalflow/clasp/g' {} \;
find . -name "*.html" -exec sed -i '' 's/signalflow/clasp/g' {} \;
```

## Testing Checklist

After implementing each protocol:
- [ ] Unit tests pass
- [ ] Integration test with real service
- [ ] Bridge creates successfully in app
- [ ] Mapping works end-to-end
- [ ] Signal monitor shows traffic
- [ ] Learn mode captures addresses

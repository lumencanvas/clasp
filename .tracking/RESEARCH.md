# Research Notes

## Rust Crates Evaluation

### MessagePack
| Crate | Stars | Notes |
|-------|-------|-------|
| rmp-serde | 800+ | Serde integration, most popular |
| msgpack-rust | - | Lower level |
| **Decision**: rmp-serde

### OSC
| Crate | Stars | Notes |
|-------|-------|-------|
| rosc | 100+ | Pure Rust, well maintained |
| nanoosc | - | Minimal |
| **Decision**: rosc

### MIDI
| Crate | Stars | Notes |
|-------|-------|-------|
| midir | 500+ | Cross-platform I/O |
| midi-msg | - | Message parsing |
| wmidi | - | Type-safe messages |
| **Decision**: midir + midi-msg

### Art-Net
| Crate | Stars | Notes |
|-------|-------|-------|
| artnet_protocol | 30+ | Basic implementation |
| **Decision**: artnet_protocol (may need to extend)

### mDNS
| Crate | Stars | Notes |
|-------|-------|-------|
| mdns-sd | 100+ | Service discovery |
| mdns | - | Older |
| **Decision**: mdns-sd

### WebSocket
| Crate | Stars | Notes |
|-------|-------|-------|
| tokio-tungstenite | 1500+ | Async, mature |
| async-tungstenite | - | Alternative |
| **Decision**: tokio-tungstenite

### Async Runtime
| Crate | Notes |
|-------|-------|
| tokio | Standard, full-featured |
| async-std | Alternative |
| **Decision**: tokio

---

## Electron Research

### IPC Patterns for High-Throughput
- Use `ipcRenderer.send` for fire-and-forget (streams)
- Use `ipcRenderer.invoke` for request/response (params)
- Consider SharedArrayBuffer for very high rates
- Can use native addon to bypass IPC entirely

### Native Addon Options
| Tool | Notes |
|------|-------|
| napi-rs | Rust → Node.js, modern |
| node-bindgen | Alternative |
| neon | Older, less maintained |
| **Decision**: napi-rs

### Electron + Rust Integration
Two approaches:
1. **Sidecar**: Rust binary as child process, communicate via stdio/IPC
2. **Native addon**: Compile Rust as .node addon

**Decision**: Native addon via napi-rs for performance

---

## Protocol Research

### MIDI 2.0 Support in Rust
- No mature crate yet
- May need to implement UMP parsing ourselves
- Consider contributing to midi-msg or creating new crate

### sACN/E1.31
| Crate | Notes |
|-------|-------|
| sacn | Basic, unmaintained |
| **Decision**: May need custom implementation

### Serial/DMX USB
| Crate | Notes |
|-------|-------|
| serialport | Cross-platform |
| **Decision**: serialport for DMX interfaces (Enttec, DMXKing)

---

## Web Research Queue
- [ ] Latest Electron best practices 2026
- [ ] napi-rs performance benchmarks
- [ ] WASM threads/SharedArrayBuffer status
- [ ] WebRTC DataChannel in pure Rust


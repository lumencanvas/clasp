# Architecture Decisions Record

## ADR-001: Core Library Language
**Date**: 2026-01-15
**Status**: Accepted

### Context
Need a language for the core SignalFlow library that:
- Compiles to WASM for browser support
- Has good FFI for language bindings
- Is performant for real-time signal processing
- Has ecosystem support for protocols (OSC, MIDI, Art-Net)

### Decision
**Rust**

### Rationale
- First-class WASM target
- Excellent FFI via cbindgen, PyO3, wasm-bindgen
- Zero-cost abstractions for performance
- Rich ecosystem: rosc, midir, artnet_protocol, mdns-sd
- Memory safety without GC

---

## ADR-002: Desktop App Framework
**Date**: 2026-01-15
**Status**: Accepted

### Context
Need cross-platform desktop app for bridge UI.

### Decision
**Electron** (not Tauri)

### Rationale
- User preference
- Mature ecosystem
- Consistent behavior across platforms
- Rich debugging tools
- Can use native Node.js addons for Rust integration

### Trade-offs
- Larger bundle size (~150MB vs ~10MB)
- Higher memory usage
- Includes Chromium

---

## ADR-003: Serialization Format
**Date**: 2026-01-15
**Status**: Accepted (from spec)

### Context
Need efficient serialization for protocol messages.

### Decision
**MessagePack**

### Rationale
- Per SF/2 spec
- Smaller than JSON
- Self-describing
- Implementations in all target languages
- Good for embedded (msgpack-c is ~50KB)

---

## ADR-004: Monorepo Structure
**Date**: 2026-01-15
**Status**: Accepted

### Decision
Single monorepo with:
- `/crates` - Rust libraries
- `/bindings` - Language bindings
- `/apps` - Applications
- `/tools` - CLI tools
- `/docs` - Documentation

### Rationale
- Atomic commits across related changes
- Easier CI/CD
- Shared tooling configuration
- Single source of truth for protocol

---

## ADR-005: JS Binding Strategy
**Date**: 2026-01-15
**Status**: Accepted

### Decision
Dual approach:
1. **WASM** - For browser via wasm-bindgen
2. **Native addon** - For Node.js/Electron via napi-rs

### Rationale
- WASM works in browser but has limitations (no raw sockets)
- Native addon in Electron gives full system access
- Same Rust core, different compilation targets


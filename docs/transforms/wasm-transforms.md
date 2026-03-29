---
title: WASM Transforms
description: Custom signal transforms via LensVM WebAssembly modules
order: 1
---

# WASM Transforms

CLASP supports custom signal transforms written as WebAssembly modules using the LensVM protocol. This lets you author transforms in Rust (or any language that compiles to `wasm32-unknown-unknown`) and load them at runtime without modifying CLASP source code.

## Why WASM Transforms

CLASP ships 18+ built-in transform types (scale, clamp, expression, curve, quantize, etc.). These cover common use cases, but some signal processing needs are domain-specific:

- Butterworth filters for sensor data
- PID controllers for motor feedback loops
- Custom easing functions for lighting automation
- Domain-specific normalization for medical devices

WASM transforms give you unlimited extensibility with sandboxed execution. A WASM module cannot crash the bridge, access the filesystem, or interfere with other transforms.

## How It Works

A WASM transform is a compiled `.wasm` file that implements the LensVM protocol. The CLASP host loads the module, feeds signal values through it, and reads the transformed output.

```
Signal In --> [WASM Module] --> Signal Out
               |
               set_param(config)
               next() --> input value
               transform() --> output value
```

Each module can optionally implement an `inverse()` function for bidirectional transforms (e.g., encoding/decoding).

## Using WASM Transforms

### In the Bridge UI

1. Create or edit a signal route
2. Set the transform type to "WASM"
3. Upload a `.wasm` file or select from the built-in library
4. Configure parameters (JSON) for the module
5. Use the preview to verify the transform works

### In Rust (Server-Side)

Enable the `lens` feature on `clasp-bridge`:

```toml
[dependencies]
clasp-bridge = { version = "4.3", features = ["lens"] }
```

Then use the `Transform::Wasm` variant:

```rust
use clasp_bridge::Transform;
use serde_json::json;

let transform = Transform::Wasm {
    module_id: "scale-lens".to_string(),
    params: Some(json!({"scale_factor": 2.0, "offset": 0.5})),
};
```

Or use `clasp-lens` directly for lower-level control:

```rust
use clasp_lens::LensHost;
use serde_json::json;

// Load a WASM module
let wasm_bytes = std::fs::read("my-transform.wasm")?;
let mut host = LensHost::new(&wasm_bytes)?;

// Configure parameters
host.set_params(json!({"scale_factor": 2.0, "offset": 0.5}));

// Transform a value
let input = json!({"value": 0.25});
let output = host.transform(&input)?;
// output = {"value": 1.0}  (0.25 * 2.0 + 0.5)

// Inverse (if supported)
if host.has_inverse() {
    let restored = host.inverse(&output)?;
    // restored = {"value": 0.25}
}
```

## LensVM Protocol

WASM modules must export these functions:

| Export | Signature | Required | Description |
|--------|-----------|----------|-------------|
| `alloc` | `(i64) -> i32` | Yes | Allocate N bytes in WASM memory, return pointer |
| `transform` | `() -> i32` | Yes | Run forward transform, return pointer to result |
| `inverse` | `() -> i32` | No | Run inverse transform, return pointer to result |
| `set_param` | `(i32) -> i32` | No | Receive configuration, return pointer to status |

And import this function from the `"lens"` module:

| Import | Signature | Description |
|--------|-----------|-------------|
| `lens::next` | `() -> i32` | Pull the next input item from the host |

### Transport Buffer Format

Data crosses the WASM boundary as a transport buffer:

```
[TypeId: i8] [Length: u32 LE] [Payload: bytes]

TypeId values:
  -1 = error (payload is UTF-8 error string)
   0 = nil (no length/payload)
   1 = JSON item (payload is JSON bytes)
 127 = end of stream (no length/payload)
```

The host encodes input values as JSON items and writes them into WASM memory via `alloc()`. The module reads them via `next()`, processes them, and returns a transport buffer pointer from `transform()`.

## Performance

WASM modules are compiled once and cached. Each transform call instantiates the module (fast, sub-millisecond) and runs the transform function. For high-frequency signal paths (>1000 Hz), the per-call overhead is typically under 100 microseconds.

Built-in transforms are still faster for simple operations (scale, clamp, etc.) since they avoid the WASM boundary overhead. Use WASM transforms for operations that are too complex for the expression engine or need external library code.

## Feature Flag

WASM transform support is behind the `lens` Cargo feature flag. It is not included in the default feature set to avoid pulling in the `wasmtime` runtime (~50MB) for deployments that don't need custom transforms.

```toml
# Enable WASM transforms
clasp-bridge = { version = "4.3", features = ["lens"] }
```

The bridge app (Electron) uses the browser's built-in WebAssembly API for client-side preview, which has no additional dependencies.

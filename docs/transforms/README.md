---
title: Transforms
description: Signal value transformation system
order: 0
---

# Transforms

CLASP transforms modify signal values as they flow through routes. Every signal route can have a transform applied between its source and target endpoints.

## Built-in Transforms

18 transform types are available out of the box, with no additional dependencies:

| Category | Types |
|----------|-------|
| Passthrough | direct |
| Arithmetic | scale, invert, negate, modulo, power |
| Limiting | clamp, deadzone, threshold, gate, trigger, toggle |
| Smoothing | smooth (EMA), quantize, round |
| Curves | linear, ease-in, ease-out, ease-in-out, exponential, logarithmic |
| Custom | expression (math), javascript (code) |

These cover the vast majority of signal processing needs for creative applications.

## WASM Transforms

For operations too complex for the built-in types, CLASP supports custom transforms written as WebAssembly modules using the LensVM protocol.

WASM transforms run in a sandboxed runtime and cannot crash the bridge or access the filesystem. They are loaded at runtime without restarting the bridge.

Three bundled lenses ship with CLASP:

| Lens | Parameters | Use Case |
|------|-----------|----------|
| lowpass | `alpha: 0.0-1.0` | Smooth noisy sensor data |
| hysteresis | `low, high` | Debounce jittery signals near a threshold |
| moving-average | `window: N` | Average over N recent samples |

See [WASM Transforms](wasm-transforms.md) for usage details and [Authoring Lenses](authoring-lenses.md) for writing your own.

## Transforms in the Bridge App

In the bridge UI, select a transform type when creating or editing a signal route. The preview shows the transform formula, and for WASM transforms, you can configure parameters as JSON.

## Transforms in Rust

The `clasp-bridge` crate provides the `Transform` enum with all built-in types. Enable the `lens` feature for WASM support:

```toml
clasp-bridge = { version = "4.3", features = ["lens"] }
```

The `clasp-lens` crate provides the `LensHost` for direct WASM module interaction.

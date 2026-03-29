# Bundled CLASP Lenses

Pre-built LensVM WASM transform modules for common signal processing operations. These lenses can be loaded into any CLASP bridge via the WASM transform type.

## Available Lenses

| Lens | Parameters | Description |
|------|-----------|-------------|
| **lowpass** | `alpha: 0.0-1.0` | First-order IIR low-pass filter. Higher alpha = more smoothing. |
| **hysteresis** | `low: f64, high: f64` | Schmitt trigger. Outputs 1.0 above high, 0.0 below low, holds between. |
| **moving-average** | `window: usize` | Simple moving average over N recent samples. |

## Building

Each lens is a standalone Rust crate that compiles to `wasm32-unknown-unknown`:

```bash
# Build a single lens
cd lenses/lowpass
cargo build --target wasm32-unknown-unknown --release

# Output: target/wasm32-unknown-unknown/release/lens_lowpass.wasm
```

Build all lenses:

```bash
for dir in lenses/*/; do
  (cd "$dir" && cargo build --target wasm32-unknown-unknown --release)
done
```

Requires: `rustup target add wasm32-unknown-unknown`

## Writing Your Own

See `docs/transforms/authoring-lenses.md` for a guide on writing custom lenses.

Each lens must export `alloc` and `transform`, and import `next` from the `"lens"` module. Optional exports: `inverse` (for bidirectional transforms) and `set_param` (for configurable parameters).

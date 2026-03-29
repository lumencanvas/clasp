# test-lens-scale

Minimal LensVM-compatible WASM module used as a test fixture for `clasp-lens`.

Implements: `output = input * scale_factor + offset`

## Rebuilding

```bash
cd crates/clasp-lens/tests/test-lens-scale
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/test_lens_scale.wasm ../fixtures/scale_lens.wasm
```

Requires: `rustup target add wasm32-unknown-unknown`

## Protocol

Exports: `alloc`, `transform`, `inverse`, `set_param`
Imports: `lens::next`

Default params (when `set_param` is not called): `scale_factor=1.0, offset=0.0` (identity transform).

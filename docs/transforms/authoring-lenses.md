---
title: Authoring Lenses
description: How to write custom WASM transforms using Rust
order: 2
---

# Authoring Lenses

This guide walks through creating a custom WASM transform module from scratch in Rust. The resulting `.wasm` file can be loaded into any CLASP bridge.

## Prerequisites

- Rust toolchain with the WASM target: `rustup target add wasm32-unknown-unknown`
- Basic familiarity with Rust

## Project Setup

Create a new Rust library crate:

```bash
cargo init --lib my-lens
cd my-lens
```

Edit `Cargo.toml`:

```toml
[package]
name = "my-lens"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = "z"
lto = true
```

The `cdylib` crate type produces a standalone `.wasm` file. The release profile minimizes binary size (typical lenses are 30-100KB).

## Minimal Implementation

A lens module needs four things:
1. Import `next()` from the `"lens"` module
2. Export `alloc()` for memory management
3. Export `transform()` for the forward transform
4. Handle the transport buffer format for input/output

Here is a complete example that scales a value by a configurable factor:

```rust
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

// Transport buffer type IDs
const TYPE_JSON: u8 = 1;
const TYPE_EOS: u8 = 127;

#[derive(Serialize, Deserialize)]
struct Params {
    scale_factor: f64,
}

#[derive(Serialize, Deserialize)]
struct SignalValue {
    value: f64,
}

thread_local! {
    static PARAMS: RefCell<Option<Params>> = RefCell::new(None);
}

// Import: host feeds input through this function
#[link(wasm_import_module = "lens")]
extern "C" {
    fn next() -> *mut u8;
}

// Export: host calls this to allocate memory for writing input data
#[no_mangle]
pub extern "C" fn alloc(size: i64) -> *mut u8 {
    let mut buf = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

// Export: receive configuration from host
#[no_mangle]
pub extern "C" fn set_param(ptr: *mut u8) -> *mut u8 {
    let json_bytes = read_json_payload(ptr);
    match json_bytes {
        Some(bytes) => {
            if let Ok(params) = serde_json::from_slice::<Params>(&bytes) {
                PARAMS.with(|p| *p.borrow_mut() = Some(params));
            }
            encode_nil()
        }
        None => encode_nil(),
    }
}

// Export: forward transform
#[no_mangle]
pub extern "C" fn transform() -> *mut u8 {
    let input_ptr = unsafe { next() };
    match read_json_payload(input_ptr) {
        Some(bytes) => {
            if let Ok(input) = serde_json::from_slice::<SignalValue>(&bytes) {
                PARAMS.with(|p| {
                    let params = p.borrow();
                    let factor = params
                        .as_ref()
                        .map(|p| p.scale_factor)
                        .unwrap_or(1.0);
                    encode_json(&SignalValue {
                        value: input.value * factor,
                    })
                })
            } else {
                encode_eos()
            }
        }
        None => encode_eos(),
    }
}

// --- Transport buffer helpers ---

fn read_json_payload(ptr: *mut u8) -> Option<Vec<u8>> {
    if ptr.is_null() {
        return None;
    }
    unsafe {
        if *ptr != TYPE_JSON {
            return None;
        }
        let len_bytes = std::slice::from_raw_parts(ptr.add(1), 4);
        let len = u32::from_le_bytes([
            len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3],
        ]) as usize;
        Some(std::slice::from_raw_parts(ptr.add(5), len).to_vec())
    }
}

fn encode_json<T: Serialize>(value: &T) -> *mut u8 {
    let payload = serde_json::to_vec(value).unwrap();
    let mut buf = Vec::with_capacity(1 + 4 + payload.len());
    buf.push(TYPE_JSON);
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(&payload);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

fn encode_nil() -> *mut u8 {
    let mut buf = vec![0u8]; // TYPE_NIL
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

fn encode_eos() -> *mut u8 {
    let mut buf = vec![TYPE_EOS];
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}
```

## Building

```bash
cargo build --target wasm32-unknown-unknown --release
```

The output file is at `target/wasm32-unknown-unknown/release/my_lens.wasm`.

## Adding an Inverse

If your transform is reversible, export an `inverse()` function. This enables bidirectional signal conversion (e.g., encoding on one end, decoding on the other).

```rust
#[no_mangle]
pub extern "C" fn inverse() -> *mut u8 {
    let input_ptr = unsafe { next() };
    match read_json_payload(input_ptr) {
        Some(bytes) => {
            if let Ok(input) = serde_json::from_slice::<SignalValue>(&bytes) {
                PARAMS.with(|p| {
                    let params = p.borrow();
                    let factor = params
                        .as_ref()
                        .map(|p| p.scale_factor)
                        .unwrap_or(1.0);
                    if factor == 0.0 {
                        return encode_eos();
                    }
                    encode_json(&SignalValue {
                        value: input.value / factor,
                    })
                })
            } else {
                encode_eos()
            }
        }
        None => encode_eos(),
    }
}
```

## Using lens_sdk

For production lenses, you can use the `lens_sdk` crate (v0.8.1 on crates.io) which provides macros that handle the transport buffer boilerplate:

```toml
[dependencies]
lens_sdk = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

```rust
use lens_sdk::StreamOption;
use serde::{Serialize, Deserialize};

lens_sdk::define!(PARAMS: Params, try_transform);

#[derive(Serialize, Deserialize)]
struct Params {
    scale_factor: f64,
}

#[derive(Serialize, Deserialize)]
struct SignalValue {
    value: f64,
}

fn try_transform(
    iter: &mut dyn Iterator<Item = lens_sdk::Result<Option<SignalValue>>>,
) -> Result<StreamOption<SignalValue>, Box<dyn std::error::Error>> {
    let params = PARAMS.read().unwrap();
    let p = params.as_ref().unwrap();

    match iter.next() {
        Some(Ok(Some(input))) => {
            Ok(StreamOption::Some(SignalValue {
                value: input.value * p.scale_factor,
            }))
        }
        _ => Ok(StreamOption::EndOfStream),
    }
}
```

The `define!` macro generates the `alloc`, `transform`, and `set_param` exports for you.

## Testing Your Lens

Test locally without CLASP by loading the WASM into the `clasp-lens` host:

```rust
use clasp_lens::LensHost;
use serde_json::json;

#[test]
fn test_my_lens() {
    let wasm = include_bytes!("path/to/my_lens.wasm");
    let mut host = LensHost::new(wasm).unwrap();
    host.set_params(json!({"scale_factor": 3.0}));

    let output = host.transform(&json!({"value": 0.5})).unwrap();
    assert_eq!(output["value"].as_f64().unwrap(), 1.5);
}
```

## Signal Value Format

CLASP signal values are JSON objects with a `value` field. Your lens should expect and return this format:

```json
{"value": 0.5}
```

The `value` field is typically `f64`, but can be any JSON type depending on the signal. Your lens should handle unexpected types gracefully (return the input unchanged or return an end-of-stream marker).

## Size Optimization

To minimize WASM binary size:

1. Use `opt-level = "z"` and `lto = true` in release profile
2. Avoid pulling in large dependencies (regex, chrono, etc.)
3. Use `wasm-opt` from the binaryen toolkit for further optimization:
   ```bash
   wasm-opt -Oz -o optimized.wasm my_lens.wasm
   ```
4. Typical lens sizes: 30-100KB uncompressed, 10-30KB gzipped

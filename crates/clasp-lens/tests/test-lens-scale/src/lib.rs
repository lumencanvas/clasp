//! Minimal LensVM-compatible WASM module for testing.
//!
//! Implements the LensVM host protocol without depending on lens_sdk.
//! This lens scales a value: output = input * scale_factor + offset.

use serde::{Deserialize, Serialize};
use std::cell::RefCell;

// Transport buffer type IDs
const TYPE_ERROR: u8 = 0xFF; // -1 as u8
const TYPE_JSON: u8 = 1;
const TYPE_EOS: u8 = 127;

#[derive(Serialize, Deserialize)]
struct Params {
    scale_factor: f64,
    offset: f64,
}

#[derive(Serialize, Deserialize)]
struct SignalValue {
    value: f64,
}

thread_local! {
    static PARAMS: RefCell<Option<Params>> = RefCell::new(None);
}

// Import: host provides next() to feed input (LensVM protocol: "lens" module)
#[link(wasm_import_module = "lens")]
extern "C" {
    fn next() -> *mut u8;
}

/// Export: allocate memory for the host to write into
#[no_mangle]
pub extern "C" fn alloc(size: i64) -> *mut u8 {
    let mut buf = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Export: receive parameters from host
#[no_mangle]
pub extern "C" fn set_param(ptr: *mut u8) -> *mut u8 {
    let data = read_transport_buffer(ptr);
    match data {
        Some(json_bytes) => {
            if let Ok(params) = serde_json::from_slice::<Params>(&json_bytes) {
                PARAMS.with(|p| *p.borrow_mut() = Some(params));
                encode_nil()
            } else {
                encode_error(b"invalid params JSON")
            }
        }
        None => encode_error(b"failed to read params"),
    }
}

/// Export: forward transform
#[no_mangle]
pub extern "C" fn transform() -> *mut u8 {
    let input_ptr = unsafe { next() };
    let input_data = read_transport_buffer(input_ptr);

    match input_data {
        Some(json_bytes) => {
            if let Ok(input) = serde_json::from_slice::<SignalValue>(&json_bytes) {
                PARAMS.with(|p| {
                    let params = p.borrow();
                    let p = params.as_ref().unwrap_or(&Params {
                        scale_factor: 1.0,
                        offset: 0.0,
                    });
                    let output = SignalValue {
                        value: input.value * p.scale_factor + p.offset,
                    };
                    encode_json(&output)
                })
            } else {
                encode_error(b"invalid input JSON")
            }
        }
        None => encode_eos(),
    }
}

/// Export: inverse transform
#[no_mangle]
pub extern "C" fn inverse() -> *mut u8 {
    let input_ptr = unsafe { next() };
    let input_data = read_transport_buffer(input_ptr);

    match input_data {
        Some(json_bytes) => {
            if let Ok(input) = serde_json::from_slice::<SignalValue>(&json_bytes) {
                PARAMS.with(|p| {
                    let params = p.borrow();
                    let p = params.as_ref().unwrap_or(&Params {
                        scale_factor: 1.0,
                        offset: 0.0,
                    });
                    if p.scale_factor == 0.0 {
                        return encode_error(b"scale_factor is zero");
                    }
                    let output = SignalValue {
                        value: (input.value - p.offset) / p.scale_factor,
                    };
                    encode_json(&output)
                })
            } else {
                encode_error(b"invalid input JSON")
            }
        }
        None => encode_eos(),
    }
}

// --- Transport buffer helpers ---

fn read_transport_buffer(ptr: *mut u8) -> Option<Vec<u8>> {
    if ptr.is_null() {
        return None;
    }
    unsafe {
        let type_id = *ptr;
        match type_id {
            TYPE_JSON => {
                let len_bytes = std::slice::from_raw_parts(ptr.add(1), 4);
                let len = u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;
                let payload = std::slice::from_raw_parts(ptr.add(5), len);
                Some(payload.to_vec())
            }
            TYPE_EOS => None,
            _ => None,
        }
    }
}

fn encode_json<T: Serialize>(value: &T) -> *mut u8 {
    let payload = serde_json::to_vec(value).unwrap();
    let total = 1 + 4 + payload.len();
    let mut buf = Vec::with_capacity(total);
    buf.push(TYPE_JSON);
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(&payload);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

fn encode_error(msg: &[u8]) -> *mut u8 {
    let total = 1 + 4 + msg.len();
    let mut buf = Vec::with_capacity(total);
    buf.push(TYPE_ERROR);
    buf.extend_from_slice(&(msg.len() as u32).to_le_bytes());
    buf.extend_from_slice(msg);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

fn encode_nil() -> *mut u8 {
    let mut buf = vec![0u8]; // TYPE_NIL = 0
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

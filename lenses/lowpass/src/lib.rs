//! Low-pass filter lens.
//!
//! Smooths signal values using a first-order IIR (infinite impulse response)
//! filter. Higher alpha values produce more smoothing (slower response).
//!
//! Parameters:
//!   alpha: f64 (0.0 to 1.0) -- smoothing factor. 0.0 = no smoothing, 0.99 = heavy smoothing.
//!
//! Formula: output = alpha * previous + (1 - alpha) * input

use serde::{Deserialize, Serialize};
use std::cell::RefCell;

const TYPE_JSON: u8 = 1;
const TYPE_EOS: u8 = 127;

#[derive(Serialize, Deserialize)]
struct Params {
    alpha: f64,
}

#[derive(Serialize, Deserialize)]
struct SignalValue {
    value: f64,
}

thread_local! {
    static PARAMS: RefCell<Option<Params>> = RefCell::new(None);
    static PREV: RefCell<Option<f64>> = RefCell::new(None);
}

#[link(wasm_import_module = "lens")]
extern "C" {
    fn next() -> *mut u8;
}

#[no_mangle]
pub extern "C" fn alloc(size: i64) -> *mut u8 {
    let mut buf = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn set_param(ptr: *mut u8) -> *mut u8 {
    if let Some(bytes) = read_json_payload(ptr) {
        if let Ok(params) = serde_json::from_slice::<Params>(&bytes) {
            PARAMS.with(|p| *p.borrow_mut() = Some(params));
        }
    }
    encode_nil()
}

#[no_mangle]
pub extern "C" fn transform() -> *mut u8 {
    let input_ptr = unsafe { next() };
    match read_json_payload(input_ptr) {
        Some(bytes) => {
            if let Ok(input) = serde_json::from_slice::<SignalValue>(&bytes) {
                let alpha = PARAMS.with(|p| {
                    p.borrow().as_ref().map(|p| p.alpha).unwrap_or(0.5)
                });
                let alpha = alpha.clamp(0.0, 1.0);

                let output = PREV.with(|prev| {
                    let prev_val = prev.borrow().unwrap_or(input.value);
                    let filtered = alpha * prev_val + (1.0 - alpha) * input.value;
                    *prev.borrow_mut() = Some(filtered);
                    filtered
                });

                encode_json(&SignalValue { value: output })
            } else {
                encode_eos()
            }
        }
        None => encode_eos(),
    }
}

fn read_json_payload(ptr: *mut u8) -> Option<Vec<u8>> {
    if ptr.is_null() { return None; }
    unsafe {
        if *ptr != TYPE_JSON { return None; }
        let len_bytes = std::slice::from_raw_parts(ptr.add(1), 4);
        let len = u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;
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
    let mut buf = vec![0u8];
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

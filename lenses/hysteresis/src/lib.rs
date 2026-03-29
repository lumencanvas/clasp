//! Schmitt trigger / hysteresis lens.
//!
//! Outputs 1.0 when input rises above `high` threshold, 0.0 when it drops
//! below `low` threshold. Between the thresholds, holds the previous output.
//! Prevents signal jitter around a single threshold.
//!
//! Parameters:
//!   low: f64  -- lower threshold (default 0.3)
//!   high: f64 -- upper threshold (default 0.7)

use serde::{Deserialize, Serialize};
use std::cell::RefCell;

const TYPE_JSON: u8 = 1;
const TYPE_EOS: u8 = 127;

#[derive(Serialize, Deserialize)]
struct Params {
    low: f64,
    high: f64,
}

#[derive(Serialize, Deserialize)]
struct SignalValue {
    value: f64,
}

thread_local! {
    static PARAMS: RefCell<Option<Params>> = RefCell::new(None);
    static STATE: RefCell<f64> = RefCell::new(0.0);
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
                let (low, high) = PARAMS.with(|p| {
                    let params = p.borrow();
                    let p = params.as_ref();
                    (p.map(|p| p.low).unwrap_or(0.3), p.map(|p| p.high).unwrap_or(0.7))
                });

                let output = STATE.with(|state| {
                    let prev = *state.borrow();
                    let next = if input.value >= high {
                        1.0
                    } else if input.value <= low {
                        0.0
                    } else {
                        prev
                    };
                    *state.borrow_mut() = next;
                    next
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

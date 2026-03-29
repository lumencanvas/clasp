//! Moving average lens.
//!
//! Computes a simple moving average over a configurable window of
//! recent values. Useful for smoothing noisy sensor data.
//!
//! Parameters:
//!   window: usize -- number of samples to average (default 5)

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::VecDeque;

const TYPE_JSON: u8 = 1;
const TYPE_EOS: u8 = 127;

#[derive(Serialize, Deserialize)]
struct Params {
    window: usize,
}

#[derive(Serialize, Deserialize)]
struct SignalValue {
    value: f64,
}

thread_local! {
    static PARAMS: RefCell<Option<Params>> = RefCell::new(None);
    static BUFFER: RefCell<VecDeque<f64>> = RefCell::new(VecDeque::new());
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
                let window = PARAMS.with(|p| {
                    p.borrow().as_ref().map(|p| p.window).unwrap_or(5)
                });
                let window = window.max(1);

                let avg = BUFFER.with(|buf| {
                    let mut buf = buf.borrow_mut();
                    buf.push_back(input.value);
                    while buf.len() > window {
                        buf.pop_front();
                    }
                    let sum: f64 = buf.iter().sum();
                    sum / buf.len() as f64
                });

                encode_json(&SignalValue { value: avg })
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

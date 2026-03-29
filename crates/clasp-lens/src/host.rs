//! LensVM WASM host implementation.
//!
//! Loads a lens WASM module and provides the host-side protocol:
//! - Implements the `next()` import (feeds input data to the lens)
//! - Calls `alloc()` to write input into WASM memory
//! - Calls `set_param()` to configure lens parameters
//! - Calls `transform()` / `inverse()` and reads the transport buffer result

use crate::error::{LensError, Result};
use crate::transport::{self, TransportItem};
use std::sync::{Arc, Mutex};
use tracing::debug;
use wasmtime::*;

/// Shared state between the host and the WASM module, used by the `next()` import.
#[derive(Default)]
struct HostState {
    /// Queued input items (transport-encoded) for the lens to consume via next().
    input_queue: Vec<Vec<u8>>,
    /// Current position in the input queue.
    input_pos: usize,
}

/// A compiled and instantiated LensVM WASM module.
///
/// Reuse this struct across multiple transform calls to avoid re-compilation.
/// The WASM module is compiled once and can be instantiated per-call or reused.
pub struct LensHost {
    engine: Engine,
    module: Module,
    params: Option<serde_json::Value>,
}

impl LensHost {
    /// Create a new LensHost from raw WASM bytes.
    ///
    /// Validates that the module exports the required `alloc` and `transform`
    /// functions. The `inverse` and `set_param` exports are optional.
    pub fn new(wasm_bytes: &[u8]) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::new(&engine, wasm_bytes)?;

        // Validate required exports exist
        let export_names: Vec<&str> = module.exports().map(|e| e.name()).collect();
        if !export_names.contains(&"alloc") {
            return Err(LensError::MissingExport {
                name: "alloc".into(),
            });
        }
        if !export_names.contains(&"transform") {
            return Err(LensError::MissingExport {
                name: "transform".into(),
            });
        }

        Ok(Self {
            engine,
            module,
            params: None,
        })
    }

    /// Set parameters that will be passed to the lens via `set_param()`.
    pub fn set_params(&mut self, params: serde_json::Value) {
        self.params = Some(params);
    }

    /// Check whether the module exports an `inverse` function.
    pub fn has_inverse(&self) -> bool {
        self.module.exports().any(|e| e.name() == "inverse")
    }

    /// Run the forward transform on a JSON value.
    pub fn transform(&self, input: &serde_json::Value) -> Result<serde_json::Value> {
        self.run_lens("transform", input)
    }

    /// Run the inverse transform on a JSON value.
    ///
    /// Returns an error if the module does not export `inverse`.
    pub fn inverse(&self, input: &serde_json::Value) -> Result<serde_json::Value> {
        if !self.has_inverse() {
            return Err(LensError::MissingExport {
                name: "inverse".into(),
            });
        }
        self.run_lens("inverse", input)
    }

    /// Internal: instantiate the module and run a named transform function.
    fn run_lens(&self, func_name: &str, input: &serde_json::Value) -> Result<serde_json::Value> {
        let host_state = Arc::new(Mutex::new(HostState::default()));

        // Queue the input value + EOS marker for next() to return
        {
            let mut state = host_state.lock().unwrap();
            state.input_queue.push(transport::encode_json(input));
            state.input_queue.push(transport::encode_eos());
        }

        let mut store = Store::new(&self.engine, host_state.clone());

        // Create the linker with the `next()` import
        let mut linker = Linker::new(&self.engine);
        let host_for_next = host_state.clone();

        linker.func_wrap(
            "lens",
            "next",
            move |mut caller: Caller<'_, Arc<Mutex<HostState>>>| -> i32 {
                let state = host_for_next.lock().unwrap();
                let pos = state.input_pos;
                let data = if pos < state.input_queue.len() {
                    state.input_queue[pos].clone()
                } else {
                    transport::encode_eos()
                };
                drop(state);

                // Increment position
                host_for_next.lock().unwrap().input_pos += 1;

                // Write data into WASM memory via alloc
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(m)) => m,
                    _ => return 0,
                };
                let alloc = match caller.get_export("alloc") {
                    Some(Extern::Func(f)) => f,
                    _ => return 0,
                };

                let ptr = match alloc.typed::<i64, i32>(&caller) {
                    Ok(f) => match f.call(&mut caller, data.len() as i64) {
                        Ok(p) => p,
                        Err(_) => return 0,
                    },
                    Err(_) => return 0,
                };

                if let Err(_) = memory.write(&mut caller, ptr as usize, &data) {
                    return 0;
                }

                ptr
            },
        )?;

        let instance = linker.instantiate(&mut store, &self.module)?;

        // If params are set and set_param export exists, call it
        if let Some(ref params) = self.params {
            if let Some(set_param_fn) = instance.get_func(&mut store, "set_param") {
                let param_data = transport::encode_json(params);
                let memory = instance
                    .get_memory(&mut store, "memory")
                    .ok_or_else(|| LensError::Memory("no memory export".into()))?;
                let alloc_fn = instance.get_typed_func::<i64, i32>(&mut store, "alloc")?;

                let ptr = alloc_fn.call(&mut store, param_data.len() as i64)?;
                memory.write(&mut store, ptr as usize, &param_data)?;

                let typed = set_param_fn.typed::<i32, i32>(&store)?;
                let result_ptr = typed.call(&mut store, ptr)?;

                // Check for errors in set_param result
                if result_ptr != 0 {
                    let result_data = read_transport_buffer(&memory, &store, result_ptr)?;
                    if let Err(e) = transport::decode(&result_data) {
                        debug!("set_param returned error: {}", e);
                    }
                }
            }
        }

        // Call the transform/inverse function
        let transform_fn = instance.get_typed_func::<(), i32>(&mut store, func_name)?;
        let result_ptr = transform_fn.call(&mut store, ())?;

        if result_ptr == 0 {
            return Err(LensError::Memory("transform returned null pointer".into()));
        }

        // Read the result from WASM memory
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| LensError::Memory("no memory export".into()))?;
        let result_data = read_transport_buffer(&memory, &store, result_ptr)?;

        match transport::decode(&result_data)? {
            (TransportItem::Json(value), _) => Ok(value),
            (TransportItem::Nil, _) => Ok(serde_json::Value::Null),
            (TransportItem::EndOfStream, _) => Err(LensError::EndOfStream),
        }
    }
}

/// Read a transport buffer from WASM memory at the given pointer.
///
/// First reads the type ID byte, then determines how many more bytes to read
/// based on the type.
fn read_transport_buffer(
    memory: &Memory,
    store: &Store<Arc<Mutex<HostState>>>,
    ptr: i32,
) -> Result<Vec<u8>> {
    let ptr = ptr as usize;
    let mem_data = memory.data(store);

    if ptr >= mem_data.len() {
        return Err(LensError::Memory(format!(
            "pointer {} out of bounds (memory size: {})",
            ptr,
            mem_data.len()
        )));
    }

    let type_id = mem_data[ptr] as i8;

    match type_id {
        transport::TYPE_NIL | transport::TYPE_EOS => {
            // Single byte, no payload
            Ok(vec![mem_data[ptr]])
        }
        transport::TYPE_JSON | transport::TYPE_ERROR => {
            // Need 1 (type) + 4 (length) + N (payload) bytes
            if ptr + 5 > mem_data.len() {
                return Err(LensError::Memory(
                    "not enough bytes for length header".into(),
                ));
            }
            let len = u32::from_le_bytes([
                mem_data[ptr + 1],
                mem_data[ptr + 2],
                mem_data[ptr + 3],
                mem_data[ptr + 4],
            ]) as usize;
            let total = 5 + len;
            if ptr + total > mem_data.len() {
                return Err(LensError::Memory(format!(
                    "payload extends past memory: need {} bytes at offset {}",
                    total, ptr
                )));
            }
            Ok(mem_data[ptr..ptr + total].to_vec())
        }
        _ => Err(LensError::Decode(format!(
            "unknown type ID at pointer: {}",
            type_id
        ))),
    }
}

impl std::fmt::Debug for LensHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LensHost")
            .field("has_inverse", &self.has_inverse())
            .field("has_params", &self.params.is_some())
            .finish()
    }
}

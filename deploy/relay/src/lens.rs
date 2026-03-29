//! LensVM WASM transform integration for the relay server.

use anyhow::{Context, Result};
use clasp_core::Value;
use clasp_lens::LensHost;
use clasp_router::SignalTransform;
use std::path::PathBuf;
use std::sync::Mutex;

/// A single lens transform bound to an address pattern.
struct BoundLens {
    pattern: String,
    host: Mutex<LensHost>,
}

/// Pipeline of WASM lens transforms matched by address pattern.
pub struct LensTransformPipeline {
    lenses: Vec<BoundLens>,
}

impl LensTransformPipeline {
    pub fn from_configs(configs: &[LensConfigEntry]) -> Result<Self> {
        let mut lenses = Vec::new();
        for config in configs {
            let wasm_bytes = std::fs::read(&config.wasm)
                .with_context(|| format!("Failed to read WASM lens: {}", config.wasm.display()))?;
            let mut host = LensHost::new(&wasm_bytes)
                .with_context(|| format!("Failed to load WASM lens: {}", config.wasm.display()))?;

            if let Some(ref params) = config.params {
                host.set_params(params.clone());
            }

            lenses.push(BoundLens {
                pattern: config.address.clone(),
                host: Mutex::new(host),
            });
        }
        Ok(Self { lenses })
    }
}

/// Lens configuration entry (deserialized from JSON).
#[derive(serde::Deserialize)]
pub struct LensConfigEntry {
    pub address: String,
    pub wasm: PathBuf,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

impl SignalTransform for LensTransformPipeline {
    fn transform(&self, address: &str, value: &Value) -> Option<Value> {
        for lens in &self.lenses {
            if clasp_core::address::glob_match(&lens.pattern, address) {
                let json_value = serde_json::to_value(value).ok()?;
                let host = lens.host.lock().ok()?;
                match host.transform(&json_value) {
                    Ok(result) => {
                        return serde_json::from_value(result).ok();
                    }
                    Err(e) => {
                        tracing::warn!("Lens transform failed for {}: {}", address, e);
                        return None;
                    }
                }
            }
        }
        None
    }
}

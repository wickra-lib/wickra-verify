//! WebAssembly bindings for `wickra-verify` (wasm-bindgen).
//!
//! Recompute a claimed backtest report and confirm or refute it, compiled to
//! WebAssembly for the browser: create a `Verifier`, drive it with a command
//! JSON (`verify`, `explain`, `canonicalize`, `version`) and read back the
//! response JSON. The same command protocol crosses every binding, so a browser
//! front-end verifies against the exact same core as the native CLI.
//!
//! The backtest engine runs sequentially here (no rayon thread pool in a
//! browser sandbox), which is byte-identical to the native run — the exact
//! cross-language golden check.

use wasm_bindgen::prelude::*;

use verify_core::Verifier as CoreVerifier;

/// A verifier driven by JSON commands.
#[wasm_bindgen]
pub struct Verifier {
    inner: CoreVerifier,
}

#[wasm_bindgen]
impl Verifier {
    /// Create a verifier from a config JSON (`{"atol":..,"rtol":..}`); missing
    /// fields fall back to the defaults. Pass `undefined` (or `"{}"`) for the
    /// default tolerances.
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: Option<String>) -> Result<Verifier, JsError> {
        let config = config_json.unwrap_or_else(|| "{}".to_string());
        CoreVerifier::new(&config)
            .map(|inner| Self { inner })
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Apply a command JSON and return the resulting response JSON.
    pub fn command(&self, cmd_json: &str) -> Result<String, JsError> {
        self.inner
            .command_json(cmd_json)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// The library version.
    #[wasm_bindgen(js_name = version)]
    pub fn instance_version(&self) -> String {
        CoreVerifier::version().to_string()
    }
}

/// The library version.
#[wasm_bindgen]
pub fn version() -> String {
    CoreVerifier::version().to_string()
}

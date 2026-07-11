//! Node.js bindings for `wickra-verify` (napi-rs).
//!
//! Thin glue over the verify core's command surface: create a `Verifier`, drive
//! it with a command JSON (`verify`, `explain`, `canonicalize`, `version`) and
//! read back the response JSON. The same command protocol crosses every binding,
//! so a Node front-end drives the exact same core as the native CLI.

#![allow(missing_debug_implementations)]
// napi exposes owned `String` arguments; the bodies only need to borrow them.
#![allow(clippy::needless_pass_by_value)]

use napi::Result;
use napi_derive::napi;

use verify_core::Verifier as CoreVerifier;

/// Build a napi error from a message.
fn err(message: impl Into<String>) -> napi::Error {
    napi::Error::from_reason(message.into())
}

/// The library version.
#[napi]
pub fn version() -> String {
    CoreVerifier::version().to_string()
}

/// A verifier driven by JSON commands.
#[napi]
pub struct Verifier {
    inner: CoreVerifier,
}

#[napi]
impl Verifier {
    /// Create a verifier from a config JSON (`{"atol":..,"rtol":..}`); missing
    /// fields fall back to the defaults. Omit the argument (or pass `"{}"`) for
    /// the default tolerances.
    #[napi(constructor)]
    pub fn new(config_json: Option<String>) -> Result<Self> {
        let config = config_json.unwrap_or_else(|| "{}".to_string());
        CoreVerifier::new(&config)
            .map(|inner| Self { inner })
            .map_err(|e| err(e.to_string()))
    }

    /// Apply a command JSON and return the resulting response JSON.
    #[napi]
    pub fn command(&self, cmd_json: String) -> Result<String> {
        self.inner
            .command_json(&cmd_json)
            .map_err(|e| err(e.to_string()))
    }

    /// The library version.
    #[napi]
    pub fn version(&self) -> String {
        CoreVerifier::version().to_string()
    }
}

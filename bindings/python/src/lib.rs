//! Python bindings for `wickra-verify`, exposed under the `wickra_verify`
//! package.
//!
//! Thin glue over the verify core's command surface: create a [`Verifier`],
//! drive it with a command JSON (`verify`, `explain`, `canonicalize`,
//! `version`) and read back the response JSON. The same command protocol
//! crosses every binding, so a Python front-end drives the exact same core as
//! the native CLI.

// PyO3 protocol methods take `self` by value/ref regardless of use.
#![allow(clippy::needless_pass_by_value)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use verify_core::Verifier;

/// A verifier driven by JSON commands.
#[pyclass(name = "Verifier")]
struct PyVerifier {
    inner: Verifier,
}

#[pymethods]
impl PyVerifier {
    /// Create a verifier from a config JSON (`{"atol":..,"rtol":..}`); missing
    /// fields fall back to the defaults. Pass `"{}"` (the default) for the
    /// default tolerances.
    #[new]
    #[pyo3(signature = (config_json = "{}"))]
    fn new(config_json: &str) -> PyResult<Self> {
        Verifier::new(config_json)
            .map(|inner| Self { inner })
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Apply a command JSON and return the resulting response JSON.
    fn command(&self, cmd_json: &str) -> PyResult<String> {
        self.inner
            .command_json(cmd_json)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// The library version.
    #[staticmethod]
    fn version() -> &'static str {
        Verifier::version()
    }
}

/// The native module (`wickra_verify._wickra_verify`).
#[pymodule]
fn _wickra_verify(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<PyVerifier>()?;
    Ok(())
}

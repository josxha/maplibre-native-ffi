#![deny(unsafe_op_in_unsafe_fn)]

use maplibre_native_core::{self as maplibre_core, Error, ErrorKind, NetworkStatus};
use pyo3::prelude::*;

mod py_errors {
    pyo3::import_exception!(maplibre_native.errors, InvalidArgumentError);
    pyo3::import_exception!(maplibre_native.errors, InvalidStateError);
    pyo3::import_exception!(maplibre_native.errors, NativeError);
    pyo3::import_exception!(maplibre_native.errors, UnknownStatusError);
    pyo3::import_exception!(maplibre_native.errors, UnsupportedFeatureError);
    pyo3::import_exception!(maplibre_native.errors, WrongThreadError);
}

fn map_error(error: Error) -> PyErr {
    let raw_status = error.raw_status();
    let diagnostic = error.diagnostic().to_owned();
    match error.kind() {
        ErrorKind::InvalidArgument => {
            py_errors::InvalidArgumentError::new_err((raw_status, diagnostic))
        }
        ErrorKind::InvalidState => py_errors::InvalidStateError::new_err((raw_status, diagnostic)),
        ErrorKind::WrongThread => py_errors::WrongThreadError::new_err((raw_status, diagnostic)),
        ErrorKind::Unsupported => {
            py_errors::UnsupportedFeatureError::new_err((raw_status, diagnostic))
        }
        ErrorKind::NativeError => py_errors::NativeError::new_err((raw_status, diagnostic)),
        ErrorKind::UnknownStatus => {
            py_errors::UnknownStatusError::new_err((raw_status.unwrap_or_default(), diagnostic))
        }
        ErrorKind::AbiVersionMismatch => {
            py_errors::UnsupportedFeatureError::new_err((raw_status, diagnostic))
        }
        _ => py_errors::NativeError::new_err((raw_status, diagnostic)),
    }
}

/// Returns the C ABI version expected by the shared Rust adaptation layer.
#[pyfunction]
fn expected_c_abi_version() -> u32 {
    maplibre_core::EXPECTED_C_ABI_VERSION
}

/// Returns the native C ABI contract version reported by the linked library.
#[pyfunction]
fn c_version() -> u32 {
    // SAFETY: mln_c_version takes no arguments and returns the process-global C
    // ABI version for the linked native library.
    unsafe { maplibre_native_sys::mln_c_version() }
}

/// Returns the raw render-backend support mask reported by the linked library.
#[pyfunction]
fn supported_render_backends_raw() -> u32 {
    // SAFETY: mln_supported_render_backend_mask takes no arguments and returns
    // a value mask. The Python layer preserves unknown future bits.
    unsafe { maplibre_native_sys::mln_supported_render_backend_mask() }
}

/// Returns the raw process-global network status reported by the linked library.
#[pyfunction]
fn network_status_raw() -> PyResult<u32> {
    maplibre_core::network_status()
        .map(NetworkStatus::raw_value)
        .map_err(map_error)
}

/// Sets the process-global network status from a raw C enum value.
#[pyfunction]
fn set_network_status_raw(raw_status: u32) -> PyResult<()> {
    maplibre_core::set_network_status(NetworkStatus::from_raw(raw_status)).map_err(map_error)
}

/// Test helper that lets native status conversion see C validation failures.
#[pyfunction]
fn set_network_status_raw_unchecked_for_test(raw_status: u32) -> PyResult<()> {
    maplibre_core::set_network_status_raw(raw_status).map_err(map_error)
}

/// Private PyO3 extension for the public maplibre_native package.
#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(expected_c_abi_version, module)?)?;
    module.add_function(wrap_pyfunction!(c_version, module)?)?;
    module.add_function(wrap_pyfunction!(supported_render_backends_raw, module)?)?;
    module.add_function(wrap_pyfunction!(network_status_raw, module)?)?;
    module.add_function(wrap_pyfunction!(set_network_status_raw, module)?)?;
    module.add_function(wrap_pyfunction!(
        set_network_status_raw_unchecked_for_test,
        module
    )?)?;
    Ok(())
}

#![deny(unsafe_op_in_unsafe_fn)]

use maplibre_native_core::{self as maplibre_core, Error, ErrorKind, NetworkStatus};
use maplibre_native_sys as sys;
use pyo3::prelude::*;
use std::sync::{Mutex, MutexGuard};

mod py_errors {
    pyo3::import_exception!(maplibre_native.errors, InvalidArgumentError);
    pyo3::import_exception!(maplibre_native.errors, InvalidStateError);
    pyo3::import_exception!(maplibre_native.errors, NativeError);
    pyo3::import_exception!(maplibre_native.errors, UnknownStatusError);
    pyo3::import_exception!(maplibre_native.errors, UnsupportedFeatureError);
    pyo3::import_exception!(maplibre_native.errors, WrongThreadError);
}

#[pyclass(name = "_RuntimeHandle")]
struct RuntimeHandle {
    state: Mutex<maplibre_core::handle::NativeHandleState<sys::mln_runtime>>,
}

impl RuntimeHandle {
    fn state(&self) -> MutexGuard<'_, maplibre_core::handle::NativeHandleState<sys::mln_runtime>> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[pymethods]
impl RuntimeHandle {
    fn close(&self) -> PyResult<()> {
        let state = self.state();
        // SAFETY: state owns an mln_runtime pointer created by mln_runtime_create
        // and pairs it with the matching status-returning destroy function.
        unsafe { state.close_status(sys::mln_runtime_destroy) }.map_err(map_error)
    }

    fn run_once(&self) -> PyResult<()> {
        let state = self.state();
        // SAFETY: The C API validates that the pointer is a live runtime handle
        // and that the call occurs on the runtime owner thread.
        maplibre_core::check(unsafe { sys::mln_runtime_run_once(state.as_ptr()) })
            .map_err(map_error)
    }

    #[getter]
    fn closed(&self) -> bool {
        self.state().is_closed()
    }
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
    unsafe { sys::mln_c_version() }
}

/// Returns the raw render-backend support mask reported by the linked library.
#[pyfunction]
fn supported_render_backends_raw() -> u32 {
    // SAFETY: mln_supported_render_backend_mask takes no arguments and returns
    // a value mask. The Python layer preserves unknown future bits.
    unsafe { sys::mln_supported_render_backend_mask() }
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

/// Creates a runtime handle on the current thread.
#[pyfunction]
fn create_runtime(
    asset_path: Option<String>,
    cache_path: Option<String>,
    maximum_cache_size: Option<u64>,
) -> PyResult<RuntimeHandle> {
    maplibre_core::validate_abi_version().map_err(map_error)?;
    let mut options = maplibre_core::RuntimeOptions::new();
    if let Some(asset_path) = asset_path {
        options = options.with_asset_path(asset_path);
    }
    if let Some(cache_path) = cache_path {
        options = options.with_cache_path(cache_path);
    }
    if let Some(maximum_cache_size) = maximum_cache_size {
        options = options.with_maximum_cache_size(maximum_cache_size);
    }
    let native_options =
        maplibre_core::runtime::runtime_options_to_native(&options).map_err(map_error)?;
    let raw_options = native_options.to_raw();
    let mut out = maplibre_core::ptr::OutPtr::<sys::mln_runtime>::new();
    // SAFETY: raw_options points to a materialized mln_runtime_options value
    // whose backing strings live for this call. out is a valid
    // null-initialized out-pointer owned by this call.
    maplibre_core::check(unsafe { sys::mln_runtime_create(&raw_options, out.as_mut_ptr()) })
        .map_err(map_error)?;
    let ptr = out.into_non_null("mln_runtime").map_err(map_error)?;
    // SAFETY: ptr came from successful mln_runtime_create and is paired with
    // the matching status-returning destroy function in RuntimeHandle.close.
    let state = unsafe { maplibre_core::handle::NativeHandleState::from_raw(ptr, "mln_runtime") };
    Ok(RuntimeHandle {
        state: Mutex::new(state),
    })
}

/// Private PyO3 extension for the public maplibre_native package.
#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<RuntimeHandle>()?;
    module.add_function(wrap_pyfunction!(expected_c_abi_version, module)?)?;
    module.add_function(wrap_pyfunction!(c_version, module)?)?;
    module.add_function(wrap_pyfunction!(supported_render_backends_raw, module)?)?;
    module.add_function(wrap_pyfunction!(network_status_raw, module)?)?;
    module.add_function(wrap_pyfunction!(set_network_status_raw, module)?)?;
    module.add_function(wrap_pyfunction!(
        set_network_status_raw_unchecked_for_test,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(create_runtime, module)?)?;
    Ok(())
}

#![deny(unsafe_op_in_unsafe_fn)]

use maplibre_native_core::{
    self as maplibre_core, Error, ErrorKind, NetworkStatus, RenderMode, RuntimeEventPayload,
    RuntimeEventType, TileOperation,
};
use maplibre_native_sys as sys;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
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

#[pyclass(name = "_MapHandle")]
struct MapHandle {
    state: Mutex<maplibre_core::handle::NativeHandleState<sys::mln_map>>,
}

impl RuntimeHandle {
    fn state(&self) -> MutexGuard<'_, maplibre_core::handle::NativeHandleState<sys::mln_runtime>> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl MapHandle {
    fn state(&self) -> MutexGuard<'_, maplibre_core::handle::NativeHandleState<sys::mln_map>> {
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

    fn poll_event(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let state = self.state();
        let mut event = maplibre_core::events::empty_runtime_event();
        let mut has_event = false;
        // SAFETY: The C API validates that the pointer is a live runtime handle.
        // event has the correct size field and has_event points to writable
        // storage for one bool.
        maplibre_core::check(unsafe {
            sys::mln_runtime_poll_event(state.as_ptr(), &mut event, &mut has_event)
        })
        .map_err(map_error)?;
        if !has_event {
            return Ok(None);
        }

        // SAFETY: event and its payload/text pointers remain valid until the
        // next poll for this runtime. This call copies before another poll can
        // occur because the runtime state mutex is still held.
        let copied = unsafe { maplibre_core::events::runtime_event_from_native(&event) }
            .map_err(map_error)?;
        event_to_py(py, copied).map(Some)
    }

    #[getter]
    fn closed(&self) -> bool {
        self.state().is_closed()
    }
}

#[pymethods]
impl MapHandle {
    fn close(&self) -> PyResult<()> {
        let state = self.state();
        // SAFETY: state owns an mln_map pointer created by mln_map_create and
        // pairs it with the matching status-returning destroy function.
        unsafe { state.close_status(sys::mln_map_destroy) }.map_err(map_error)
    }

    fn request_repaint(&self) -> PyResult<()> {
        let state = self.state();
        // SAFETY: The C API validates that the pointer is a live map handle and
        // that the call occurs on the map owner thread.
        maplibre_core::check(unsafe { sys::mln_map_request_repaint(state.as_ptr()) })
            .map_err(map_error)
    }

    fn set_style_json(&self, json: String) -> PyResult<()> {
        let state = self.state();
        let json = maplibre_core::string::c_string(&json).map_err(map_error)?;
        // SAFETY: The C API validates that the pointer is a live map handle.
        // json is a null-terminated C string whose storage lives for this call.
        maplibre_core::check(unsafe { sys::mln_map_set_style_json(state.as_ptr(), json.as_ptr()) })
            .map_err(map_error)
    }

    #[getter]
    fn closed(&self) -> bool {
        self.state().is_closed()
    }
}

fn event_to_py(py: Python<'_>, event: maplibre_core::CopiedRuntimeEvent) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("event_type", event_type_raw(event.event_type))?;
    dict.set_item("source_type", event.source.source_type)?;
    dict.set_item("source_address", event.source.source_address)?;
    dict.set_item("code", event.code)?;
    dict.set_item("message", event.message)?;
    dict.set_item("payload", payload_to_py(py, event.payload)?)?;
    Ok(dict.into_any().unbind())
}

fn payload_to_py(py: Python<'_>, payload: RuntimeEventPayload) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    match payload {
        RuntimeEventPayload::None => {
            dict.set_item("kind", "none")?;
        }
        RuntimeEventPayload::RenderFrame(payload) => {
            dict.set_item("kind", "render_frame")?;
            dict.set_item("mode", render_mode_raw(payload.mode))?;
            dict.set_item("needs_repaint", payload.needs_repaint)?;
            dict.set_item("placement_changed", payload.placement_changed)?;
        }
        RuntimeEventPayload::RenderMap(payload) => {
            dict.set_item("kind", "render_map")?;
            dict.set_item("mode", render_mode_raw(payload.mode))?;
        }
        RuntimeEventPayload::StyleImageMissing(payload) => {
            dict.set_item("kind", "style_image_missing")?;
            dict.set_item("image_id", payload.image_id)?;
        }
        RuntimeEventPayload::TileAction(payload) => {
            dict.set_item("kind", "tile_action")?;
            dict.set_item("operation", tile_operation_raw(payload.operation))?;
            dict.set_item("source_id", payload.source_id)?;
        }
        RuntimeEventPayload::OfflineRegionStatus(payload) => {
            dict.set_item("kind", "offline_region_status")?;
            dict.set_item("region_id", payload.region_id)?;
        }
        RuntimeEventPayload::OfflineRegionResponseError(payload) => {
            dict.set_item("kind", "offline_region_response_error")?;
            dict.set_item("region_id", payload.region_id)?;
            dict.set_item("reason", payload.reason.raw_value())?;
        }
        RuntimeEventPayload::OfflineRegionTileCountLimit(payload) => {
            dict.set_item("kind", "offline_region_tile_count_limit")?;
            dict.set_item("region_id", payload.region_id)?;
            dict.set_item("limit", payload.limit)?;
        }
        RuntimeEventPayload::OfflineOperationCompleted(payload) => {
            dict.set_item("kind", "offline_operation_completed")?;
            dict.set_item("operation_id", payload.operation_id)?;
            dict.set_item("operation_kind", payload.raw_operation_kind)?;
            dict.set_item("result_kind", payload.raw_result_kind)?;
            dict.set_item("result_status", payload.result_status)?;
            dict.set_item("found", payload.found)?;
        }
        RuntimeEventPayload::Unknown(payload) => {
            dict.set_item("kind", "unknown")?;
            dict.set_item("raw_type", payload.raw_type)?;
            dict.set_item("bytes", PyBytes::new(py, &payload.bytes))?;
        }
        _ => {
            dict.set_item("kind", "unknown")?;
        }
    }
    Ok(dict.into_any().unbind())
}

fn render_mode_raw(mode: RenderMode) -> u32 {
    match mode {
        RenderMode::Partial => sys::MLN_RENDER_MODE_PARTIAL,
        RenderMode::Full => sys::MLN_RENDER_MODE_FULL,
        RenderMode::Unknown(raw) => raw,
        _ => 0,
    }
}

fn tile_operation_raw(operation: TileOperation) -> u32 {
    match operation {
        TileOperation::RequestedFromCache => sys::MLN_TILE_OPERATION_REQUESTED_FROM_CACHE,
        TileOperation::RequestedFromNetwork => sys::MLN_TILE_OPERATION_REQUESTED_FROM_NETWORK,
        TileOperation::LoadFromNetwork => sys::MLN_TILE_OPERATION_LOAD_FROM_NETWORK,
        TileOperation::LoadFromCache => sys::MLN_TILE_OPERATION_LOAD_FROM_CACHE,
        TileOperation::StartParse => sys::MLN_TILE_OPERATION_START_PARSE,
        TileOperation::EndParse => sys::MLN_TILE_OPERATION_END_PARSE,
        TileOperation::Error => sys::MLN_TILE_OPERATION_ERROR,
        TileOperation::Cancelled => sys::MLN_TILE_OPERATION_CANCELLED,
        TileOperation::Null => sys::MLN_TILE_OPERATION_NULL,
        TileOperation::Unknown(raw) => raw,
        _ => 0,
    }
}

fn event_type_raw(event_type: RuntimeEventType) -> u32 {
    match event_type {
        RuntimeEventType::MapCameraWillChange => sys::MLN_RUNTIME_EVENT_MAP_CAMERA_WILL_CHANGE,
        RuntimeEventType::MapCameraIsChanging => sys::MLN_RUNTIME_EVENT_MAP_CAMERA_IS_CHANGING,
        RuntimeEventType::MapCameraDidChange => sys::MLN_RUNTIME_EVENT_MAP_CAMERA_DID_CHANGE,
        RuntimeEventType::MapStyleLoaded => sys::MLN_RUNTIME_EVENT_MAP_STYLE_LOADED,
        RuntimeEventType::MapLoadingStarted => sys::MLN_RUNTIME_EVENT_MAP_LOADING_STARTED,
        RuntimeEventType::MapLoadingFinished => sys::MLN_RUNTIME_EVENT_MAP_LOADING_FINISHED,
        RuntimeEventType::MapLoadingFailed => sys::MLN_RUNTIME_EVENT_MAP_LOADING_FAILED,
        RuntimeEventType::MapIdle => sys::MLN_RUNTIME_EVENT_MAP_IDLE,
        RuntimeEventType::MapRenderUpdateAvailable => {
            sys::MLN_RUNTIME_EVENT_MAP_RENDER_UPDATE_AVAILABLE
        }
        RuntimeEventType::MapRenderError => sys::MLN_RUNTIME_EVENT_MAP_RENDER_ERROR,
        RuntimeEventType::MapStillImageFinished => sys::MLN_RUNTIME_EVENT_MAP_STILL_IMAGE_FINISHED,
        RuntimeEventType::MapStillImageFailed => sys::MLN_RUNTIME_EVENT_MAP_STILL_IMAGE_FAILED,
        RuntimeEventType::MapRenderFrameStarted => sys::MLN_RUNTIME_EVENT_MAP_RENDER_FRAME_STARTED,
        RuntimeEventType::MapRenderFrameFinished => {
            sys::MLN_RUNTIME_EVENT_MAP_RENDER_FRAME_FINISHED
        }
        RuntimeEventType::MapRenderMapStarted => sys::MLN_RUNTIME_EVENT_MAP_RENDER_MAP_STARTED,
        RuntimeEventType::MapRenderMapFinished => sys::MLN_RUNTIME_EVENT_MAP_RENDER_MAP_FINISHED,
        RuntimeEventType::MapStyleImageMissing => sys::MLN_RUNTIME_EVENT_MAP_STYLE_IMAGE_MISSING,
        RuntimeEventType::MapTileAction => sys::MLN_RUNTIME_EVENT_MAP_TILE_ACTION,
        RuntimeEventType::OfflineRegionStatusChanged => {
            sys::MLN_RUNTIME_EVENT_OFFLINE_REGION_STATUS_CHANGED
        }
        RuntimeEventType::OfflineRegionResponseError => {
            sys::MLN_RUNTIME_EVENT_OFFLINE_REGION_RESPONSE_ERROR
        }
        RuntimeEventType::OfflineRegionTileCountLimitExceeded => {
            sys::MLN_RUNTIME_EVENT_OFFLINE_REGION_TILE_COUNT_LIMIT_EXCEEDED
        }
        RuntimeEventType::OfflineOperationCompleted => {
            sys::MLN_RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED
        }
        RuntimeEventType::Unknown(raw) => raw,
        _ => 0,
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

/// Creates a map handle owned by a runtime.
#[pyfunction]
fn create_map(
    runtime: &RuntimeHandle,
    width: u32,
    height: u32,
    scale_factor: f64,
    map_mode: u32,
) -> PyResult<MapHandle> {
    let Some(mode) = maplibre_core::MapMode::from_raw(map_mode) else {
        return Err(py_errors::InvalidArgumentError::new_err((
            Option::<i32>::None,
            format!("unknown map mode: {map_mode}"),
        )));
    };
    let options = maplibre_core::MapOptions::new(width, height, scale_factor).with_mode(mode);
    let raw_options = maplibre_core::options::map_options_to_native(&options);
    let runtime_state = runtime.state();
    let mut out = maplibre_core::ptr::OutPtr::<sys::mln_map>::new();
    // SAFETY: runtime_state owns or has released the runtime pointer. The C API
    // validates that it is live. raw_options is a fully initialized value, and
    // out is a valid null-initialized out-pointer owned by this call.
    maplibre_core::check(unsafe {
        sys::mln_map_create(runtime_state.as_ptr(), &raw_options, out.as_mut_ptr())
    })
    .map_err(map_error)?;
    let ptr = out.into_non_null("mln_map").map_err(map_error)?;
    // SAFETY: ptr came from successful mln_map_create and is paired with the
    // matching status-returning destroy function in MapHandle.close.
    let state = unsafe { maplibre_core::handle::NativeHandleState::from_raw(ptr, "mln_map") };
    Ok(MapHandle {
        state: Mutex::new(state),
    })
}

/// Private PyO3 extension for the public maplibre_native package.
#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<RuntimeHandle>()?;
    module.add_class::<MapHandle>()?;
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
    module.add_function(wrap_pyfunction!(create_map, module)?)?;
    Ok(())
}

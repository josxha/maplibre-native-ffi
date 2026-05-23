#![deny(unsafe_op_in_unsafe_fn)]

use maplibre_native_core::{
    self as maplibre_core, Error, ErrorKind, NetworkStatus, RenderMode, RuntimeEventPayload,
    RuntimeEventType, TileOperation,
};
use maplibre_native_sys as sys;
use pyo3::buffer::PyBuffer;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyDict};
use std::ffi::c_void;
use std::sync::{Arc, Mutex, MutexGuard};

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

struct RenderSessionState {
    handle: maplibre_core::handle::NativeHandleState<sys::mln_render_session>,
    detached: bool,
    frame_acquired: bool,
}

#[pyclass(name = "_RenderSessionHandle")]
struct RenderSessionHandle {
    state: Arc<Mutex<RenderSessionState>>,
}

#[pyclass(name = "_DetachedRenderSessionHandle")]
struct DetachedRenderSessionHandle {
    state: Arc<Mutex<RenderSessionState>>,
}

#[derive(Debug, Clone, Copy)]
struct MetalOwnedTextureFrameRaw {
    generation: u64,
    width: u32,
    height: u32,
    scale_factor: f64,
    frame_id: u64,
    texture_address: usize,
    device_address: usize,
    pixel_format: u64,
}

#[pyclass(name = "_MetalOwnedTextureFrameHandle")]
struct MetalOwnedTextureFrameHandle {
    session: Arc<Mutex<RenderSessionState>>,
    raw: MetalOwnedTextureFrameRaw,
    closed: Mutex<bool>,
}

#[derive(Debug, Clone, Copy)]
struct VulkanOwnedTextureFrameRaw {
    generation: u64,
    width: u32,
    height: u32,
    scale_factor: f64,
    frame_id: u64,
    image_address: usize,
    image_view_address: usize,
    device_address: usize,
    format: u32,
    layout: u32,
}

#[pyclass(name = "_VulkanOwnedTextureFrameHandle")]
struct VulkanOwnedTextureFrameHandle {
    session: Arc<Mutex<RenderSessionState>>,
    raw: VulkanOwnedTextureFrameRaw,
    closed: Mutex<bool>,
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

impl RenderSessionState {
    fn new(ptr: std::ptr::NonNull<sys::mln_render_session>) -> Self {
        // SAFETY: ptr came from a successful render-session attach function and
        // is paired with mln_render_session_destroy in close.
        let handle = unsafe {
            maplibre_core::handle::NativeHandleState::from_raw(ptr, "mln_render_session")
        };
        Self {
            handle,
            detached: false,
            frame_acquired: false,
        }
    }

    fn as_ptr(&self) -> *mut sys::mln_render_session {
        self.handle.as_ptr()
    }

    fn is_closed(&self) -> bool {
        self.handle.is_closed()
    }

    fn ensure_no_frame_acquired(&self) -> PyResult<()> {
        if self.frame_acquired {
            Err(invalid_state_error(
                "render session has an acquired texture frame",
            ))
        } else {
            Ok(())
        }
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

#[pymethods]
impl RenderSessionHandle {
    fn close(&self) -> PyResult<()> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.is_closed() {
            return Ok(());
        }
        state.ensure_no_frame_acquired()?;
        // SAFETY: state owns an mln_render_session pointer created by an attach
        // function and pairs it with the matching status-returning destroy.
        unsafe { state.handle.close_status(sys::mln_render_session_destroy) }.map_err(map_error)
    }

    fn resize(&self, width: u32, height: u32, scale_factor: f64) -> PyResult<()> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.ensure_no_frame_acquired()?;
        // SAFETY: The C API validates that the pointer is live, attached, and
        // called on the owner thread.
        maplibre_core::check(unsafe {
            sys::mln_render_session_resize(state.as_ptr(), width, height, scale_factor)
        })
        .map_err(map_error)
    }

    fn render_update(&self) -> PyResult<()> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.ensure_no_frame_acquired()?;
        // SAFETY: The C API validates the render-session pointer and state.
        maplibre_core::check(unsafe { sys::mln_render_session_render_update(state.as_ptr()) })
            .map_err(map_error)
    }

    fn detach(&self) -> PyResult<DetachedRenderSessionHandle> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.ensure_no_frame_acquired()?;
        // SAFETY: The C API validates the render-session pointer and state.
        maplibre_core::check(unsafe { sys::mln_render_session_detach(state.as_ptr()) })
            .map_err(map_error)?;
        state.detached = true;
        drop(state);
        Ok(DetachedRenderSessionHandle {
            state: Arc::clone(&self.state),
        })
    }

    fn reduce_memory_use(&self) -> PyResult<()> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.ensure_no_frame_acquired()?;
        // SAFETY: The C API validates the render-session pointer and state.
        maplibre_core::check(unsafe { sys::mln_render_session_reduce_memory_use(state.as_ptr()) })
            .map_err(map_error)
    }

    fn clear_data(&self) -> PyResult<()> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.ensure_no_frame_acquired()?;
        // SAFETY: The C API validates the render-session pointer and state.
        maplibre_core::check(unsafe { sys::mln_render_session_clear_data(state.as_ptr()) })
            .map_err(map_error)
    }

    fn dump_debug_logs(&self) -> PyResult<()> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.ensure_no_frame_acquired()?;
        // SAFETY: The C API validates the render-session pointer and state.
        maplibre_core::check(unsafe { sys::mln_render_session_dump_debug_logs(state.as_ptr()) })
            .map_err(map_error)
    }

    fn texture_image_info(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.ensure_no_frame_acquired()?;
        let info = probe_texture_image_info(state.as_ptr())?;
        texture_image_info_to_py(py, info)
    }

    fn read_premultiplied_rgba8(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.ensure_no_frame_acquired()?;
        let info = probe_texture_image_info(state.as_ptr())?;
        let mut data = vec![0; info.byte_length];
        let info = read_texture_image_into(state.as_ptr(), &mut data)?;
        image_to_py(py, info, &data)
    }

    fn read_premultiplied_rgba8_into(
        &self,
        py: Python<'_>,
        buffer: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        let py_buffer = PyBuffer::<u8>::get(buffer).map_err(|error| {
            invalid_argument_error(format!("expected writable contiguous u8 buffer: {error}"))
        })?;
        let Some(cells) = py_buffer.as_mut_slice(py) else {
            return Err(invalid_argument_error(
                "expected writable contiguous u8 buffer",
            ));
        };
        let data = if cells.is_empty() {
            std::ptr::null_mut()
        } else {
            cells.as_ptr().cast::<u8>().cast_mut()
        };
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.ensure_no_frame_acquired()?;
        // SAFETY: data points to the writable contiguous Python buffer borrowed
        // above for cells.len() u8 elements, or is null when the buffer is empty.
        let info = read_texture_image_raw(state.as_ptr(), data, cells.len())?;
        texture_image_info_to_py(py, info)
    }

    fn acquire_metal_owned_texture_frame(&self) -> PyResult<MetalOwnedTextureFrameHandle> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.ensure_no_frame_acquired()?;
        let mut raw = empty_metal_owned_texture_frame();
        // SAFETY: raw points to initialized writable frame storage, and the C
        // API validates the session pointer and texture-session state.
        maplibre_core::check(unsafe {
            sys::mln_metal_owned_texture_acquire_frame(state.as_ptr(), &mut raw)
        })
        .map_err(map_error)?;
        state.frame_acquired = true;
        Ok(MetalOwnedTextureFrameHandle {
            session: Arc::clone(&self.state),
            raw: MetalOwnedTextureFrameRaw::from_native(&raw),
            closed: Mutex::new(false),
        })
    }

    fn acquire_vulkan_owned_texture_frame(&self) -> PyResult<VulkanOwnedTextureFrameHandle> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.ensure_no_frame_acquired()?;
        let mut raw = empty_vulkan_owned_texture_frame();
        // SAFETY: raw points to initialized writable frame storage, and the C
        // API validates the session pointer and texture-session state.
        maplibre_core::check(unsafe {
            sys::mln_vulkan_owned_texture_acquire_frame(state.as_ptr(), &mut raw)
        })
        .map_err(map_error)?;
        state.frame_acquired = true;
        Ok(VulkanOwnedTextureFrameHandle {
            session: Arc::clone(&self.state),
            raw: VulkanOwnedTextureFrameRaw::from_native(&raw),
            closed: Mutex::new(false),
        })
    }

    #[getter]
    fn closed(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_closed()
    }

    #[getter]
    fn detached(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .detached
    }
}

#[pymethods]
impl DetachedRenderSessionHandle {
    fn close(&self) -> PyResult<()> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.is_closed() {
            return Ok(());
        }
        state.ensure_no_frame_acquired()?;
        // SAFETY: state owns an mln_render_session pointer created by an attach
        // function and pairs it with the matching status-returning destroy.
        unsafe { state.handle.close_status(sys::mln_render_session_destroy) }.map_err(map_error)
    }

    #[getter]
    fn closed(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_closed()
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

fn invalid_argument_error(diagnostic: impl Into<String>) -> PyErr {
    py_errors::InvalidArgumentError::new_err((Option::<i32>::None, diagnostic.into()))
}

fn invalid_state_error(diagnostic: impl Into<String>) -> PyErr {
    py_errors::InvalidStateError::new_err((Option::<i32>::None, diagnostic.into()))
}

fn texture_image_info_to_py(
    py: Python<'_>,
    info: maplibre_core::TextureImageInfo,
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("width", info.width)?;
    dict.set_item("height", info.height)?;
    dict.set_item("stride", info.stride)?;
    dict.set_item("byte_length", info.byte_length)?;
    Ok(dict.into_any().unbind())
}

fn image_to_py(
    py: Python<'_>,
    info: maplibre_core::TextureImageInfo,
    data: &[u8],
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("info", texture_image_info_to_py(py, info)?)?;
    dict.set_item("data", PyBytes::new(py, data))?;
    Ok(dict.into_any().unbind())
}

fn probe_texture_image_info(
    session: *mut sys::mln_render_session,
) -> PyResult<maplibre_core::TextureImageInfo> {
    // SAFETY: Default constructor takes no arguments and initializes size.
    let mut info = unsafe { sys::mln_texture_image_info_default() };
    // SAFETY: The C API validates session. Null data and zero capacity are the
    // documented metadata probe path, with info pointing to initialized storage.
    let status = unsafe {
        sys::mln_texture_read_premultiplied_rgba8(session, std::ptr::null_mut(), 0, &mut info)
    };
    if status == sys::MLN_STATUS_OK
        || (status == sys::MLN_STATUS_INVALID_ARGUMENT && info.byte_length > 0)
    {
        Ok(maplibre_core::values::texture_image_info_from_native(&info))
    } else {
        Err(map_error(Error::from_status(status)))
    }
}

fn read_texture_image_raw(
    session: *mut sys::mln_render_session,
    data: *mut u8,
    capacity: usize,
) -> PyResult<maplibre_core::TextureImageInfo> {
    // SAFETY: Default constructor takes no arguments and initializes size.
    let mut info = unsafe { sys::mln_texture_image_info_default() };
    // SAFETY: The caller guarantees data points to capacity writable bytes or
    // is null for an empty buffer. The C API validates session and capacity.
    maplibre_core::check(unsafe {
        sys::mln_texture_read_premultiplied_rgba8(session, data, capacity, &mut info)
    })
    .map_err(map_error)?;
    Ok(maplibre_core::values::texture_image_info_from_native(&info))
}

fn read_texture_image_into(
    session: *mut sys::mln_render_session,
    data: &mut [u8],
) -> PyResult<maplibre_core::TextureImageInfo> {
    let data_ptr = if data.is_empty() {
        std::ptr::null_mut()
    } else {
        data.as_mut_ptr()
    };
    read_texture_image_raw(session, data_ptr, data.len())
}

impl MetalOwnedTextureFrameRaw {
    fn from_native(raw: &sys::mln_metal_owned_texture_frame) -> Self {
        Self {
            generation: raw.generation,
            width: raw.width,
            height: raw.height,
            scale_factor: raw.scale_factor,
            frame_id: raw.frame_id,
            texture_address: raw.texture as usize,
            device_address: raw.device as usize,
            pixel_format: raw.pixel_format,
        }
    }

    fn to_native(self) -> sys::mln_metal_owned_texture_frame {
        sys::mln_metal_owned_texture_frame {
            size: std::mem::size_of::<sys::mln_metal_owned_texture_frame>() as u32,
            generation: self.generation,
            width: self.width,
            height: self.height,
            scale_factor: self.scale_factor,
            frame_id: self.frame_id,
            texture: self.texture_address as *mut c_void,
            device: self.device_address as *mut c_void,
            pixel_format: self.pixel_format,
        }
    }
}

impl VulkanOwnedTextureFrameRaw {
    fn from_native(raw: &sys::mln_vulkan_owned_texture_frame) -> Self {
        Self {
            generation: raw.generation,
            width: raw.width,
            height: raw.height,
            scale_factor: raw.scale_factor,
            frame_id: raw.frame_id,
            image_address: raw.image as usize,
            image_view_address: raw.image_view as usize,
            device_address: raw.device as usize,
            format: raw.format,
            layout: raw.layout,
        }
    }

    fn to_native(self) -> sys::mln_vulkan_owned_texture_frame {
        sys::mln_vulkan_owned_texture_frame {
            size: std::mem::size_of::<sys::mln_vulkan_owned_texture_frame>() as u32,
            generation: self.generation,
            width: self.width,
            height: self.height,
            scale_factor: self.scale_factor,
            frame_id: self.frame_id,
            image: self.image_address as *mut c_void,
            image_view: self.image_view_address as *mut c_void,
            device: self.device_address as *mut c_void,
            format: self.format,
            layout: self.layout,
        }
    }
}

fn empty_metal_owned_texture_frame() -> sys::mln_metal_owned_texture_frame {
    sys::mln_metal_owned_texture_frame {
        size: std::mem::size_of::<sys::mln_metal_owned_texture_frame>() as u32,
        generation: 0,
        width: 0,
        height: 0,
        scale_factor: 0.0,
        frame_id: 0,
        texture: std::ptr::null_mut(),
        device: std::ptr::null_mut(),
        pixel_format: 0,
    }
}

fn empty_vulkan_owned_texture_frame() -> sys::mln_vulkan_owned_texture_frame {
    sys::mln_vulkan_owned_texture_frame {
        size: std::mem::size_of::<sys::mln_vulkan_owned_texture_frame>() as u32,
        generation: 0,
        width: 0,
        height: 0,
        scale_factor: 0.0,
        frame_id: 0,
        image: std::ptr::null_mut(),
        image_view: std::ptr::null_mut(),
        device: std::ptr::null_mut(),
        format: 0,
        layout: 0,
    }
}

#[pymethods]
impl MetalOwnedTextureFrameHandle {
    fn close(&self) -> PyResult<()> {
        let mut closed = self
            .closed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *closed {
            return Ok(());
        }
        let mut session = self
            .session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let raw = self.raw.to_native();
        // SAFETY: raw reconstructs the frame returned by the successful native
        // acquire call for this session and has not been released yet.
        maplibre_core::check(unsafe {
            sys::mln_metal_owned_texture_release_frame(session.as_ptr(), &raw)
        })
        .map_err(map_error)?;
        session.frame_acquired = false;
        *closed = true;
        Ok(())
    }

    fn frame(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if *self
            .closed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
        {
            return Err(invalid_state_error(
                "MetalOwnedTextureFrameHandle is closed",
            ));
        }
        let dict = PyDict::new(py);
        dict.set_item("generation", self.raw.generation)?;
        dict.set_item("width", self.raw.width)?;
        dict.set_item("height", self.raw.height)?;
        dict.set_item("scale_factor", self.raw.scale_factor)?;
        dict.set_item("frame_id", self.raw.frame_id)?;
        dict.set_item("pixel_format", self.raw.pixel_format)?;
        Ok(dict.into_any().unbind())
    }

    fn texture_address(&self) -> PyResult<usize> {
        if *self
            .closed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
        {
            Err(invalid_state_error(
                "MetalOwnedTextureFrameHandle is closed",
            ))
        } else {
            Ok(self.raw.texture_address)
        }
    }

    fn device_address(&self) -> PyResult<usize> {
        if *self
            .closed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
        {
            Err(invalid_state_error(
                "MetalOwnedTextureFrameHandle is closed",
            ))
        } else {
            Ok(self.raw.device_address)
        }
    }

    #[getter]
    fn closed(&self) -> bool {
        *self
            .closed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for MetalOwnedTextureFrameHandle {
    fn drop(&mut self) {
        let Ok(mut closed) = self.closed.lock() else {
            return;
        };
        if *closed {
            return;
        }
        let Ok(mut session) = self.session.lock() else {
            return;
        };
        let raw = self.raw.to_native();
        // SAFETY: Best-effort release of the active frame. Drop cannot report
        // errors and never panics.
        let status = unsafe { sys::mln_metal_owned_texture_release_frame(session.as_ptr(), &raw) };
        if status == sys::MLN_STATUS_OK {
            session.frame_acquired = false;
            *closed = true;
        }
    }
}

#[pymethods]
impl VulkanOwnedTextureFrameHandle {
    fn close(&self) -> PyResult<()> {
        let mut closed = self
            .closed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *closed {
            return Ok(());
        }
        let mut session = self
            .session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let raw = self.raw.to_native();
        // SAFETY: raw reconstructs the frame returned by the successful native
        // acquire call for this session and has not been released yet.
        maplibre_core::check(unsafe {
            sys::mln_vulkan_owned_texture_release_frame(session.as_ptr(), &raw)
        })
        .map_err(map_error)?;
        session.frame_acquired = false;
        *closed = true;
        Ok(())
    }

    fn frame(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if *self
            .closed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
        {
            return Err(invalid_state_error(
                "VulkanOwnedTextureFrameHandle is closed",
            ));
        }
        let dict = PyDict::new(py);
        dict.set_item("generation", self.raw.generation)?;
        dict.set_item("width", self.raw.width)?;
        dict.set_item("height", self.raw.height)?;
        dict.set_item("scale_factor", self.raw.scale_factor)?;
        dict.set_item("frame_id", self.raw.frame_id)?;
        dict.set_item("format", self.raw.format)?;
        dict.set_item("layout", self.raw.layout)?;
        Ok(dict.into_any().unbind())
    }

    fn image_address(&self) -> PyResult<usize> {
        if *self
            .closed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
        {
            Err(invalid_state_error(
                "VulkanOwnedTextureFrameHandle is closed",
            ))
        } else {
            Ok(self.raw.image_address)
        }
    }

    fn image_view_address(&self) -> PyResult<usize> {
        if *self
            .closed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
        {
            Err(invalid_state_error(
                "VulkanOwnedTextureFrameHandle is closed",
            ))
        } else {
            Ok(self.raw.image_view_address)
        }
    }

    fn device_address(&self) -> PyResult<usize> {
        if *self
            .closed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
        {
            Err(invalid_state_error(
                "VulkanOwnedTextureFrameHandle is closed",
            ))
        } else {
            Ok(self.raw.device_address)
        }
    }

    #[getter]
    fn closed(&self) -> bool {
        *self
            .closed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for VulkanOwnedTextureFrameHandle {
    fn drop(&mut self) {
        let Ok(mut closed) = self.closed.lock() else {
            return;
        };
        if *closed {
            return;
        }
        let Ok(mut session) = self.session.lock() else {
            return;
        };
        let raw = self.raw.to_native();
        // SAFETY: Best-effort release of the active frame. Drop cannot report
        // errors and never panics.
        let status = unsafe { sys::mln_vulkan_owned_texture_release_frame(session.as_ptr(), &raw) };
        if status == sys::MLN_STATUS_OK {
            session.frame_acquired = false;
            *closed = true;
        }
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

fn attach_render_session<F>(map: &MapHandle, attach: F) -> PyResult<RenderSessionHandle>
where
    F: FnOnce(*mut sys::mln_map, *mut *mut sys::mln_render_session) -> sys::mln_status,
{
    let map_state = map.state();
    let mut out = maplibre_core::ptr::OutPtr::<sys::mln_render_session>::new();
    maplibre_core::check(attach(map_state.as_ptr(), out.as_mut_ptr())).map_err(map_error)?;
    let ptr = out.into_non_null("mln_render_session").map_err(map_error)?;
    Ok(RenderSessionHandle {
        state: Arc::new(Mutex::new(RenderSessionState::new(ptr))),
    })
}

#[pyfunction]
fn attach_metal_surface(
    map: &MapHandle,
    width: u32,
    height: u32,
    scale_factor: f64,
    device_address: usize,
    layer_address: usize,
) -> PyResult<RenderSessionHandle> {
    let descriptor = maplibre_core::render::metal_surface_descriptor_to_native(
        maplibre_core::render::MetalSurfaceDescriptorFields {
            extent: maplibre_core::render::RenderTargetExtentFields {
                width,
                height,
                scale_factor,
            },
            context: maplibre_core::render::MetalContextDescriptorFields {
                device: device_address as *mut c_void,
            },
            layer: layer_address as *mut c_void,
        },
    );
    attach_render_session(map, |map_ptr, out| {
        // SAFETY: descriptor is fully initialized and lives for this call. The C
        // API validates the map pointer, descriptor fields, and out pointer.
        unsafe { sys::mln_metal_surface_attach(map_ptr, &descriptor, out) }
    })
}

#[pyfunction]
fn attach_vulkan_surface(
    map: &MapHandle,
    width: u32,
    height: u32,
    scale_factor: f64,
    instance_address: usize,
    physical_device_address: usize,
    device_address: usize,
    graphics_queue_address: usize,
    graphics_queue_family_index: u32,
    surface_address: usize,
) -> PyResult<RenderSessionHandle> {
    let descriptor = maplibre_core::render::vulkan_surface_descriptor_to_native(
        maplibre_core::render::VulkanSurfaceDescriptorFields {
            extent: maplibre_core::render::RenderTargetExtentFields {
                width,
                height,
                scale_factor,
            },
            context: vulkan_context_fields(
                instance_address,
                physical_device_address,
                device_address,
                graphics_queue_address,
                graphics_queue_family_index,
            ),
            surface: surface_address as *mut c_void,
        },
    );
    attach_render_session(map, |map_ptr, out| {
        // SAFETY: descriptor is fully initialized and lives for this call. The C
        // API validates the map pointer, descriptor fields, and out pointer.
        unsafe { sys::mln_vulkan_surface_attach(map_ptr, &descriptor, out) }
    })
}

#[pyfunction]
fn attach_metal_owned_texture(
    map: &MapHandle,
    width: u32,
    height: u32,
    scale_factor: f64,
    device_address: usize,
) -> PyResult<RenderSessionHandle> {
    let descriptor = maplibre_core::render::metal_owned_texture_descriptor_to_native(
        maplibre_core::render::MetalOwnedTextureDescriptorFields {
            extent: maplibre_core::render::RenderTargetExtentFields {
                width,
                height,
                scale_factor,
            },
            context: maplibre_core::render::MetalContextDescriptorFields {
                device: device_address as *mut c_void,
            },
        },
    );
    attach_render_session(map, |map_ptr, out| {
        // SAFETY: descriptor is fully initialized and lives for this call. The C
        // API validates the map pointer, descriptor fields, and out pointer.
        unsafe { sys::mln_metal_owned_texture_attach(map_ptr, &descriptor, out) }
    })
}

#[pyfunction]
fn attach_metal_borrowed_texture(
    map: &MapHandle,
    width: u32,
    height: u32,
    scale_factor: f64,
    texture_address: usize,
) -> PyResult<RenderSessionHandle> {
    let descriptor = maplibre_core::render::metal_borrowed_texture_descriptor_to_native(
        maplibre_core::render::MetalBorrowedTextureDescriptorFields {
            extent: maplibre_core::render::RenderTargetExtentFields {
                width,
                height,
                scale_factor,
            },
            texture: texture_address as *mut c_void,
        },
    );
    attach_render_session(map, |map_ptr, out| {
        // SAFETY: descriptor is fully initialized and lives for this call. The C
        // API validates the map pointer, descriptor fields, and out pointer.
        unsafe { sys::mln_metal_borrowed_texture_attach(map_ptr, &descriptor, out) }
    })
}

#[pyfunction]
fn attach_vulkan_owned_texture(
    map: &MapHandle,
    width: u32,
    height: u32,
    scale_factor: f64,
    instance_address: usize,
    physical_device_address: usize,
    device_address: usize,
    graphics_queue_address: usize,
    graphics_queue_family_index: u32,
) -> PyResult<RenderSessionHandle> {
    let descriptor = maplibre_core::render::vulkan_owned_texture_descriptor_to_native(
        maplibre_core::render::VulkanOwnedTextureDescriptorFields {
            extent: maplibre_core::render::RenderTargetExtentFields {
                width,
                height,
                scale_factor,
            },
            context: vulkan_context_fields(
                instance_address,
                physical_device_address,
                device_address,
                graphics_queue_address,
                graphics_queue_family_index,
            ),
        },
    );
    attach_render_session(map, |map_ptr, out| {
        // SAFETY: descriptor is fully initialized and lives for this call. The C
        // API validates the map pointer, descriptor fields, and out pointer.
        unsafe { sys::mln_vulkan_owned_texture_attach(map_ptr, &descriptor, out) }
    })
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn attach_vulkan_borrowed_texture(
    map: &MapHandle,
    width: u32,
    height: u32,
    scale_factor: f64,
    instance_address: usize,
    physical_device_address: usize,
    device_address: usize,
    graphics_queue_address: usize,
    graphics_queue_family_index: u32,
    image_address: usize,
    image_view_address: usize,
    format: u32,
    initial_layout: u32,
    final_layout: u32,
) -> PyResult<RenderSessionHandle> {
    let descriptor = maplibre_core::render::vulkan_borrowed_texture_descriptor_to_native(
        maplibre_core::render::VulkanBorrowedTextureDescriptorFields {
            extent: maplibre_core::render::RenderTargetExtentFields {
                width,
                height,
                scale_factor,
            },
            context: vulkan_context_fields(
                instance_address,
                physical_device_address,
                device_address,
                graphics_queue_address,
                graphics_queue_family_index,
            ),
            image: image_address as *mut c_void,
            image_view: image_view_address as *mut c_void,
            format,
            initial_layout,
            final_layout,
        },
    );
    attach_render_session(map, |map_ptr, out| {
        // SAFETY: descriptor is fully initialized and lives for this call. The C
        // API validates the map pointer, descriptor fields, and out pointer.
        unsafe { sys::mln_vulkan_borrowed_texture_attach(map_ptr, &descriptor, out) }
    })
}

fn vulkan_context_fields(
    instance_address: usize,
    physical_device_address: usize,
    device_address: usize,
    graphics_queue_address: usize,
    graphics_queue_family_index: u32,
) -> maplibre_core::render::VulkanContextDescriptorFields {
    maplibre_core::render::VulkanContextDescriptorFields {
        instance: instance_address as *mut c_void,
        physical_device: physical_device_address as *mut c_void,
        device: device_address as *mut c_void,
        graphics_queue: graphics_queue_address as *mut c_void,
        graphics_queue_family_index,
    }
}

/// Private PyO3 extension for the public maplibre_native package.
#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<RuntimeHandle>()?;
    module.add_class::<MapHandle>()?;
    module.add_class::<RenderSessionHandle>()?;
    module.add_class::<DetachedRenderSessionHandle>()?;
    module.add_class::<MetalOwnedTextureFrameHandle>()?;
    module.add_class::<VulkanOwnedTextureFrameHandle>()?;
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
    module.add_function(wrap_pyfunction!(attach_metal_surface, module)?)?;
    module.add_function(wrap_pyfunction!(attach_vulkan_surface, module)?)?;
    module.add_function(wrap_pyfunction!(attach_metal_owned_texture, module)?)?;
    module.add_function(wrap_pyfunction!(attach_metal_borrowed_texture, module)?)?;
    module.add_function(wrap_pyfunction!(attach_vulkan_owned_texture, module)?)?;
    module.add_function(wrap_pyfunction!(attach_vulkan_borrowed_texture, module)?)?;
    Ok(())
}

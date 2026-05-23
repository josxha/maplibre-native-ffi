use std::ffi::{CStr, CString, c_void};
use std::os::raw::c_char;
use std::sync::{Arc, Mutex, mpsc};

use maplibre_native_core::{self as core, handle::NativeHandleState};
use maplibre_native_sys as sys;
use napi::bindgen_prelude::{BigInt, Result};
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;

use crate::error;

#[napi(object)]
pub struct RuntimeOptions {
    pub asset_path: Option<String>,
    pub cache_path: Option<String>,
    pub maximum_cache_size: Option<BigInt>,
}

#[napi(object)]
pub struct OfflineOperationStart {
    pub operation_id: String,
}

#[napi(object)]
pub struct RuntimeEvent {
    pub event_type: String,
    pub raw_event_type: u32,
    pub source_type: String,
    pub raw_source_type: u32,
    pub source_address: String,
    pub code: i32,
    pub message: Option<String>,
    pub payload_kind: String,
}

#[napi(object)]
pub struct ResourceTransformRequest {
    pub kind: String,
    pub raw_kind: u32,
    pub url: String,
}

struct ResourceTransformState {
    callback: ThreadsafeFunction<ResourceTransformRequest, Option<String>>,
    replacements: Mutex<Vec<CString>>,
}

#[napi(js_name = "NativeRuntimeHandle")]
pub struct NativeRuntimeHandle {
    state: NativeHandleState<sys::mln_runtime>,
    resource_transform: Mutex<Option<Arc<ResourceTransformState>>>,
}

#[napi(js_name = "createNativeRuntimeHandle")]
pub fn create_native_runtime_handle(
    options: Option<RuntimeOptions>,
) -> Result<NativeRuntimeHandle> {
    let options = options.unwrap_or_default().into_core()?;
    let native_options =
        core::runtime::runtime_options_to_native(&options).map_err(error::from_core)?;
    let mut runtime = std::ptr::null_mut();

    core::check(unsafe { sys::mln_runtime_create(&native_options.to_raw(), &mut runtime) })
        .map_err(error::from_core)?;
    let state = unsafe { NativeHandleState::from_raw_ptr(runtime, "RuntimeHandle") }
        .map_err(error::from_core)?;
    Ok(NativeRuntimeHandle {
        state,
        resource_transform: Mutex::new(None),
    })
}

#[napi]
impl NativeRuntimeHandle {
    #[napi]
    pub fn close(&self) -> Result<()> {
        unsafe { self.state.close_status(sys::mln_runtime_destroy) }.map_err(error::from_core)
    }

    #[napi(getter)]
    pub fn closed(&self) -> bool {
        self.state.is_closed()
    }

    #[napi(js_name = "runOnce")]
    pub fn run_once(&self) -> Result<()> {
        core::check(unsafe { sys::mln_runtime_run_once(self.state.as_ptr()) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "setResourceTransform")]
    pub fn set_resource_transform(
        &self,
        callback: ThreadsafeFunction<ResourceTransformRequest, Option<String>>,
    ) -> Result<()> {
        let transform = Arc::new(ResourceTransformState {
            callback,
            replacements: Mutex::new(Vec::new()),
        });
        let descriptor = core::resource::resource_transform_descriptor(
            Some(resource_transform_trampoline),
            Arc::as_ptr(&transform) as *mut c_void,
        );
        core::check(unsafe {
            sys::mln_runtime_set_resource_transform(self.state.as_ptr(), &descriptor)
        })
        .map_err(error::from_core)?;
        *self
            .resource_transform
            .lock()
            .map_err(|_| error::invalid_argument("resource transform state lock is poisoned"))? =
            Some(transform);
        Ok(())
    }

    #[napi(js_name = "clearResourceTransform")]
    pub fn clear_resource_transform(&self) -> Result<()> {
        core::check(unsafe { sys::mln_runtime_clear_resource_transform(self.state.as_ptr()) })
            .map_err(error::from_core)?;
        *self
            .resource_transform
            .lock()
            .map_err(|_| error::invalid_argument("resource transform state lock is poisoned"))? =
            None;
        Ok(())
    }

    #[napi(js_name = "runAmbientCacheOperation")]
    pub fn run_ambient_cache_operation(&self, operation: String) -> Result<OfflineOperationStart> {
        let operation = ambient_cache_operation_from_string(&operation)?;
        let mut operation_id = 0;
        core::check(unsafe {
            sys::mln_runtime_run_ambient_cache_operation_start(
                self.state.as_ptr(),
                operation,
                &mut operation_id,
            )
        })
        .map_err(error::from_core)?;
        Ok(OfflineOperationStart {
            operation_id: operation_id.to_string(),
        })
    }

    #[napi(js_name = "discardOfflineOperation")]
    pub fn discard_offline_operation(&self, operation_id: BigInt) -> Result<()> {
        core::check(unsafe {
            sys::mln_runtime_offline_operation_discard(
                self.state.as_ptr(),
                bigint_to_u64(operation_id, "operationId")?,
            )
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "pollEvent")]
    pub fn poll_event(&self) -> Result<Option<RuntimeEvent>> {
        let mut raw = core::events::empty_runtime_event();
        let mut has_event = false;
        core::check(unsafe {
            sys::mln_runtime_poll_event(self.state.as_ptr(), &mut raw, &mut has_event)
        })
        .map_err(error::from_core)?;
        if !has_event {
            return Ok(None);
        }

        let copied =
            unsafe { core::events::runtime_event_from_native(&raw) }.map_err(error::from_core)?;
        Ok(Some(RuntimeEvent::from_copied(copied, raw.type_)))
    }
}

unsafe extern "C" fn resource_transform_trampoline(
    user_data: *mut c_void,
    kind: u32,
    url: *const c_char,
    out_response: *mut sys::mln_resource_transform_response,
) -> sys::mln_status {
    let init_status =
        unsafe { core::resource::initialize_resource_transform_response(out_response) };
    if init_status != sys::MLN_STATUS_OK {
        return init_status;
    }
    if user_data.is_null() || url.is_null() {
        return sys::MLN_STATUS_INVALID_ARGUMENT;
    }

    let transform = unsafe { &*(user_data as *const ResourceTransformState) };
    let url = unsafe { CStr::from_ptr(url) }
        .to_string_lossy()
        .into_owned();
    let request = ResourceTransformRequest {
        kind: resource_kind_name(core::enums::resource_kind_from_raw(kind)).to_owned(),
        raw_kind: kind,
        url,
    };
    let (sender, receiver) = mpsc::sync_channel(1);
    let status = transform.callback.call_with_return_value(
        Ok(request),
        ThreadsafeFunctionCallMode::Blocking,
        move |result, _env| {
            let _ = sender.send(result.ok().flatten());
            Ok(())
        },
    );
    if !matches!(status, napi::Status::Ok) {
        return sys::MLN_STATUS_OK;
    }

    let Ok(Some(replacement)) = receiver.recv() else {
        return sys::MLN_STATUS_OK;
    };
    if replacement.is_empty() {
        return sys::MLN_STATUS_OK;
    }
    let Ok(replacement) = CString::new(replacement) else {
        return sys::MLN_STATUS_OK;
    };
    let Ok(mut replacements) = transform.replacements.lock() else {
        return sys::MLN_STATUS_OK;
    };
    replacements.push(replacement);
    if let Some(replacement) = replacements.last() {
        unsafe {
            (*out_response).url = replacement.as_ptr();
        }
    }
    sys::MLN_STATUS_OK
}

impl RuntimeEvent {
    fn from_copied(event: core::CopiedRuntimeEvent, raw_event_type: u32) -> Self {
        Self {
            event_type: runtime_event_type_name(event.event_type).to_owned(),
            raw_event_type,
            source_type: runtime_event_source_type_name(event.source.source_type).to_owned(),
            raw_source_type: event.source.source_type,
            source_address: event.source.source_address.to_string(),
            code: event.code,
            message: event.message,
            payload_kind: runtime_event_payload_kind(&event.payload).to_owned(),
        }
    }
}

fn resource_kind_name(kind: core::ResourceKind) -> &'static str {
    match kind {
        core::ResourceKind::Unknown => "unknown",
        core::ResourceKind::Style => "style",
        core::ResourceKind::Source => "source",
        core::ResourceKind::Tile => "tile",
        core::ResourceKind::Glyphs => "glyphs",
        core::ResourceKind::SpriteImage => "sprite-image",
        core::ResourceKind::SpriteJson => "sprite-json",
        core::ResourceKind::Image => "image",
        core::ResourceKind::UnknownRaw(_) => "unknown",
        _ => "unknown",
    }
}

fn runtime_event_source_type_name(raw: u32) -> &'static str {
    match raw {
        sys::MLN_RUNTIME_EVENT_SOURCE_RUNTIME => "runtime",
        sys::MLN_RUNTIME_EVENT_SOURCE_MAP => "map",
        _ => "unknown",
    }
}

fn runtime_event_payload_kind(payload: &core::RuntimeEventPayload) -> &'static str {
    match payload {
        core::RuntimeEventPayload::None => "none",
        core::RuntimeEventPayload::RenderFrame(_) => "render-frame",
        core::RuntimeEventPayload::RenderMap(_) => "render-map",
        core::RuntimeEventPayload::StyleImageMissing(_) => "style-image-missing",
        core::RuntimeEventPayload::TileAction(_) => "tile-action",
        core::RuntimeEventPayload::OfflineRegionStatus(_) => "offline-region-status",
        core::RuntimeEventPayload::OfflineRegionResponseError(_) => "offline-region-response-error",
        core::RuntimeEventPayload::OfflineRegionTileCountLimit(_) => {
            "offline-region-tile-count-limit"
        }
        core::RuntimeEventPayload::OfflineOperationCompleted(_) => "offline-operation-completed",
        core::RuntimeEventPayload::Unknown(_) => "unknown",
        _ => "unknown",
    }
}

fn runtime_event_type_name(event_type: core::RuntimeEventType) -> &'static str {
    match event_type {
        core::RuntimeEventType::MapCameraWillChange => "map-camera-will-change",
        core::RuntimeEventType::MapCameraIsChanging => "map-camera-is-changing",
        core::RuntimeEventType::MapCameraDidChange => "map-camera-did-change",
        core::RuntimeEventType::MapStyleLoaded => "map-style-loaded",
        core::RuntimeEventType::MapLoadingStarted => "map-loading-started",
        core::RuntimeEventType::MapLoadingFinished => "map-loading-finished",
        core::RuntimeEventType::MapLoadingFailed => "map-loading-failed",
        core::RuntimeEventType::MapIdle => "map-idle",
        core::RuntimeEventType::MapRenderUpdateAvailable => "map-render-update-available",
        core::RuntimeEventType::MapRenderError => "map-render-error",
        core::RuntimeEventType::MapStillImageFinished => "map-still-image-finished",
        core::RuntimeEventType::MapStillImageFailed => "map-still-image-failed",
        core::RuntimeEventType::MapRenderFrameStarted => "map-render-frame-started",
        core::RuntimeEventType::MapRenderFrameFinished => "map-render-frame-finished",
        core::RuntimeEventType::MapRenderMapStarted => "map-render-map-started",
        core::RuntimeEventType::MapRenderMapFinished => "map-render-map-finished",
        core::RuntimeEventType::MapStyleImageMissing => "map-style-image-missing",
        core::RuntimeEventType::MapTileAction => "map-tile-action",
        core::RuntimeEventType::OfflineRegionStatusChanged => "offline-region-status-changed",
        core::RuntimeEventType::OfflineRegionResponseError => "offline-region-response-error",
        core::RuntimeEventType::OfflineRegionTileCountLimitExceeded => {
            "offline-region-tile-count-limit-exceeded"
        }
        core::RuntimeEventType::OfflineOperationCompleted => "offline-operation-completed",
        core::RuntimeEventType::Unknown(_) => "unknown",
        _ => "unknown",
    }
}

impl NativeRuntimeHandle {
    pub(crate) fn as_ptr(&self) -> *mut sys::mln_runtime {
        self.state.as_ptr()
    }
}

impl Drop for NativeRuntimeHandle {
    fn drop(&mut self) {
        let _ = self.state.leak_for_report();
    }
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            asset_path: None,
            cache_path: None,
            maximum_cache_size: None,
        }
    }
}

impl RuntimeOptions {
    fn into_core(self) -> Result<core::RuntimeOptions> {
        let mut options = core::RuntimeOptions::new();
        if let Some(asset_path) = self.asset_path {
            options = options.with_asset_path(asset_path);
        }
        if let Some(cache_path) = self.cache_path {
            options = options.with_cache_path(cache_path);
        }
        if let Some(maximum_cache_size) = self.maximum_cache_size {
            options =
                options.with_maximum_cache_size(maximum_cache_size_to_u64(maximum_cache_size)?);
        }
        Ok(options)
    }
}

fn ambient_cache_operation_from_string(operation: &str) -> Result<u32> {
    match operation {
        "resetDatabase" => Ok(sys::MLN_AMBIENT_CACHE_OPERATION_RESET_DATABASE),
        "packDatabase" => Ok(sys::MLN_AMBIENT_CACHE_OPERATION_PACK_DATABASE),
        "invalidate" => Ok(sys::MLN_AMBIENT_CACHE_OPERATION_INVALIDATE),
        "clear" => Ok(sys::MLN_AMBIENT_CACHE_OPERATION_CLEAR),
        other => Err(error::invalid_argument(format!(
            "ambient cache operation must be 'resetDatabase', 'packDatabase', 'invalidate', or 'clear', got '{other}'"
        ))),
    }
}

fn maximum_cache_size_to_u64(value: BigInt) -> Result<u64> {
    bigint_to_u64(value, "maximumCacheSize")
}

fn bigint_to_u64(value: BigInt, field_name: &str) -> Result<u64> {
    let (signed, value, lossless) = value.get_u64();
    if signed || !lossless {
        return Err(error::invalid_argument(format!(
            "{field_name} must be a non-negative 64-bit bigint"
        )));
    }
    Ok(value)
}

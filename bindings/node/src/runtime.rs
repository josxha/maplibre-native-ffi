use maplibre_native_core::{self as core, handle::NativeHandleState};
use maplibre_native_sys as sys;
use napi::bindgen_prelude::{BigInt, Result};
use napi_derive::napi;

use crate::error;

#[napi(object)]
pub struct RuntimeOptions {
    pub asset_path: Option<String>,
    pub cache_path: Option<String>,
    pub maximum_cache_size: Option<BigInt>,
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

#[napi(js_name = "NativeRuntimeHandle")]
pub struct NativeRuntimeHandle {
    state: NativeHandleState<sys::mln_runtime>,
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
    Ok(NativeRuntimeHandle { state })
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

fn maximum_cache_size_to_u64(value: BigInt) -> Result<u64> {
    let (signed, value, lossless) = value.get_u64();
    if signed || !lossless {
        return Err(error::invalid_argument(
            "maximumCacheSize must be a non-negative 64-bit bigint",
        ));
    }
    Ok(value)
}

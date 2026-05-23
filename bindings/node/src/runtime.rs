use std::collections::HashMap;
use std::ffi::{CStr, CString, c_void};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, mpsc};

use maplibre_native_core::{self as core, handle::NativeHandleState};
use maplibre_native_sys as sys;
use napi::bindgen_prelude::{BigInt, Result, Uint8Array};
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
pub struct OfflineRegionDefinitionInput {
    pub kind: String,
    pub style_url: String,
    pub bounds: Option<crate::values::LatLngBounds>,
    pub geometry: Option<String>,
    pub min_zoom: f64,
    pub max_zoom: f64,
    pub pixel_ratio: f64,
    pub include_ideographs: Option<bool>,
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

#[napi(object)]
pub struct ResourceByteRange {
    pub start: String,
    pub end: String,
}

#[napi(object)]
pub struct ResourceProviderRequest {
    pub url: String,
    pub kind: String,
    pub raw_kind: u32,
    pub loading_method: String,
    pub raw_loading_method: u32,
    pub priority: String,
    pub raw_priority: u32,
    pub usage: String,
    pub raw_usage: u32,
    pub storage_policy: String,
    pub raw_storage_policy: u32,
    pub range: Option<ResourceByteRange>,
    pub prior_modified_unix_ms: Option<i64>,
    pub prior_expires_unix_ms: Option<i64>,
    pub prior_etag: Option<String>,
    pub prior_data: Uint8Array,
    pub handle_id: String,
}

#[napi(object)]
pub struct ResourceResponseInput {
    pub status: Option<String>,
    pub error_reason: Option<String>,
    pub bytes: Option<Uint8Array>,
    pub error_message: Option<String>,
    pub must_revalidate: Option<bool>,
    pub modified_unix_ms: Option<i64>,
    pub expires_unix_ms: Option<i64>,
    pub etag: Option<String>,
    pub retry_after_unix_ms: Option<i64>,
}

static RESOURCE_REQUEST_HANDLE_IDS: AtomicU64 = AtomicU64::new(1);
static RESOURCE_REQUEST_HANDLES: OnceLock<
    Mutex<HashMap<u64, Arc<core::resource::ResourceRequestHandleState>>>,
> = OnceLock::new();

struct ResourceTransformState {
    callback: ThreadsafeFunction<ResourceTransformRequest, Option<String>>,
    replacements: Mutex<Vec<CString>>,
}

struct ResourceProviderState {
    callback: ThreadsafeFunction<ResourceProviderRequest, Option<String>>,
}

#[napi(js_name = "NativeRuntimeHandle")]
pub struct NativeRuntimeHandle {
    state: NativeHandleState<sys::mln_runtime>,
    resource_transform: Mutex<Option<Arc<ResourceTransformState>>>,
    resource_provider: Mutex<Option<Arc<ResourceProviderState>>>,
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
        resource_provider: Mutex::new(None),
    })
}

#[napi(js_name = "nativeResourceRequestComplete")]
pub fn native_resource_request_complete(
    handle_id: String,
    response: ResourceResponseInput,
) -> Result<()> {
    let handle_id = parse_resource_request_handle_id(&handle_id)?;
    let handle = resource_request_handles()
        .lock()
        .map_err(|_| error::invalid_argument("resource request registry lock is poisoned"))?
        .remove(&handle_id)
        .ok_or_else(|| error::invalid_argument("ResourceRequestHandle is closed"))?;
    handle
        .complete(&resource_response_from_input(response)?)
        .map_err(error::from_core)
}

#[napi(js_name = "nativeResourceRequestCancelled")]
pub fn native_resource_request_cancelled(handle_id: String) -> Result<bool> {
    let handle_id = parse_resource_request_handle_id(&handle_id)?;
    let handle = resource_request_handles()
        .lock()
        .map_err(|_| error::invalid_argument("resource request registry lock is poisoned"))?
        .get(&handle_id)
        .cloned()
        .ok_or_else(|| error::invalid_argument("ResourceRequestHandle is closed"))?;
    handle.is_cancelled().map_err(error::from_core)
}

#[napi(js_name = "nativeResourceRequestClose")]
pub fn native_resource_request_close(handle_id: String) -> Result<()> {
    let handle_id = parse_resource_request_handle_id(&handle_id)?;
    if let Some(handle) = resource_request_handles()
        .lock()
        .map_err(|_| error::invalid_argument("resource request registry lock is poisoned"))?
        .remove(&handle_id)
    {
        handle.close();
    }
    Ok(())
}

#[napi]
impl NativeRuntimeHandle {
    #[napi]
    pub fn close(&self) -> Result<()> {
        unsafe { self.state.close_status(sys::mln_runtime_destroy) }.map_err(error::from_core)?;
        if let Ok(mut transform) = self.resource_transform.lock() {
            *transform = None;
        }
        if let Ok(mut provider) = self.resource_provider.lock() {
            *provider = None;
        }
        Ok(())
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

    #[napi(js_name = "setResourceProvider")]
    pub fn set_resource_provider(
        &self,
        callback: ThreadsafeFunction<ResourceProviderRequest, Option<String>>,
    ) -> Result<()> {
        let provider = Arc::new(ResourceProviderState { callback });
        let descriptor = core::resource::resource_provider_descriptor(
            Some(resource_provider_trampoline),
            Arc::as_ptr(&provider) as *mut c_void,
        );
        core::check(unsafe {
            sys::mln_runtime_set_resource_provider(self.state.as_ptr(), &descriptor)
        })
        .map_err(error::from_core)?;
        *self
            .resource_provider
            .lock()
            .map_err(|_| error::invalid_argument("resource provider state lock is poisoned"))? =
            Some(provider);
        Ok(())
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
        Ok(offline_operation_start(operation_id))
    }

    #[napi(js_name = "offlineRegionsList")]
    pub fn offline_regions_list(&self) -> Result<OfflineOperationStart> {
        let mut operation_id = 0;
        core::check(unsafe {
            sys::mln_runtime_offline_regions_list_start(self.state.as_ptr(), &mut operation_id)
        })
        .map_err(error::from_core)?;
        Ok(offline_operation_start(operation_id))
    }

    #[napi(js_name = "offlineRegionGet")]
    pub fn offline_region_get(&self, region_id: BigInt) -> Result<OfflineOperationStart> {
        let mut operation_id = 0;
        core::check(unsafe {
            sys::mln_runtime_offline_region_get_start(
                self.state.as_ptr(),
                bigint_to_i64(region_id, "regionId")?,
                &mut operation_id,
            )
        })
        .map_err(error::from_core)?;
        Ok(offline_operation_start(operation_id))
    }

    #[napi(js_name = "offlineRegionSetObserved")]
    pub fn offline_region_set_observed(
        &self,
        region_id: BigInt,
        observed: bool,
    ) -> Result<OfflineOperationStart> {
        let mut operation_id = 0;
        core::check(unsafe {
            sys::mln_runtime_offline_region_set_observed_start(
                self.state.as_ptr(),
                bigint_to_i64(region_id, "regionId")?,
                observed,
                &mut operation_id,
            )
        })
        .map_err(error::from_core)?;
        Ok(offline_operation_start(operation_id))
    }

    #[napi(js_name = "offlineRegionSetDownloadState")]
    pub fn offline_region_set_download_state(
        &self,
        region_id: BigInt,
        state: String,
    ) -> Result<OfflineOperationStart> {
        let mut operation_id = 0;
        core::check(unsafe {
            sys::mln_runtime_offline_region_set_download_state_start(
                self.state.as_ptr(),
                bigint_to_i64(region_id, "regionId")?,
                offline_region_download_state_from_string(&state)?,
                &mut operation_id,
            )
        })
        .map_err(error::from_core)?;
        Ok(offline_operation_start(operation_id))
    }

    #[napi(js_name = "offlineRegionInvalidate")]
    pub fn offline_region_invalidate(&self, region_id: BigInt) -> Result<OfflineOperationStart> {
        let mut operation_id = 0;
        core::check(unsafe {
            sys::mln_runtime_offline_region_invalidate_start(
                self.state.as_ptr(),
                bigint_to_i64(region_id, "regionId")?,
                &mut operation_id,
            )
        })
        .map_err(error::from_core)?;
        Ok(offline_operation_start(operation_id))
    }

    #[napi(js_name = "offlineRegionDelete")]
    pub fn offline_region_delete(&self, region_id: BigInt) -> Result<OfflineOperationStart> {
        let mut operation_id = 0;
        core::check(unsafe {
            sys::mln_runtime_offline_region_delete_start(
                self.state.as_ptr(),
                bigint_to_i64(region_id, "regionId")?,
                &mut operation_id,
            )
        })
        .map_err(error::from_core)?;
        Ok(offline_operation_start(operation_id))
    }

    #[napi(js_name = "offlineRegionCreate")]
    pub fn offline_region_create(
        &self,
        definition: OfflineRegionDefinitionInput,
        metadata: Option<Uint8Array>,
    ) -> Result<OfflineOperationStart> {
        let definition = offline_region_definition_from_input(definition)?;
        let native_definition = core::runtime::offline_region_definition_to_native(&definition)
            .map_err(error::from_core)?;
        let raw_definition = native_definition.to_raw();
        let metadata = metadata
            .map(|metadata| metadata.to_vec())
            .unwrap_or_default();
        let mut operation_id = 0;
        core::check(unsafe {
            sys::mln_runtime_offline_region_create_start(
                self.state.as_ptr(),
                &raw_definition,
                core::runtime::metadata_ptr(&metadata),
                metadata.len(),
                &mut operation_id,
            )
        })
        .map_err(error::from_core)?;
        Ok(offline_operation_start(operation_id))
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

unsafe extern "C" fn resource_provider_trampoline(
    user_data: *mut c_void,
    request: *const sys::mln_resource_request,
    handle: *mut sys::mln_resource_request_handle,
) -> u32 {
    if user_data.is_null() || request.is_null() || handle.is_null() {
        return core::resource::UNKNOWN_PROVIDER_DECISION;
    }
    let provider = unsafe { &*(user_data as *const ResourceProviderState) };
    let request = match unsafe { core::resource::copy_resource_request(&*request) } {
        Ok(request) => request,
        Err(_) => return core::resource::UNKNOWN_PROVIDER_DECISION,
    };
    let handle_state = match unsafe {
        core::resource::ResourceRequestHandleState::new(
            handle,
            core::resource::ResourceRequestHandleFns::NATIVE,
        )
    } {
        Ok(handle_state) => handle_state,
        Err(_) => return core::resource::UNKNOWN_PROVIDER_DECISION,
    };
    let handle_id = register_resource_request_handle(handle_state.clone());
    let provider_request = resource_provider_request_from_core(request, handle_id);
    let (sender, receiver) = mpsc::sync_channel(1);
    let decision_state = handle_state.clone();
    let decision_handle_id = handle_id;
    let status = provider.callback.call_with_return_value(
        Ok(provider_request),
        ThreadsafeFunctionCallMode::Blocking,
        move |result, _env| {
            let decision = match result.ok().flatten().as_deref() {
                Some("handle") => core::resource::ResourceProviderDecision::Handle,
                _ => core::resource::ResourceProviderDecision::PassThrough,
            };
            if !matches!(decision, core::resource::ResourceProviderDecision::Handle) {
                unregister_resource_request_handle(decision_handle_id);
            }
            let _ = sender.send(decision_state.finish_provider_decision(decision));
            Ok(())
        },
    );
    if !matches!(status, napi::Status::Ok) {
        unregister_resource_request_handle(handle_id);
        return handle_state.finish_provider_exception();
    }
    receiver.recv().unwrap_or_else(|_| {
        unregister_resource_request_handle(handle_id);
        handle_state.finish_provider_exception()
    })
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

fn resource_request_handles()
-> &'static Mutex<HashMap<u64, Arc<core::resource::ResourceRequestHandleState>>> {
    RESOURCE_REQUEST_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn register_resource_request_handle(
    handle: Arc<core::resource::ResourceRequestHandleState>,
) -> u64 {
    let handle_id = RESOURCE_REQUEST_HANDLE_IDS.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut handles) = resource_request_handles().lock() {
        handles.insert(handle_id, handle);
    }
    handle_id
}

fn unregister_resource_request_handle(handle_id: u64) {
    if let Ok(mut handles) = resource_request_handles().lock() {
        handles.remove(&handle_id);
    }
}

fn parse_resource_request_handle_id(handle_id: &str) -> Result<u64> {
    handle_id
        .parse::<u64>()
        .map_err(|_| error::invalid_argument("ResourceRequestHandle id is invalid"))
}

fn resource_provider_request_from_core(
    request: core::resource::ResourceRequest,
    handle_id: u64,
) -> ResourceProviderRequest {
    ResourceProviderRequest {
        url: request.url,
        kind: resource_kind_name(request.kind).to_owned(),
        raw_kind: request.raw_kind,
        loading_method: resource_loading_method_name(request.loading_method).to_owned(),
        raw_loading_method: request.raw_loading_method,
        priority: resource_priority_name(request.priority).to_owned(),
        raw_priority: request.raw_priority,
        usage: resource_usage_name(request.usage).to_owned(),
        raw_usage: request.raw_usage,
        storage_policy: resource_storage_policy_name(request.storage_policy).to_owned(),
        raw_storage_policy: request.raw_storage_policy,
        range: request.range.map(|range| ResourceByteRange {
            start: range.start.to_string(),
            end: range.end.to_string(),
        }),
        prior_modified_unix_ms: request.prior_modified_unix_ms,
        prior_expires_unix_ms: request.prior_expires_unix_ms,
        prior_etag: request.prior_etag,
        prior_data: Uint8Array::from(request.prior_data),
        handle_id: handle_id.to_string(),
    }
}

fn resource_response_from_input(input: ResourceResponseInput) -> Result<core::ResourceResponse> {
    let status = resource_response_status_from_string(input.status.as_deref().unwrap_or("ok"))?;
    let error_reason =
        resource_error_reason_from_string(input.error_reason.as_deref().unwrap_or("none"))?;
    let mut response = core::ResourceResponse::default();
    response.status = status;
    response.error_reason = error_reason;
    response.bytes = input.bytes.map(|bytes| bytes.to_vec()).unwrap_or_default();
    response.error_message = input.error_message;
    response.must_revalidate = input.must_revalidate.unwrap_or(false);
    response.modified_unix_ms = input.modified_unix_ms;
    response.expires_unix_ms = input.expires_unix_ms;
    response.etag = input.etag;
    response.retry_after_unix_ms = input.retry_after_unix_ms;
    Ok(response)
}

fn resource_response_status_from_string(value: &str) -> Result<core::ResourceResponseStatus> {
    match value {
        "ok" => Ok(core::ResourceResponseStatus::Ok),
        "error" => Ok(core::ResourceResponseStatus::Error),
        "noContent" => Ok(core::ResourceResponseStatus::NoContent),
        "notModified" => Ok(core::ResourceResponseStatus::NotModified),
        other => Err(error::invalid_argument(format!(
            "resource response status must be 'ok', 'error', 'noContent', or 'notModified', got '{other}'"
        ))),
    }
}

fn resource_error_reason_from_string(value: &str) -> Result<core::ResourceErrorReason> {
    match value {
        "none" => Ok(core::ResourceErrorReason::None),
        "notFound" => Ok(core::ResourceErrorReason::NotFound),
        "server" => Ok(core::ResourceErrorReason::Server),
        "connection" => Ok(core::ResourceErrorReason::Connection),
        "rateLimit" => Ok(core::ResourceErrorReason::RateLimit),
        "other" => Ok(core::ResourceErrorReason::Other),
        other => Err(error::invalid_argument(format!(
            "resource error reason must be 'none', 'notFound', 'server', 'connection', 'rateLimit', or 'other', got '{other}'"
        ))),
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

fn resource_loading_method_name(value: core::ResourceLoadingMethod) -> &'static str {
    match value {
        core::ResourceLoadingMethod::All => "all",
        core::ResourceLoadingMethod::CacheOnly => "cacheOnly",
        core::ResourceLoadingMethod::NetworkOnly => "networkOnly",
        core::ResourceLoadingMethod::Unknown(_) => "unknown",
        _ => "unknown",
    }
}

fn resource_priority_name(value: core::ResourcePriority) -> &'static str {
    match value {
        core::ResourcePriority::Low => "low",
        core::ResourcePriority::Regular => "regular",
        core::ResourcePriority::Unknown(_) => "unknown",
        _ => "unknown",
    }
}

fn resource_usage_name(value: core::ResourceUsage) -> &'static str {
    match value {
        core::ResourceUsage::Online => "online",
        core::ResourceUsage::Offline => "offline",
        core::ResourceUsage::Unknown(_) => "unknown",
        _ => "unknown",
    }
}

fn resource_storage_policy_name(value: core::ResourceStoragePolicy) -> &'static str {
    match value {
        core::ResourceStoragePolicy::Permanent => "permanent",
        core::ResourceStoragePolicy::Volatile => "volatile",
        core::ResourceStoragePolicy::Unknown(_) => "unknown",
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

fn offline_operation_start(operation_id: u64) -> OfflineOperationStart {
    OfflineOperationStart {
        operation_id: operation_id.to_string(),
    }
}

fn offline_region_definition_from_input(
    input: OfflineRegionDefinitionInput,
) -> Result<core::OfflineRegionDefinition> {
    let include_ideographs = input.include_ideographs.unwrap_or(true);
    match input.kind.as_str() {
        "tilePyramid" => Ok(core::OfflineRegionDefinition::TilePyramid {
            style_url: input.style_url,
            bounds: input
                .bounds
                .ok_or_else(|| {
                    error::invalid_argument("tile pyramid offline region requires bounds")
                })?
                .into_core(),
            min_zoom: input.min_zoom,
            max_zoom: input.max_zoom,
            pixel_ratio: input.pixel_ratio as f32,
            include_ideographs,
        }),
        "geometry" => Ok(core::OfflineRegionDefinition::GeometryRegion {
            style_url: input.style_url,
            geometry: crate::map::parse_geometry(input.geometry.ok_or_else(|| {
                error::invalid_argument("geometry offline region requires geometry")
            })?)?,
            min_zoom: input.min_zoom,
            max_zoom: input.max_zoom,
            pixel_ratio: input.pixel_ratio as f32,
            include_ideographs,
        }),
        other => Err(error::invalid_argument(format!(
            "offline region kind must be 'tilePyramid' or 'geometry', got '{other}'"
        ))),
    }
}

fn offline_region_download_state_from_string(state: &str) -> Result<u32> {
    match state {
        "inactive" => core::OfflineRegionDownloadState::Inactive
            .raw_for_set()
            .map_err(error::from_core),
        "active" => core::OfflineRegionDownloadState::Active
            .raw_for_set()
            .map_err(error::from_core),
        other => Err(error::invalid_argument(format!(
            "offline region download state must be 'inactive' or 'active', got '{other}'"
        ))),
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

fn bigint_to_i64(value: BigInt, field_name: &str) -> Result<i64> {
    let (value, lossless) = value.get_i64();
    if !lossless {
        return Err(error::invalid_argument(format!(
            "{field_name} must be a signed 64-bit bigint"
        )));
    }
    Ok(value)
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

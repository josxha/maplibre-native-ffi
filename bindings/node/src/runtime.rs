use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString, c_void};
use std::os::raw::c_char;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, ThreadId};

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
pub struct OfflineRegionDefinitionValue {
    pub kind: String,
    pub style_url: String,
    pub bounds: Option<crate::values::LatLngBounds>,
    pub geometry: Option<String>,
    pub min_zoom: f64,
    pub max_zoom: f64,
    pub pixel_ratio: f64,
    pub include_ideographs: bool,
}

#[napi(object)]
pub struct OfflineRegionInfoValue {
    pub id: String,
    pub definition: OfflineRegionDefinitionValue,
    pub metadata: Uint8Array,
}

#[napi(object)]
pub struct OfflineRegionStatusValue {
    pub download_state: String,
    pub raw_download_state: u32,
    pub completed_resource_count: String,
    pub completed_resource_size: String,
    pub completed_tile_count: String,
    pub completed_tile_size: String,
    pub required_resource_count: String,
    pub required_resource_count_is_precise: bool,
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
pub struct ResourceRouteInput {
    pub kind: Option<String>,
    pub url: Option<String>,
    pub url_prefix: Option<String>,
}

#[napi(object)]
pub struct ResourceTransformRuleInput {
    pub kind: Option<String>,
    pub url: Option<String>,
    pub url_prefix: Option<String>,
    pub replacement_url: Option<String>,
    pub replacement_url_prefix: Option<String>,
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
static RESOURCE_REQUEST_HANDLES: OnceLock<Mutex<HashMap<u64, ResourceRequestRegistration>>> =
    OnceLock::new();

#[derive(Clone)]
struct ResourceRequestRegistration {
    handle: Arc<core::resource::ResourceRequestHandleState>,
    provider: usize,
}

struct ResourceMatcher {
    kind: Option<u32>,
    url: Option<String>,
    url_prefix: Option<String>,
}

struct ResourceTransformRule {
    matcher: ResourceMatcher,
    replacement_url: Option<CString>,
    replacement_url_prefix: Option<String>,
}

struct ResourceTransformState {
    rules: Vec<ResourceTransformRule>,
    replacements: Mutex<HashMap<ThreadId, CString>>,
}

struct ResourceProviderState {
    routes: Vec<ResourceMatcher>,
    callback: ThreadsafeFunction<ResourceProviderRequest>,
    pending_handle_ids: Mutex<HashSet<u64>>,
}

#[napi(js_name = "NativeRuntimeHandle")]
pub struct NativeRuntimeHandle {
    state: NativeHandleState<sys::mln_runtime>,
    resource_transform: Mutex<Option<Arc<ResourceTransformState>>>,
    retired_resource_transforms: Mutex<Vec<Arc<ResourceTransformState>>>,
    resource_provider: Mutex<Option<Arc<ResourceProviderState>>>,
    retired_resource_providers: Mutex<Vec<Arc<ResourceProviderState>>>,
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
        retired_resource_transforms: Mutex::new(Vec::new()),
        resource_provider: Mutex::new(None),
        retired_resource_providers: Mutex::new(Vec::new()),
    })
}

#[napi(js_name = "nativeResourceRequestComplete")]
pub fn native_resource_request_complete(
    handle_id: String,
    response: ResourceResponseInput,
) -> Result<()> {
    let handle_id = parse_resource_request_handle_id(&handle_id)?;
    let response = resource_response_from_input(response)?;
    let registration = resource_request_handles()
        .lock()
        .map_err(|_| error::invalid_argument("resource request registry lock is poisoned"))?
        .get(&handle_id)
        .cloned()
        .ok_or_else(|| error::invalid_argument("ResourceRequestHandle is closed"))?;
    registration
        .handle
        .complete(&response)
        .map_err(error::from_core)?;
    unregister_resource_request_handle(handle_id);
    Ok(())
}

#[napi(js_name = "nativeResourceRequestCancelled")]
pub fn native_resource_request_cancelled(handle_id: String) -> Result<bool> {
    let handle_id = parse_resource_request_handle_id(&handle_id)?;
    let registration = resource_request_handles()
        .lock()
        .map_err(|_| error::invalid_argument("resource request registry lock is poisoned"))?
        .get(&handle_id)
        .cloned()
        .ok_or_else(|| error::invalid_argument("ResourceRequestHandle is closed"))?;
    registration.handle.is_cancelled().map_err(error::from_core)
}

#[napi(js_name = "nativeResourceRequestClose")]
pub fn native_resource_request_close(handle_id: String) -> Result<()> {
    let handle_id = parse_resource_request_handle_id(&handle_id)?;
    if let Some(registration) = unregister_resource_request_handle(handle_id) {
        registration.handle.close();
    }
    Ok(())
}

#[napi]
impl NativeRuntimeHandle {
    #[napi]
    pub fn close(&self) -> Result<()> {
        unsafe { self.state.close_status(sys::mln_runtime_destroy) }.map_err(error::from_core)?;
        self.release_resource_callback_state();
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

    #[napi(js_name = "setResourceProviderRoutes")]
    pub fn set_resource_provider_routes(
        &self,
        routes: Vec<ResourceRouteInput>,
        callback: ThreadsafeFunction<ResourceProviderRequest>,
    ) -> Result<()> {
        let provider = Arc::new(ResourceProviderState {
            routes: routes
                .into_iter()
                .map(resource_matcher_from_input)
                .collect::<Result<Vec<_>>>()?,
            callback,
            pending_handle_ids: Mutex::new(HashSet::new()),
        });
        let descriptor = core::resource::resource_provider_descriptor(
            Some(resource_provider_trampoline),
            Arc::as_ptr(&provider) as *mut c_void,
        );
        core::check(unsafe {
            sys::mln_runtime_set_resource_provider(self.state.as_ptr(), &descriptor)
        })
        .map_err(error::from_core)?;
        let replaced = self
            .resource_provider
            .lock()
            .map_err(|_| error::invalid_argument("resource provider state lock is poisoned"))?
            .replace(provider);
        if let Some(replaced) = replaced {
            self.retire_resource_provider(replaced);
        }
        Ok(())
    }

    #[napi(js_name = "setResourceTransformRules")]
    pub fn set_resource_transform_rules(
        &self,
        rules: Vec<ResourceTransformRuleInput>,
    ) -> Result<()> {
        let transform = Arc::new(ResourceTransformState {
            rules: rules
                .into_iter()
                .map(resource_transform_rule_from_input)
                .collect::<Result<Vec<_>>>()?,
            replacements: Mutex::new(HashMap::new()),
        });
        let descriptor = core::resource::resource_transform_descriptor(
            Some(resource_transform_trampoline),
            Arc::as_ptr(&transform) as *mut c_void,
        );
        core::check(unsafe {
            sys::mln_runtime_set_resource_transform(self.state.as_ptr(), &descriptor)
        })
        .map_err(error::from_core)?;
        let replaced = self
            .resource_transform
            .lock()
            .map_err(|_| error::invalid_argument("resource transform state lock is poisoned"))?
            .replace(transform);
        if let Some(replaced) = replaced {
            self.retire_resource_transform(replaced);
        }
        Ok(())
    }

    #[napi(js_name = "clearResourceTransform")]
    pub fn clear_resource_transform(&self) -> Result<()> {
        core::check(unsafe { sys::mln_runtime_clear_resource_transform(self.state.as_ptr()) })
            .map_err(error::from_core)?;
        let replaced = self
            .resource_transform
            .lock()
            .map_err(|_| error::invalid_argument("resource transform state lock is poisoned"))?
            .take();
        if let Some(replaced) = replaced {
            self.retire_resource_transform(replaced);
        }
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

    #[napi(js_name = "offlineRegionsMergeDatabase")]
    pub fn offline_regions_merge_database(&self, path: String) -> Result<OfflineOperationStart> {
        let path = core::string::c_string(&path).map_err(error::from_core)?;
        let mut operation_id = 0;
        core::check(unsafe {
            sys::mln_runtime_offline_regions_merge_database_start(
                self.state.as_ptr(),
                path.as_ptr(),
                &mut operation_id,
            )
        })
        .map_err(error::from_core)?;
        Ok(offline_operation_start(operation_id))
    }

    #[napi(js_name = "offlineRegionUpdateMetadata")]
    pub fn offline_region_update_metadata(
        &self,
        region_id: BigInt,
        metadata: Option<Uint8Array>,
    ) -> Result<OfflineOperationStart> {
        let metadata = metadata
            .map(|metadata| metadata.to_vec())
            .unwrap_or_default();
        let mut operation_id = 0;
        core::check(unsafe {
            sys::mln_runtime_offline_region_update_metadata_start(
                self.state.as_ptr(),
                bigint_to_i64(region_id, "regionId")?,
                core::runtime::metadata_ptr(&metadata),
                metadata.len(),
                &mut operation_id,
            )
        })
        .map_err(error::from_core)?;
        Ok(offline_operation_start(operation_id))
    }

    #[napi(js_name = "offlineRegionGetStatus")]
    pub fn offline_region_get_status(&self, region_id: BigInt) -> Result<OfflineOperationStart> {
        let mut operation_id = 0;
        core::check(unsafe {
            sys::mln_runtime_offline_region_get_status_start(
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

    #[napi(js_name = "offlineRegionCreateTakeResult")]
    pub fn offline_region_create_take_result(
        &self,
        operation_id: BigInt,
    ) -> Result<OfflineRegionInfoValue> {
        let mut snapshot = std::ptr::null_mut();
        core::check(unsafe {
            sys::mln_runtime_offline_region_create_take_result(
                self.state.as_ptr(),
                bigint_to_u64(operation_id, "operationId")?,
                &mut snapshot,
            )
        })
        .map_err(error::from_core)?;
        copy_offline_region_snapshot_value(snapshot)
    }

    #[napi(js_name = "offlineRegionGetTakeResult")]
    pub fn offline_region_get_take_result(
        &self,
        operation_id: BigInt,
    ) -> Result<Option<OfflineRegionInfoValue>> {
        let mut snapshot = std::ptr::null_mut();
        let mut found = false;
        core::check(unsafe {
            sys::mln_runtime_offline_region_get_take_result(
                self.state.as_ptr(),
                bigint_to_u64(operation_id, "operationId")?,
                &mut snapshot,
                &mut found,
            )
        })
        .map_err(error::from_core)?;
        if !found {
            return Ok(None);
        }
        Ok(Some(copy_offline_region_snapshot_value(snapshot)?))
    }

    #[napi(js_name = "offlineRegionsListTakeResult")]
    pub fn offline_regions_list_take_result(
        &self,
        operation_id: BigInt,
    ) -> Result<Vec<OfflineRegionInfoValue>> {
        let mut list = std::ptr::null_mut();
        core::check(unsafe {
            sys::mln_runtime_offline_regions_list_take_result(
                self.state.as_ptr(),
                bigint_to_u64(operation_id, "operationId")?,
                &mut list,
            )
        })
        .map_err(error::from_core)?;
        copy_offline_region_list_value(list)
    }

    #[napi(js_name = "offlineRegionsMergeDatabaseTakeResult")]
    pub fn offline_regions_merge_database_take_result(
        &self,
        operation_id: BigInt,
    ) -> Result<Vec<OfflineRegionInfoValue>> {
        let mut list = std::ptr::null_mut();
        core::check(unsafe {
            sys::mln_runtime_offline_regions_merge_database_take_result(
                self.state.as_ptr(),
                bigint_to_u64(operation_id, "operationId")?,
                &mut list,
            )
        })
        .map_err(error::from_core)?;
        copy_offline_region_list_value(list)
    }

    #[napi(js_name = "offlineRegionUpdateMetadataTakeResult")]
    pub fn offline_region_update_metadata_take_result(
        &self,
        operation_id: BigInt,
    ) -> Result<OfflineRegionInfoValue> {
        let mut snapshot = std::ptr::null_mut();
        core::check(unsafe {
            sys::mln_runtime_offline_region_update_metadata_take_result(
                self.state.as_ptr(),
                bigint_to_u64(operation_id, "operationId")?,
                &mut snapshot,
            )
        })
        .map_err(error::from_core)?;
        copy_offline_region_snapshot_value(snapshot)
    }

    #[napi(js_name = "offlineRegionGetStatusTakeResult")]
    pub fn offline_region_get_status_take_result(
        &self,
        operation_id: BigInt,
    ) -> Result<OfflineRegionStatusValue> {
        let mut raw_status = core::events::empty_offline_region_status_native();
        core::check(unsafe {
            sys::mln_runtime_offline_region_get_status_take_result(
                self.state.as_ptr(),
                bigint_to_u64(operation_id, "operationId")?,
                &mut raw_status,
            )
        })
        .map_err(error::from_core)?;
        Ok(offline_region_status_value_from_native(raw_status))
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
    if !provider
        .routes
        .iter()
        .any(|route| resource_matcher_matches(route, request.raw_kind, &request.url))
    {
        return sys::MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH;
    }
    let handle_state = match unsafe {
        core::resource::ResourceRequestHandleState::new(
            handle,
            core::resource::ResourceRequestHandleFns::NATIVE,
        )
    } {
        Ok(handle_state) => handle_state,
        Err(_) => return core::resource::UNKNOWN_PROVIDER_DECISION,
    };
    let handle_id = register_resource_request_handle(handle_state.clone(), provider);
    let provider_request = resource_provider_request_from_core(request, handle_id);
    let status = provider.callback.call(
        Ok(provider_request),
        ThreadsafeFunctionCallMode::NonBlocking,
    );
    if !matches!(status, napi::Status::Ok) {
        unregister_resource_request_handle(handle_id);
        return handle_state.finish_provider_exception();
    }
    handle_state.finish_provider_decision(core::resource::ResourceProviderDecision::Handle)
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
    let url = unsafe { CStr::from_ptr(url) }.to_string_lossy();
    let Some(rule) = transform
        .rules
        .iter()
        .find(|rule| resource_matcher_matches(&rule.matcher, kind, &url))
    else {
        return sys::MLN_STATUS_OK;
    };

    if let Some(replacement) = &rule.replacement_url {
        unsafe {
            (*out_response).url = replacement.as_ptr();
        }
        return sys::MLN_STATUS_OK;
    }

    let Some(replacement_url_prefix) = &rule.replacement_url_prefix else {
        return sys::MLN_STATUS_OK;
    };
    let Some(url_prefix) = &rule.matcher.url_prefix else {
        return sys::MLN_STATUS_OK;
    };
    let suffix = url.strip_prefix(url_prefix).unwrap_or_default();
    let Ok(replacement) = CString::new(format!("{replacement_url_prefix}{suffix}")) else {
        return sys::MLN_STATUS_OK;
    };
    let Ok(mut replacements) = transform.replacements.lock() else {
        return sys::MLN_STATUS_OK;
    };
    let thread_id = thread::current().id();
    replacements.insert(thread_id, replacement);
    if let Some(replacement) = replacements.get(&thread_id) {
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

fn resource_request_handles() -> &'static Mutex<HashMap<u64, ResourceRequestRegistration>> {
    RESOURCE_REQUEST_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn register_resource_request_handle(
    handle: Arc<core::resource::ResourceRequestHandleState>,
    provider: &ResourceProviderState,
) -> u64 {
    let handle_id = RESOURCE_REQUEST_HANDLE_IDS.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut handles) = resource_request_handles().lock() {
        handles.insert(
            handle_id,
            ResourceRequestRegistration {
                handle,
                provider: provider as *const ResourceProviderState as usize,
            },
        );
    }
    if let Ok(mut pending) = provider.pending_handle_ids.lock() {
        pending.insert(handle_id);
    }
    handle_id
}

fn unregister_resource_request_handle(handle_id: u64) -> Option<ResourceRequestRegistration> {
    let registration = resource_request_handles()
        .lock()
        .ok()
        .and_then(|mut handles| handles.remove(&handle_id));
    if let Some(registration) = &registration {
        // SAFETY: a registration can only remain in the global registry while
        // its provider state is active or retained in the runtime's retired
        // provider list. Provider drop drains and removes all its registrations.
        let provider = unsafe { &*(registration.provider as *const ResourceProviderState) };
        if let Ok(mut pending) = provider.pending_handle_ids.lock() {
            pending.remove(&handle_id);
        }
    }
    registration
}

fn resource_matcher_from_input(input: ResourceRouteInput) -> Result<ResourceMatcher> {
    Ok(ResourceMatcher {
        kind: input
            .kind
            .as_deref()
            .map(resource_kind_from_name)
            .transpose()?,
        url: input.url,
        url_prefix: input.url_prefix,
    })
}

fn resource_transform_rule_from_input(
    input: ResourceTransformRuleInput,
) -> Result<ResourceTransformRule> {
    let replacement_url = input
        .replacement_url
        .map(|url| {
            CString::new(url)
                .map_err(|_| error::invalid_argument("replacementUrl must not contain null bytes"))
        })
        .transpose()?;
    let replacement_url_prefix = input.replacement_url_prefix;
    if replacement_url.is_some() == replacement_url_prefix.is_some() {
        return Err(error::invalid_argument(
            "resource transform rule must set exactly one of replacementUrl or replacementUrlPrefix",
        ));
    }
    if replacement_url_prefix.is_some() && input.url_prefix.is_none() {
        return Err(error::invalid_argument(
            "resource transform rule with replacementUrlPrefix must also set urlPrefix",
        ));
    }
    if let Some(prefix) = &replacement_url_prefix {
        CString::new(prefix.as_str()).map_err(|_| {
            error::invalid_argument("replacementUrlPrefix must not contain null bytes")
        })?;
    }
    Ok(ResourceTransformRule {
        matcher: resource_matcher_from_input(ResourceRouteInput {
            kind: input.kind,
            url: input.url,
            url_prefix: input.url_prefix,
        })?,
        replacement_url,
        replacement_url_prefix,
    })
}

fn resource_matcher_matches(matcher: &ResourceMatcher, raw_kind: u32, url: &str) -> bool {
    if matcher.kind.is_some_and(|kind| kind != raw_kind) {
        return false;
    }
    if matcher
        .url
        .as_deref()
        .is_some_and(|expected| expected != url)
    {
        return false;
    }
    if matcher
        .url_prefix
        .as_deref()
        .is_some_and(|prefix| !url.starts_with(prefix))
    {
        return false;
    }
    true
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

fn resource_kind_from_name(kind: &str) -> Result<u32> {
    match kind {
        "unknown" => Ok(sys::MLN_RESOURCE_KIND_UNKNOWN),
        "style" => Ok(sys::MLN_RESOURCE_KIND_STYLE),
        "source" => Ok(sys::MLN_RESOURCE_KIND_SOURCE),
        "tile" => Ok(sys::MLN_RESOURCE_KIND_TILE),
        "glyphs" => Ok(sys::MLN_RESOURCE_KIND_GLYPHS),
        "sprite-image" => Ok(sys::MLN_RESOURCE_KIND_SPRITE_IMAGE),
        "sprite-json" => Ok(sys::MLN_RESOURCE_KIND_SPRITE_JSON),
        "image" => Ok(sys::MLN_RESOURCE_KIND_IMAGE),
        other => Err(error::invalid_argument(format!(
            "resource kind must be 'unknown', 'style', 'source', 'tile', 'glyphs', 'sprite-image', 'sprite-json', or 'image', got '{other}'"
        ))),
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

    fn retire_resource_transform(&self, transform: Arc<ResourceTransformState>) {
        if let Ok(mut retired) = self.retired_resource_transforms.lock() {
            retired.push(transform);
        } else {
            std::mem::forget(transform);
        }
    }

    fn retire_resource_provider(&self, provider: Arc<ResourceProviderState>) {
        if let Ok(mut retired) = self.retired_resource_providers.lock() {
            retired.push(provider);
        } else {
            std::mem::forget(provider);
        }
    }

    fn release_resource_callback_state(&self) {
        if let Ok(mut transform) = self.resource_transform.lock() {
            *transform = None;
        }
        if let Ok(mut retired_transforms) = self.retired_resource_transforms.lock() {
            retired_transforms.clear();
        }
        if let Ok(mut provider) = self.resource_provider.lock() {
            *provider = None;
        }
        if let Ok(mut retired_providers) = self.retired_resource_providers.lock() {
            retired_providers.clear();
        }
    }
}

impl Drop for ResourceProviderState {
    fn drop(&mut self) {
        let pending = self
            .pending_handle_ids
            .lock()
            .map(|mut pending| pending.drain().collect::<Vec<_>>())
            .unwrap_or_default();
        for handle_id in pending {
            if let Some(registration) = resource_request_handles()
                .lock()
                .ok()
                .and_then(|mut handles| handles.remove(&handle_id))
            {
                registration.handle.close();
            }
        }
    }
}

impl Drop for NativeRuntimeHandle {
    fn drop(&mut self) {
        if self.state.leak_for_report().is_some() {
            if let Ok(mut transform) = self.resource_transform.lock() {
                if let Some(transform) = transform.take() {
                    std::mem::forget(transform);
                }
            }
            if let Ok(mut retired_transforms) = self.retired_resource_transforms.lock() {
                for transform in retired_transforms.drain(..) {
                    std::mem::forget(transform);
                }
            }
            if let Ok(mut provider) = self.resource_provider.lock() {
                if let Some(provider) = provider.take() {
                    std::mem::forget(provider);
                }
            }
            if let Ok(mut retired_providers) = self.retired_resource_providers.lock() {
                for provider in retired_providers.drain(..) {
                    std::mem::forget(provider);
                }
            }
        }
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

fn copy_offline_region_snapshot_value(
    snapshot: *mut sys::mln_offline_region_snapshot,
) -> Result<OfflineRegionInfoValue> {
    let snapshot = NonNull::new(snapshot)
        .ok_or_else(|| error::invalid_argument("offline region snapshot result was null"))?;
    let info = unsafe { core::runtime::copy_offline_region_snapshot(snapshot) }
        .map_err(error::from_core)?;
    offline_region_info_to_value(info)
}

fn copy_offline_region_list_value(
    list: *mut sys::mln_offline_region_list,
) -> Result<Vec<OfflineRegionInfoValue>> {
    let list = NonNull::new(list)
        .ok_or_else(|| error::invalid_argument("offline region list result was null"))?;
    let regions =
        unsafe { core::runtime::copy_offline_region_list(list) }.map_err(error::from_core)?;
    regions
        .into_iter()
        .map(offline_region_info_to_value)
        .collect()
}

fn offline_region_info_to_value(info: core::OfflineRegionInfo) -> Result<OfflineRegionInfoValue> {
    Ok(OfflineRegionInfoValue {
        id: info.id.to_string(),
        definition: offline_region_definition_to_value(info.definition)?,
        metadata: Uint8Array::from(info.metadata),
    })
}

fn offline_region_definition_to_value(
    definition: core::OfflineRegionDefinition,
) -> Result<OfflineRegionDefinitionValue> {
    match definition {
        core::OfflineRegionDefinition::TilePyramid {
            style_url,
            bounds,
            min_zoom,
            max_zoom,
            pixel_ratio,
            include_ideographs,
        } => Ok(OfflineRegionDefinitionValue {
            kind: "tilePyramid".to_owned(),
            style_url,
            bounds: Some(crate::values::LatLngBounds::from_core(bounds)),
            geometry: None,
            min_zoom,
            max_zoom,
            pixel_ratio: f64::from(pixel_ratio),
            include_ideographs,
        }),
        core::OfflineRegionDefinition::GeometryRegion {
            style_url,
            geometry,
            min_zoom,
            max_zoom,
            pixel_ratio,
            include_ideographs,
        } => Ok(OfflineRegionDefinitionValue {
            kind: "geometry".to_owned(),
            style_url,
            bounds: None,
            geometry: Some(json_string_from_serde(geometry_to_serde(geometry))?),
            min_zoom,
            max_zoom,
            pixel_ratio: f64::from(pixel_ratio),
            include_ideographs,
        }),
        _ => Err(error::invalid_argument("unknown offline region definition")),
    }
}

fn offline_region_status_value_from_native(
    raw: sys::mln_offline_region_status,
) -> OfflineRegionStatusValue {
    OfflineRegionStatusValue {
        download_state: offline_region_download_state_name(raw.download_state).to_owned(),
        raw_download_state: raw.download_state,
        completed_resource_count: raw.completed_resource_count.to_string(),
        completed_resource_size: raw.completed_resource_size.to_string(),
        completed_tile_count: raw.completed_tile_count.to_string(),
        completed_tile_size: raw.completed_tile_size.to_string(),
        required_resource_count: raw.required_resource_count.to_string(),
        required_resource_count_is_precise: raw.required_resource_count_is_precise,
    }
}

fn geometry_to_serde(geometry: core::Geometry) -> serde_json::Value {
    match geometry {
        core::Geometry::Empty => serde_json::Value::Null,
        core::Geometry::Point(coordinate) => serde_json::json!({
            "type": "Point",
            "coordinates": [coordinate.longitude, coordinate.latitude]
        }),
        core::Geometry::LineString(coordinates) => serde_json::json!({
            "type": "LineString",
            "coordinates": coordinates_to_serde(coordinates)
        }),
        core::Geometry::Polygon(rings) => serde_json::json!({
            "type": "Polygon",
            "coordinates": rings.into_iter().map(coordinates_to_serde).collect::<Vec<_>>()
        }),
        core::Geometry::MultiPoint(coordinates) => serde_json::json!({
            "type": "MultiPoint",
            "coordinates": coordinates_to_serde(coordinates)
        }),
        core::Geometry::MultiLineString(lines) => serde_json::json!({
            "type": "MultiLineString",
            "coordinates": lines.into_iter().map(coordinates_to_serde).collect::<Vec<_>>()
        }),
        core::Geometry::MultiPolygon(polygons) => serde_json::json!({
            "type": "MultiPolygon",
            "coordinates": polygons
                .into_iter()
                .map(|rings| rings.into_iter().map(coordinates_to_serde).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        }),
        core::Geometry::GeometryCollection(geometries) => serde_json::json!({
            "type": "GeometryCollection",
            "geometries": geometries.into_iter().map(geometry_to_serde).collect::<Vec<_>>()
        }),
        _ => serde_json::Value::Null,
    }
}

fn coordinates_to_serde(coordinates: Vec<core::LatLng>) -> serde_json::Value {
    serde_json::Value::Array(
        coordinates
            .into_iter()
            .map(|coordinate| serde_json::json!([coordinate.longitude, coordinate.latitude]))
            .collect(),
    )
}

fn json_string_from_serde(value: serde_json::Value) -> Result<String> {
    serde_json::to_string(&value).map_err(|serialize_error| {
        error::invalid_argument(format!(
            "JSON value could not be serialized: {serialize_error}"
        ))
    })
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

fn offline_region_download_state_name(raw: u32) -> &'static str {
    match core::OfflineRegionDownloadState::from_raw(raw) {
        core::OfflineRegionDownloadState::Inactive => "inactive",
        core::OfflineRegionDownloadState::Active => "active",
        core::OfflineRegionDownloadState::Unknown(_) => "unknown",
        _ => "unknown",
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

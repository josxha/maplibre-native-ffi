#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr;

use maplibre_native_core::RuntimeEventPayload;
use maplibre_native_core::events as core_events;
use maplibre_native_sys as sys;

use crate::glib::{self, GBoolean, GBytes, GFALSE, GTRUE, GType};
use crate::native_pointer::NativePointer;

const RUNTIME_EVENT_TYPE_NAME: &CStr = c"MlnValaRuntimeEvent";

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValaRuntimeEventType {
    Unknown = 0,
    MapCameraWillChange = 1,
    MapCameraIsChanging = 2,
    MapCameraDidChange = 3,
    MapStyleLoaded = 4,
    MapLoadingStarted = 5,
    MapLoadingFinished = 6,
    MapLoadingFailed = 7,
    MapIdle = 8,
    MapRenderUpdateAvailable = 9,
    MapRenderError = 10,
    MapStillImageFinished = 11,
    MapStillImageFailed = 12,
    MapRenderFrameStarted = 13,
    MapRenderFrameFinished = 14,
    MapRenderMapStarted = 15,
    MapRenderMapFinished = 16,
    MapStyleImageMissing = 17,
    MapTileAction = 18,
    OfflineRegionStatusChanged = 19,
    OfflineRegionResponseError = 20,
    OfflineRegionTileCountLimitExceeded = 21,
    OfflineOperationCompleted = 22,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValaRuntimeEventSourceType {
    Runtime = 0,
    Map = 1,
    Unknown = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValaRuntimeEventPayloadType {
    Unknown = -1,
    None = 0,
    RenderFrame = 1,
    RenderMap = 2,
    StyleImageMissing = 3,
    TileAction = 4,
    OfflineRegionStatus = 5,
    OfflineRegionResponseError = 6,
    OfflineRegionTileCountLimit = 7,
    OfflineOperationCompleted = 8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValaRenderMode {
    Unknown = -1,
    Partial = 0,
    Full = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValaTileOperation {
    Unknown = -1,
    RequestedFromCache = 0,
    RequestedFromNetwork = 1,
    LoadFromNetwork = 2,
    LoadFromCache = 3,
    StartParse = 4,
    EndParse = 5,
    Error = 6,
    Cancelled = 7,
    Null = 8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ValaRenderingStats {
    pub encoding_time: f64,
    pub rendering_time: f64,
    pub frame_count: i64,
    pub draw_call_count: i64,
    pub total_draw_call_count: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValaRuntimeEventRenderFrame {
    pub mode: ValaRenderMode,
    pub raw_mode: u32,
    pub needs_repaint: bool,
    pub placement_changed: bool,
    pub stats: ValaRenderingStats,
}

impl Default for ValaRuntimeEventRenderFrame {
    fn default() -> Self {
        Self {
            mode: ValaRenderMode::Unknown,
            raw_mode: 0,
            needs_repaint: false,
            placement_changed: false,
            stats: ValaRenderingStats::default(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValaRuntimeEventRenderMap {
    pub mode: ValaRenderMode,
    pub raw_mode: u32,
}

impl Default for ValaRuntimeEventRenderMap {
    fn default() -> Self {
        Self {
            mode: ValaRenderMode::Unknown,
            raw_mode: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ValaTileId {
    pub overscaled_z: u32,
    pub wrap: i32,
    pub canonical_z: u32,
    pub canonical_x: u32,
    pub canonical_y: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValaRuntimeEventTileAction {
    pub operation: ValaTileOperation,
    pub raw_operation: u32,
    pub tile_id: ValaTileId,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ValaOfflineRegionStatus {
    pub size: u32,
    pub download_state: u32,
    pub completed_resource_count: u64,
    pub completed_resource_size: u64,
    pub completed_tile_count: u64,
    pub required_tile_count: u64,
    pub completed_tile_size: u64,
    pub required_resource_count: u64,
    pub required_resource_count_is_precise: bool,
    pub complete: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ValaRuntimeEventOfflineRegionStatus {
    pub region_id: i64,
    pub status: ValaOfflineRegionStatus,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ValaRuntimeEventOfflineRegionResponseError {
    pub region_id: i64,
    pub reason: u32,
    pub raw_reason: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ValaRuntimeEventOfflineRegionTileCountLimit {
    pub region_id: i64,
    pub limit: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ValaRuntimeEventOfflineOperationCompleted {
    pub operation_id: u64,
    pub operation_kind: u32,
    pub raw_operation_kind: u32,
    pub result_kind: u32,
    pub raw_result_kind: u32,
    pub result_status: i32,
    pub found: bool,
}

impl Default for ValaRuntimeEventTileAction {
    fn default() -> Self {
        Self {
            operation: ValaTileOperation::Unknown,
            raw_operation: 0,
            tile_id: ValaTileId::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RuntimeEvent {
    raw_type: u32,
    source_type: u32,
    source_address: usize,
    code: i32,
    payload_type: u32,
    message: Option<CString>,
    style_image_missing_image_id: Option<CString>,
    tile_action_source_id: Option<CString>,
    payload: RuntimeEventPayload,
}

impl RuntimeEvent {
    pub fn from_native(
        event: &sys::mln_runtime_event,
    ) -> maplibre_native_core::error::Result<Self> {
        // SAFETY: The caller polls this event from native storage and copies it
        // before the next poll.
        let copied = unsafe { core_events::runtime_event_from_native(event) }?;
        let message = copied
            .message
            .and_then(|message| CString::new(message).ok());
        let style_image_missing_image_id = match &copied.payload {
            RuntimeEventPayload::StyleImageMissing(payload) => {
                CString::new(payload.image_id.as_str()).ok()
            }
            _ => None,
        };
        let tile_action_source_id = match &copied.payload {
            RuntimeEventPayload::TileAction(payload) => {
                CString::new(payload.source_id.as_str()).ok()
            }
            _ => None,
        };
        Ok(Self {
            raw_type: event.type_,
            source_type: copied.source.source_type,
            source_address: copied.source.source_address,
            code: copied.code,
            payload_type: event.payload_type,
            message,
            style_image_missing_image_id,
            tile_action_source_id,
            payload: copied.payload,
        })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_type() -> GType {
    glib::register_boxed_type(
        RUNTIME_EVENT_TYPE_NAME,
        mln_vala_runtime_event_copy_erased,
        mln_vala_runtime_event_free_erased,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_copy(event: *const RuntimeEvent) -> *mut RuntimeEvent {
    runtime_event_copy(event)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_free(event: *mut RuntimeEvent) {
    runtime_event_free(event);
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_event_type(
    event: *const RuntimeEvent,
) -> ValaRuntimeEventType {
    event_ref(event).map_or(ValaRuntimeEventType::Unknown, |event| {
        event_type_from_raw(event.raw_type)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_raw_type(event: *const RuntimeEvent) -> u32 {
    event_ref(event).map_or(0, |event| event.raw_type)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_source_type(
    event: *const RuntimeEvent,
) -> ValaRuntimeEventSourceType {
    event_ref(event).map_or(ValaRuntimeEventSourceType::Unknown, |event| {
        source_type_from_raw(event.source_type)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_raw_source_type(event: *const RuntimeEvent) -> u32 {
    event_ref(event).map_or(0, |event| event.source_type)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_source_pointer(
    event: *const RuntimeEvent,
) -> *mut NativePointer {
    event_ref(event)
        .and_then(|event| NativePointer::from_ptr(event.source_address as *mut c_void))
        .map_or(ptr::null_mut(), |pointer| Box::into_raw(Box::new(pointer)))
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_code(event: *const RuntimeEvent) -> i32 {
    event_ref(event).map_or(0, |event| event.code)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_payload_type(
    event: *const RuntimeEvent,
) -> ValaRuntimeEventPayloadType {
    event_ref(event).map_or(ValaRuntimeEventPayloadType::Unknown, |event| {
        payload_type_from_raw(event.payload_type)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_raw_payload_type(event: *const RuntimeEvent) -> u32 {
    event_ref(event).map_or(0, |event| event.payload_type)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_message(event: *const RuntimeEvent) -> *const c_char {
    event_ref(event)
        .and_then(|event| event.message.as_ref())
        .map_or(ptr::null(), |message| message.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_render_frame(
    event: *const RuntimeEvent,
    out_payload: *mut ValaRuntimeEventRenderFrame,
) -> GBoolean {
    let Some(event) = event_ref(event) else {
        return GFALSE;
    };
    let RuntimeEventPayload::RenderFrame(payload) = &event.payload else {
        return GFALSE;
    };
    if out_payload.is_null() {
        return GFALSE;
    }
    let raw_mode = render_mode_raw(payload.mode);
    // SAFETY: out_payload is non-null writable storage.
    unsafe {
        *out_payload = ValaRuntimeEventRenderFrame {
            mode: render_mode_from_raw(raw_mode),
            raw_mode,
            needs_repaint: payload.needs_repaint,
            placement_changed: payload.placement_changed,
            stats: ValaRenderingStats {
                encoding_time: payload.stats.encoding_time,
                rendering_time: payload.stats.rendering_time,
                frame_count: payload.stats.frame_count,
                draw_call_count: payload.stats.draw_call_count,
                total_draw_call_count: payload.stats.total_draw_call_count,
            },
        };
    }
    GTRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_render_map(
    event: *const RuntimeEvent,
    out_payload: *mut ValaRuntimeEventRenderMap,
) -> GBoolean {
    let Some(event) = event_ref(event) else {
        return GFALSE;
    };
    let RuntimeEventPayload::RenderMap(payload) = &event.payload else {
        return GFALSE;
    };
    if out_payload.is_null() {
        return GFALSE;
    }
    let raw_mode = render_mode_raw(payload.mode);
    // SAFETY: out_payload is non-null writable storage.
    unsafe {
        *out_payload = ValaRuntimeEventRenderMap {
            mode: render_mode_from_raw(raw_mode),
            raw_mode,
        };
    }
    GTRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_style_image_missing_image_id(
    event: *const RuntimeEvent,
) -> *const c_char {
    let Some(event) = event_ref(event) else {
        return ptr::null();
    };
    match &event.payload {
        RuntimeEventPayload::StyleImageMissing(_) => event
            .style_image_missing_image_id
            .as_ref()
            .map_or(ptr::null(), |image_id| image_id.as_ptr()),
        _ => ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_tile_action(
    event: *const RuntimeEvent,
    out_payload: *mut ValaRuntimeEventTileAction,
) -> GBoolean {
    let Some(event) = event_ref(event) else {
        return GFALSE;
    };
    let RuntimeEventPayload::TileAction(payload) = &event.payload else {
        return GFALSE;
    };
    if out_payload.is_null() {
        return GFALSE;
    }
    let raw_operation = tile_operation_raw(payload.operation);
    // SAFETY: out_payload is non-null writable storage.
    unsafe {
        *out_payload = ValaRuntimeEventTileAction {
            operation: tile_operation_from_raw(raw_operation),
            raw_operation,
            tile_id: ValaTileId {
                overscaled_z: payload.tile_id.overscaled_z,
                wrap: payload.tile_id.wrap,
                canonical_z: payload.tile_id.canonical_z,
                canonical_x: payload.tile_id.canonical_x,
                canonical_y: payload.tile_id.canonical_y,
            },
        };
    }
    GTRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_tile_action_source_id(
    event: *const RuntimeEvent,
) -> *const c_char {
    let Some(event) = event_ref(event) else {
        return ptr::null();
    };
    match &event.payload {
        RuntimeEventPayload::TileAction(_) => event
            .tile_action_source_id
            .as_ref()
            .map_or(ptr::null(), |source_id| source_id.as_ptr()),
        _ => ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_offline_region_status(
    event: *const RuntimeEvent,
    out_payload: *mut ValaRuntimeEventOfflineRegionStatus,
) -> GBoolean {
    let Some(event) = event_ref(event) else {
        return GFALSE;
    };
    let RuntimeEventPayload::OfflineRegionStatus(payload) = &event.payload else {
        return GFALSE;
    };
    if out_payload.is_null() {
        return GFALSE;
    }
    unsafe {
        *out_payload = ValaRuntimeEventOfflineRegionStatus {
            region_id: payload.region_id,
            status: ValaOfflineRegionStatus {
                size: std::mem::size_of::<ValaOfflineRegionStatus>() as u32,
                download_state: offline_download_state_raw(payload.status.download_state),
                completed_resource_count: payload.status.completed_resource_count,
                completed_resource_size: payload.status.completed_resource_size,
                completed_tile_count: payload.status.completed_tile_count,
                required_tile_count: payload.status.required_tile_count,
                completed_tile_size: payload.status.completed_tile_size,
                required_resource_count: payload.status.required_resource_count,
                required_resource_count_is_precise: payload
                    .status
                    .required_resource_count_is_precise,
                complete: payload.status.complete,
            },
        };
    }
    GTRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_offline_region_response_error(
    event: *const RuntimeEvent,
    out_payload: *mut ValaRuntimeEventOfflineRegionResponseError,
) -> GBoolean {
    let Some(event) = event_ref(event) else {
        return GFALSE;
    };
    let RuntimeEventPayload::OfflineRegionResponseError(payload) = &event.payload else {
        return GFALSE;
    };
    if out_payload.is_null() {
        return GFALSE;
    }
    let raw_reason = payload.reason.raw_value();
    unsafe {
        *out_payload = ValaRuntimeEventOfflineRegionResponseError {
            region_id: payload.region_id,
            reason: raw_reason,
            raw_reason,
        };
    }
    GTRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_offline_region_tile_count_limit(
    event: *const RuntimeEvent,
    out_payload: *mut ValaRuntimeEventOfflineRegionTileCountLimit,
) -> GBoolean {
    let Some(event) = event_ref(event) else {
        return GFALSE;
    };
    let RuntimeEventPayload::OfflineRegionTileCountLimit(payload) = &event.payload else {
        return GFALSE;
    };
    if out_payload.is_null() {
        return GFALSE;
    }
    unsafe {
        *out_payload = ValaRuntimeEventOfflineRegionTileCountLimit {
            region_id: payload.region_id,
            limit: payload.limit,
        };
    }
    GTRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_offline_operation_completed(
    event: *const RuntimeEvent,
    out_payload: *mut ValaRuntimeEventOfflineOperationCompleted,
) -> GBoolean {
    let Some(event) = event_ref(event) else {
        return GFALSE;
    };
    let RuntimeEventPayload::OfflineOperationCompleted(payload) = &event.payload else {
        return GFALSE;
    };
    if out_payload.is_null() {
        return GFALSE;
    }
    unsafe {
        *out_payload = ValaRuntimeEventOfflineOperationCompleted {
            operation_id: payload.operation_id,
            operation_kind: payload.operation_kind.raw_value(),
            raw_operation_kind: payload.raw_operation_kind,
            result_kind: payload.result_kind.raw_value(),
            raw_result_kind: payload.raw_result_kind,
            result_status: payload.result_status,
            found: payload.found,
        };
    }
    GTRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_dup_unknown_payload(
    event: *const RuntimeEvent,
) -> *mut GBytes {
    let Some(event) = event_ref(event) else {
        return ptr::null_mut();
    };
    match &event.payload {
        RuntimeEventPayload::Unknown(payload) => glib::bytes_new(&payload.bytes),
        _ => ptr::null_mut(),
    }
}

fn event_ref(event: *const RuntimeEvent) -> Option<&'static RuntimeEvent> {
    if event.is_null() {
        None
    } else {
        Some(unsafe { &*event })
    }
}

fn runtime_event_copy(event: *const RuntimeEvent) -> *mut RuntimeEvent {
    event_ref(event).map_or(ptr::null_mut(), |event| {
        Box::into_raw(Box::new(event.clone()))
    })
}

fn runtime_event_free(event: *mut RuntimeEvent) {
    if event.is_null() {
        return;
    }
    // SAFETY: The caller transfers ownership of a RuntimeEvent allocated by this adapter.
    unsafe {
        drop(Box::from_raw(event));
    }
}

unsafe extern "C" fn mln_vala_runtime_event_copy_erased(event: *mut c_void) -> *mut c_void {
    mln_vala_runtime_event_copy(event.cast::<RuntimeEvent>()).cast::<c_void>()
}

unsafe extern "C" fn mln_vala_runtime_event_free_erased(event: *mut c_void) {
    mln_vala_runtime_event_free(event.cast::<RuntimeEvent>());
}

fn event_type_from_raw(raw: u32) -> ValaRuntimeEventType {
    match raw {
        sys::MLN_RUNTIME_EVENT_MAP_CAMERA_WILL_CHANGE => ValaRuntimeEventType::MapCameraWillChange,
        sys::MLN_RUNTIME_EVENT_MAP_CAMERA_IS_CHANGING => ValaRuntimeEventType::MapCameraIsChanging,
        sys::MLN_RUNTIME_EVENT_MAP_CAMERA_DID_CHANGE => ValaRuntimeEventType::MapCameraDidChange,
        sys::MLN_RUNTIME_EVENT_MAP_STYLE_LOADED => ValaRuntimeEventType::MapStyleLoaded,
        sys::MLN_RUNTIME_EVENT_MAP_LOADING_STARTED => ValaRuntimeEventType::MapLoadingStarted,
        sys::MLN_RUNTIME_EVENT_MAP_LOADING_FINISHED => ValaRuntimeEventType::MapLoadingFinished,
        sys::MLN_RUNTIME_EVENT_MAP_LOADING_FAILED => ValaRuntimeEventType::MapLoadingFailed,
        sys::MLN_RUNTIME_EVENT_MAP_IDLE => ValaRuntimeEventType::MapIdle,
        sys::MLN_RUNTIME_EVENT_MAP_RENDER_UPDATE_AVAILABLE => {
            ValaRuntimeEventType::MapRenderUpdateAvailable
        }
        sys::MLN_RUNTIME_EVENT_MAP_RENDER_ERROR => ValaRuntimeEventType::MapRenderError,
        sys::MLN_RUNTIME_EVENT_MAP_STILL_IMAGE_FINISHED => {
            ValaRuntimeEventType::MapStillImageFinished
        }
        sys::MLN_RUNTIME_EVENT_MAP_STILL_IMAGE_FAILED => ValaRuntimeEventType::MapStillImageFailed,
        sys::MLN_RUNTIME_EVENT_MAP_RENDER_FRAME_STARTED => {
            ValaRuntimeEventType::MapRenderFrameStarted
        }
        sys::MLN_RUNTIME_EVENT_MAP_RENDER_FRAME_FINISHED => {
            ValaRuntimeEventType::MapRenderFrameFinished
        }
        sys::MLN_RUNTIME_EVENT_MAP_RENDER_MAP_STARTED => ValaRuntimeEventType::MapRenderMapStarted,
        sys::MLN_RUNTIME_EVENT_MAP_RENDER_MAP_FINISHED => {
            ValaRuntimeEventType::MapRenderMapFinished
        }
        sys::MLN_RUNTIME_EVENT_MAP_STYLE_IMAGE_MISSING => {
            ValaRuntimeEventType::MapStyleImageMissing
        }
        sys::MLN_RUNTIME_EVENT_MAP_TILE_ACTION => ValaRuntimeEventType::MapTileAction,
        sys::MLN_RUNTIME_EVENT_OFFLINE_REGION_STATUS_CHANGED => {
            ValaRuntimeEventType::OfflineRegionStatusChanged
        }
        sys::MLN_RUNTIME_EVENT_OFFLINE_REGION_RESPONSE_ERROR => {
            ValaRuntimeEventType::OfflineRegionResponseError
        }
        sys::MLN_RUNTIME_EVENT_OFFLINE_REGION_TILE_COUNT_LIMIT_EXCEEDED => {
            ValaRuntimeEventType::OfflineRegionTileCountLimitExceeded
        }
        sys::MLN_RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED => {
            ValaRuntimeEventType::OfflineOperationCompleted
        }
        _ => ValaRuntimeEventType::Unknown,
    }
}

fn source_type_from_raw(raw: u32) -> ValaRuntimeEventSourceType {
    match raw {
        sys::MLN_RUNTIME_EVENT_SOURCE_RUNTIME => ValaRuntimeEventSourceType::Runtime,
        sys::MLN_RUNTIME_EVENT_SOURCE_MAP => ValaRuntimeEventSourceType::Map,
        _ => ValaRuntimeEventSourceType::Unknown,
    }
}

fn payload_type_from_raw(raw: u32) -> ValaRuntimeEventPayloadType {
    match raw {
        sys::MLN_RUNTIME_EVENT_PAYLOAD_NONE => ValaRuntimeEventPayloadType::None,
        sys::MLN_RUNTIME_EVENT_PAYLOAD_RENDER_FRAME => ValaRuntimeEventPayloadType::RenderFrame,
        sys::MLN_RUNTIME_EVENT_PAYLOAD_RENDER_MAP => ValaRuntimeEventPayloadType::RenderMap,
        sys::MLN_RUNTIME_EVENT_PAYLOAD_STYLE_IMAGE_MISSING => {
            ValaRuntimeEventPayloadType::StyleImageMissing
        }
        sys::MLN_RUNTIME_EVENT_PAYLOAD_TILE_ACTION => ValaRuntimeEventPayloadType::TileAction,
        sys::MLN_RUNTIME_EVENT_PAYLOAD_OFFLINE_REGION_STATUS => {
            ValaRuntimeEventPayloadType::OfflineRegionStatus
        }
        sys::MLN_RUNTIME_EVENT_PAYLOAD_OFFLINE_REGION_RESPONSE_ERROR => {
            ValaRuntimeEventPayloadType::OfflineRegionResponseError
        }
        sys::MLN_RUNTIME_EVENT_PAYLOAD_OFFLINE_REGION_TILE_COUNT_LIMIT => {
            ValaRuntimeEventPayloadType::OfflineRegionTileCountLimit
        }
        sys::MLN_RUNTIME_EVENT_PAYLOAD_OFFLINE_OPERATION_COMPLETED => {
            ValaRuntimeEventPayloadType::OfflineOperationCompleted
        }
        _ => ValaRuntimeEventPayloadType::Unknown,
    }
}

fn offline_download_state_raw(state: maplibre_native_core::OfflineRegionDownloadState) -> u32 {
    match state {
        maplibre_native_core::OfflineRegionDownloadState::Inactive => {
            sys::MLN_OFFLINE_REGION_DOWNLOAD_INACTIVE
        }
        maplibre_native_core::OfflineRegionDownloadState::Active => {
            sys::MLN_OFFLINE_REGION_DOWNLOAD_ACTIVE
        }
        maplibre_native_core::OfflineRegionDownloadState::Unknown(raw) => raw,
        _ => 0,
    }
}

fn render_mode_raw(mode: maplibre_native_core::RenderMode) -> u32 {
    match mode {
        maplibre_native_core::RenderMode::Partial => sys::MLN_RENDER_MODE_PARTIAL,
        maplibre_native_core::RenderMode::Full => sys::MLN_RENDER_MODE_FULL,
        maplibre_native_core::RenderMode::Unknown(raw) => raw,
        _ => 0xffff_ffff,
    }
}

fn render_mode_from_raw(raw: u32) -> ValaRenderMode {
    match raw {
        sys::MLN_RENDER_MODE_PARTIAL => ValaRenderMode::Partial,
        sys::MLN_RENDER_MODE_FULL => ValaRenderMode::Full,
        _ => ValaRenderMode::Unknown,
    }
}

fn tile_operation_raw(operation: maplibre_native_core::TileOperation) -> u32 {
    match operation {
        maplibre_native_core::TileOperation::RequestedFromCache => {
            sys::MLN_TILE_OPERATION_REQUESTED_FROM_CACHE
        }
        maplibre_native_core::TileOperation::RequestedFromNetwork => {
            sys::MLN_TILE_OPERATION_REQUESTED_FROM_NETWORK
        }
        maplibre_native_core::TileOperation::LoadFromNetwork => {
            sys::MLN_TILE_OPERATION_LOAD_FROM_NETWORK
        }
        maplibre_native_core::TileOperation::LoadFromCache => {
            sys::MLN_TILE_OPERATION_LOAD_FROM_CACHE
        }
        maplibre_native_core::TileOperation::StartParse => sys::MLN_TILE_OPERATION_START_PARSE,
        maplibre_native_core::TileOperation::EndParse => sys::MLN_TILE_OPERATION_END_PARSE,
        maplibre_native_core::TileOperation::Error => sys::MLN_TILE_OPERATION_ERROR,
        maplibre_native_core::TileOperation::Cancelled => sys::MLN_TILE_OPERATION_CANCELLED,
        maplibre_native_core::TileOperation::Null => sys::MLN_TILE_OPERATION_NULL,
        maplibre_native_core::TileOperation::Unknown(raw) => raw,
        _ => 0xffff_ffff,
    }
}

fn tile_operation_from_raw(raw: u32) -> ValaTileOperation {
    match raw {
        sys::MLN_TILE_OPERATION_REQUESTED_FROM_CACHE => ValaTileOperation::RequestedFromCache,
        sys::MLN_TILE_OPERATION_REQUESTED_FROM_NETWORK => ValaTileOperation::RequestedFromNetwork,
        sys::MLN_TILE_OPERATION_LOAD_FROM_NETWORK => ValaTileOperation::LoadFromNetwork,
        sys::MLN_TILE_OPERATION_LOAD_FROM_CACHE => ValaTileOperation::LoadFromCache,
        sys::MLN_TILE_OPERATION_START_PARSE => ValaTileOperation::StartParse,
        sys::MLN_TILE_OPERATION_END_PARSE => ValaTileOperation::EndParse,
        sys::MLN_TILE_OPERATION_ERROR => ValaTileOperation::Error,
        sys::MLN_TILE_OPERATION_CANCELLED => ValaTileOperation::Cancelled,
        sys::MLN_TILE_OPERATION_NULL => ValaTileOperation::Null,
        _ => ValaTileOperation::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_event_boxed_type_is_registered() {
        assert_ne!(mln_vala_runtime_event_get_type(), 0);
    }

    #[test]
    fn runtime_event_copy_owns_message() {
        let event = RuntimeEvent {
            raw_type: sys::MLN_RUNTIME_EVENT_MAP_IDLE,
            source_type: sys::MLN_RUNTIME_EVENT_SOURCE_RUNTIME,
            source_address: 0,
            code: 3,
            payload_type: sys::MLN_RUNTIME_EVENT_PAYLOAD_NONE,
            message: Some(CString::new("hello").unwrap()),
            style_image_missing_image_id: None,
            tile_action_source_id: None,
            payload: RuntimeEventPayload::None,
        };
        let copy = mln_vala_runtime_event_copy(&event);

        assert!(!copy.is_null());
        unsafe {
            assert_eq!(
                CStr::from_ptr(mln_vala_runtime_event_get_message(copy))
                    .to_str()
                    .unwrap(),
                "hello"
            );
        }
        assert_eq!(
            mln_vala_runtime_event_get_event_type(copy),
            ValaRuntimeEventType::MapIdle
        );

        mln_vala_runtime_event_free(copy);
    }
}

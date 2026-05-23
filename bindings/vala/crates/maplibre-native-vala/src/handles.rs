use std::collections::{HashMap, hash_map::DefaultHasher};
use std::ffi::{CStr, CString, c_char, c_void};
use std::hash::{Hash, Hasher};
use std::ptr;
use std::sync::{Condvar, Mutex, OnceLock};

use maplibre_native_core::error::{self, Error};
use maplibre_native_sys as sys;

use crate::events::RuntimeEvent;
use crate::geo::{LatLng, ScreenPoint};
use crate::glib::{self, GBoolean, GDestroyNotify, GError, GFALSE, GObject, GTRUE, GType};
use crate::resource::{
    ResourceRequestHandle, resource_request_handle_finish_provider_decision,
    resource_request_handle_from_native,
};
use crate::string_list::{self, StringList};
use crate::values::{self, GeoJson, Geometry, JsonValue};

const RUNTIME_TYPE_NAME: &CStr = c"MlnValaRuntimeHandle";
const MAP_TYPE_NAME: &CStr = c"MlnValaMapHandle";
const STYLE_ID_LIST_TYPE_NAME: &CStr = c"MlnValaStyleIdListHandle";
const JSON_SNAPSHOT_TYPE_NAME: &CStr = c"MlnValaJsonSnapshotHandle";
const OFFLINE_REGION_SNAPSHOT_TYPE_NAME: &CStr = c"MlnValaOfflineRegionSnapshotHandle";
const OFFLINE_REGION_LIST_TYPE_NAME: &CStr = c"MlnValaOfflineRegionListHandle";

#[repr(C)]
pub struct RuntimeHandle {
    parent_instance: GObject,
    native: *mut sys::mln_runtime,
    resource_transform: *mut ResourceTransformState,
    resource_provider: *mut ResourceProviderState,
    owner_thread: u64,
}

pub type ResourceTransformCallback =
    unsafe extern "C" fn(kind: u32, url: *const c_char, user_data: *mut c_void) -> *mut c_char;

pub type ResourceProviderCallback = unsafe extern "C" fn(
    request: *const sys::mln_resource_request,
    handle: *mut ResourceRequestHandle,
    user_data: *mut c_void,
) -> u32;

pub type CustomGeometrySourceTileDelegate =
    unsafe extern "C" fn(tile_id: sys::mln_canonical_tile_id, user_data: *mut c_void);

struct CustomGeometrySourceCallbackState {
    source_id: CString,
    fetch_tile: CustomGeometrySourceTileDelegate,
    fetch_user_data: *mut c_void,
    fetch_destroy_notify: GDestroyNotify,
    cancel_tile: Option<CustomGeometrySourceTileDelegate>,
    cancel_user_data: *mut c_void,
    cancel_destroy_notify: GDestroyNotify,
    lifecycle: Mutex<CustomGeometrySourceLifecycle>,
    idle: Condvar,
}

#[derive(Debug)]
struct CustomGeometrySourceLifecycle {
    closing: bool,
    active_callbacks: usize,
}

struct CustomGeometryCallbackGuard<'a> {
    state: &'a CustomGeometrySourceCallbackState,
}

impl Drop for CustomGeometryCallbackGuard<'_> {
    fn drop(&mut self) {
        let Ok(mut lifecycle) = self.state.lifecycle.lock() else {
            return;
        };
        lifecycle.active_callbacks = lifecycle.active_callbacks.saturating_sub(1);
        if lifecycle.active_callbacks == 0 {
            self.state.idle.notify_all();
        }
    }
}

unsafe extern "C" fn custom_geometry_fetch_trampoline(
    user_data: *mut c_void,
    tile_id: sys::mln_canonical_tile_id,
) {
    let state = user_data.cast::<CustomGeometrySourceCallbackState>();
    if state.is_null() {
        return;
    }
    let state = unsafe { &*state };
    let Some(_guard) = custom_geometry_enter_callback(state) else {
        return;
    };
    // SAFETY: state is retained by the owning MapHandle while native can invoke
    // the source callback, and the guard prevents closure destruction until this
    // upcall returns.
    unsafe { (state.fetch_tile)(tile_id, state.fetch_user_data) };
}

unsafe extern "C" fn custom_geometry_cancel_trampoline(
    user_data: *mut c_void,
    tile_id: sys::mln_canonical_tile_id,
) {
    let state = user_data.cast::<CustomGeometrySourceCallbackState>();
    if state.is_null() {
        return;
    }
    let state = unsafe { &*state };
    let Some(_guard) = custom_geometry_enter_callback(state) else {
        return;
    };
    // SAFETY: state is retained by the owning MapHandle while native can invoke
    // the source callback, and the guard prevents closure destruction until this
    // upcall returns.
    if let Some(cancel_tile) = state.cancel_tile {
        unsafe { cancel_tile(tile_id, state.cancel_user_data) };
    }
}

fn custom_geometry_enter_callback(
    state: &CustomGeometrySourceCallbackState,
) -> Option<CustomGeometryCallbackGuard<'_>> {
    let Ok(mut lifecycle) = state.lifecycle.lock() else {
        return None;
    };
    if lifecycle.closing {
        return None;
    }
    lifecycle.active_callbacks += 1;
    Some(CustomGeometryCallbackGuard { state })
}

struct ResourceTransformState {
    callback: ResourceTransformCallback,
    user_data: *mut c_void,
    destroy_notify: GDestroyNotify,
    returned_urls: Mutex<Vec<*mut c_char>>,
}

struct ResourceProviderState {
    callback: ResourceProviderCallback,
    user_data: *mut c_void,
    destroy_notify: GDestroyNotify,
}

fn current_thread_token() -> u64 {
    let mut hasher = DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    hasher.finish()
}

fn runtime_should_finalize_on_owner_thread(handle: *mut RuntimeHandle) -> bool {
    if handle.is_null() {
        return false;
    }
    // SAFETY: The caller passes a RuntimeHandle GObject during finalization.
    unsafe { !(*handle).native.is_null() && (*handle).owner_thread == current_thread_token() }
}

fn map_should_finalize_on_owner_thread(handle: *mut MapHandle) -> bool {
    if handle.is_null() {
        return false;
    }
    // SAFETY: The caller passes a MapHandle GObject during finalization.
    unsafe { !(*handle).native.is_null() && (*handle).owner_thread == current_thread_token() }
}

#[repr(C)]
pub struct MapHandle {
    parent_instance: GObject,
    native: *mut sys::mln_map,
    runtime: *mut RuntimeHandle,
    owner_thread: u64,
    custom_geometry_callbacks: *mut Vec<*mut CustomGeometrySourceCallbackState>,
}

#[repr(C)]
pub struct StyleIdListHandle {
    parent_instance: GObject,
    native: *mut sys::mln_style_id_list,
}

#[repr(C)]
pub struct JsonSnapshotHandle {
    parent_instance: GObject,
    native: *mut sys::mln_json_snapshot,
}

#[repr(C)]
pub struct OfflineRegionSnapshotHandle {
    parent_instance: GObject,
    native: *mut sys::mln_offline_region_snapshot,
}

#[repr(C)]
pub struct OfflineRegionListHandle {
    parent_instance: GObject,
    native: *mut sys::mln_offline_region_list,
}

impl glib::ObjectFinalize for RuntimeHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        let handle = object.cast::<RuntimeHandle>();
        if runtime_should_finalize_on_owner_thread(handle) {
            let _ = close_runtime_handle(handle);
        } else if unsafe { !(*handle).native.is_null() } {
            eprintln!(
                "RuntimeHandle finalized off its owner thread; call close() on the owner thread to release native state"
            );
        }
    }
}

impl glib::ObjectFinalize for MapHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        let handle = object.cast::<MapHandle>();
        if map_should_finalize_on_owner_thread(handle) {
            let _ = close_map_handle(handle);
        } else if unsafe { !(*handle).native.is_null() } {
            eprintln!(
                "MapHandle finalized off its owner thread; call close() on the owner thread to release native state"
            );
        }
    }
}

impl glib::ObjectFinalize for StyleIdListHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        close_style_id_list(object.cast::<StyleIdListHandle>());
    }
}

impl glib::ObjectFinalize for JsonSnapshotHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        close_json_snapshot(object.cast::<JsonSnapshotHandle>());
    }
}

impl glib::ObjectFinalize for OfflineRegionSnapshotHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        close_offline_region_snapshot(object.cast::<OfflineRegionSnapshotHandle>());
    }
}

impl glib::ObjectFinalize for OfflineRegionListHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        close_offline_region_list(object.cast::<OfflineRegionListHandle>());
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_get_type() -> GType {
    glib::register_object_type::<RuntimeHandle>(RUNTIME_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_type() -> GType {
    glib::register_object_type::<MapHandle>(MAP_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_id_list_handle_get_type() -> GType {
    glib::register_object_type::<StyleIdListHandle>(STYLE_ID_LIST_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_snapshot_handle_get_type() -> GType {
    glib::register_object_type::<JsonSnapshotHandle>(JSON_SNAPSHOT_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_snapshot_handle_get_type() -> GType {
    glib::register_object_type::<OfflineRegionSnapshotHandle>(OFFLINE_REGION_SNAPSHOT_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_list_handle_get_type() -> GType {
    glib::register_object_type::<OfflineRegionListHandle>(OFFLINE_REGION_LIST_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_options_default(
    out_options: *mut sys::mln_runtime_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_runtime_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_options_default(
    out_options: *mut sys::mln_map_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_map_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_new(error_out: *mut *mut GError) -> *mut RuntimeHandle {
    match create_runtime_handle(ptr::null()) {
        Ok(handle) => handle,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_new_with_options(
    options: *const sys::mln_runtime_options,
    error_out: *mut *mut GError,
) -> *mut RuntimeHandle {
    match create_runtime_handle(options) {
        Ok(handle) => handle,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_close(
    handle: *mut RuntimeHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match close_runtime_handle(handle) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_run_once(
    handle: *mut RuntimeHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match runtime_native(handle).and_then(|runtime| {
        // SAFETY: `runtime_native` returns a live native runtime pointer.
        error::check(unsafe { sys::mln_runtime_run_once(runtime) })
    }) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_poll_event(
    handle: *mut RuntimeHandle,
    out_event: *mut *mut RuntimeEvent,
    error_out: *mut *mut GError,
) -> GBoolean {
    match poll_runtime_event(handle, out_event) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_set_resource_transform(
    handle: *mut RuntimeHandle,
    callback: Option<ResourceTransformCallback>,
    user_data: *mut c_void,
    destroy_notify: GDestroyNotify,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_resource_transform(handle, callback, user_data, destroy_notify) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_clear_resource_transform(
    handle: *mut RuntimeHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match clear_resource_transform(handle) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_set_resource_provider(
    handle: *mut RuntimeHandle,
    callback: Option<ResourceProviderCallback>,
    user_data: *mut c_void,
    destroy_notify: GDestroyNotify,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_resource_provider(handle, callback, user_data, destroy_notify) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_run_ambient_cache_operation_start(
    handle: *mut RuntimeHandle,
    operation: u32,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match run_ambient_cache_operation_start(handle, operation, out_operation_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_create_start(
    handle: *mut RuntimeHandle,
    definition: *const sys::mln_offline_region_definition,
    metadata: *const u8,
    metadata_size: usize,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_region_create_start(
        handle,
        definition,
        metadata,
        metadata_size,
        out_operation_id,
    ) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_get_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_region_get_start(handle, region_id, out_operation_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_regions_list_start(
    handle: *mut RuntimeHandle,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_regions_list_start(handle, out_operation_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_regions_merge_database_start(
    handle: *mut RuntimeHandle,
    side_database_path: *const c_char,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_regions_merge_database_start(handle, side_database_path, out_operation_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_update_metadata_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    metadata: *const u8,
    metadata_size: usize,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_region_update_metadata_start(
        handle,
        region_id,
        metadata,
        metadata_size,
        out_operation_id,
    ) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_get_status_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_region_get_status_start(handle, region_id, out_operation_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_set_observed_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    observed: GBoolean,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_region_set_observed_start(handle, region_id, observed != GFALSE, out_operation_id)
    {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_set_download_state_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    state: u32,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_region_set_download_state_start(handle, region_id, state, out_operation_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_invalidate_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_region_invalidate_start(handle, region_id, out_operation_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_delete_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_region_delete_start(handle, region_id, out_operation_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_create_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionSnapshotHandle {
    match offline_region_create_take_result(handle, operation_id) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_get_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionSnapshotHandle {
    match offline_region_get_take_result(handle, operation_id, out_found) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_regions_list_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionListHandle {
    match offline_regions_list_take_result(handle, operation_id) {
        Ok(list) => list,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_regions_merge_database_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionListHandle {
    match offline_regions_merge_database_take_result(handle, operation_id) {
        Ok(list) => list,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_update_metadata_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionSnapshotHandle {
    match offline_region_update_metadata_take_result(handle, operation_id) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_get_status_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    out_status: *mut sys::mln_offline_region_status,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_region_get_status_take_result(handle, operation_id, out_status) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_operation_discard(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_operation_discard(handle, operation_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_camera_options_default(
    out_options: *mut sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_camera_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_animation_options_default(
    out_options: *mut sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_animation_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_camera_fit_options_default(
    out_options: *mut sys::mln_camera_fit_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_camera_fit_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_bound_options_default(
    out_options: *mut sys::mln_bound_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_bound_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_free_camera_options_default(
    out_options: *mut sys::mln_free_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_free_camera_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_projection_mode_default(
    out_mode: *mut sys::mln_projection_mode,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_projection_mode(out_mode) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_viewport_options_default(
    out_options: *mut sys::mln_map_viewport_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_map_viewport_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_tile_options_default(
    out_options: *mut sys::mln_map_tile_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_map_tile_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_tile_source_options_default(
    out_options: *mut sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_style_tile_source_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_tile_source_options_set_min_zoom(
    options: *mut sys::mln_style_tile_source_options,
    min_zoom: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_style_tile_min_zoom(options, min_zoom) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_tile_source_options_set_max_zoom(
    options: *mut sys::mln_style_tile_source_options,
    max_zoom: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_style_tile_max_zoom(options, max_zoom) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_tile_source_options_set_tile_size(
    options: *mut sys::mln_style_tile_source_options,
    tile_size: u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_style_tile_tile_size(options, tile_size) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_tile_source_options_set_attribution(
    options: *mut sys::mln_style_tile_source_options,
    attribution: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_style_tile_attribution(options, attribution) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_custom_geometry_source_options_default(
    out_options: *mut sys::mln_custom_geometry_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_custom_geometry_source_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_premultiplied_rgba8_image_default(
    out_image: *mut sys::mln_premultiplied_rgba8_image,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_premultiplied_rgba8_image(out_image) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_premultiplied_rgba8_image_init(
    out_image: *mut sys::mln_premultiplied_rgba8_image,
    width: u32,
    height: u32,
    stride: u32,
    pixels: *const u8,
    byte_length: usize,
    error_out: *mut *mut GError,
) -> GBoolean {
    match init_premultiplied_rgba8_image(out_image, width, height, stride, pixels, byte_length) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_image_options_default(
    out_options: *mut sys::mln_style_image_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_style_image_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_image_info_default(
    out_info: *mut sys::mln_style_image_info,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_style_image_info(out_info) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_options_set_maximum_cache_size(
    options: *mut sys::mln_runtime_options,
    maximum_cache_size: u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_runtime_maximum_cache_size(options, maximum_cache_size) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

macro_rules! export_bool_setter {
    ($name:ident, $helper:ident, $options:ty, $value:ty, $arg:ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(
            options: *mut $options,
            $arg: $value,
            error_out: *mut *mut GError,
        ) -> GBoolean {
            match $helper(options, $arg) {
                Ok(()) => GTRUE,
                Err(error) => {
                    glib::set_error(error_out, error);
                    GFALSE
                }
            }
        }
    };
}

export_bool_setter!(
    mln_vala_camera_options_set_center,
    set_camera_center,
    sys::mln_camera_options,
    *const sys::mln_lat_lng,
    center
);
export_bool_setter!(
    mln_vala_camera_options_set_zoom,
    set_camera_zoom,
    sys::mln_camera_options,
    f64,
    zoom
);
export_bool_setter!(
    mln_vala_camera_options_set_bearing,
    set_camera_bearing,
    sys::mln_camera_options,
    f64,
    bearing
);
export_bool_setter!(
    mln_vala_camera_options_set_pitch,
    set_camera_pitch,
    sys::mln_camera_options,
    f64,
    pitch
);
export_bool_setter!(
    mln_vala_camera_options_set_center_altitude,
    set_camera_center_altitude,
    sys::mln_camera_options,
    f64,
    center_altitude
);
export_bool_setter!(
    mln_vala_camera_options_set_padding,
    set_camera_padding,
    sys::mln_camera_options,
    *const sys::mln_edge_insets,
    padding
);
export_bool_setter!(
    mln_vala_camera_options_set_anchor,
    set_camera_anchor,
    sys::mln_camera_options,
    *const sys::mln_screen_point,
    anchor
);
export_bool_setter!(
    mln_vala_camera_options_set_roll,
    set_camera_roll,
    sys::mln_camera_options,
    f64,
    roll
);
export_bool_setter!(
    mln_vala_camera_options_set_field_of_view,
    set_camera_field_of_view,
    sys::mln_camera_options,
    f64,
    field_of_view
);
export_bool_setter!(
    mln_vala_animation_options_set_duration_ms,
    set_animation_duration_ms,
    sys::mln_animation_options,
    f64,
    duration_ms
);
export_bool_setter!(
    mln_vala_animation_options_set_velocity,
    set_animation_velocity,
    sys::mln_animation_options,
    f64,
    velocity
);
export_bool_setter!(
    mln_vala_animation_options_set_min_zoom,
    set_animation_min_zoom,
    sys::mln_animation_options,
    f64,
    min_zoom
);
export_bool_setter!(
    mln_vala_animation_options_set_easing,
    set_animation_easing,
    sys::mln_animation_options,
    *const sys::mln_unit_bezier,
    easing
);
export_bool_setter!(
    mln_vala_camera_fit_options_set_padding,
    set_camera_fit_padding,
    sys::mln_camera_fit_options,
    *const sys::mln_edge_insets,
    padding
);
export_bool_setter!(
    mln_vala_camera_fit_options_set_bearing,
    set_camera_fit_bearing,
    sys::mln_camera_fit_options,
    f64,
    bearing
);
export_bool_setter!(
    mln_vala_camera_fit_options_set_pitch,
    set_camera_fit_pitch,
    sys::mln_camera_fit_options,
    f64,
    pitch
);
export_bool_setter!(
    mln_vala_bound_options_set_bounds,
    set_bound_bounds,
    sys::mln_bound_options,
    *const sys::mln_lat_lng_bounds,
    bounds
);
export_bool_setter!(
    mln_vala_bound_options_set_min_zoom,
    set_bound_min_zoom,
    sys::mln_bound_options,
    f64,
    min_zoom
);
export_bool_setter!(
    mln_vala_bound_options_set_max_zoom,
    set_bound_max_zoom,
    sys::mln_bound_options,
    f64,
    max_zoom
);
export_bool_setter!(
    mln_vala_bound_options_set_min_pitch,
    set_bound_min_pitch,
    sys::mln_bound_options,
    f64,
    min_pitch
);
export_bool_setter!(
    mln_vala_bound_options_set_max_pitch,
    set_bound_max_pitch,
    sys::mln_bound_options,
    f64,
    max_pitch
);
export_bool_setter!(
    mln_vala_free_camera_options_set_position,
    set_free_camera_position,
    sys::mln_free_camera_options,
    *const sys::mln_vec3,
    position
);
export_bool_setter!(
    mln_vala_free_camera_options_set_orientation,
    set_free_camera_orientation,
    sys::mln_free_camera_options,
    *const sys::mln_quaternion,
    orientation
);
export_bool_setter!(
    mln_vala_projection_mode_set_axonometric,
    set_projection_axonometric,
    sys::mln_projection_mode,
    bool,
    axonometric
);
export_bool_setter!(
    mln_vala_projection_mode_set_x_skew,
    set_projection_x_skew,
    sys::mln_projection_mode,
    f64,
    x_skew
);
export_bool_setter!(
    mln_vala_projection_mode_set_y_skew,
    set_projection_y_skew,
    sys::mln_projection_mode,
    f64,
    y_skew
);
export_bool_setter!(
    mln_vala_map_viewport_options_set_north_orientation,
    set_viewport_north_orientation,
    sys::mln_map_viewport_options,
    u32,
    north_orientation
);
export_bool_setter!(
    mln_vala_map_viewport_options_set_constrain_mode,
    set_viewport_constrain_mode,
    sys::mln_map_viewport_options,
    u32,
    constrain_mode
);
export_bool_setter!(
    mln_vala_map_viewport_options_set_viewport_mode,
    set_viewport_viewport_mode,
    sys::mln_map_viewport_options,
    u32,
    viewport_mode
);
export_bool_setter!(
    mln_vala_map_viewport_options_set_frustum_offset,
    set_viewport_frustum_offset,
    sys::mln_map_viewport_options,
    *const sys::mln_edge_insets,
    frustum_offset
);
export_bool_setter!(
    mln_vala_map_tile_options_set_prefetch_zoom_delta,
    set_tile_prefetch_zoom_delta,
    sys::mln_map_tile_options,
    u32,
    prefetch_zoom_delta
);
export_bool_setter!(
    mln_vala_map_tile_options_set_lod_min_radius,
    set_tile_lod_min_radius,
    sys::mln_map_tile_options,
    f64,
    lod_min_radius
);
export_bool_setter!(
    mln_vala_map_tile_options_set_lod_scale,
    set_tile_lod_scale,
    sys::mln_map_tile_options,
    f64,
    lod_scale
);
export_bool_setter!(
    mln_vala_map_tile_options_set_lod_pitch_threshold,
    set_tile_lod_pitch_threshold,
    sys::mln_map_tile_options,
    f64,
    lod_pitch_threshold
);
export_bool_setter!(
    mln_vala_map_tile_options_set_lod_zoom_shift,
    set_tile_lod_zoom_shift,
    sys::mln_map_tile_options,
    f64,
    lod_zoom_shift
);
export_bool_setter!(
    mln_vala_map_tile_options_set_lod_mode,
    set_tile_lod_mode,
    sys::mln_map_tile_options,
    u32,
    lod_mode
);
export_bool_setter!(
    mln_vala_style_tile_source_options_set_scheme,
    set_style_tile_scheme,
    sys::mln_style_tile_source_options,
    u32,
    scheme
);
export_bool_setter!(
    mln_vala_style_tile_source_options_set_bounds,
    set_style_tile_bounds,
    sys::mln_style_tile_source_options,
    *const sys::mln_lat_lng_bounds,
    bounds
);
export_bool_setter!(
    mln_vala_style_tile_source_options_set_vector_encoding,
    set_style_tile_vector_encoding,
    sys::mln_style_tile_source_options,
    u32,
    vector_encoding
);
export_bool_setter!(
    mln_vala_style_tile_source_options_set_raster_encoding,
    set_style_tile_raster_encoding,
    sys::mln_style_tile_source_options,
    u32,
    raster_encoding
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_min_zoom,
    set_custom_geometry_min_zoom,
    sys::mln_custom_geometry_source_options,
    f64,
    min_zoom
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_max_zoom,
    set_custom_geometry_max_zoom,
    sys::mln_custom_geometry_source_options,
    f64,
    max_zoom
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_tolerance,
    set_custom_geometry_tolerance,
    sys::mln_custom_geometry_source_options,
    f64,
    tolerance
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_tile_size,
    set_custom_geometry_tile_size,
    sys::mln_custom_geometry_source_options,
    u32,
    tile_size
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_buffer,
    set_custom_geometry_buffer,
    sys::mln_custom_geometry_source_options,
    u32,
    buffer
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_clip,
    set_custom_geometry_clip,
    sys::mln_custom_geometry_source_options,
    bool,
    clip
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_wrap,
    set_custom_geometry_wrap,
    sys::mln_custom_geometry_source_options,
    bool,
    wrap
);
export_bool_setter!(
    mln_vala_style_image_options_set_pixel_ratio,
    set_style_image_pixel_ratio,
    sys::mln_style_image_options,
    f32,
    pixel_ratio
);
export_bool_setter!(
    mln_vala_style_image_options_set_sdf,
    set_style_image_sdf,
    sys::mln_style_image_options,
    bool,
    sdf
);
export_bool_setter!(
    mln_vala_feature_state_selector_set_source_id,
    set_feature_state_source_id,
    sys::mln_feature_state_selector,
    *const c_char,
    source_id
);
export_bool_setter!(
    mln_vala_feature_state_selector_set_source_layer_id,
    set_feature_state_source_layer_id,
    sys::mln_feature_state_selector,
    *const c_char,
    source_layer_id
);
export_bool_setter!(
    mln_vala_feature_state_selector_set_feature_id,
    set_feature_state_feature_id,
    sys::mln_feature_state_selector,
    *const c_char,
    feature_id
);
export_bool_setter!(
    mln_vala_feature_state_selector_set_state_key,
    set_feature_state_state_key,
    sys::mln_feature_state_selector,
    *const c_char,
    state_key
);

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_new(
    runtime: *mut RuntimeHandle,
    width: u32,
    height: u32,
    scale_factor: f64,
    error_out: *mut *mut GError,
) -> *mut MapHandle {
    match create_map_handle(runtime, width, height, scale_factor) {
        Ok(handle) => handle,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_new_with_options(
    runtime: *mut RuntimeHandle,
    options: *const sys::mln_map_options,
    error_out: *mut *mut GError,
) -> *mut MapHandle {
    match create_map_handle_with_options(runtime, options) {
        Ok(handle) => handle,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_close(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match close_map_handle(handle) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_request_repaint(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match request_repaint(handle) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_request_still_image(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match request_still_image(handle) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_debug_options(
    handle: *mut MapHandle,
    options: u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_debug_options(handle, options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_debug_options(
    handle: *mut MapHandle,
    out_options: *mut u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_debug_options(handle, out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_rendering_stats_view_enabled(
    handle: *mut MapHandle,
    enabled: GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_rendering_stats_view_enabled(handle, enabled != GFALSE) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_rendering_stats_view_enabled(
    handle: *mut MapHandle,
    out_enabled: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_rendering_stats_view_enabled(handle, out_enabled) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_is_fully_loaded(
    handle: *mut MapHandle,
    out_loaded: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match is_fully_loaded(handle, out_loaded) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_dump_debug_logs(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match dump_debug_logs(handle) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_camera(
    handle: *mut MapHandle,
    out_camera: *mut sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_camera(handle, out_camera) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_viewport_options(
    handle: *mut MapHandle,
    out_options: *mut sys::mln_map_viewport_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_viewport_options(handle, out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_viewport_options(
    handle: *mut MapHandle,
    options: *const sys::mln_map_viewport_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_viewport_options(handle, options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_tile_options(
    handle: *mut MapHandle,
    out_options: *mut sys::mln_map_tile_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_tile_options(handle, out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_tile_options(
    handle: *mut MapHandle,
    options: *const sys::mln_map_tile_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_tile_options(handle, options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_bounds(
    handle: *mut MapHandle,
    out_options: *mut sys::mln_bound_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_bounds(handle, out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_bounds(
    handle: *mut MapHandle,
    options: *const sys::mln_bound_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_bounds(handle, options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_free_camera_options(
    handle: *mut MapHandle,
    out_options: *mut sys::mln_free_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_free_camera_options(handle, out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_free_camera_options(
    handle: *mut MapHandle,
    options: *const sys::mln_free_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_free_camera_options(handle, options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_projection_mode(
    handle: *mut MapHandle,
    out_mode: *mut sys::mln_projection_mode,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_projection_mode(handle, out_mode) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_projection_mode(
    handle: *mut MapHandle,
    mode: *const sys::mln_projection_mode,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_projection_mode(handle, mode) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_camera_for_lat_lng_bounds(
    handle: *mut MapHandle,
    bounds: *const sys::mln_lat_lng_bounds,
    fit_options: *const sys::mln_camera_fit_options,
    out_camera: *mut sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match camera_for_lat_lng_bounds(handle, bounds, fit_options, out_camera) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_camera_for_lat_lngs(
    handle: *mut MapHandle,
    coordinates: *const LatLng,
    coordinate_count: usize,
    fit_options: *const sys::mln_camera_fit_options,
    out_camera: *mut sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match camera_for_lat_lngs(
        handle,
        coordinates,
        coordinate_count,
        fit_options,
        out_camera,
    ) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_camera_for_geometry(
    handle: *mut MapHandle,
    geometry: *const Geometry,
    fit_options: *const sys::mln_camera_fit_options,
    out_camera: *mut sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match camera_for_geometry(handle, geometry, fit_options, out_camera) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_lat_lng_bounds_for_camera(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    out_bounds: *mut sys::mln_lat_lng_bounds,
    error_out: *mut *mut GError,
) -> GBoolean {
    match lat_lng_bounds_for_camera(handle, camera, out_bounds, false) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_lat_lng_bounds_for_camera_unwrapped(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    out_bounds: *mut sys::mln_lat_lng_bounds,
    error_out: *mut *mut GError,
) -> GBoolean {
    match lat_lng_bounds_for_camera(handle, camera, out_bounds, true) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_jump_to(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match jump_to(handle, camera) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_ease_to(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    animation: *const sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match ease_to(handle, camera, animation) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_fly_to(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    animation: *const sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match fly_to(handle, camera, animation) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_move_by(
    handle: *mut MapHandle,
    delta_x: f64,
    delta_y: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match move_by(handle, delta_x, delta_y) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_move_by_animated(
    handle: *mut MapHandle,
    delta_x: f64,
    delta_y: f64,
    animation: *const sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match move_by_animated(handle, delta_x, delta_y, animation) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_scale_by(
    handle: *mut MapHandle,
    scale: f64,
    anchor: *const sys::mln_screen_point,
    error_out: *mut *mut GError,
) -> GBoolean {
    match scale_by(handle, scale, anchor) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_scale_by_animated(
    handle: *mut MapHandle,
    scale: f64,
    anchor: *const sys::mln_screen_point,
    animation: *const sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match scale_by_animated(handle, scale, anchor, animation) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_rotate_by(
    handle: *mut MapHandle,
    first: *const sys::mln_screen_point,
    second: *const sys::mln_screen_point,
    error_out: *mut *mut GError,
) -> GBoolean {
    match rotate_by(handle, first, second) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_rotate_by_animated(
    handle: *mut MapHandle,
    first: *const sys::mln_screen_point,
    second: *const sys::mln_screen_point,
    animation: *const sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match rotate_by_animated(handle, first, second, animation) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_pitch_by(
    handle: *mut MapHandle,
    pitch: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match pitch_by(handle, pitch) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_pitch_by_animated(
    handle: *mut MapHandle,
    pitch: f64,
    animation: *const sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match pitch_by_animated(handle, pitch, animation) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_cancel_transitions(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match cancel_transitions(handle) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_pixel_for_lat_lng(
    handle: *mut MapHandle,
    coordinate: *const LatLng,
    out_point: *mut ScreenPoint,
    error_out: *mut *mut GError,
) -> GBoolean {
    match map_pixel_for_lat_lng(handle, coordinate, out_point) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_lat_lng_for_pixel(
    handle: *mut MapHandle,
    point: *const ScreenPoint,
    out_coordinate: *mut LatLng,
    error_out: *mut *mut GError,
) -> GBoolean {
    match map_lat_lng_for_pixel(handle, point, out_coordinate) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_pixels_for_lat_lngs(
    handle: *mut MapHandle,
    coordinates: *const LatLng,
    coordinate_count: usize,
    out_points: *mut ScreenPoint,
    error_out: *mut *mut GError,
) -> GBoolean {
    match map_pixels_for_lat_lngs(handle, coordinates, coordinate_count, out_points) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_lat_lngs_for_pixels(
    handle: *mut MapHandle,
    points: *const ScreenPoint,
    point_count: usize,
    out_coordinates: *mut LatLng,
    error_out: *mut *mut GError,
) -> GBoolean {
    match map_lat_lngs_for_pixels(handle, points, point_count, out_coordinates) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_style_url(
    handle: *mut MapHandle,
    url: *const std::ffi::c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_style_url(handle, url) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_style_json(
    handle: *mut MapHandle,
    json: *const std::ffi::c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_style_json(handle, json) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_geojson_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_geojson_source_url(handle, source_id, url) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_geojson_source_data(
    handle: *mut MapHandle,
    source_id: *const c_char,
    data: *const GeoJson,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_geojson_source_data(handle, source_id, data) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_geojson_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_geojson_source_url(handle, source_id, url) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_geojson_source_data(
    handle: *mut MapHandle,
    source_id: *const c_char,
    data: *const GeoJson,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_geojson_source_data(handle, source_id, data) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_vector_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    options: *const sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_tile_source_url(handle, source_id, url, options, TileSourceKind::Vector) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_raster_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    options: *const sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_tile_source_url(handle, source_id, url, options, TileSourceKind::Raster) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_raster_dem_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    options: *const sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_tile_source_url(handle, source_id, url, options, TileSourceKind::RasterDem) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_vector_source_tiles(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tiles: *const StringList,
    options: *const sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_tile_source_tiles(handle, source_id, tiles, options, TileSourceKind::Vector) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_raster_source_tiles(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tiles: *const StringList,
    options: *const sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_tile_source_tiles(handle, source_id, tiles, options, TileSourceKind::Raster) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_raster_dem_source_tiles(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tiles: *const StringList,
    options: *const sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_tile_source_tiles(handle, source_id, tiles, options, TileSourceKind::RasterDem) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_custom_geometry_source(
    handle: *mut MapHandle,
    source_id: *const c_char,
    options: *const sys::mln_custom_geometry_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_custom_geometry_source(handle, source_id, options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_custom_geometry_source_with_callbacks(
    handle: *mut MapHandle,
    source_id: *const c_char,
    fetch_tile: Option<CustomGeometrySourceTileDelegate>,
    fetch_user_data: *mut c_void,
    fetch_destroy_notify: GDestroyNotify,
    cancel_tile: Option<CustomGeometrySourceTileDelegate>,
    cancel_user_data: *mut c_void,
    cancel_destroy_notify: GDestroyNotify,
    options: *const sys::mln_custom_geometry_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_custom_geometry_source_with_callbacks(
        handle,
        source_id,
        fetch_tile,
        fetch_user_data,
        fetch_destroy_notify,
        cancel_tile,
        cancel_user_data,
        cancel_destroy_notify,
        options,
    ) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_custom_geometry_source_tile_data(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tile_id: *const sys::mln_canonical_tile_id,
    data: *const GeoJson,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_custom_geometry_source_tile_data(handle, source_id, tile_id, data) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_invalidate_custom_geometry_source_tile(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tile_id: *const sys::mln_canonical_tile_id,
    error_out: *mut *mut GError,
) -> GBoolean {
    match invalidate_custom_geometry_source_tile(handle, source_id, tile_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_invalidate_custom_geometry_source_region(
    handle: *mut MapHandle,
    source_id: *const c_char,
    bounds: *const sys::mln_lat_lng_bounds,
    error_out: *mut *mut GError,
) -> GBoolean {
    match invalidate_custom_geometry_source_region(handle, source_id, bounds) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_style_source_exists(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_exists: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match style_source_exists(handle, source_id, out_exists) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_style_source_type(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_source_type: *mut u32,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_style_source_type(handle, source_id, out_source_type, out_found) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_style_source_info(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_info: *mut sys::mln_style_source_info,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_style_source_info(handle, source_id, out_info, out_found) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_copy_style_source_attribution(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_attribution: *mut c_char,
    attribution_capacity: usize,
    out_attribution_size: *mut usize,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match copy_style_source_attribution(
        handle,
        source_id,
        out_attribution,
        attribution_capacity,
        out_attribution_size,
        out_found,
    ) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_remove_style_source(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_removed: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match remove_style_source(handle, source_id, out_removed) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_style_image(
    handle: *mut MapHandle,
    image_id: *const c_char,
    image: *const sys::mln_premultiplied_rgba8_image,
    options: *const sys::mln_style_image_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_style_image(handle, image_id, image, options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_remove_style_image(
    handle: *mut MapHandle,
    image_id: *const c_char,
    out_removed: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match remove_style_image(handle, image_id, out_removed) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_style_image_exists(
    handle: *mut MapHandle,
    image_id: *const c_char,
    out_exists: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match style_image_exists(handle, image_id, out_exists) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_style_image_info(
    handle: *mut MapHandle,
    image_id: *const c_char,
    out_info: *mut sys::mln_style_image_info,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_style_image_info(handle, image_id, out_info, out_found) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_copy_style_image_premultiplied_rgba8(
    handle: *mut MapHandle,
    image_id: *const c_char,
    out_pixels: *mut u8,
    pixel_capacity: usize,
    out_byte_length: *mut usize,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match copy_style_image_premultiplied_rgba8(
        handle,
        image_id,
        out_pixels,
        pixel_capacity,
        out_byte_length,
        out_found,
    ) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_image_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    coordinates: *const sys::mln_lat_lng,
    coordinate_count: usize,
    url: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_image_source_url(handle, source_id, coordinates, coordinate_count, url) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_image_source_image(
    handle: *mut MapHandle,
    source_id: *const c_char,
    coordinates: *const sys::mln_lat_lng,
    coordinate_count: usize,
    image: *const sys::mln_premultiplied_rgba8_image,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_image_source_image(handle, source_id, coordinates, coordinate_count, image) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_image_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_image_source_url(handle, source_id, url) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_image_source_image(
    handle: *mut MapHandle,
    source_id: *const c_char,
    image: *const sys::mln_premultiplied_rgba8_image,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_image_source_image(handle, source_id, image) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_image_source_coordinates(
    handle: *mut MapHandle,
    source_id: *const c_char,
    coordinates: *const sys::mln_lat_lng,
    coordinate_count: usize,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_image_source_coordinates(handle, source_id, coordinates, coordinate_count) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_image_source_coordinates(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_coordinates: *mut sys::mln_lat_lng,
    coordinate_capacity: usize,
    out_coordinate_count: *mut usize,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_image_source_coordinates(
        handle,
        source_id,
        out_coordinates,
        coordinate_capacity,
        out_coordinate_count,
        out_found,
    ) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_hillshade_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    source_id: *const c_char,
    before_layer_id: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_dem_layer(
        handle,
        layer_id,
        source_id,
        before_layer_id,
        DemLayerKind::Hillshade,
    ) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_color_relief_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    source_id: *const c_char,
    before_layer_id: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_dem_layer(
        handle,
        layer_id,
        source_id,
        before_layer_id,
        DemLayerKind::ColorRelief,
    ) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_location_indicator_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    before_layer_id: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_location_indicator_layer(handle, layer_id, before_layer_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_location_indicator_location(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    coordinate: *const sys::mln_lat_lng,
    altitude: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_location_indicator_location(handle, layer_id, coordinate, altitude) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_location_indicator_bearing(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    bearing: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_location_indicator_bearing(handle, layer_id, bearing) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_location_indicator_accuracy_radius(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    radius: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_location_indicator_accuracy_radius(handle, layer_id, radius) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_location_indicator_image_name(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    image_kind: u32,
    image_id: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_location_indicator_image_name(handle, layer_id, image_kind, image_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_style_source_json(
    handle: *mut MapHandle,
    source_id: *const c_char,
    source_json: *const JsonValue,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_style_source_json(handle, source_id, source_json) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_style_layer_json(
    handle: *mut MapHandle,
    layer_json: *const JsonValue,
    before_layer_id: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match add_style_layer_json(handle, layer_json, before_layer_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_style_layer_exists(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    out_exists: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match style_layer_exists(handle, layer_id, out_exists) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_move_style_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    before_layer_id: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    match move_style_layer(handle, layer_id, before_layer_id) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_remove_style_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    out_removed: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match remove_style_layer(handle, layer_id, out_removed) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_style_layer_type(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    out_layer_type: *mut *mut c_char,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_style_layer_type(handle, layer_id, out_layer_type, out_found) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_list_style_source_ids(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> *mut StyleIdListHandle {
    match list_style_source_ids(handle) {
        Ok(list) => list,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_style_layer_json(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> *mut JsonSnapshotHandle {
    match get_style_layer_json(handle, layer_id, out_found) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_style_light_json(
    handle: *mut MapHandle,
    light_json: *const JsonValue,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_style_light_json(handle, light_json) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_style_light_property(
    handle: *mut MapHandle,
    property_name: *const c_char,
    value: *const JsonValue,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_style_light_property(handle, property_name, value) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_style_light_property(
    handle: *mut MapHandle,
    property_name: *const c_char,
    error_out: *mut *mut GError,
) -> *mut JsonSnapshotHandle {
    match get_style_light_property(handle, property_name) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_layer_property(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    property_name: *const c_char,
    value: *const JsonValue,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_layer_property(handle, layer_id, property_name, value) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_layer_property(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    property_name: *const c_char,
    error_out: *mut *mut GError,
) -> *mut JsonSnapshotHandle {
    match get_layer_property(handle, layer_id, property_name) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_layer_filter(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    filter: *const JsonValue,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_layer_filter(handle, layer_id, filter) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_layer_filter(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    error_out: *mut *mut GError,
) -> *mut JsonSnapshotHandle {
    match get_layer_filter(handle, layer_id) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_list_style_layer_ids(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> *mut StyleIdListHandle {
    match list_style_layer_ids(handle) {
        Ok(list) => list,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_id_list_handle_count(
    handle: *mut StyleIdListHandle,
    out_count: *mut usize,
    error_out: *mut *mut GError,
) -> GBoolean {
    match style_id_list_count(handle, out_count) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_id_list_handle_get(
    handle: *mut StyleIdListHandle,
    index: usize,
    error_out: *mut *mut GError,
) -> *mut c_char {
    match style_id_list_get(handle, index) {
        Ok(id) => id,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_id_list_handle_close(handle: *mut StyleIdListHandle) {
    close_style_id_list(handle);
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_snapshot_handle_get(
    handle: *mut JsonSnapshotHandle,
    out_value: *mut *mut JsonValue,
    error_out: *mut *mut GError,
) -> GBoolean {
    match json_snapshot_get(handle, out_value) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_snapshot_handle_close(handle: *mut JsonSnapshotHandle) {
    close_json_snapshot(handle);
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_snapshot_handle_get(
    handle: *mut OfflineRegionSnapshotHandle,
    out_info: *mut sys::mln_offline_region_info,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_region_snapshot_get(handle, out_info) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

fn offline_region_info_dup_metadata(
    info: *const sys::mln_offline_region_info,
) -> *mut glib::GBytes {
    if info.is_null() {
        return ptr::null_mut();
    }
    // SAFETY: info points to a copied offline-region info struct. Copy the
    // metadata bytes immediately into a GBytes owned by the caller.
    let info = unsafe { &*info };
    if info.metadata.is_null() || info.metadata_size == 0 {
        return ptr::null_mut();
    }
    // SAFETY: metadata is valid for metadata_size bytes while the copied info
    // remains live; bytes_new copies it.
    let bytes = unsafe { std::slice::from_raw_parts(info.metadata, info.metadata_size) };
    glib::bytes_new(bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_info_dup_metadata(
    info: *const sys::mln_offline_region_info,
) -> *mut glib::GBytes {
    offline_region_info_dup_metadata(info)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_snapshot_handle_close(
    handle: *mut OfflineRegionSnapshotHandle,
) {
    close_offline_region_snapshot(handle);
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_list_handle_count(
    handle: *mut OfflineRegionListHandle,
    out_count: *mut usize,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_region_list_count(handle, out_count) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_list_handle_get(
    handle: *mut OfflineRegionListHandle,
    index: usize,
    out_info: *mut sys::mln_offline_region_info,
    error_out: *mut *mut GError,
) -> GBoolean {
    match offline_region_list_get(handle, index, out_info) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_list_handle_close(handle: *mut OfflineRegionListHandle) {
    close_offline_region_list(handle);
}

fn default_runtime_options(out_options: *mut sys::mln_runtime_options) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_runtime_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn default_map_options(out_options: *mut sys::mln_map_options) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_map_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn request_repaint(handle: *mut MapHandle) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map_native` returns a live native map pointer.
    error::check(unsafe { sys::mln_map_request_repaint(map) })
}

fn request_still_image(handle: *mut MapHandle) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map_native` returns a live native map pointer.
    error::check(unsafe { sys::mln_map_request_still_image(map) })
}

fn set_debug_options(handle: *mut MapHandle, options: u32) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map_native` returns a live native map pointer. The C API
    // validates debug-option bits.
    error::check(unsafe { sys::mln_map_set_debug_options(map, options) })
}

fn get_debug_options(handle: *mut MapHandle, out_options: *mut u32) -> error::Result<()> {
    let map = map_native(handle)?;
    if out_options.is_null() {
        return Err(Error::invalid_argument(
            "debug options output pointer is null",
        ));
    }
    // SAFETY: `map` is live and `out_options` points to writable storage.
    error::check(unsafe { sys::mln_map_get_debug_options(map, out_options) })
}

fn set_rendering_stats_view_enabled(handle: *mut MapHandle, enabled: bool) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map_native` returns a live native map pointer.
    error::check(unsafe { sys::mln_map_set_rendering_stats_view_enabled(map, enabled) })
}

fn get_rendering_stats_view_enabled(
    handle: *mut MapHandle,
    out_enabled: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let mut enabled = false;
    // SAFETY: `map` is live and `enabled` is valid output storage.
    error::check(unsafe { sys::mln_map_get_rendering_stats_view_enabled(map, &mut enabled) })?;
    glib::clear_optional_out_pointer(out_enabled, if enabled { GTRUE } else { GFALSE })
}

fn default_camera_options(out_options: *mut sys::mln_camera_options) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_camera_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn default_animation_options(out_options: *mut sys::mln_animation_options) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_animation_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn default_camera_fit_options(out_options: *mut sys::mln_camera_fit_options) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_camera_fit_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn default_bound_options(out_options: *mut sys::mln_bound_options) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_bound_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn default_free_camera_options(
    out_options: *mut sys::mln_free_camera_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_free_camera_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn default_projection_mode(out_mode: *mut sys::mln_projection_mode) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mode = unsafe { sys::mln_projection_mode_default() };
    glib::clear_optional_out_pointer(out_mode, mode)
}

fn default_map_viewport_options(
    out_options: *mut sys::mln_map_viewport_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_map_viewport_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn default_map_tile_options(out_options: *mut sys::mln_map_tile_options) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_map_tile_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn default_style_tile_source_options(
    out_options: *mut sys::mln_style_tile_source_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_style_tile_source_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn style_tile_options_mut(
    options: *mut sys::mln_style_tile_source_options,
) -> error::Result<&'static mut sys::mln_style_tile_source_options> {
    if options.is_null() {
        return Err(Error::invalid_argument(
            "style tile source options are null",
        ));
    }
    Ok(unsafe { &mut *options })
}

fn set_style_tile_min_zoom(
    options: *mut sys::mln_style_tile_source_options,
    min_zoom: f64,
) -> error::Result<()> {
    let options = style_tile_options_mut(options)?;
    options.min_zoom = min_zoom;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_MIN_ZOOM;
    Ok(())
}

fn set_style_tile_max_zoom(
    options: *mut sys::mln_style_tile_source_options,
    max_zoom: f64,
) -> error::Result<()> {
    let options = style_tile_options_mut(options)?;
    options.max_zoom = max_zoom;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_MAX_ZOOM;
    Ok(())
}

fn set_style_tile_tile_size(
    options: *mut sys::mln_style_tile_source_options,
    tile_size: u32,
) -> error::Result<()> {
    let options = style_tile_options_mut(options)?;
    options.tile_size = tile_size;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_TILE_SIZE;
    Ok(())
}

fn style_tile_attribution_storage() -> &'static Mutex<HashMap<usize, CString>> {
    static STORAGE: OnceLock<Mutex<HashMap<usize, CString>>> = OnceLock::new();
    STORAGE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn set_style_tile_attribution(
    options: *mut sys::mln_style_tile_source_options,
    attribution: *const c_char,
) -> error::Result<()> {
    let options_ref = style_tile_options_mut(options)?;
    let (owned, view) = owned_string_view(attribution, "style tile attribution")?;
    style_tile_attribution_storage()
        .lock()
        .expect("style tile attribution storage lock poisoned")
        .insert(options as usize, owned);
    options_ref.attribution = view;
    options_ref.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_ATTRIBUTION;
    Ok(())
}

fn mut_struct<T>(ptr: *mut T, name: &str) -> error::Result<&'static mut T> {
    if ptr.is_null() {
        return Err(Error::invalid_argument(format!("{name} is null")));
    }
    // SAFETY: The caller supplies writable struct storage for this call.
    Ok(unsafe { &mut *ptr })
}

fn copy_struct<T: Copy>(ptr: *const T, name: &str) -> error::Result<T> {
    if ptr.is_null() {
        return Err(Error::invalid_argument(format!("{name} is null")));
    }
    // SAFETY: The caller supplies readable struct storage for this call.
    Ok(unsafe { *ptr })
}

fn set_runtime_maximum_cache_size(
    options: *mut sys::mln_runtime_options,
    maximum_cache_size: u64,
) -> error::Result<()> {
    let options = mut_struct(options, "runtime options")?;
    options.maximum_cache_size = maximum_cache_size;
    options.flags |= sys::MLN_RUNTIME_OPTION_MAXIMUM_CACHE_SIZE;
    Ok(())
}

fn set_camera_center(
    options: *mut sys::mln_camera_options,
    center: *const sys::mln_lat_lng,
) -> error::Result<()> {
    let center = copy_struct(center, "camera center")?;
    let options = mut_struct(options, "camera options")?;
    options.latitude = center.latitude;
    options.longitude = center.longitude;
    options.fields |= sys::MLN_CAMERA_OPTION_CENTER;
    Ok(())
}

fn set_camera_zoom(options: *mut sys::mln_camera_options, zoom: f64) -> error::Result<()> {
    let options = mut_struct(options, "camera options")?;
    options.zoom = zoom;
    options.fields |= sys::MLN_CAMERA_OPTION_ZOOM;
    Ok(())
}

fn set_camera_bearing(options: *mut sys::mln_camera_options, bearing: f64) -> error::Result<()> {
    let options = mut_struct(options, "camera options")?;
    options.bearing = bearing;
    options.fields |= sys::MLN_CAMERA_OPTION_BEARING;
    Ok(())
}

fn set_camera_pitch(options: *mut sys::mln_camera_options, pitch: f64) -> error::Result<()> {
    let options = mut_struct(options, "camera options")?;
    options.pitch = pitch;
    options.fields |= sys::MLN_CAMERA_OPTION_PITCH;
    Ok(())
}

fn set_camera_center_altitude(
    options: *mut sys::mln_camera_options,
    center_altitude: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "camera options")?;
    options.center_altitude = center_altitude;
    options.fields |= sys::MLN_CAMERA_OPTION_CENTER_ALTITUDE;
    Ok(())
}

fn set_camera_padding(
    options: *mut sys::mln_camera_options,
    padding: *const sys::mln_edge_insets,
) -> error::Result<()> {
    let padding = copy_struct(padding, "camera padding")?;
    let options = mut_struct(options, "camera options")?;
    options.padding = padding;
    options.fields |= sys::MLN_CAMERA_OPTION_PADDING;
    Ok(())
}

fn set_camera_anchor(
    options: *mut sys::mln_camera_options,
    anchor: *const sys::mln_screen_point,
) -> error::Result<()> {
    let anchor = copy_struct(anchor, "camera anchor")?;
    let options = mut_struct(options, "camera options")?;
    options.anchor = anchor;
    options.fields |= sys::MLN_CAMERA_OPTION_ANCHOR;
    Ok(())
}

fn set_camera_roll(options: *mut sys::mln_camera_options, roll: f64) -> error::Result<()> {
    let options = mut_struct(options, "camera options")?;
    options.roll = roll;
    options.fields |= sys::MLN_CAMERA_OPTION_ROLL;
    Ok(())
}

fn set_camera_field_of_view(
    options: *mut sys::mln_camera_options,
    field_of_view: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "camera options")?;
    options.field_of_view = field_of_view;
    options.fields |= sys::MLN_CAMERA_OPTION_FOV;
    Ok(())
}

fn set_animation_duration_ms(
    options: *mut sys::mln_animation_options,
    duration_ms: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "animation options")?;
    options.duration_ms = duration_ms;
    options.fields |= sys::MLN_ANIMATION_OPTION_DURATION;
    Ok(())
}

fn set_animation_velocity(
    options: *mut sys::mln_animation_options,
    velocity: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "animation options")?;
    options.velocity = velocity;
    options.fields |= sys::MLN_ANIMATION_OPTION_VELOCITY;
    Ok(())
}

fn set_animation_min_zoom(
    options: *mut sys::mln_animation_options,
    min_zoom: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "animation options")?;
    options.min_zoom = min_zoom;
    options.fields |= sys::MLN_ANIMATION_OPTION_MIN_ZOOM;
    Ok(())
}

fn set_animation_easing(
    options: *mut sys::mln_animation_options,
    easing: *const sys::mln_unit_bezier,
) -> error::Result<()> {
    let easing = copy_struct(easing, "animation easing")?;
    let options = mut_struct(options, "animation options")?;
    options.easing = easing;
    options.fields |= sys::MLN_ANIMATION_OPTION_EASING;
    Ok(())
}

fn set_camera_fit_padding(
    options: *mut sys::mln_camera_fit_options,
    padding: *const sys::mln_edge_insets,
) -> error::Result<()> {
    let padding = copy_struct(padding, "camera fit padding")?;
    let options = mut_struct(options, "camera fit options")?;
    options.padding = padding;
    options.fields |= sys::MLN_CAMERA_FIT_OPTION_PADDING;
    Ok(())
}

fn set_camera_fit_bearing(
    options: *mut sys::mln_camera_fit_options,
    bearing: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "camera fit options")?;
    options.bearing = bearing;
    options.fields |= sys::MLN_CAMERA_FIT_OPTION_BEARING;
    Ok(())
}

fn set_camera_fit_pitch(
    options: *mut sys::mln_camera_fit_options,
    pitch: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "camera fit options")?;
    options.pitch = pitch;
    options.fields |= sys::MLN_CAMERA_FIT_OPTION_PITCH;
    Ok(())
}

fn set_bound_bounds(
    options: *mut sys::mln_bound_options,
    bounds: *const sys::mln_lat_lng_bounds,
) -> error::Result<()> {
    let bounds = copy_struct(bounds, "bound options bounds")?;
    let options = mut_struct(options, "bound options")?;
    options.bounds = bounds;
    options.fields |= sys::MLN_BOUND_OPTION_BOUNDS;
    Ok(())
}

fn set_bound_min_zoom(options: *mut sys::mln_bound_options, min_zoom: f64) -> error::Result<()> {
    let options = mut_struct(options, "bound options")?;
    options.min_zoom = min_zoom;
    options.fields |= sys::MLN_BOUND_OPTION_MIN_ZOOM;
    Ok(())
}

fn set_bound_max_zoom(options: *mut sys::mln_bound_options, max_zoom: f64) -> error::Result<()> {
    let options = mut_struct(options, "bound options")?;
    options.max_zoom = max_zoom;
    options.fields |= sys::MLN_BOUND_OPTION_MAX_ZOOM;
    Ok(())
}

fn set_bound_min_pitch(options: *mut sys::mln_bound_options, min_pitch: f64) -> error::Result<()> {
    let options = mut_struct(options, "bound options")?;
    options.min_pitch = min_pitch;
    options.fields |= sys::MLN_BOUND_OPTION_MIN_PITCH;
    Ok(())
}

fn set_bound_max_pitch(options: *mut sys::mln_bound_options, max_pitch: f64) -> error::Result<()> {
    let options = mut_struct(options, "bound options")?;
    options.max_pitch = max_pitch;
    options.fields |= sys::MLN_BOUND_OPTION_MAX_PITCH;
    Ok(())
}

fn set_free_camera_position(
    options: *mut sys::mln_free_camera_options,
    position: *const sys::mln_vec3,
) -> error::Result<()> {
    let position = copy_struct(position, "free camera position")?;
    let options = mut_struct(options, "free camera options")?;
    options.position = position;
    options.fields |= sys::MLN_FREE_CAMERA_OPTION_POSITION;
    Ok(())
}

fn set_free_camera_orientation(
    options: *mut sys::mln_free_camera_options,
    orientation: *const sys::mln_quaternion,
) -> error::Result<()> {
    let orientation = copy_struct(orientation, "free camera orientation")?;
    let options = mut_struct(options, "free camera options")?;
    options.orientation = orientation;
    options.fields |= sys::MLN_FREE_CAMERA_OPTION_ORIENTATION;
    Ok(())
}

fn set_projection_axonometric(
    mode: *mut sys::mln_projection_mode,
    axonometric: bool,
) -> error::Result<()> {
    let mode = mut_struct(mode, "projection mode")?;
    mode.axonometric = axonometric;
    mode.fields |= sys::MLN_PROJECTION_MODE_AXONOMETRIC;
    Ok(())
}

fn set_projection_x_skew(mode: *mut sys::mln_projection_mode, x_skew: f64) -> error::Result<()> {
    let mode = mut_struct(mode, "projection mode")?;
    mode.x_skew = x_skew;
    mode.fields |= sys::MLN_PROJECTION_MODE_X_SKEW;
    Ok(())
}

fn set_projection_y_skew(mode: *mut sys::mln_projection_mode, y_skew: f64) -> error::Result<()> {
    let mode = mut_struct(mode, "projection mode")?;
    mode.y_skew = y_skew;
    mode.fields |= sys::MLN_PROJECTION_MODE_Y_SKEW;
    Ok(())
}

fn set_viewport_north_orientation(
    options: *mut sys::mln_map_viewport_options,
    north_orientation: u32,
) -> error::Result<()> {
    let options = mut_struct(options, "map viewport options")?;
    options.north_orientation = north_orientation;
    options.fields |= sys::MLN_MAP_VIEWPORT_OPTION_NORTH_ORIENTATION;
    Ok(())
}

fn set_viewport_constrain_mode(
    options: *mut sys::mln_map_viewport_options,
    constrain_mode: u32,
) -> error::Result<()> {
    let options = mut_struct(options, "map viewport options")?;
    options.constrain_mode = constrain_mode;
    options.fields |= sys::MLN_MAP_VIEWPORT_OPTION_CONSTRAIN_MODE;
    Ok(())
}

fn set_viewport_viewport_mode(
    options: *mut sys::mln_map_viewport_options,
    viewport_mode: u32,
) -> error::Result<()> {
    let options = mut_struct(options, "map viewport options")?;
    options.viewport_mode = viewport_mode;
    options.fields |= sys::MLN_MAP_VIEWPORT_OPTION_VIEWPORT_MODE;
    Ok(())
}

fn set_viewport_frustum_offset(
    options: *mut sys::mln_map_viewport_options,
    frustum_offset: *const sys::mln_edge_insets,
) -> error::Result<()> {
    let frustum_offset = copy_struct(frustum_offset, "viewport frustum offset")?;
    let options = mut_struct(options, "map viewport options")?;
    options.frustum_offset = frustum_offset;
    options.fields |= sys::MLN_MAP_VIEWPORT_OPTION_FRUSTUM_OFFSET;
    Ok(())
}

fn set_tile_prefetch_zoom_delta(
    options: *mut sys::mln_map_tile_options,
    prefetch_zoom_delta: u32,
) -> error::Result<()> {
    let options = mut_struct(options, "map tile options")?;
    options.prefetch_zoom_delta = prefetch_zoom_delta;
    options.fields |= sys::MLN_MAP_TILE_OPTION_PREFETCH_ZOOM_DELTA;
    Ok(())
}

fn set_tile_lod_min_radius(
    options: *mut sys::mln_map_tile_options,
    lod_min_radius: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "map tile options")?;
    options.lod_min_radius = lod_min_radius;
    options.fields |= sys::MLN_MAP_TILE_OPTION_LOD_MIN_RADIUS;
    Ok(())
}

fn set_tile_lod_scale(
    options: *mut sys::mln_map_tile_options,
    lod_scale: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "map tile options")?;
    options.lod_scale = lod_scale;
    options.fields |= sys::MLN_MAP_TILE_OPTION_LOD_SCALE;
    Ok(())
}

fn set_tile_lod_pitch_threshold(
    options: *mut sys::mln_map_tile_options,
    lod_pitch_threshold: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "map tile options")?;
    options.lod_pitch_threshold = lod_pitch_threshold;
    options.fields |= sys::MLN_MAP_TILE_OPTION_LOD_PITCH_THRESHOLD;
    Ok(())
}

fn set_tile_lod_zoom_shift(
    options: *mut sys::mln_map_tile_options,
    lod_zoom_shift: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "map tile options")?;
    options.lod_zoom_shift = lod_zoom_shift;
    options.fields |= sys::MLN_MAP_TILE_OPTION_LOD_ZOOM_SHIFT;
    Ok(())
}

fn set_tile_lod_mode(options: *mut sys::mln_map_tile_options, lod_mode: u32) -> error::Result<()> {
    let options = mut_struct(options, "map tile options")?;
    options.lod_mode = lod_mode;
    options.fields |= sys::MLN_MAP_TILE_OPTION_LOD_MODE;
    Ok(())
}

fn set_style_tile_scheme(
    options: *mut sys::mln_style_tile_source_options,
    scheme: u32,
) -> error::Result<()> {
    let options = style_tile_options_mut(options)?;
    options.scheme = scheme;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_SCHEME;
    Ok(())
}

fn set_style_tile_bounds(
    options: *mut sys::mln_style_tile_source_options,
    bounds: *const sys::mln_lat_lng_bounds,
) -> error::Result<()> {
    let bounds = copy_struct(bounds, "style tile bounds")?;
    let options = style_tile_options_mut(options)?;
    options.bounds = bounds;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_BOUNDS;
    Ok(())
}

fn set_style_tile_vector_encoding(
    options: *mut sys::mln_style_tile_source_options,
    vector_encoding: u32,
) -> error::Result<()> {
    let options = style_tile_options_mut(options)?;
    options.vector_encoding = vector_encoding;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_VECTOR_ENCODING;
    Ok(())
}

fn set_style_tile_raster_encoding(
    options: *mut sys::mln_style_tile_source_options,
    raster_encoding: u32,
) -> error::Result<()> {
    let options = style_tile_options_mut(options)?;
    options.raster_encoding = raster_encoding;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_RASTER_ENCODING;
    Ok(())
}

fn custom_geometry_options_mut(
    options: *mut sys::mln_custom_geometry_source_options,
) -> error::Result<&'static mut sys::mln_custom_geometry_source_options> {
    mut_struct(options, "custom geometry source options")
}

fn set_custom_geometry_min_zoom(
    options: *mut sys::mln_custom_geometry_source_options,
    min_zoom: f64,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.min_zoom = min_zoom;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_MIN_ZOOM;
    Ok(())
}

fn set_custom_geometry_max_zoom(
    options: *mut sys::mln_custom_geometry_source_options,
    max_zoom: f64,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.max_zoom = max_zoom;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_MAX_ZOOM;
    Ok(())
}

fn set_custom_geometry_tolerance(
    options: *mut sys::mln_custom_geometry_source_options,
    tolerance: f64,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.tolerance = tolerance;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_TOLERANCE;
    Ok(())
}

fn set_custom_geometry_tile_size(
    options: *mut sys::mln_custom_geometry_source_options,
    tile_size: u32,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.tile_size = tile_size;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_TILE_SIZE;
    Ok(())
}

fn set_custom_geometry_buffer(
    options: *mut sys::mln_custom_geometry_source_options,
    buffer: u32,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.buffer = buffer;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_BUFFER;
    Ok(())
}

fn set_custom_geometry_clip(
    options: *mut sys::mln_custom_geometry_source_options,
    clip: bool,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.clip = clip;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_CLIP;
    Ok(())
}

fn set_custom_geometry_wrap(
    options: *mut sys::mln_custom_geometry_source_options,
    wrap: bool,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.wrap = wrap;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_WRAP;
    Ok(())
}

fn set_style_image_pixel_ratio(
    options: *mut sys::mln_style_image_options,
    pixel_ratio: f32,
) -> error::Result<()> {
    let options = mut_struct(options, "style image options")?;
    options.pixel_ratio = pixel_ratio;
    options.fields |= sys::MLN_STYLE_IMAGE_OPTION_PIXEL_RATIO;
    Ok(())
}

fn set_style_image_sdf(options: *mut sys::mln_style_image_options, sdf: bool) -> error::Result<()> {
    let options = mut_struct(options, "style image options")?;
    options.sdf = sdf;
    options.fields |= sys::MLN_STYLE_IMAGE_OPTION_SDF;
    Ok(())
}

#[derive(Default)]
struct FeatureStateSelectorStringStorage {
    source_id: Option<CString>,
    source_layer_id: Option<CString>,
    feature_id: Option<CString>,
    state_key: Option<CString>,
}

fn feature_state_selector_storage()
-> &'static Mutex<HashMap<usize, FeatureStateSelectorStringStorage>> {
    static STORAGE: OnceLock<Mutex<HashMap<usize, FeatureStateSelectorStringStorage>>> =
        OnceLock::new();
    STORAGE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn owned_string_view(
    value: *const c_char,
    label: &str,
) -> error::Result<(CString, sys::mln_string_view)> {
    if value.is_null() {
        return Err(Error::invalid_argument(format!("{label} is null")));
    }
    let bytes = unsafe { CStr::from_ptr(value) }.to_bytes();
    let owned = CString::new(bytes)
        .map_err(|_| Error::invalid_argument(format!("{label} contains embedded NUL")))?;
    let view = sys::mln_string_view {
        data: owned.as_ptr(),
        size: owned.as_bytes().len(),
    };
    Ok((owned, view))
}

fn set_feature_state_source_id(
    selector: *mut sys::mln_feature_state_selector,
    source_id: *const c_char,
) -> error::Result<()> {
    let (owned, view) = owned_string_view(source_id, "feature-state source ID")?;
    let selector_ref = mut_struct(selector, "feature-state selector")?;
    feature_state_selector_storage()
        .lock()
        .expect("feature-state selector storage lock poisoned")
        .entry(selector as usize)
        .or_default()
        .source_id = Some(owned);
    selector_ref.source_id = view;
    Ok(())
}

fn set_feature_state_source_layer_id(
    selector: *mut sys::mln_feature_state_selector,
    source_layer_id: *const c_char,
) -> error::Result<()> {
    let (owned, view) = owned_string_view(source_layer_id, "feature-state source layer ID")?;
    let selector_ref = mut_struct(selector, "feature-state selector")?;
    feature_state_selector_storage()
        .lock()
        .expect("feature-state selector storage lock poisoned")
        .entry(selector as usize)
        .or_default()
        .source_layer_id = Some(owned);
    selector_ref.source_layer_id = view;
    selector_ref.fields |= sys::MLN_FEATURE_STATE_SELECTOR_SOURCE_LAYER_ID;
    Ok(())
}

fn set_feature_state_feature_id(
    selector: *mut sys::mln_feature_state_selector,
    feature_id: *const c_char,
) -> error::Result<()> {
    let (owned, view) = owned_string_view(feature_id, "feature-state feature ID")?;
    let selector_ref = mut_struct(selector, "feature-state selector")?;
    feature_state_selector_storage()
        .lock()
        .expect("feature-state selector storage lock poisoned")
        .entry(selector as usize)
        .or_default()
        .feature_id = Some(owned);
    selector_ref.feature_id = view;
    selector_ref.fields |= sys::MLN_FEATURE_STATE_SELECTOR_FEATURE_ID;
    Ok(())
}

fn set_feature_state_state_key(
    selector: *mut sys::mln_feature_state_selector,
    state_key: *const c_char,
) -> error::Result<()> {
    let (owned, view) = owned_string_view(state_key, "feature-state state key")?;
    let selector_ref = mut_struct(selector, "feature-state selector")?;
    feature_state_selector_storage()
        .lock()
        .expect("feature-state selector storage lock poisoned")
        .entry(selector as usize)
        .or_default()
        .state_key = Some(owned);
    selector_ref.state_key = view;
    selector_ref.fields |= sys::MLN_FEATURE_STATE_SELECTOR_STATE_KEY;
    Ok(())
}

fn default_custom_geometry_source_options(
    out_options: *mut sys::mln_custom_geometry_source_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_custom_geometry_source_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn default_premultiplied_rgba8_image(
    out_image: *mut sys::mln_premultiplied_rgba8_image,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let image = unsafe { sys::mln_premultiplied_rgba8_image_default() };
    glib::clear_optional_out_pointer(out_image, image)
}

fn init_premultiplied_rgba8_image(
    out_image: *mut sys::mln_premultiplied_rgba8_image,
    width: u32,
    height: u32,
    stride: u32,
    pixels: *const u8,
    byte_length: usize,
) -> error::Result<()> {
    if pixels.is_null() && byte_length != 0 {
        return Err(Error::invalid_argument("image pixel buffer is null"));
    }
    let image = sys::mln_premultiplied_rgba8_image {
        size: std::mem::size_of::<sys::mln_premultiplied_rgba8_image>() as u32,
        width,
        height,
        stride,
        pixels,
        byte_length,
    };
    glib::clear_optional_out_pointer(out_image, image)
}

fn default_style_image_options(
    out_options: *mut sys::mln_style_image_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_style_image_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn default_style_image_info(out_info: *mut sys::mln_style_image_info) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let info = unsafe { sys::mln_style_image_info_default() };
    glib::clear_optional_out_pointer(out_info, info)
}

fn get_camera(
    handle: *mut MapHandle,
    out_camera: *mut sys::mln_camera_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if out_camera.is_null() {
        return Err(Error::invalid_argument("camera output pointer is null"));
    }
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mut camera = unsafe { sys::mln_camera_options_default() };
    // SAFETY: `map` is live and `camera` is valid output storage initialized
    // with the current ABI size.
    error::check(unsafe { sys::mln_map_get_camera(map, &mut camera) })?;
    glib::clear_optional_out_pointer(out_camera, camera)
}

fn get_viewport_options(
    handle: *mut MapHandle,
    out_options: *mut sys::mln_map_viewport_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if out_options.is_null() {
        return Err(Error::invalid_argument(
            "viewport options output pointer is null",
        ));
    }
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mut options = unsafe { sys::mln_map_viewport_options_default() };
    // SAFETY: `map` is live and `options` is valid output storage initialized
    // with the current ABI size.
    error::check(unsafe { sys::mln_map_get_viewport_options(map, &mut options) })?;
    glib::clear_optional_out_pointer(out_options, options)
}

fn set_viewport_options(
    handle: *mut MapHandle,
    options: *const sys::mln_map_viewport_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if options.is_null() {
        return Err(Error::invalid_argument("viewport options are null"));
    }
    // SAFETY: `map` is live and `options` points to caller-owned options.
    error::check(unsafe { sys::mln_map_set_viewport_options(map, options) })
}

fn get_tile_options(
    handle: *mut MapHandle,
    out_options: *mut sys::mln_map_tile_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if out_options.is_null() {
        return Err(Error::invalid_argument(
            "tile options output pointer is null",
        ));
    }
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mut options = unsafe { sys::mln_map_tile_options_default() };
    // SAFETY: `map` is live and `options` is valid output storage initialized
    // with the current ABI size.
    error::check(unsafe { sys::mln_map_get_tile_options(map, &mut options) })?;
    glib::clear_optional_out_pointer(out_options, options)
}

fn set_tile_options(
    handle: *mut MapHandle,
    options: *const sys::mln_map_tile_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if options.is_null() {
        return Err(Error::invalid_argument("tile options are null"));
    }
    // SAFETY: `map` is live and `options` points to caller-owned options.
    error::check(unsafe { sys::mln_map_set_tile_options(map, options) })
}

fn get_bounds(
    handle: *mut MapHandle,
    out_options: *mut sys::mln_bound_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if out_options.is_null() {
        return Err(Error::invalid_argument(
            "bound options output pointer is null",
        ));
    }
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mut options = unsafe { sys::mln_bound_options_default() };
    // SAFETY: `map` is live and `options` is valid output storage initialized
    // with the current ABI size.
    error::check(unsafe { sys::mln_map_get_bounds(map, &mut options) })?;
    glib::clear_optional_out_pointer(out_options, options)
}

fn set_bounds(handle: *mut MapHandle, options: *const sys::mln_bound_options) -> error::Result<()> {
    let map = map_native(handle)?;
    if options.is_null() {
        return Err(Error::invalid_argument("bound options are null"));
    }
    // SAFETY: `map` is live and `options` points to caller-owned options.
    error::check(unsafe { sys::mln_map_set_bounds(map, options) })
}

fn get_free_camera_options(
    handle: *mut MapHandle,
    out_options: *mut sys::mln_free_camera_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if out_options.is_null() {
        return Err(Error::invalid_argument(
            "free camera options output pointer is null",
        ));
    }
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mut options = unsafe { sys::mln_free_camera_options_default() };
    // SAFETY: `map` is live and `options` is valid output storage initialized
    // with the current ABI size.
    error::check(unsafe { sys::mln_map_get_free_camera_options(map, &mut options) })?;
    glib::clear_optional_out_pointer(out_options, options)
}

fn set_free_camera_options(
    handle: *mut MapHandle,
    options: *const sys::mln_free_camera_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if options.is_null() {
        return Err(Error::invalid_argument("free camera options are null"));
    }
    // SAFETY: `map` is live and `options` points to caller-owned options.
    error::check(unsafe { sys::mln_map_set_free_camera_options(map, options) })
}

fn get_projection_mode(
    handle: *mut MapHandle,
    out_mode: *mut sys::mln_projection_mode,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if out_mode.is_null() {
        return Err(Error::invalid_argument(
            "projection mode output pointer is null",
        ));
    }
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mut mode = unsafe { sys::mln_projection_mode_default() };
    // SAFETY: `map` is live and `mode` is valid output storage initialized with
    // the current ABI size.
    error::check(unsafe { sys::mln_map_get_projection_mode(map, &mut mode) })?;
    glib::clear_optional_out_pointer(out_mode, mode)
}

fn set_projection_mode(
    handle: *mut MapHandle,
    mode: *const sys::mln_projection_mode,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if mode.is_null() {
        return Err(Error::invalid_argument("projection mode is null"));
    }
    // SAFETY: `map` is live and `mode` points to caller-owned options.
    error::check(unsafe { sys::mln_map_set_projection_mode(map, mode) })
}

fn camera_for_lat_lng_bounds(
    handle: *mut MapHandle,
    bounds: *const sys::mln_lat_lng_bounds,
    fit_options: *const sys::mln_camera_fit_options,
    out_camera: *mut sys::mln_camera_options,
) -> error::Result<()> {
    if bounds.is_null() {
        return Err(Error::invalid_argument("bounds are null"));
    }
    if out_camera.is_null() {
        return Err(Error::invalid_argument("camera output pointer is null"));
    }
    let map = map_native(handle)?;
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mut camera = unsafe { sys::mln_camera_options_default() };
    // SAFETY: `bounds` is checked non-null and copied by value for the C API;
    // fit_options may be null; local output storage has the required size.
    let bounds = unsafe { *bounds };
    error::check(unsafe {
        sys::mln_map_camera_for_lat_lng_bounds(map, bounds, fit_options, &mut camera)
    })?;
    glib::clear_optional_out_pointer(out_camera, camera)
}

fn camera_for_lat_lngs(
    handle: *mut MapHandle,
    coordinates: *const LatLng,
    coordinate_count: usize,
    fit_options: *const sys::mln_camera_fit_options,
    out_camera: *mut sys::mln_camera_options,
) -> error::Result<()> {
    if coordinates.is_null() {
        return Err(Error::invalid_argument("coordinates are null"));
    }
    if out_camera.is_null() {
        return Err(Error::invalid_argument("camera output pointer is null"));
    }
    let map = map_native(handle)?;
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mut camera = unsafe { sys::mln_camera_options_default() };
    // SAFETY: `LatLng` is repr-compatible with `mln_lat_lng`; coordinates are
    // borrowed for this call and the C API validates count and values. Local
    // output storage has the required size.
    error::check(unsafe {
        sys::mln_map_camera_for_lat_lngs(
            map,
            coordinates.cast::<sys::mln_lat_lng>(),
            coordinate_count,
            fit_options,
            &mut camera,
        )
    })?;
    glib::clear_optional_out_pointer(out_camera, camera)
}

fn camera_for_geometry(
    handle: *mut MapHandle,
    geometry: *const Geometry,
    fit_options: *const sys::mln_camera_fit_options,
    out_camera: *mut sys::mln_camera_options,
) -> error::Result<()> {
    let geometry = values::geometry_ref(geometry)
        .ok_or_else(|| Error::invalid_argument("geometry is null"))?;
    if out_camera.is_null() {
        return Err(Error::invalid_argument("camera output pointer is null"));
    }
    let map = map_native(handle)?;
    let geometry = geometry.materialize()?;
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mut camera = unsafe { sys::mln_camera_options_default() };
    // SAFETY: `map` is live, geometry materializer owns native storage for this
    // call, and local output storage has the required size.
    error::check(unsafe {
        sys::mln_map_camera_for_geometry(map, geometry.as_ptr(), fit_options, &mut camera)
    })?;
    glib::clear_optional_out_pointer(out_camera, camera)
}

fn lat_lng_bounds_for_camera(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    out_bounds: *mut sys::mln_lat_lng_bounds,
    unwrapped: bool,
) -> error::Result<()> {
    if camera.is_null() {
        return Err(Error::invalid_argument("camera options are null"));
    }
    if out_bounds.is_null() {
        return Err(Error::invalid_argument("bounds output pointer is null"));
    }
    let map = map_native(handle)?;
    // SAFETY: `map` is live and pointers are valid for this call.
    let status = unsafe {
        if unwrapped {
            sys::mln_map_lat_lng_bounds_for_camera_unwrapped(map, camera, out_bounds)
        } else {
            sys::mln_map_lat_lng_bounds_for_camera(map, camera, out_bounds)
        }
    };
    error::check(status)
}

fn jump_to(handle: *mut MapHandle, camera: *const sys::mln_camera_options) -> error::Result<()> {
    let map = map_native(handle)?;
    if camera.is_null() {
        return Err(Error::invalid_argument("camera options are null"));
    }
    // SAFETY: `map` is live, and `camera` points to caller-owned options.
    error::check(unsafe { sys::mln_map_jump_to(map, camera) })
}

fn ease_to(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    animation: *const sys::mln_animation_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if camera.is_null() {
        return Err(Error::invalid_argument("camera options are null"));
    }
    // SAFETY: `map` is live, `camera` points to caller-owned options, and the
    // nullable animation pointer is borrowed for this call.
    error::check(unsafe { sys::mln_map_ease_to(map, camera, animation) })
}

fn fly_to(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    animation: *const sys::mln_animation_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if camera.is_null() {
        return Err(Error::invalid_argument("camera options are null"));
    }
    // SAFETY: `map` is live, `camera` points to caller-owned options, and the
    // nullable animation pointer is borrowed for this call.
    error::check(unsafe { sys::mln_map_fly_to(map, camera, animation) })
}

fn move_by(handle: *mut MapHandle, delta_x: f64, delta_y: f64) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map` is live. The C API validates numeric values.
    error::check(unsafe { sys::mln_map_move_by(map, delta_x, delta_y) })
}

fn move_by_animated(
    handle: *mut MapHandle,
    delta_x: f64,
    delta_y: f64,
    animation: *const sys::mln_animation_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map` is live, and nullable animation is borrowed for this call.
    error::check(unsafe { sys::mln_map_move_by_animated(map, delta_x, delta_y, animation) })
}

fn scale_by(
    handle: *mut MapHandle,
    scale: f64,
    anchor: *const sys::mln_screen_point,
) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map` is live, and nullable anchor is borrowed for this call.
    error::check(unsafe { sys::mln_map_scale_by(map, scale, anchor) })
}

fn scale_by_animated(
    handle: *mut MapHandle,
    scale: f64,
    anchor: *const sys::mln_screen_point,
    animation: *const sys::mln_animation_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map` is live. Nullable anchor and animation are borrowed for
    // this call.
    error::check(unsafe { sys::mln_map_scale_by_animated(map, scale, anchor, animation) })
}

fn rotate_by(
    handle: *mut MapHandle,
    first: *const sys::mln_screen_point,
    second: *const sys::mln_screen_point,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if first.is_null() || second.is_null() {
        return Err(Error::invalid_argument("rotation screen point is null"));
    }
    // SAFETY: `map` is live and point pointers were null-checked above.
    error::check(unsafe { sys::mln_map_rotate_by(map, *first, *second) })
}

fn rotate_by_animated(
    handle: *mut MapHandle,
    first: *const sys::mln_screen_point,
    second: *const sys::mln_screen_point,
    animation: *const sys::mln_animation_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if first.is_null() || second.is_null() {
        return Err(Error::invalid_argument("rotation screen point is null"));
    }
    // SAFETY: `map` is live, point pointers were null-checked above, and
    // nullable animation is borrowed for this call.
    error::check(unsafe { sys::mln_map_rotate_by_animated(map, *first, *second, animation) })
}

fn pitch_by(handle: *mut MapHandle, pitch: f64) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map` is live. The C API validates pitch.
    error::check(unsafe { sys::mln_map_pitch_by(map, pitch) })
}

fn pitch_by_animated(
    handle: *mut MapHandle,
    pitch: f64,
    animation: *const sys::mln_animation_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map` is live, and nullable animation is borrowed for this call.
    error::check(unsafe { sys::mln_map_pitch_by_animated(map, pitch, animation) })
}

fn cancel_transitions(handle: *mut MapHandle) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map` is live.
    error::check(unsafe { sys::mln_map_cancel_transitions(map) })
}

fn map_pixel_for_lat_lng(
    handle: *mut MapHandle,
    coordinate: *const LatLng,
    out_point: *mut ScreenPoint,
) -> error::Result<()> {
    if coordinate.is_null() {
        return Err(Error::invalid_argument("coordinate is null"));
    }
    let map = map_native(handle)?;
    // SAFETY: `coordinate` was null-checked and points to one LatLng.
    let coordinate = unsafe { *coordinate };
    let mut native_point = sys::mln_screen_point { x: 0.0, y: 0.0 };
    // SAFETY: `map` is live and output storage is valid.
    error::check(unsafe {
        sys::mln_map_pixel_for_lat_lng(map, coordinate.into(), &mut native_point)
    })?;
    glib::clear_optional_out_pointer(out_point, native_point.into())
}

fn map_lat_lng_for_pixel(
    handle: *mut MapHandle,
    point: *const ScreenPoint,
    out_coordinate: *mut LatLng,
) -> error::Result<()> {
    if point.is_null() {
        return Err(Error::invalid_argument("screen point is null"));
    }
    let map = map_native(handle)?;
    // SAFETY: `point` was null-checked and points to one ScreenPoint.
    let point = unsafe { *point };
    let mut native_coordinate = sys::mln_lat_lng {
        latitude: 0.0,
        longitude: 0.0,
    };
    // SAFETY: `map` is live and output storage is valid.
    error::check(unsafe {
        sys::mln_map_lat_lng_for_pixel(map, point.into(), &mut native_coordinate)
    })?;
    glib::clear_optional_out_pointer(out_coordinate, native_coordinate.into())
}

fn map_pixels_for_lat_lngs(
    handle: *mut MapHandle,
    coordinates: *const LatLng,
    coordinate_count: usize,
    out_points: *mut ScreenPoint,
) -> error::Result<()> {
    if coordinates.is_null() {
        return Err(Error::invalid_argument("coordinates are null"));
    }
    if out_points.is_null() {
        return Err(Error::invalid_argument("points output is null"));
    }
    let map = map_native(handle)?;
    // SAFETY: `LatLng`/`ScreenPoint` are repr-compatible with native structs;
    // pointers are borrowed for this call and native validates counts/capacity.
    error::check(unsafe {
        sys::mln_map_pixels_for_lat_lngs(
            map,
            coordinates.cast::<sys::mln_lat_lng>(),
            coordinate_count,
            out_points.cast::<sys::mln_screen_point>(),
        )
    })
}

fn map_lat_lngs_for_pixels(
    handle: *mut MapHandle,
    points: *const ScreenPoint,
    point_count: usize,
    out_coordinates: *mut LatLng,
) -> error::Result<()> {
    if points.is_null() {
        return Err(Error::invalid_argument("screen points are null"));
    }
    if out_coordinates.is_null() {
        return Err(Error::invalid_argument("coordinates output is null"));
    }
    let map = map_native(handle)?;
    // SAFETY: `LatLng`/`ScreenPoint` are repr-compatible with native structs;
    // pointers are borrowed for this call and native validates counts/capacity.
    error::check(unsafe {
        sys::mln_map_lat_lngs_for_pixels(
            map,
            points.cast::<sys::mln_screen_point>(),
            point_count,
            out_coordinates.cast::<sys::mln_lat_lng>(),
        )
    })
}

fn is_fully_loaded(handle: *mut MapHandle, out_loaded: *mut GBoolean) -> error::Result<()> {
    let map = map_native(handle)?;
    let mut loaded = false;
    // SAFETY: `map` is live and `loaded` is valid output storage.
    error::check(unsafe { sys::mln_map_is_fully_loaded(map, &mut loaded) })?;
    glib::clear_optional_out_pointer(out_loaded, if loaded { GTRUE } else { GFALSE })
}

fn dump_debug_logs(handle: *mut MapHandle) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map_native` returns a live native map pointer.
    error::check(unsafe { sys::mln_map_dump_debug_logs(map) })
}

fn set_style_url(handle: *mut MapHandle, url: *const std::ffi::c_char) -> error::Result<()> {
    let map = map_native(handle)?;
    if url.is_null() {
        return Err(Error::invalid_argument("style URL is null"));
    }
    // SAFETY: `url` is a caller-provided NUL-terminated string pointer and
    // `map_native` returns a live map pointer. The C API copies the input.
    error::check(unsafe { sys::mln_map_set_style_url(map, url) })
}

fn set_style_json(handle: *mut MapHandle, json: *const std::ffi::c_char) -> error::Result<()> {
    let map = map_native(handle)?;
    if json.is_null() {
        return Err(Error::invalid_argument("style JSON is null"));
    }
    // SAFETY: `json` is a caller-provided NUL-terminated string pointer and
    // `map_native` returns a live map pointer. The C API copies the input.
    error::check(unsafe { sys::mln_map_set_style_json(map, json) })?;
    unsafe { destroy_all_custom_geometry_callbacks(handle) };
    Ok(())
}

#[derive(Clone, Copy)]
enum TileSourceKind {
    Vector,
    Raster,
    RasterDem,
}

fn add_geojson_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let url = string_view_from_c(url, "GeoJSON source URL")?;
    // SAFETY: `map` is live and string views borrow NUL-terminated caller
    // strings for this call. The C API copies accepted strings.
    error::check(unsafe { sys::mln_map_add_geojson_source_url(map, source_id, url) })
}

fn add_geojson_source_data(
    handle: *mut MapHandle,
    source_id: *const c_char,
    data: *const GeoJson,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let data = values::geojson_ref(data)
        .ok_or_else(|| Error::invalid_argument("GeoJSON data is null"))?
        .materialize()?;
    // SAFETY: `map` is live, source ID borrows a caller string for this call,
    // and data materializer owns native storage for this call.
    error::check(unsafe { sys::mln_map_add_geojson_source_data(map, source_id, data.as_ptr()) })
}

fn set_geojson_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let url = string_view_from_c(url, "GeoJSON source URL")?;
    // SAFETY: `map` is live and string views borrow NUL-terminated caller
    // strings for this call. The C API copies accepted strings.
    error::check(unsafe { sys::mln_map_set_geojson_source_url(map, source_id, url) })
}

fn set_geojson_source_data(
    handle: *mut MapHandle,
    source_id: *const c_char,
    data: *const GeoJson,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let data = values::geojson_ref(data)
        .ok_or_else(|| Error::invalid_argument("GeoJSON data is null"))?
        .materialize()?;
    // SAFETY: `map` is live, source ID borrows a caller string for this call,
    // and data materializer owns native storage for this call.
    error::check(unsafe { sys::mln_map_set_geojson_source_data(map, source_id, data.as_ptr()) })
}

fn add_tile_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    options: *const sys::mln_style_tile_source_options,
    kind: TileSourceKind,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let url = string_view_from_c(url, "tile source URL")?;
    if options.is_null() {
        return Err(Error::invalid_argument(
            "style tile source options are null",
        ));
    }
    // SAFETY: `map` is live, string views borrow caller strings for this call,
    // and options points to caller-owned options borrowed for this call.
    let status = unsafe {
        match kind {
            TileSourceKind::Vector => {
                sys::mln_map_add_vector_source_url(map, source_id, url, options)
            }
            TileSourceKind::Raster => {
                sys::mln_map_add_raster_source_url(map, source_id, url, options)
            }
            TileSourceKind::RasterDem => {
                sys::mln_map_add_raster_dem_source_url(map, source_id, url, options)
            }
        }
    };
    error::check(status)
}

fn add_tile_source_tiles(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tiles: *const StringList,
    options: *const sys::mln_style_tile_source_options,
    kind: TileSourceKind,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let Some(tiles) = string_list::string_list_ref(tiles) else {
        return Err(Error::invalid_argument("tile URLs are null"));
    };
    if options.is_null() {
        return Err(Error::invalid_argument(
            "style tile source options are null",
        ));
    }
    // SAFETY: `map` is live, `source_id` borrows a caller string for this call,
    // tiles owns validated string views borrowed for this call, and options
    // points to caller-owned options borrowed for this call.
    let status = unsafe {
        match kind {
            TileSourceKind::Vector => sys::mln_map_add_vector_source_tiles(
                map,
                source_id,
                tiles.as_ptr(),
                tiles.len(),
                options,
            ),
            TileSourceKind::Raster => sys::mln_map_add_raster_source_tiles(
                map,
                source_id,
                tiles.as_ptr(),
                tiles.len(),
                options,
            ),
            TileSourceKind::RasterDem => sys::mln_map_add_raster_dem_source_tiles(
                map,
                source_id,
                tiles.as_ptr(),
                tiles.len(),
                options,
            ),
        }
    };
    error::check(status)
}

fn add_custom_geometry_source(
    handle: *mut MapHandle,
    source_id: *const c_char,
    options: *const sys::mln_custom_geometry_source_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    if options.is_null() {
        return Err(Error::invalid_argument(
            "custom geometry source options are null",
        ));
    }
    // SAFETY: `map` is live, source ID borrows a caller string for this call,
    // and options points to caller-owned options borrowed for this call.
    error::check(unsafe { sys::mln_map_add_custom_geometry_source(map, source_id, options) })
}

#[allow(clippy::too_many_arguments)]
fn add_custom_geometry_source_with_callbacks(
    handle: *mut MapHandle,
    source_id: *const c_char,
    fetch_tile: Option<CustomGeometrySourceTileDelegate>,
    fetch_user_data: *mut c_void,
    fetch_destroy_notify: GDestroyNotify,
    cancel_tile: Option<CustomGeometrySourceTileDelegate>,
    cancel_user_data: *mut c_void,
    cancel_destroy_notify: GDestroyNotify,
    options: *const sys::mln_custom_geometry_source_options,
) -> error::Result<()> {
    let Some(fetch_tile) = fetch_tile else {
        return Err(Error::invalid_argument(
            "custom geometry fetch delegate is null",
        ));
    };
    let map = map_native(handle)?;
    if source_id.is_null() {
        return Err(Error::invalid_argument("source ID is null"));
    }
    let source_id_bytes = unsafe { CStr::from_ptr(source_id) }.to_bytes();
    let source_id_copy = CString::new(source_id_bytes)
        .map_err(|_| Error::invalid_argument("source ID contains embedded NUL"))?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let mut native_options = if options.is_null() {
        unsafe { sys::mln_custom_geometry_source_options_default() }
    } else {
        unsafe { *options }
    };
    let state = Box::into_raw(Box::new(CustomGeometrySourceCallbackState {
        source_id: source_id_copy,
        fetch_tile,
        fetch_user_data,
        fetch_destroy_notify,
        cancel_tile,
        cancel_user_data,
        cancel_destroy_notify,
        lifecycle: Mutex::new(CustomGeometrySourceLifecycle {
            closing: false,
            active_callbacks: 0,
        }),
        idle: Condvar::new(),
    }));
    native_options.fetch_tile = Some(custom_geometry_fetch_trampoline);
    native_options.cancel_tile =
        cancel_tile.map(|_| custom_geometry_cancel_trampoline as unsafe extern "C" fn(_, _));
    native_options.user_data = state.cast::<c_void>();
    let status =
        unsafe { sys::mln_map_add_custom_geometry_source(map, source_id, &native_options) };
    if let Err(error) = error::check(status) {
        destroy_custom_geometry_state(state);
        return Err(error);
    }
    unsafe {
        let callbacks = (*handle).custom_geometry_callbacks;
        if callbacks.is_null() {
            destroy_custom_geometry_state(state);
            return Err(Error::new(
                maplibre_native_core::error::ErrorKind::InvalidState,
                None,
                "custom geometry callback list is not initialized",
            ));
        }
        (*callbacks).push(state);
    }
    Ok(())
}

fn destroy_custom_geometry_state(state: *mut CustomGeometrySourceCallbackState) {
    if state.is_null() {
        return;
    }
    let state = unsafe { Box::from_raw(state) };
    if let Ok(mut lifecycle) = state.lifecycle.lock() {
        lifecycle.closing = true;
        while lifecycle.active_callbacks != 0 {
            lifecycle = state
                .idle
                .wait(lifecycle)
                .expect("custom geometry lifecycle lock poisoned while waiting for callbacks");
        }
    }
    if let Some(destroy) = state.fetch_destroy_notify {
        unsafe { destroy(state.fetch_user_data) };
    }
    if let Some(destroy) = state.cancel_destroy_notify {
        unsafe { destroy(state.cancel_user_data) };
    }
}

fn destroy_custom_geometry_callbacks(callbacks: *mut Vec<*mut CustomGeometrySourceCallbackState>) {
    if callbacks.is_null() {
        return;
    }
    let callbacks = unsafe { Box::from_raw(callbacks) };
    for state in callbacks.into_iter() {
        destroy_custom_geometry_state(state);
    }
}

unsafe fn destroy_all_custom_geometry_callbacks(handle: *mut MapHandle) {
    if handle.is_null() {
        return;
    }
    let callbacks = unsafe { (*handle).custom_geometry_callbacks };
    if callbacks.is_null() {
        return;
    }
    let callbacks = unsafe { &mut *callbacks };
    for state in callbacks.drain(..) {
        destroy_custom_geometry_state(state);
    }
}

unsafe fn destroy_custom_geometry_callbacks_for_source(handle: *mut MapHandle, source_id: &[u8]) {
    if handle.is_null() {
        return;
    }
    let callbacks = unsafe { (*handle).custom_geometry_callbacks };
    if callbacks.is_null() {
        return;
    }
    let callbacks = unsafe { &mut *callbacks };
    let mut index = 0;
    while index < callbacks.len() {
        let state = callbacks[index];
        let matches = !state.is_null() && unsafe { (*state).source_id.as_bytes() == source_id };
        if matches {
            let state = callbacks.remove(index);
            destroy_custom_geometry_state(state);
        } else {
            index += 1;
        }
    }
}

fn set_custom_geometry_source_tile_data(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tile_id: *const sys::mln_canonical_tile_id,
    data: *const GeoJson,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    if tile_id.is_null() {
        return Err(Error::invalid_argument("tile ID is null"));
    }
    let data = values::geojson_ref(data)
        .ok_or_else(|| Error::invalid_argument("GeoJSON data is null"))?
        .materialize()?;
    let tile_id = unsafe { *tile_id };
    error::check(unsafe {
        sys::mln_map_set_custom_geometry_source_tile_data(map, source_id, tile_id, data.as_ptr())
    })
}

fn invalidate_custom_geometry_source_tile(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tile_id: *const sys::mln_canonical_tile_id,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    if tile_id.is_null() {
        return Err(Error::invalid_argument("tile ID is null"));
    }
    // SAFETY: `map` is live, source ID borrows a caller string for this call,
    // and tile_id points to a caller-owned descriptor copied by value.
    let tile_id = unsafe { *tile_id };
    error::check(unsafe {
        sys::mln_map_invalidate_custom_geometry_source_tile(map, source_id, tile_id)
    })
}

fn invalidate_custom_geometry_source_region(
    handle: *mut MapHandle,
    source_id: *const c_char,
    bounds: *const sys::mln_lat_lng_bounds,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    if bounds.is_null() {
        return Err(Error::invalid_argument("bounds are null"));
    }
    // SAFETY: `map` is live, source ID borrows a caller string for this call,
    // and bounds points to a caller-owned descriptor borrowed for this call.
    let bounds = unsafe { *bounds };
    error::check(unsafe {
        sys::mln_map_invalidate_custom_geometry_source_region(map, source_id, bounds)
    })
}

fn style_source_exists(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_exists: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let mut exists = false;
    // SAFETY: `map` is live, `source_id` borrows a caller string for this call,
    // and `exists` is valid output storage.
    error::check(unsafe { sys::mln_map_style_source_exists(map, source_id, &mut exists) })?;
    glib::clear_optional_out_pointer(out_exists, if exists { GTRUE } else { GFALSE })
}

fn get_style_source_type(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_source_type: *mut u32,
    out_found: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if out_source_type.is_null() {
        return Err(Error::invalid_argument(
            "source type output pointer is null",
        ));
    }
    let source_id = string_view_from_c(source_id, "source ID")?;
    let mut source_type = sys::MLN_STYLE_SOURCE_TYPE_UNKNOWN;
    let mut found = false;
    // SAFETY: `map` is live, `source_id` borrows a caller string for this call,
    // and output pointers refer to writable storage.
    error::check(unsafe {
        sys::mln_map_get_style_source_type(map, source_id, &mut source_type, &mut found)
    })?;
    glib::clear_optional_out_pointer(out_source_type, source_type)?;
    glib::clear_optional_out_pointer(out_found, if found { GTRUE } else { GFALSE })
}

fn get_style_source_info(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_info: *mut sys::mln_style_source_info,
    out_found: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if out_info.is_null() {
        return Err(Error::invalid_argument(
            "source info output pointer is null",
        ));
    }
    let source_id = string_view_from_c(source_id, "source ID")?;
    let mut info = sys::mln_style_source_info {
        size: std::mem::size_of::<sys::mln_style_source_info>() as u32,
        type_: sys::MLN_STYLE_SOURCE_TYPE_UNKNOWN,
        id_size: 0,
        is_volatile: false,
        has_attribution: false,
        attribution_size: 0,
    };
    let mut found = false;
    // SAFETY: `map` is live, `source_id` borrows a caller string for this call,
    // and output pointers refer to writable storage.
    error::check(unsafe {
        sys::mln_map_get_style_source_info(map, source_id, &mut info, &mut found)
    })?;
    glib::clear_optional_out_pointer(out_info, info)?;
    glib::clear_optional_out_pointer(out_found, if found { GTRUE } else { GFALSE })
}

fn copy_style_source_attribution(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_attribution: *mut c_char,
    attribution_capacity: usize,
    out_attribution_size: *mut usize,
    out_found: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let mut attribution_size = 0;
    let mut found = false;
    // SAFETY: `map` is live, `source_id` borrows a caller string for this call,
    // and the C API validates caller-allocated output storage.
    error::check(unsafe {
        sys::mln_map_copy_style_source_attribution(
            map,
            source_id,
            out_attribution,
            attribution_capacity,
            &mut attribution_size,
            &mut found,
        )
    })?;
    glib::clear_optional_out_pointer(out_attribution_size, attribution_size)?;
    glib::clear_optional_out_pointer(out_found, if found { GTRUE } else { GFALSE })
}

fn remove_style_source(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_removed: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if source_id.is_null() {
        return Err(Error::invalid_argument("source ID is null"));
    }
    let source_id_bytes = unsafe { CStr::from_ptr(source_id) }.to_bytes().to_vec();
    let source_id = string_view_from_c(source_id, "source ID")?;
    let mut removed = false;
    // SAFETY: `map` is live, `source_id` borrows a caller string for this call,
    // and `removed` is valid output storage.
    error::check(unsafe { sys::mln_map_remove_style_source(map, source_id, &mut removed) })?;
    if removed {
        unsafe { destroy_custom_geometry_callbacks_for_source(handle, &source_id_bytes) };
    }
    glib::clear_optional_out_pointer(out_removed, if removed { GTRUE } else { GFALSE })
}

fn set_style_image(
    handle: *mut MapHandle,
    image_id: *const c_char,
    image: *const sys::mln_premultiplied_rgba8_image,
    options: *const sys::mln_style_image_options,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let image_id = string_view_from_c(image_id, "style image ID")?;
    if image.is_null() {
        return Err(Error::invalid_argument("style image is null"));
    }
    if options.is_null() {
        return Err(Error::invalid_argument("style image options are null"));
    }
    // SAFETY: `map` is live, `image_id` borrows a caller string for this call,
    // and image/options point to caller-owned descriptors borrowed for this call.
    error::check(unsafe { sys::mln_map_set_style_image(map, image_id, image, options) })
}

fn remove_style_image(
    handle: *mut MapHandle,
    image_id: *const c_char,
    out_removed: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let image_id = string_view_from_c(image_id, "style image ID")?;
    let mut removed = false;
    // SAFETY: `map` is live, `image_id` borrows a caller string for this call,
    // and `removed` is valid output storage.
    error::check(unsafe { sys::mln_map_remove_style_image(map, image_id, &mut removed) })?;
    glib::clear_optional_out_pointer(out_removed, if removed { GTRUE } else { GFALSE })
}

fn style_image_exists(
    handle: *mut MapHandle,
    image_id: *const c_char,
    out_exists: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let image_id = string_view_from_c(image_id, "style image ID")?;
    let mut exists = false;
    // SAFETY: `map` is live, `image_id` borrows a caller string for this call,
    // and `exists` is valid output storage.
    error::check(unsafe { sys::mln_map_style_image_exists(map, image_id, &mut exists) })?;
    glib::clear_optional_out_pointer(out_exists, if exists { GTRUE } else { GFALSE })
}

fn get_style_image_info(
    handle: *mut MapHandle,
    image_id: *const c_char,
    out_info: *mut sys::mln_style_image_info,
    out_found: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if out_info.is_null() {
        return Err(Error::invalid_argument("style image info output is null"));
    }
    let image_id = string_view_from_c(image_id, "style image ID")?;
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mut info = unsafe { sys::mln_style_image_info_default() };
    let mut found = false;
    // SAFETY: `map` is live, `image_id` borrows a caller string for this call,
    // and output pointers refer to writable storage.
    error::check(unsafe {
        sys::mln_map_get_style_image_info(map, image_id, &mut info, &mut found)
    })?;
    glib::clear_optional_out_pointer(out_info, info)?;
    glib::clear_optional_out_pointer(out_found, if found { GTRUE } else { GFALSE })
}

fn copy_style_image_premultiplied_rgba8(
    handle: *mut MapHandle,
    image_id: *const c_char,
    out_pixels: *mut u8,
    pixel_capacity: usize,
    out_byte_length: *mut usize,
    out_found: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let image_id = string_view_from_c(image_id, "style image ID")?;
    let mut byte_length = 0;
    let mut found = false;
    // SAFETY: `map` is live, `image_id` borrows a caller string for this call,
    // and the C API validates the caller-allocated output buffer.
    error::check(unsafe {
        sys::mln_map_copy_style_image_premultiplied_rgba8(
            map,
            image_id,
            out_pixels,
            pixel_capacity,
            &mut byte_length,
            &mut found,
        )
    })?;
    glib::clear_optional_out_pointer(out_byte_length, byte_length)?;
    glib::clear_optional_out_pointer(out_found, if found { GTRUE } else { GFALSE })
}

fn add_image_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    coordinates: *const sys::mln_lat_lng,
    coordinate_count: usize,
    url: *const c_char,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "image source ID")?;
    let url = string_view_from_c(url, "image source URL")?;
    if coordinates.is_null() {
        return Err(Error::invalid_argument("image source coordinates are null"));
    }
    // SAFETY: `map` is live, string views and coordinates borrow caller storage
    // for this call, and the C API copies accepted values.
    error::check(unsafe {
        sys::mln_map_add_image_source_url(map, source_id, coordinates, coordinate_count, url)
    })
}

fn add_image_source_image(
    handle: *mut MapHandle,
    source_id: *const c_char,
    coordinates: *const sys::mln_lat_lng,
    coordinate_count: usize,
    image: *const sys::mln_premultiplied_rgba8_image,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "image source ID")?;
    if coordinates.is_null() {
        return Err(Error::invalid_argument("image source coordinates are null"));
    }
    if image.is_null() {
        return Err(Error::invalid_argument("image source image is null"));
    }
    // SAFETY: `map` is live and borrowed pointers remain valid for this call.
    error::check(unsafe {
        sys::mln_map_add_image_source_image(map, source_id, coordinates, coordinate_count, image)
    })
}

fn set_image_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "image source ID")?;
    let url = string_view_from_c(url, "image source URL")?;
    // SAFETY: `map` is live and the C API copies the accepted URL.
    error::check(unsafe { sys::mln_map_set_image_source_url(map, source_id, url) })
}

fn set_image_source_image(
    handle: *mut MapHandle,
    source_id: *const c_char,
    image: *const sys::mln_premultiplied_rgba8_image,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "image source ID")?;
    if image.is_null() {
        return Err(Error::invalid_argument("image source image is null"));
    }
    // SAFETY: `map` is live and image points to a borrowed descriptor for this call.
    error::check(unsafe { sys::mln_map_set_image_source_image(map, source_id, image) })
}

fn set_image_source_coordinates(
    handle: *mut MapHandle,
    source_id: *const c_char,
    coordinates: *const sys::mln_lat_lng,
    coordinate_count: usize,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "image source ID")?;
    if coordinates.is_null() {
        return Err(Error::invalid_argument("image source coordinates are null"));
    }
    // SAFETY: `map` is live and coordinates borrow caller storage for this call.
    error::check(unsafe {
        sys::mln_map_set_image_source_coordinates(map, source_id, coordinates, coordinate_count)
    })
}

fn get_image_source_coordinates(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_coordinates: *mut sys::mln_lat_lng,
    coordinate_capacity: usize,
    out_coordinate_count: *mut usize,
    out_found: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "image source ID")?;
    let mut coordinate_count = 0;
    let mut found = false;
    // SAFETY: `map` is live and the C API validates caller-allocated output storage.
    error::check(unsafe {
        sys::mln_map_get_image_source_coordinates(
            map,
            source_id,
            out_coordinates,
            coordinate_capacity,
            &mut coordinate_count,
            &mut found,
        )
    })?;
    glib::clear_optional_out_pointer(out_coordinate_count, coordinate_count)?;
    glib::clear_optional_out_pointer(out_found, if found { GTRUE } else { GFALSE })
}

#[derive(Clone, Copy)]
enum DemLayerKind {
    Hillshade,
    ColorRelief,
}

fn add_dem_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    source_id: *const c_char,
    before_layer_id: *const c_char,
    kind: DemLayerKind,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "DEM layer ID")?;
    let source_id = string_view_from_c(source_id, "DEM source ID")?;
    let before_layer_id = string_view_from_c(before_layer_id, "before layer ID")?;
    // SAFETY: `map` is live and string views borrow caller strings for this call.
    let status = unsafe {
        match kind {
            DemLayerKind::Hillshade => {
                sys::mln_map_add_hillshade_layer(map, layer_id, source_id, before_layer_id)
            }
            DemLayerKind::ColorRelief => {
                sys::mln_map_add_color_relief_layer(map, layer_id, source_id, before_layer_id)
            }
        }
    };
    error::check(status)
}

fn add_location_indicator_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    before_layer_id: *const c_char,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "location indicator layer ID")?;
    let before_layer_id = string_view_from_c(before_layer_id, "before layer ID")?;
    // SAFETY: `map` is live and string views borrow caller strings for this call.
    error::check(unsafe {
        sys::mln_map_add_location_indicator_layer(map, layer_id, before_layer_id)
    })
}

fn set_location_indicator_location(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    coordinate: *const sys::mln_lat_lng,
    altitude: f64,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "location indicator layer ID")?;
    if coordinate.is_null() {
        return Err(Error::invalid_argument(
            "location indicator coordinate is null",
        ));
    }
    // SAFETY: `coordinate` was checked non-null and points to a borrowed value
    // for this call. `map` is live and the C API validates values.
    error::check(unsafe {
        sys::mln_map_set_location_indicator_location(map, layer_id, *coordinate, altitude)
    })
}

fn set_location_indicator_bearing(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    bearing: f64,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "location indicator layer ID")?;
    // SAFETY: `map` is live and the C API validates bearing.
    error::check(unsafe { sys::mln_map_set_location_indicator_bearing(map, layer_id, bearing) })
}

fn set_location_indicator_accuracy_radius(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    radius: f64,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "location indicator layer ID")?;
    // SAFETY: `map` is live and the C API validates radius.
    error::check(unsafe {
        sys::mln_map_set_location_indicator_accuracy_radius(map, layer_id, radius)
    })
}

fn set_location_indicator_image_name(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    image_kind: u32,
    image_id: *const c_char,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "location indicator layer ID")?;
    let image_id = string_view_from_c(image_id, "location indicator image ID")?;
    // SAFETY: `map` is live and the C API validates image kind.
    error::check(unsafe {
        sys::mln_map_set_location_indicator_image_name(map, layer_id, image_kind, image_id)
    })
}

fn style_layer_exists(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    out_exists: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "style layer ID")?;
    let mut exists = false;
    // SAFETY: `map` is live, `layer_id` borrows a caller string for this call,
    // and `exists` is valid output storage.
    error::check(unsafe { sys::mln_map_style_layer_exists(map, layer_id, &mut exists) })?;
    glib::clear_optional_out_pointer(out_exists, if exists { GTRUE } else { GFALSE })
}

fn move_style_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    before_layer_id: *const c_char,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "style layer ID")?;
    let before_layer_id = string_view_from_c(before_layer_id, "before layer ID")?;
    // SAFETY: `map` is live and string views borrow caller strings for this call.
    error::check(unsafe { sys::mln_map_move_style_layer(map, layer_id, before_layer_id) })
}

fn remove_style_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    out_removed: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "style layer ID")?;
    let mut removed = false;
    // SAFETY: `map` is live, `layer_id` borrows a caller string for this call,
    // and `removed` is valid output storage.
    error::check(unsafe { sys::mln_map_remove_style_layer(map, layer_id, &mut removed) })?;
    glib::clear_optional_out_pointer(out_removed, if removed { GTRUE } else { GFALSE })
}

fn get_style_layer_type(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    out_layer_type: *mut *mut c_char,
    out_found: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if out_layer_type.is_null() {
        return Err(Error::invalid_argument(
            "style layer type output pointer is null",
        ));
    }
    let layer_id = string_view_from_c(layer_id, "style layer ID")?;
    let mut layer_type = sys::mln_string_view {
        data: ptr::null(),
        size: 0,
    };
    let mut found = false;
    // SAFETY: `map` is live and output pointers are writable for this call.
    error::check(unsafe {
        sys::mln_map_get_style_layer_type(map, layer_id, &mut layer_type, &mut found)
    })?;
    let layer_type_copy = if found {
        glib::copy_string_view(layer_type)?
    } else {
        ptr::null_mut()
    };
    glib::clear_optional_out_pointer(out_layer_type, layer_type_copy)?;
    glib::clear_optional_out_pointer(out_found, if found { GTRUE } else { GFALSE })
}

fn add_style_source_json(
    handle: *mut MapHandle,
    source_id: *const c_char,
    source_json: *const JsonValue,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let source_json = json_value_native(source_json, "source JSON")?;
    // SAFETY: `map` is live and descriptors are borrowed for this call.
    error::check(unsafe {
        sys::mln_map_add_style_source_json(map, source_id, source_json.as_ptr())
    })
}

fn add_style_layer_json(
    handle: *mut MapHandle,
    layer_json: *const JsonValue,
    before_layer_id: *const c_char,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let layer_json = json_value_native(layer_json, "layer JSON")?;
    let before_layer_id = string_view_from_c(before_layer_id, "before layer ID")?;
    // SAFETY: `map` is live and descriptors are borrowed for this call.
    error::check(unsafe {
        sys::mln_map_add_style_layer_json(map, layer_json.as_ptr(), before_layer_id)
    })
}

fn get_style_layer_json(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    out_found: *mut GBoolean,
) -> error::Result<*mut JsonSnapshotHandle> {
    if out_found.is_null() {
        return Err(Error::invalid_argument(
            "style layer found output pointer is null",
        ));
    }
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "style layer ID")?;
    let mut snapshot = ptr::null_mut();
    let mut found = false;
    // SAFETY: `map` is live and output storage is valid.
    error::check(unsafe {
        sys::mln_map_get_style_layer_json(map, layer_id, &mut snapshot, &mut found)
    })?;
    glib::clear_optional_out_pointer(out_found, if found { GTRUE } else { GFALSE })?;
    if found {
        wrap_json_snapshot(snapshot)
    } else {
        Ok(ptr::null_mut())
    }
}

fn set_style_light_json(handle: *mut MapHandle, light_json: *const JsonValue) -> error::Result<()> {
    let map = map_native(handle)?;
    let light_json = json_value_native(light_json, "style light JSON")?;
    // SAFETY: `map` is live and JSON is borrowed for this call.
    error::check(unsafe { sys::mln_map_set_style_light_json(map, light_json.as_ptr()) })
}

fn set_style_light_property(
    handle: *mut MapHandle,
    property_name: *const c_char,
    value: *const JsonValue,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let property_name = string_view_from_c(property_name, "style light property name")?;
    let value = json_value_native(value, "style light property value")?;
    // SAFETY: `map` is live and descriptors are borrowed for this call.
    error::check(unsafe {
        sys::mln_map_set_style_light_property(map, property_name, value.as_ptr())
    })
}

fn get_style_light_property(
    handle: *mut MapHandle,
    property_name: *const c_char,
) -> error::Result<*mut JsonSnapshotHandle> {
    let map = map_native(handle)?;
    let property_name = string_view_from_c(property_name, "style light property name")?;
    let mut snapshot = ptr::null_mut();
    // SAFETY: `map` is live and output storage is valid and null-initialized.
    error::check(unsafe {
        sys::mln_map_get_style_light_property(map, property_name, &mut snapshot)
    })?;
    wrap_nullable_json_snapshot(snapshot)
}

fn set_layer_property(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    property_name: *const c_char,
    value: *const JsonValue,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "style layer ID")?;
    let property_name = string_view_from_c(property_name, "style layer property name")?;
    let value = json_value_native(value, "style layer property value")?;
    // SAFETY: `map` is live and descriptors are borrowed for this call.
    error::check(unsafe {
        sys::mln_map_set_layer_property(map, layer_id, property_name, value.as_ptr())
    })
}

fn get_layer_property(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    property_name: *const c_char,
) -> error::Result<*mut JsonSnapshotHandle> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "style layer ID")?;
    let property_name = string_view_from_c(property_name, "style layer property name")?;
    let mut snapshot = ptr::null_mut();
    // SAFETY: `map` is live and output storage is valid and null-initialized.
    error::check(unsafe {
        sys::mln_map_get_layer_property(map, layer_id, property_name, &mut snapshot)
    })?;
    wrap_nullable_json_snapshot(snapshot)
}

fn set_layer_filter(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    filter: *const JsonValue,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "style layer ID")?;
    let filter = if filter.is_null() {
        None
    } else {
        Some(json_value_native(filter, "style layer filter")?)
    };
    let filter_ptr = filter
        .as_ref()
        .map_or(ptr::null(), |filter| filter.as_ptr());
    // SAFETY: `map` is live and filter is nullable by contract.
    error::check(unsafe { sys::mln_map_set_layer_filter(map, layer_id, filter_ptr) })
}

fn get_layer_filter(
    handle: *mut MapHandle,
    layer_id: *const c_char,
) -> error::Result<*mut JsonSnapshotHandle> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "style layer ID")?;
    let mut snapshot = ptr::null_mut();
    // SAFETY: `map` is live and output storage is valid and null-initialized.
    error::check(unsafe { sys::mln_map_get_layer_filter(map, layer_id, &mut snapshot) })?;
    wrap_nullable_json_snapshot(snapshot)
}

fn list_style_source_ids(handle: *mut MapHandle) -> error::Result<*mut StyleIdListHandle> {
    let map = map_native(handle)?;
    let mut native = ptr::null_mut();
    // SAFETY: `map` is live and `native` is null before the call as required.
    error::check(unsafe { sys::mln_map_list_style_source_ids(map, &mut native) })?;
    wrap_style_id_list(native)
}

fn list_style_layer_ids(handle: *mut MapHandle) -> error::Result<*mut StyleIdListHandle> {
    let map = map_native(handle)?;
    let mut native = ptr::null_mut();
    // SAFETY: `map` is live and `native` is null before the call as required.
    error::check(unsafe { sys::mln_map_list_style_layer_ids(map, &mut native) })?;
    wrap_style_id_list(native)
}

fn wrap_style_id_list(
    native: *mut sys::mln_style_id_list,
) -> error::Result<*mut StyleIdListHandle> {
    if native.is_null() {
        return Err(Error::invalid_argument("native style ID list is null"));
    }
    let handle = glib::new_object::<StyleIdListHandle>(mln_vala_style_id_list_handle_get_type());
    if handle.is_null() {
        // SAFETY: `native` came from a successful list operation.
        unsafe { sys::mln_style_id_list_destroy(native) };
        return Err(Error::invalid_argument(
            "failed to allocate StyleIdListHandle",
        ));
    }
    // SAFETY: `handle` points to a newly allocated list handle wrapper.
    unsafe {
        (*handle).native = native;
    }
    Ok(handle)
}

fn style_id_list_native(
    handle: *mut StyleIdListHandle,
) -> error::Result<*mut sys::mln_style_id_list> {
    if handle.is_null() {
        return Err(Error::invalid_argument("StyleIdListHandle is null"));
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe { (*handle).native };
    if native.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "StyleIdListHandle is closed",
        ));
    }
    Ok(native)
}

fn style_id_list_count(handle: *mut StyleIdListHandle, out_count: *mut usize) -> error::Result<()> {
    let native = style_id_list_native(handle)?;
    let mut count = 0;
    // SAFETY: `native` is live and `count` is valid output storage.
    error::check(unsafe { sys::mln_style_id_list_count(native, &mut count) })?;
    glib::clear_optional_out_pointer(out_count, count)
}

fn style_id_list_get(handle: *mut StyleIdListHandle, index: usize) -> error::Result<*mut c_char> {
    let native = style_id_list_native(handle)?;
    let mut id = sys::mln_string_view {
        data: ptr::null(),
        size: 0,
    };
    // SAFETY: `native` is live and `id` is valid output storage.
    error::check(unsafe { sys::mln_style_id_list_get(native, index, &mut id) })?;
    glib::copy_string_view(id)
}

fn close_style_id_list(handle: *mut StyleIdListHandle) {
    if handle.is_null() {
        return;
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe {
        let native = (*handle).native;
        (*handle).native = ptr::null_mut();
        native
    };
    if !native.is_null() {
        // SAFETY: This wrapper owns the native list and closes it exactly once.
        unsafe { sys::mln_style_id_list_destroy(native) };
    }
}

fn json_value_native(
    value: *const JsonValue,
    label: &str,
) -> error::Result<maplibre_native_core::json::NativeJsonValue> {
    values::json_ref(value)
        .ok_or_else(|| Error::invalid_argument(format!("{label} is null")))?
        .materialize()
}

fn wrap_nullable_json_snapshot(
    native: *mut sys::mln_json_snapshot,
) -> error::Result<*mut JsonSnapshotHandle> {
    if native.is_null() {
        Ok(ptr::null_mut())
    } else {
        wrap_json_snapshot(native)
    }
}

pub(crate) fn wrap_json_snapshot(
    native: *mut sys::mln_json_snapshot,
) -> error::Result<*mut JsonSnapshotHandle> {
    if native.is_null() {
        return Err(Error::invalid_argument("native JSON snapshot is null"));
    }
    let handle = glib::new_object::<JsonSnapshotHandle>(mln_vala_json_snapshot_handle_get_type());
    if handle.is_null() {
        // SAFETY: `native` came from a successful snapshot-producing operation.
        unsafe { sys::mln_json_snapshot_destroy(native) };
        return Err(Error::invalid_argument(
            "failed to allocate JsonSnapshotHandle",
        ));
    }
    // SAFETY: `handle` points to a newly allocated snapshot wrapper.
    unsafe {
        (*handle).native = native;
    }
    Ok(handle)
}

fn json_snapshot_native(
    handle: *mut JsonSnapshotHandle,
) -> error::Result<*mut sys::mln_json_snapshot> {
    if handle.is_null() {
        return Err(Error::invalid_argument("JsonSnapshotHandle is null"));
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe { (*handle).native };
    if native.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "JsonSnapshotHandle is closed",
        ));
    }
    Ok(native)
}

fn json_snapshot_get(
    handle: *mut JsonSnapshotHandle,
    out_value: *mut *mut JsonValue,
) -> error::Result<()> {
    let native = json_snapshot_native(handle)?;
    if out_value.is_null() {
        return Err(Error::invalid_argument("JSON value output pointer is null"));
    }
    let mut borrowed = ptr::null();
    // SAFETY: `native` is live and output storage is valid.
    error::check(unsafe { sys::mln_json_snapshot_get(native, &mut borrowed) })?;
    if borrowed.is_null() {
        return glib::clear_optional_out_pointer(out_value, ptr::null_mut());
    }
    // SAFETY: borrowed JSON storage is valid while snapshot is live. Copy it
    // into an owned boxed JsonValue before returning to Vala.
    let value = unsafe { maplibre_native_core::json::json_value_from_native(&*borrowed) }?;
    glib::clear_optional_out_pointer(out_value, Box::into_raw(Box::new(JsonValue { value })))
}

fn close_json_snapshot(handle: *mut JsonSnapshotHandle) {
    if handle.is_null() {
        return;
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe {
        let native = (*handle).native;
        (*handle).native = ptr::null_mut();
        native
    };
    if !native.is_null() {
        // SAFETY: This wrapper owns the native snapshot and closes it exactly once.
        unsafe { sys::mln_json_snapshot_destroy(native) };
    }
}

fn wrap_offline_region_snapshot(
    native: *mut sys::mln_offline_region_snapshot,
) -> error::Result<*mut OfflineRegionSnapshotHandle> {
    if native.is_null() {
        return Err(Error::invalid_argument(
            "native offline region snapshot is null",
        ));
    }
    let handle = glib::new_object::<OfflineRegionSnapshotHandle>(
        mln_vala_offline_region_snapshot_handle_get_type(),
    );
    if handle.is_null() {
        // SAFETY: `native` came from a successful take-result operation.
        unsafe { sys::mln_offline_region_snapshot_destroy(native) };
        return Err(Error::invalid_argument(
            "failed to allocate OfflineRegionSnapshotHandle",
        ));
    }
    // SAFETY: `handle` points to a newly allocated snapshot wrapper.
    unsafe {
        (*handle).native = native;
    }
    Ok(handle)
}

fn wrap_offline_region_list(
    native: *mut sys::mln_offline_region_list,
) -> error::Result<*mut OfflineRegionListHandle> {
    if native.is_null() {
        return Err(Error::invalid_argument(
            "native offline region list is null",
        ));
    }
    let handle =
        glib::new_object::<OfflineRegionListHandle>(mln_vala_offline_region_list_handle_get_type());
    if handle.is_null() {
        // SAFETY: `native` came from a successful take-result operation.
        unsafe { sys::mln_offline_region_list_destroy(native) };
        return Err(Error::invalid_argument(
            "failed to allocate OfflineRegionListHandle",
        ));
    }
    // SAFETY: `handle` points to a newly allocated list wrapper.
    unsafe {
        (*handle).native = native;
    }
    Ok(handle)
}

fn offline_region_snapshot_native(
    handle: *mut OfflineRegionSnapshotHandle,
) -> error::Result<*mut sys::mln_offline_region_snapshot> {
    if handle.is_null() {
        return Err(Error::invalid_argument(
            "OfflineRegionSnapshotHandle is null",
        ));
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe { (*handle).native };
    if native.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "OfflineRegionSnapshotHandle is closed",
        ));
    }
    Ok(native)
}

fn offline_region_list_native(
    handle: *mut OfflineRegionListHandle,
) -> error::Result<*mut sys::mln_offline_region_list> {
    if handle.is_null() {
        return Err(Error::invalid_argument("OfflineRegionListHandle is null"));
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe { (*handle).native };
    if native.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "OfflineRegionListHandle is closed",
        ));
    }
    Ok(native)
}

fn offline_region_snapshot_get(
    handle: *mut OfflineRegionSnapshotHandle,
    out_info: *mut sys::mln_offline_region_info,
) -> error::Result<()> {
    let native = offline_region_snapshot_native(handle)?;
    if out_info.is_null() {
        return Err(Error::invalid_argument(
            "offline region info output pointer is null",
        ));
    }
    // SAFETY: `out_info` is valid output storage by the null check above.
    unsafe {
        (*out_info).size = std::mem::size_of::<sys::mln_offline_region_info>() as u32;
    }
    // SAFETY: `native` is live and output storage has the required size field.
    error::check(unsafe { sys::mln_offline_region_snapshot_get(native, out_info) })
}

fn offline_region_list_count(
    handle: *mut OfflineRegionListHandle,
    out_count: *mut usize,
) -> error::Result<()> {
    let native = offline_region_list_native(handle)?;
    let mut count = 0;
    // SAFETY: `native` is live and `count` is valid output storage.
    error::check(unsafe { sys::mln_offline_region_list_count(native, &mut count) })?;
    glib::clear_optional_out_pointer(out_count, count)
}

fn offline_region_list_get(
    handle: *mut OfflineRegionListHandle,
    index: usize,
    out_info: *mut sys::mln_offline_region_info,
) -> error::Result<()> {
    let native = offline_region_list_native(handle)?;
    if out_info.is_null() {
        return Err(Error::invalid_argument(
            "offline region info output pointer is null",
        ));
    }
    // SAFETY: `out_info` is valid output storage by the null check above.
    unsafe {
        (*out_info).size = std::mem::size_of::<sys::mln_offline_region_info>() as u32;
    }
    // SAFETY: `native` is live and output storage has the required size field.
    error::check(unsafe { sys::mln_offline_region_list_get(native, index, out_info) })
}

fn close_offline_region_snapshot(handle: *mut OfflineRegionSnapshotHandle) {
    if handle.is_null() {
        return;
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe {
        let native = (*handle).native;
        (*handle).native = ptr::null_mut();
        native
    };
    if !native.is_null() {
        // SAFETY: This wrapper owns the native snapshot and closes it exactly once.
        unsafe { sys::mln_offline_region_snapshot_destroy(native) };
    }
}

fn close_offline_region_list(handle: *mut OfflineRegionListHandle) {
    if handle.is_null() {
        return;
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe {
        let native = (*handle).native;
        (*handle).native = ptr::null_mut();
        native
    };
    if !native.is_null() {
        // SAFETY: This wrapper owns the native list and closes it exactly once.
        unsafe { sys::mln_offline_region_list_destroy(native) };
    }
}

fn string_view_from_c(value: *const c_char, label: &str) -> error::Result<sys::mln_string_view> {
    if value.is_null() {
        return Err(Error::invalid_argument(format!("{label} is null")));
    }
    // SAFETY: The caller supplies a NUL-terminated C string pointer.
    let bytes = unsafe { CStr::from_ptr(value) }.to_bytes();
    Ok(sys::mln_string_view {
        data: value,
        size: bytes.len(),
    })
}

fn set_resource_transform(
    handle: *mut RuntimeHandle,
    callback: Option<ResourceTransformCallback>,
    user_data: *mut c_void,
    destroy_notify: GDestroyNotify,
) -> error::Result<()> {
    let Some(callback) = callback else {
        return clear_resource_transform(handle);
    };

    let runtime = runtime_native(handle)?;
    let new_state = Box::into_raw(Box::new(ResourceTransformState {
        callback,
        user_data,
        destroy_notify,
        returned_urls: Mutex::new(Vec::new()),
    }));
    let transform = sys::mln_resource_transform {
        size: std::mem::size_of::<sys::mln_resource_transform>() as u32,
        callback: Some(resource_transform_trampoline),
        user_data: new_state.cast::<c_void>(),
    };

    // SAFETY: `runtime` is live and `transform` points to a descriptor borrowed
    // for this call. Native stores the callback and user data by reference;
    // `new_state` stays alive in the RuntimeHandle on success.
    if let Err(error) =
        error::check(unsafe { sys::mln_runtime_set_resource_transform(runtime, &transform) })
    {
        destroy_resource_transform_state(new_state);
        return Err(error);
    }

    let old_state = unsafe {
        let old_state = (*handle).resource_transform;
        (*handle).resource_transform = new_state;
        old_state
    };
    destroy_resource_transform_state(old_state);
    Ok(())
}

fn clear_resource_transform(handle: *mut RuntimeHandle) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    // SAFETY: `runtime` is live. The C API waits for in-flight transform
    // callbacks before returning.
    error::check(unsafe { sys::mln_runtime_clear_resource_transform(runtime) })?;
    let old_state = unsafe {
        let old_state = (*handle).resource_transform;
        (*handle).resource_transform = ptr::null_mut();
        old_state
    };
    destroy_resource_transform_state(old_state);
    Ok(())
}

unsafe extern "C" fn resource_transform_trampoline(
    user_data: *mut c_void,
    kind: u32,
    url: *const c_char,
    out_response: *mut sys::mln_resource_transform_response,
) -> sys::mln_status {
    if user_data.is_null() || out_response.is_null() {
        return sys::MLN_STATUS_INVALID_ARGUMENT;
    }

    let state = user_data.cast::<ResourceTransformState>();
    // SAFETY: Native passes the state pointer installed by
    // `set_resource_transform`; clear/replace waits for in-flight callbacks.
    let replacement_url = unsafe { ((*state).callback)(kind, url, (*state).user_data) };
    // SAFETY: `out_response` was null-checked and points to writable response
    // storage for this callback.
    unsafe {
        (*out_response).size = std::mem::size_of::<sys::mln_resource_transform_response>() as u32;
        (*out_response).url = replacement_url;
    }
    if !replacement_url.is_null() {
        // SAFETY: `state` is the installed transform state and remains valid
        // for the duration of this callback.
        unsafe { &*state }
            .returned_urls
            .lock()
            .expect("resource transform URL storage lock poisoned")
            .push(replacement_url);
    }
    sys::MLN_STATUS_OK
}

fn destroy_resource_transform_state(state: *mut ResourceTransformState) {
    if state.is_null() {
        return;
    }

    // SAFETY: `state` was allocated by Box::into_raw and is no longer reachable
    // from native code after clear/replace/destroy returns.
    let state = unsafe { Box::from_raw(state) };
    for url in state
        .returned_urls
        .into_inner()
        .expect("resource transform URL storage lock poisoned")
    {
        // SAFETY: Vala returns owned strings allocated with GLib allocation.
        unsafe { glib::free(url.cast::<c_void>()) };
    }
    if let Some(destroy_notify) = state.destroy_notify {
        // SAFETY: Destroy notify belongs to the stored user data and is called
        // exactly once when the transform state is replaced, cleared, or closed.
        unsafe { destroy_notify(state.user_data) };
    }
}

fn set_resource_provider(
    handle: *mut RuntimeHandle,
    callback: Option<ResourceProviderCallback>,
    user_data: *mut c_void,
    destroy_notify: GDestroyNotify,
) -> error::Result<()> {
    let Some(callback) = callback else {
        return Err(Error::invalid_argument(
            "resource provider callback is null",
        ));
    };

    let runtime = runtime_native(handle)?;
    let new_state = Box::into_raw(Box::new(ResourceProviderState {
        callback,
        user_data,
        destroy_notify,
    }));
    let provider = sys::mln_resource_provider {
        size: std::mem::size_of::<sys::mln_resource_provider>() as u32,
        callback: Some(resource_provider_trampoline),
        user_data: new_state.cast::<c_void>(),
    };
    // SAFETY: `runtime` is live and the provider references adapter-owned state
    // retained until replacement or runtime close.
    if let Err(error) =
        error::check(unsafe { sys::mln_runtime_set_resource_provider(runtime, &provider) })
    {
        destroy_resource_provider_state(new_state);
        return Err(error);
    }
    // SAFETY: `handle` was checked by `runtime_native`. Replacing the provider
    // is only possible before maps exist; destroy the old adapter state after
    // native accepts the new one.
    let old_state = unsafe {
        let old_state = (*handle).resource_provider;
        (*handle).resource_provider = new_state;
        old_state
    };
    destroy_resource_provider_state(old_state);
    Ok(())
}

unsafe extern "C" fn resource_provider_trampoline(
    user_data: *mut c_void,
    request: *const sys::mln_resource_request,
    native_handle: *mut sys::mln_resource_request_handle,
) -> u32 {
    let state = user_data.cast::<ResourceProviderState>();
    // SAFETY: Native passes a provider-owned request handle for the callback.
    let handle = unsafe { resource_request_handle_from_native(native_handle) };
    if handle.is_null() {
        return sys::MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH;
    }
    // SAFETY: `state` is the installed provider state and remains valid for the
    // runtime lifetime. The callback must obey provider threading rules.
    let decision = unsafe { ((*state).callback)(request, handle, (*state).user_data) };
    // SAFETY: Finalize provider ownership after the callback returns. Inline
    // completion forces HANDLE even if the callback accidentally returns
    // PASS_THROUGH, matching Rust/Java one-shot provider semantics.
    let decision = unsafe { resource_request_handle_finish_provider_decision(handle, decision) };
    glib::unref_object(handle);
    decision
}

fn destroy_resource_provider_state(state: *mut ResourceProviderState) {
    if state.is_null() {
        return;
    }
    // SAFETY: `state` was allocated by Box::into_raw and is no longer reachable
    // from native provider storage after replacement or runtime close.
    let state = unsafe { Box::from_raw(state) };
    if let Some(destroy_notify) = state.destroy_notify {
        // SAFETY: Destroy notify belongs to stored user data and is called once.
        unsafe { destroy_notify(state.user_data) };
    }
}

fn run_ambient_cache_operation_start(
    handle: *mut RuntimeHandle,
    operation: u32,
    out_operation_id: *mut u64,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    if out_operation_id.is_null() {
        return Err(Error::invalid_argument(
            "offline operation ID output pointer is null",
        ));
    }
    // SAFETY: `runtime` is live, and `out_operation_id` points to writable
    // operation ID storage. The C API validates operation values.
    error::check(unsafe {
        sys::mln_runtime_run_ambient_cache_operation_start(runtime, operation, out_operation_id)
    })
}

fn offline_region_create_start(
    handle: *mut RuntimeHandle,
    definition: *const sys::mln_offline_region_definition,
    metadata: *const u8,
    metadata_size: usize,
    out_operation_id: *mut u64,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    if definition.is_null() {
        return Err(Error::invalid_argument("offline region definition is null"));
    }
    if metadata.is_null() && metadata_size != 0 {
        return Err(Error::invalid_argument(
            "offline region metadata is null with non-zero size",
        ));
    }
    if out_operation_id.is_null() {
        return Err(Error::invalid_argument(
            "offline operation ID output pointer is null",
        ));
    }
    // SAFETY: `runtime` is live, definition and metadata are borrowed for this
    // call, and output storage is writable.
    error::check(unsafe {
        sys::mln_runtime_offline_region_create_start(
            runtime,
            definition,
            metadata,
            metadata_size,
            out_operation_id,
        )
    })
}

fn offline_region_get_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    out_operation_id: *mut u64,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    if out_operation_id.is_null() {
        return Err(Error::invalid_argument(
            "offline operation ID output pointer is null",
        ));
    }
    // SAFETY: `runtime` is live and output storage is writable. The C API
    // validates the region ID domain.
    error::check(unsafe {
        sys::mln_runtime_offline_region_get_start(runtime, region_id, out_operation_id)
    })
}

fn offline_regions_list_start(
    handle: *mut RuntimeHandle,
    out_operation_id: *mut u64,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    if out_operation_id.is_null() {
        return Err(Error::invalid_argument(
            "offline operation ID output pointer is null",
        ));
    }
    // SAFETY: `runtime` is live and output storage is writable.
    error::check(unsafe { sys::mln_runtime_offline_regions_list_start(runtime, out_operation_id) })
}

fn offline_regions_merge_database_start(
    handle: *mut RuntimeHandle,
    side_database_path: *const c_char,
    out_operation_id: *mut u64,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    if side_database_path.is_null() {
        return Err(Error::invalid_argument("side database path is null"));
    }
    if out_operation_id.is_null() {
        return Err(Error::invalid_argument(
            "offline operation ID output pointer is null",
        ));
    }
    // SAFETY: `runtime` is live, `side_database_path` is a borrowed C string,
    // and output storage is writable.
    error::check(unsafe {
        sys::mln_runtime_offline_regions_merge_database_start(
            runtime,
            side_database_path,
            out_operation_id,
        )
    })
}

fn offline_region_update_metadata_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    metadata: *const u8,
    metadata_size: usize,
    out_operation_id: *mut u64,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    if metadata.is_null() && metadata_size != 0 {
        return Err(Error::invalid_argument(
            "offline region metadata is null with non-zero size",
        ));
    }
    if out_operation_id.is_null() {
        return Err(Error::invalid_argument(
            "offline operation ID output pointer is null",
        ));
    }
    // SAFETY: `runtime` is live, metadata is either null with zero size or
    // borrowed for this call, and output storage is writable.
    error::check(unsafe {
        sys::mln_runtime_offline_region_update_metadata_start(
            runtime,
            region_id,
            metadata,
            metadata_size,
            out_operation_id,
        )
    })
}

fn offline_region_get_status_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    out_operation_id: *mut u64,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    if out_operation_id.is_null() {
        return Err(Error::invalid_argument(
            "offline operation ID output pointer is null",
        ));
    }
    // SAFETY: `runtime` is live and output storage is writable.
    error::check(unsafe {
        sys::mln_runtime_offline_region_get_status_start(runtime, region_id, out_operation_id)
    })
}

fn offline_region_set_observed_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    observed: bool,
    out_operation_id: *mut u64,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    if out_operation_id.is_null() {
        return Err(Error::invalid_argument(
            "offline operation ID output pointer is null",
        ));
    }
    // SAFETY: `runtime` is live and output storage is writable.
    error::check(unsafe {
        sys::mln_runtime_offline_region_set_observed_start(
            runtime,
            region_id,
            observed,
            out_operation_id,
        )
    })
}

fn offline_region_set_download_state_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    state: u32,
    out_operation_id: *mut u64,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    if out_operation_id.is_null() {
        return Err(Error::invalid_argument(
            "offline operation ID output pointer is null",
        ));
    }
    // SAFETY: `runtime` is live and output storage is writable. The C API
    // validates the download state enum-domain value.
    error::check(unsafe {
        sys::mln_runtime_offline_region_set_download_state_start(
            runtime,
            region_id,
            state,
            out_operation_id,
        )
    })
}

fn offline_region_invalidate_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    out_operation_id: *mut u64,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    if out_operation_id.is_null() {
        return Err(Error::invalid_argument(
            "offline operation ID output pointer is null",
        ));
    }
    // SAFETY: `runtime` is live and output storage is writable.
    error::check(unsafe {
        sys::mln_runtime_offline_region_invalidate_start(runtime, region_id, out_operation_id)
    })
}

fn offline_region_delete_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    out_operation_id: *mut u64,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    if out_operation_id.is_null() {
        return Err(Error::invalid_argument(
            "offline operation ID output pointer is null",
        ));
    }
    // SAFETY: `runtime` is live and output storage is writable.
    error::check(unsafe {
        sys::mln_runtime_offline_region_delete_start(runtime, region_id, out_operation_id)
    })
}

fn offline_region_create_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
) -> error::Result<*mut OfflineRegionSnapshotHandle> {
    let runtime = runtime_native(handle)?;
    let mut snapshot = ptr::null_mut();
    // SAFETY: `runtime` is live and output storage is valid. The C API
    // validates operation state.
    error::check(unsafe {
        sys::mln_runtime_offline_region_create_take_result(runtime, operation_id, &mut snapshot)
    })?;
    wrap_offline_region_snapshot(snapshot)
}

fn offline_region_get_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    out_found: *mut GBoolean,
) -> error::Result<*mut OfflineRegionSnapshotHandle> {
    if out_found.is_null() {
        return Err(Error::invalid_argument(
            "offline region found output pointer is null",
        ));
    }
    let runtime = runtime_native(handle)?;
    let mut snapshot = ptr::null_mut();
    let mut found = false;
    // SAFETY: `runtime` is live and output storage is valid. The C API
    // validates operation state.
    error::check(unsafe {
        sys::mln_runtime_offline_region_get_take_result(
            runtime,
            operation_id,
            &mut snapshot,
            &mut found,
        )
    })?;
    glib::clear_optional_out_pointer(out_found, if found { GTRUE } else { GFALSE })?;
    if found {
        wrap_offline_region_snapshot(snapshot)
    } else {
        Ok(ptr::null_mut())
    }
}

fn offline_regions_list_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
) -> error::Result<*mut OfflineRegionListHandle> {
    let runtime = runtime_native(handle)?;
    let mut list = ptr::null_mut();
    // SAFETY: `runtime` is live and output storage is valid. The C API
    // validates operation state.
    error::check(unsafe {
        sys::mln_runtime_offline_regions_list_take_result(runtime, operation_id, &mut list)
    })?;
    wrap_offline_region_list(list)
}

fn offline_regions_merge_database_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
) -> error::Result<*mut OfflineRegionListHandle> {
    let runtime = runtime_native(handle)?;
    let mut list = ptr::null_mut();
    // SAFETY: `runtime` is live and output storage is valid. The C API
    // validates operation state.
    error::check(unsafe {
        sys::mln_runtime_offline_regions_merge_database_take_result(
            runtime,
            operation_id,
            &mut list,
        )
    })?;
    wrap_offline_region_list(list)
}

fn offline_region_update_metadata_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
) -> error::Result<*mut OfflineRegionSnapshotHandle> {
    let runtime = runtime_native(handle)?;
    let mut snapshot = ptr::null_mut();
    // SAFETY: `runtime` is live and output storage is valid. The C API
    // validates operation state.
    error::check(unsafe {
        sys::mln_runtime_offline_region_update_metadata_take_result(
            runtime,
            operation_id,
            &mut snapshot,
        )
    })?;
    wrap_offline_region_snapshot(snapshot)
}

fn offline_region_get_status_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    out_status: *mut sys::mln_offline_region_status,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    if out_status.is_null() {
        return Err(Error::invalid_argument(
            "offline region status output pointer is null",
        ));
    }
    // SAFETY: `runtime` is live and output storage is writable. The C API
    // validates operation ID state and writes only on success.
    error::check(unsafe {
        sys::mln_runtime_offline_region_get_status_take_result(runtime, operation_id, out_status)
    })
}

fn offline_operation_discard(handle: *mut RuntimeHandle, operation_id: u64) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    // SAFETY: `runtime` is live. The C API validates operation IDs.
    error::check(unsafe { sys::mln_runtime_offline_operation_discard(runtime, operation_id) })
}

fn poll_runtime_event(
    handle: *mut RuntimeHandle,
    out_event: *mut *mut RuntimeEvent,
) -> error::Result<()> {
    if out_event.is_null() {
        return Err(Error::invalid_argument(
            "runtime event output pointer is null",
        ));
    }

    let runtime = runtime_native(handle)?;
    // SAFETY: Zero is a valid baseline before setting the public size field.
    let mut event: sys::mln_runtime_event = unsafe { std::mem::zeroed() };
    event.size = std::mem::size_of::<sys::mln_runtime_event>() as u32;
    let mut has_event = false;
    // SAFETY: `runtime` is live and output pointers are valid for this call.
    error::check(unsafe { sys::mln_runtime_poll_event(runtime, &mut event, &mut has_event) })?;

    // SAFETY: `out_event` was null-checked above.
    let copied_event = if has_event {
        Some(RuntimeEvent::from_native(&event)?)
    } else {
        None
    };

    // SAFETY: `out_event` was null-checked above.
    unsafe {
        *out_event = copied_event.map_or(ptr::null_mut(), |event| Box::into_raw(Box::new(event)));
    }
    Ok(())
}

fn create_runtime_handle(
    options: *const sys::mln_runtime_options,
) -> error::Result<*mut RuntimeHandle> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let default_options;
    let options = if options.is_null() {
        default_options = unsafe { sys::mln_runtime_options_default() };
        &default_options
    } else {
        // SAFETY: The caller supplied a non-null pointer to borrowed runtime
        // options for the duration of this call.
        unsafe { &*options }
    };
    let mut native = ptr::null_mut();
    // SAFETY: `options` and `native` out pointer are valid for this call.
    error::check(unsafe { sys::mln_runtime_create(options, &mut native) })?;

    let handle = glib::new_object::<RuntimeHandle>(mln_vala_runtime_handle_get_type());
    if handle.is_null() {
        // SAFETY: `native` came from successful runtime creation.
        let _ = error::check(unsafe { sys::mln_runtime_destroy(native) });
        return Err(Error::invalid_argument("failed to allocate RuntimeHandle"));
    }

    // SAFETY: `handle` points to an instance allocated with space for
    // `RuntimeHandle`.
    unsafe {
        (*handle).native = native;
        (*handle).resource_transform = ptr::null_mut();
        (*handle).resource_provider = ptr::null_mut();
        (*handle).owner_thread = current_thread_token();
    }
    Ok(handle)
}

fn close_runtime_handle(handle: *mut RuntimeHandle) -> error::Result<()> {
    if handle.is_null() {
        return Err(Error::invalid_argument("RuntimeHandle is null"));
    }
    // SAFETY: `handle` is non-null and expected to point to `RuntimeHandle`.
    let runtime = unsafe { (*handle).native };
    if runtime.is_null() {
        return Ok(());
    }
    // SAFETY: `runtime` is live and owned by this handle.
    error::check(unsafe { sys::mln_runtime_destroy(runtime) })?;
    // SAFETY: `handle` was checked above. Successful runtime destruction makes
    // resource transform callbacks unreachable.
    let (resource_transform, resource_provider) = unsafe {
        (*handle).native = ptr::null_mut();
        let resource_transform = (*handle).resource_transform;
        (*handle).resource_transform = ptr::null_mut();
        let resource_provider = (*handle).resource_provider;
        (*handle).resource_provider = ptr::null_mut();
        (resource_transform, resource_provider)
    };
    destroy_resource_transform_state(resource_transform);
    destroy_resource_provider_state(resource_provider);
    Ok(())
}

fn runtime_native(handle: *mut RuntimeHandle) -> error::Result<*mut sys::mln_runtime> {
    if handle.is_null() {
        return Err(Error::invalid_argument("RuntimeHandle is null"));
    }

    // SAFETY: `handle` is non-null and expected to point to `RuntimeHandle`.
    let native = unsafe { (*handle).native };
    if native.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "RuntimeHandle is closed",
        ));
    }
    Ok(native)
}

fn create_map_handle(
    runtime: *mut RuntimeHandle,
    width: u32,
    height: u32,
    scale_factor: f64,
) -> error::Result<*mut MapHandle> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mut options = unsafe { sys::mln_map_options_default() };
    options.width = width;
    options.height = height;
    options.scale_factor = scale_factor;
    create_map_handle_with_options(runtime, &options)
}

fn create_map_handle_with_options(
    runtime: *mut RuntimeHandle,
    options: *const sys::mln_map_options,
) -> error::Result<*mut MapHandle> {
    let native_runtime = runtime_native(runtime)?;
    if options.is_null() {
        return Err(Error::invalid_argument("map options are null"));
    }

    let mut native = ptr::null_mut();
    // SAFETY: `native_runtime`, `options`, and out pointer are valid for this
    // call. The C API validates option values and copies needed inputs.
    error::check(unsafe { sys::mln_map_create(native_runtime, options, &mut native) })?;

    let handle = glib::new_object::<MapHandle>(mln_vala_map_handle_get_type());
    if handle.is_null() {
        // SAFETY: `native` came from successful map creation.
        let _ = error::check(unsafe { sys::mln_map_destroy(native) });
        return Err(Error::invalid_argument("failed to allocate MapHandle"));
    }

    // SAFETY: `handle` points to an instance allocated with space for
    // `MapHandle`.
    unsafe {
        (*handle).native = native;
        (*handle).runtime = glib::ref_object(runtime);
        (*handle).owner_thread = current_thread_token();
        (*handle).custom_geometry_callbacks = Box::into_raw(Box::new(Vec::new()));
    }
    Ok(handle)
}

fn close_map_handle(handle: *mut MapHandle) -> error::Result<()> {
    if handle.is_null() {
        return Err(Error::invalid_argument("MapHandle is null"));
    }
    // SAFETY: `handle` is non-null and expected to point to `MapHandle`.
    let map = unsafe { (*handle).native };
    if map.is_null() {
        return Ok(());
    }
    // SAFETY: `map` is live and owned by this handle.
    error::check(unsafe { sys::mln_map_destroy(map) })?;
    // SAFETY: `handle` was checked above.
    unsafe {
        (*handle).native = ptr::null_mut();
        destroy_custom_geometry_callbacks((*handle).custom_geometry_callbacks);
        (*handle).custom_geometry_callbacks = ptr::null_mut();
        glib::unref_object((*handle).runtime);
        (*handle).runtime = ptr::null_mut();
    }
    Ok(())
}

pub(crate) fn map_native(handle: *mut MapHandle) -> error::Result<*mut sys::mln_map> {
    if handle.is_null() {
        return Err(Error::invalid_argument("MapHandle is null"));
    }

    // SAFETY: `handle` is non-null and expected to point to `MapHandle`.
    let native = unsafe { (*handle).native };
    if native.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "MapHandle is closed",
        ));
    }
    Ok(native)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static PROVIDER_DESTROY_COUNT: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn passthrough_transform(
        _kind: u32,
        _url: *const c_char,
        _user_data: *mut c_void,
    ) -> *mut c_char {
        ptr::null_mut()
    }

    unsafe extern "C" fn replacing_transform(
        _kind: u32,
        _url: *const c_char,
        _user_data: *mut c_void,
    ) -> *mut c_char {
        crate::glib::copy_string_view(sys::mln_string_view {
            data: c"asset://replacement".as_ptr(),
            size: "asset://replacement".len(),
        })
        .expect("test replacement URL allocation should succeed")
    }

    unsafe extern "C" fn counted_transform_destroy_notify(user_data: *mut c_void) {
        assert!(!user_data.is_null());
        // SAFETY: Tests pass a live `AtomicUsize` pointer as user_data and
        // trigger destroy-notify before the counter leaves scope.
        unsafe { &*(user_data as *const AtomicUsize) }.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn passthrough_provider(
        _request: *const sys::mln_resource_request,
        _handle: *mut ResourceRequestHandle,
        _user_data: *mut c_void,
    ) -> u32 {
        sys::MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH
    }

    unsafe extern "C" fn provider_destroy_notify(_user_data: *mut c_void) {
        PROVIDER_DESTROY_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn custom_geometry_tile_callback(
        _user_data: *mut c_void,
        _tile_id: sys::mln_canonical_tile_id,
    ) {
    }

    #[test]
    fn runtime_handle_create_run_and_close() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());

        assert_eq!(
            mln_vala_runtime_handle_run_once(runtime, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );

        glib::unref_object(runtime);
    }

    #[test]
    fn runtime_and_map_options_create_handles() {
        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut runtime_options: sys::mln_runtime_options = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_runtime_options_default(&mut runtime_options, ptr::null_mut()),
            GTRUE
        );
        let runtime = mln_vala_runtime_handle_new_with_options(&runtime_options, ptr::null_mut());
        assert!(!runtime.is_null());

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut map_options: sys::mln_map_options = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_map_options_default(&mut map_options, ptr::null_mut()),
            GTRUE
        );
        map_options.width = 128;
        map_options.height = 128;
        map_options.scale_factor = 1.0;
        let map = mln_vala_map_handle_new_with_options(runtime, &map_options, ptr::null_mut());
        assert!(!map.is_null());

        assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
        assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );

        glib::unref_object(map);
        glib::unref_object(runtime);
    }

    #[test]
    fn map_owner_thread_errors_propagate_as_gerror() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let map = mln_vala_map_handle_new(runtime, 128, 128, 1.0, ptr::null_mut());
        assert!(!map.is_null());

        let map_bits = map as usize;
        let wrong_thread_failed = std::thread::spawn(move || {
            let map = map_bits as *mut MapHandle;
            let mut loaded = GFALSE;
            let mut error = ptr::null_mut();
            let ok = mln_vala_map_handle_is_fully_loaded(map, &mut loaded, &mut error);
            ok == GFALSE && !error.is_null()
        })
        .join()
        .expect("wrong-thread probe should not panic");
        assert!(wrong_thread_failed);

        assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );
        glib::unref_object(map);
        glib::unref_object(runtime);
    }

    #[test]
    fn gobject_finalize_releases_runtime_callback_state() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let destroy_count = AtomicUsize::new(0);

        assert_eq!(
            mln_vala_runtime_handle_set_resource_transform(
                runtime,
                Some(passthrough_transform),
                &destroy_count as *const AtomicUsize as *mut c_void,
                Some(counted_transform_destroy_notify),
                ptr::null_mut(),
            ),
            GTRUE
        );
        glib::unref_object(runtime);
        assert_eq!(destroy_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn gobject_finalize_releases_map_before_runtime_close() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let map = mln_vala_map_handle_new(runtime, 128, 128, 1.0, ptr::null_mut());
        assert!(!map.is_null());

        glib::unref_object(map);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );
        glib::unref_object(runtime);
    }

    #[test]
    fn resource_transform_replacement_and_clear_run_destroy_notify() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let destroy_count = AtomicUsize::new(0);

        assert_eq!(
            mln_vala_runtime_handle_set_resource_transform(
                runtime,
                Some(passthrough_transform),
                &destroy_count as *const AtomicUsize as *mut c_void,
                Some(counted_transform_destroy_notify),
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(destroy_count.load(Ordering::SeqCst), 0);

        assert_eq!(
            mln_vala_runtime_handle_set_resource_transform(
                runtime,
                Some(passthrough_transform),
                &destroy_count as *const AtomicUsize as *mut c_void,
                Some(counted_transform_destroy_notify),
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(destroy_count.load(Ordering::SeqCst), 1);

        assert_eq!(
            mln_vala_runtime_handle_clear_resource_transform(runtime, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(destroy_count.load(Ordering::SeqCst), 2);

        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );
        glib::unref_object(runtime);
    }

    #[test]
    fn resource_transform_retains_non_null_replacement_urls_until_destroy() {
        let state = Box::into_raw(Box::new(ResourceTransformState {
            callback: replacing_transform,
            user_data: ptr::null_mut(),
            destroy_notify: None,
            returned_urls: Mutex::new(Vec::new()),
        }));
        // SAFETY: Zeroed storage is initialized by the trampoline before use.
        let mut first_response: sys::mln_resource_transform_response =
            unsafe { std::mem::zeroed() };
        // SAFETY: Zeroed storage is initialized by the trampoline before use.
        let mut second_response: sys::mln_resource_transform_response =
            unsafe { std::mem::zeroed() };

        // SAFETY: `state` is valid callback state, input URLs are static
        // NUL-terminated strings, and response pointers are writable.
        assert_eq!(
            unsafe {
                resource_transform_trampoline(
                    state.cast::<c_void>(),
                    sys::MLN_RESOURCE_KIND_STYLE,
                    c"asset://one".as_ptr(),
                    &mut first_response,
                )
            },
            sys::MLN_STATUS_OK
        );
        assert_eq!(
            unsafe {
                resource_transform_trampoline(
                    state.cast::<c_void>(),
                    sys::MLN_RESOURCE_KIND_STYLE,
                    c"asset://two".as_ptr(),
                    &mut second_response,
                )
            },
            sys::MLN_STATUS_OK
        );

        assert!(!first_response.url.is_null());
        assert!(!second_response.url.is_null());
        // SAFETY: The transform state retains both GLib-allocated URLs until
        // `destroy_resource_transform_state` below.
        assert_eq!(
            unsafe { CStr::from_ptr(first_response.url) },
            c"asset://replacement"
        );
        // SAFETY: The second response URL is likewise retained until teardown.
        assert_eq!(
            unsafe { CStr::from_ptr(second_response.url) },
            c"asset://replacement"
        );
        assert_eq!(
            unsafe { &*state }
                .returned_urls
                .lock()
                .expect("resource transform URL storage lock poisoned")
                .len(),
            2
        );

        destroy_resource_transform_state(state);
    }

    #[test]
    fn resource_provider_registration_runs_destroy_notify_on_close() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        PROVIDER_DESTROY_COUNT.store(0, Ordering::SeqCst);

        assert_eq!(
            mln_vala_runtime_handle_set_resource_provider(
                runtime,
                Some(passthrough_provider),
                ptr::null_mut(),
                Some(provider_destroy_notify),
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(PROVIDER_DESTROY_COUNT.load(Ordering::SeqCst), 0);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(PROVIDER_DESTROY_COUNT.load(Ordering::SeqCst), 1);
        glib::unref_object(runtime);
    }

    #[test]
    fn offline_operation_wrappers_report_invalid_inputs() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());

        let mut operation_id = 0;
        let mut error = ptr::null_mut();
        assert_eq!(
            mln_vala_runtime_handle_run_ambient_cache_operation_start(
                runtime,
                0,
                &mut operation_id,
                &mut error,
            ),
            GFALSE
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        assert_eq!(
            mln_vala_runtime_handle_offline_regions_list_start(
                runtime,
                ptr::null_mut(),
                &mut error
            ),
            GFALSE
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        assert_eq!(
            mln_vala_runtime_handle_offline_regions_merge_database_start(
                runtime,
                ptr::null(),
                &mut operation_id,
                &mut error,
            ),
            GFALSE
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        assert_eq!(
            mln_vala_runtime_handle_offline_region_update_metadata_start(
                runtime,
                1,
                ptr::null(),
                1,
                &mut operation_id,
                &mut error,
            ),
            GFALSE
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        assert_eq!(
            mln_vala_runtime_handle_offline_region_get_status_take_result(
                runtime,
                0,
                ptr::null_mut(),
                &mut error,
            ),
            GFALSE
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        assert_eq!(
            mln_vala_offline_region_snapshot_handle_get(
                ptr::null_mut(),
                ptr::null_mut(),
                &mut error
            ),
            GFALSE
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        assert_eq!(
            mln_vala_offline_region_list_handle_get(
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                &mut error
            ),
            GFALSE
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        assert_eq!(
            mln_vala_runtime_handle_offline_operation_discard(runtime, 0, &mut error),
            GFALSE
        );
        assert!(!error.is_null());

        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );
        glib::unref_object(runtime);
    }

    #[test]
    fn map_handle_retains_runtime_until_close() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
        assert!(!map.is_null());

        assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );

        glib::unref_object(map);
        glib::unref_object(runtime);
    }

    #[test]
    fn geojson_source_url_lifecycle_round_trips_existence() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
        assert!(!map.is_null());

        assert_eq!(
            mln_vala_map_handle_set_style_json(
                map,
                c"{\"version\":8,\"sources\":{},\"layers\":[]}".as_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_add_geojson_source_url(
                map,
                c"fixture-source".as_ptr(),
                c"asset://fixture.geojson".as_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        let mut exists = GFALSE;
        assert_eq!(
            mln_vala_map_handle_style_source_exists(
                map,
                c"fixture-source".as_ptr(),
                &mut exists,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(exists, GTRUE);
        let mut source_type = sys::MLN_STYLE_SOURCE_TYPE_UNKNOWN;
        let mut found = GFALSE;
        assert_eq!(
            mln_vala_map_handle_get_style_source_type(
                map,
                c"fixture-source".as_ptr(),
                &mut source_type,
                &mut found,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(found, GTRUE);
        assert_eq!(source_type, sys::MLN_STYLE_SOURCE_TYPE_GEOJSON);
        let mut source_info = sys::mln_style_source_info {
            size: std::mem::size_of::<sys::mln_style_source_info>() as u32,
            type_: sys::MLN_STYLE_SOURCE_TYPE_UNKNOWN,
            id_size: 0,
            is_volatile: false,
            has_attribution: false,
            attribution_size: 0,
        };
        assert_eq!(
            mln_vala_map_handle_get_style_source_info(
                map,
                c"fixture-source".as_ptr(),
                &mut source_info,
                &mut found,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(found, GTRUE);
        assert_eq!(source_info.type_, sys::MLN_STYLE_SOURCE_TYPE_GEOJSON);
        assert_eq!(
            mln_vala_map_handle_set_geojson_source_url(
                map,
                c"fixture-source".as_ptr(),
                c"asset://fixture-updated.geojson".as_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        let mut removed = GFALSE;
        assert_eq!(
            mln_vala_map_handle_remove_style_source(
                map,
                c"fixture-source".as_ptr(),
                &mut removed,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(removed, GTRUE);

        let geojson = GeoJson {
            value: maplibre_native_core::GeoJson::Geometry(maplibre_native_core::Geometry::Point(
                maplibre_native_core::LatLng::new(0.0, 0.0),
            )),
        };
        assert_eq!(
            mln_vala_map_handle_add_geojson_source_data(
                map,
                c"fixture-data-source".as_ptr(),
                &geojson,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_geojson_source_data(
                map,
                c"fixture-data-source".as_ptr(),
                &geojson,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_remove_style_source(
                map,
                c"fixture-data-source".as_ptr(),
                &mut removed,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(removed, GTRUE);

        let mut custom_options = unsafe { sys::mln_custom_geometry_source_options_default() };
        assert_eq!(
            mln_vala_custom_geometry_source_options_default(&mut custom_options, ptr::null_mut()),
            GTRUE
        );
        custom_options.fetch_tile = Some(custom_geometry_tile_callback);
        assert_eq!(
            mln_vala_map_handle_add_custom_geometry_source(
                map,
                c"custom-geometry-source".as_ptr(),
                &custom_options,
                ptr::null_mut(),
            ),
            GTRUE
        );
        let tile_id = sys::mln_canonical_tile_id { z: 0, x: 0, y: 0 };
        assert_eq!(
            mln_vala_map_handle_set_custom_geometry_source_tile_data(
                map,
                c"custom-geometry-source".as_ptr(),
                &tile_id,
                &geojson,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_invalidate_custom_geometry_source_tile(
                map,
                c"custom-geometry-source".as_ptr(),
                &tile_id,
                ptr::null_mut(),
            ),
            GTRUE
        );
        let bounds = sys::mln_lat_lng_bounds {
            southwest: sys::mln_lat_lng {
                latitude: -1.0,
                longitude: -1.0,
            },
            northeast: sys::mln_lat_lng {
                latitude: 1.0,
                longitude: 1.0,
            },
        };
        assert_eq!(
            mln_vala_map_handle_invalidate_custom_geometry_source_region(
                map,
                c"custom-geometry-source".as_ptr(),
                &bounds,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_remove_style_source(
                map,
                c"custom-geometry-source".as_ptr(),
                &mut removed,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(removed, GTRUE);

        let mut tile_options = unsafe { sys::mln_style_tile_source_options_default() };
        assert_eq!(
            mln_vala_style_tile_source_options_default(&mut tile_options, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_add_vector_source_url(
                map,
                c"vector-source".as_ptr(),
                c"asset://vector-source.json".as_ptr(),
                &tile_options,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_remove_style_source(
                map,
                c"vector-source".as_ptr(),
                &mut removed,
                ptr::null_mut(),
            ),
            GTRUE
        );
        let vector_tile_url = c"asset://vector/{z}/{x}/{y}.pbf";
        let vector_tile_values = [vector_tile_url.as_ptr()];
        let vector_tiles = crate::string_list::mln_vala_string_list_new(
            vector_tile_values.as_ptr(),
            vector_tile_values.len(),
            ptr::null_mut(),
        );
        assert!(!vector_tiles.is_null());
        assert_eq!(
            mln_vala_map_handle_add_vector_source_tiles(
                map,
                c"vector-tiles-source".as_ptr(),
                vector_tiles,
                &tile_options,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_remove_style_source(
                map,
                c"vector-tiles-source".as_ptr(),
                &mut removed,
                ptr::null_mut(),
            ),
            GTRUE
        );
        let raster_tile_url = c"asset://raster/{z}/{x}/{y}.png";
        let raster_tile_values = [raster_tile_url.as_ptr()];
        let raster_tiles = crate::string_list::mln_vala_string_list_new(
            raster_tile_values.as_ptr(),
            raster_tile_values.len(),
            ptr::null_mut(),
        );
        assert!(!raster_tiles.is_null());
        assert_eq!(
            mln_vala_map_handle_add_raster_source_tiles(
                map,
                c"raster-tiles-source".as_ptr(),
                raster_tiles,
                &tile_options,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_remove_style_source(
                map,
                c"raster-tiles-source".as_ptr(),
                &mut removed,
                ptr::null_mut(),
            ),
            GTRUE
        );
        let dem_tile_url = c"asset://dem/{z}/{x}/{y}.png";
        let dem_tile_values = [dem_tile_url.as_ptr()];
        let dem_tiles = crate::string_list::mln_vala_string_list_new(
            dem_tile_values.as_ptr(),
            dem_tile_values.len(),
            ptr::null_mut(),
        );
        assert!(!dem_tiles.is_null());
        assert_eq!(
            mln_vala_map_handle_add_raster_dem_source_tiles(
                map,
                c"dem-tiles-source".as_ptr(),
                dem_tiles,
                &tile_options,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_remove_style_source(
                map,
                c"dem-tiles-source".as_ptr(),
                &mut removed,
                ptr::null_mut(),
            ),
            GTRUE
        );

        crate::string_list::mln_vala_string_list_free(vector_tiles);
        crate::string_list::mln_vala_string_list_free(raster_tiles);
        crate::string_list::mln_vala_string_list_free(dem_tiles);

        assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );

        glib::unref_object(map);
        glib::unref_object(runtime);
    }

    #[test]
    fn style_image_lifecycle_round_trips_metadata_and_pixels() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
        assert!(!map.is_null());

        assert_eq!(
            mln_vala_map_handle_set_style_json(
                map,
                c"{\"version\":8,\"sources\":{},\"layers\":[]}".as_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );

        let pixels = [255_u8, 0, 0, 255];
        // SAFETY: Zeroed storage is immediately initialized by the public init
        // helper before use.
        let mut image: sys::mln_premultiplied_rgba8_image = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_premultiplied_rgba8_image_init(
                &mut image,
                1,
                1,
                4,
                pixels.as_ptr(),
                pixels.len(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(image.byte_length, pixels.len());

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut options: sys::mln_style_image_options = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_style_image_options_default(&mut options, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_style_image(
                map,
                c"fixture-image".as_ptr(),
                &image,
                &options,
                ptr::null_mut(),
            ),
            GTRUE
        );

        let mut exists = GFALSE;
        assert_eq!(
            mln_vala_map_handle_style_image_exists(
                map,
                c"fixture-image".as_ptr(),
                &mut exists,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(exists, GTRUE);

        // SAFETY: Zeroed storage is immediately initialized by the wrapper.
        let mut info: sys::mln_style_image_info = unsafe { std::mem::zeroed() };
        let mut found = GFALSE;
        assert_eq!(
            mln_vala_map_handle_get_style_image_info(
                map,
                c"fixture-image".as_ptr(),
                &mut info,
                &mut found,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(found, GTRUE);
        assert_eq!(info.width, 1);
        assert_eq!(info.byte_length, pixels.len());

        let mut copied = [0_u8; 4];
        let mut byte_length = 0;
        assert_eq!(
            mln_vala_map_handle_copy_style_image_premultiplied_rgba8(
                map,
                c"fixture-image".as_ptr(),
                copied.as_mut_ptr(),
                copied.len(),
                &mut byte_length,
                &mut found,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(found, GTRUE);
        assert_eq!(byte_length, pixels.len());
        assert_eq!(copied, pixels);

        let mut removed = GFALSE;
        assert_eq!(
            mln_vala_map_handle_remove_style_image(
                map,
                c"fixture-image".as_ptr(),
                &mut removed,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(removed, GTRUE);

        assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );

        glib::unref_object(map);
        glib::unref_object(runtime);
    }

    #[test]
    fn image_source_lifecycle_round_trips_coordinates() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
        assert!(!map.is_null());

        assert_eq!(
            mln_vala_map_handle_set_style_json(
                map,
                c"{\"version\":8,\"sources\":{},\"layers\":[]}".as_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );

        let coordinates = [
            sys::mln_lat_lng {
                latitude: 0.0,
                longitude: 0.0,
            },
            sys::mln_lat_lng {
                latitude: 0.0,
                longitude: 1.0,
            },
            sys::mln_lat_lng {
                latitude: 1.0,
                longitude: 1.0,
            },
            sys::mln_lat_lng {
                latitude: 1.0,
                longitude: 0.0,
            },
        ];
        assert_eq!(
            mln_vala_map_handle_add_image_source_url(
                map,
                c"image-source".as_ptr(),
                coordinates.as_ptr(),
                coordinates.len(),
                c"asset://image.png".as_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_image_source_url(
                map,
                c"image-source".as_ptr(),
                c"asset://image-updated.png".as_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_image_source_coordinates(
                map,
                c"image-source".as_ptr(),
                coordinates.as_ptr(),
                coordinates.len(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        let mut copied_coordinates = [sys::mln_lat_lng {
            latitude: 0.0,
            longitude: 0.0,
        }; 4];
        let mut coordinate_count = 0;
        let mut found = GFALSE;
        assert_eq!(
            mln_vala_map_handle_get_image_source_coordinates(
                map,
                c"image-source".as_ptr(),
                copied_coordinates.as_mut_ptr(),
                copied_coordinates.len(),
                &mut coordinate_count,
                &mut found,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(found, GTRUE);
        assert_eq!(coordinate_count, 4);
        assert_eq!(copied_coordinates[2].latitude, 1.0);
        let mut removed = GFALSE;
        assert_eq!(
            mln_vala_map_handle_remove_style_source(
                map,
                c"image-source".as_ptr(),
                &mut removed,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(removed, GTRUE);

        let pixels = [255_u8, 0, 0, 255];
        // SAFETY: Zeroed storage is immediately initialized by the public init helper.
        let mut image: sys::mln_premultiplied_rgba8_image = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_premultiplied_rgba8_image_init(
                &mut image,
                1,
                1,
                4,
                pixels.as_ptr(),
                pixels.len(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_add_image_source_image(
                map,
                c"inline-image-source".as_ptr(),
                coordinates.as_ptr(),
                coordinates.len(),
                &image,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_image_source_image(
                map,
                c"inline-image-source".as_ptr(),
                &image,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_remove_style_source(
                map,
                c"inline-image-source".as_ptr(),
                &mut removed,
                ptr::null_mut(),
            ),
            GTRUE
        );

        assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );

        glib::unref_object(map);
        glib::unref_object(runtime);
    }

    #[test]
    fn location_indicator_layer_lifecycle_round_trips_existence() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
        assert!(!map.is_null());

        assert_eq!(
            mln_vala_map_handle_set_style_json(
                map,
                c"{\"version\":8,\"sources\":{},\"layers\":[]}".as_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_add_location_indicator_layer(
                map,
                c"location-layer".as_ptr(),
                c"".as_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        let coordinate = sys::mln_lat_lng {
            latitude: 37.7749,
            longitude: -122.4194,
        };
        assert_eq!(
            mln_vala_map_handle_set_location_indicator_location(
                map,
                c"location-layer".as_ptr(),
                &coordinate,
                0.0,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_location_indicator_bearing(
                map,
                c"location-layer".as_ptr(),
                15.0,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_location_indicator_accuracy_radius(
                map,
                c"location-layer".as_ptr(),
                10.0,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_location_indicator_image_name(
                map,
                c"location-layer".as_ptr(),
                sys::MLN_LOCATION_INDICATOR_IMAGE_KIND_TOP,
                c"fixture-image".as_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        let mut exists = GFALSE;
        assert_eq!(
            mln_vala_map_handle_style_layer_exists(
                map,
                c"location-layer".as_ptr(),
                &mut exists,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(exists, GTRUE);

        let mut layer_type = ptr::null_mut();
        let mut found = GFALSE;
        assert_eq!(
            mln_vala_map_handle_get_style_layer_type(
                map,
                c"location-layer".as_ptr(),
                &mut layer_type,
                &mut found,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(found, GTRUE);
        assert!(!layer_type.is_null());
        // SAFETY: Layer type was allocated with GLib allocation by the wrapper.
        unsafe { glib::free(layer_type.cast::<c_void>()) };

        let layer_ids = mln_vala_map_handle_list_style_layer_ids(map, ptr::null_mut());
        assert!(!layer_ids.is_null());
        let mut layer_count = 0;
        assert_eq!(
            mln_vala_style_id_list_handle_count(layer_ids, &mut layer_count, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(layer_count, 0);
        let first_layer_id = mln_vala_style_id_list_handle_get(layer_ids, 0, ptr::null_mut());
        assert!(!first_layer_id.is_null());
        // SAFETY: ID was allocated with GLib allocation by the wrapper.
        unsafe { glib::free(first_layer_id.cast::<c_void>()) };
        mln_vala_style_id_list_handle_close(layer_ids);
        glib::unref_object(layer_ids);

        let source_ids = mln_vala_map_handle_list_style_source_ids(map, ptr::null_mut());
        assert!(!source_ids.is_null());
        let mut source_count = 0;
        assert_eq!(
            mln_vala_style_id_list_handle_count(source_ids, &mut source_count, ptr::null_mut()),
            GTRUE
        );
        mln_vala_style_id_list_handle_close(source_ids);
        glib::unref_object(source_ids);

        assert_eq!(
            mln_vala_map_handle_move_style_layer(
                map,
                c"location-layer".as_ptr(),
                c"".as_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        let mut removed = GFALSE;
        assert_eq!(
            mln_vala_map_handle_remove_style_layer(
                map,
                c"location-layer".as_ptr(),
                &mut removed,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(removed, GTRUE);

        assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );

        glib::unref_object(map);
        glib::unref_object(runtime);
    }

    #[test]
    fn map_debug_and_rendering_stats_options_round_trip() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
        assert!(!map.is_null());

        assert_eq!(
            mln_vala_map_handle_set_debug_options(
                map,
                sys::MLN_MAP_DEBUG_TILE_BORDERS,
                ptr::null_mut(),
            ),
            GTRUE
        );
        let mut debug_options = 0;
        assert_eq!(
            mln_vala_map_handle_get_debug_options(map, &mut debug_options, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(debug_options, sys::MLN_MAP_DEBUG_TILE_BORDERS);

        assert_eq!(
            mln_vala_map_handle_set_rendering_stats_view_enabled(map, GTRUE, ptr::null_mut()),
            GTRUE
        );
        let mut rendering_stats_enabled = GFALSE;
        assert_eq!(
            mln_vala_map_handle_get_rendering_stats_view_enabled(
                map,
                &mut rendering_stats_enabled,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(rendering_stats_enabled, GTRUE);

        assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );

        glib::unref_object(map);
        glib::unref_object(runtime);
    }

    #[test]
    fn map_camera_commands_accept_default_descriptors() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
        assert!(!map.is_null());

        // SAFETY: Zeroed storage is immediately initialized through the public
        // default entry point before use.
        let mut camera: sys::mln_camera_options = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_camera_options_default(&mut camera, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(camera.size, 0);
        assert_eq!(
            mln_vala_map_handle_get_camera(map, &mut camera, ptr::null_mut()),
            GTRUE
        );

        // SAFETY: Zeroed storage is immediately initialized through the public
        // default entry point before use.
        let mut animation: sys::mln_animation_options = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_animation_options_default(&mut animation, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(animation.size, 0);

        // SAFETY: Zeroed storage is immediately initialized through the public
        // default entry point before use.
        let mut fit_options: sys::mln_camera_fit_options = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_camera_fit_options_default(&mut fit_options, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(fit_options.size, 0);
        let bounds = sys::mln_lat_lng_bounds {
            southwest: sys::mln_lat_lng {
                latitude: -1.0,
                longitude: -1.0,
            },
            northeast: sys::mln_lat_lng {
                latitude: 1.0,
                longitude: 1.0,
            },
        };
        assert_eq!(
            mln_vala_map_handle_camera_for_lat_lng_bounds(
                map,
                &bounds,
                &fit_options,
                &mut camera,
                ptr::null_mut(),
            ),
            GTRUE
        );
        let fit_coordinates = [
            LatLng {
                latitude: -1.0,
                longitude: -1.0,
            },
            LatLng {
                latitude: 1.0,
                longitude: 1.0,
            },
        ];
        assert_eq!(
            mln_vala_map_handle_camera_for_lat_lngs(
                map,
                fit_coordinates.as_ptr(),
                fit_coordinates.len(),
                &fit_options,
                &mut camera,
                ptr::null_mut(),
            ),
            GTRUE
        );
        // SAFETY: Zeroed storage is overwritten through the public output API.
        let mut fitted_bounds: sys::mln_lat_lng_bounds = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_map_handle_lat_lng_bounds_for_camera(
                map,
                &camera,
                &mut fitted_bounds,
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_lat_lng_bounds_for_camera_unwrapped(
                map,
                &camera,
                &mut fitted_bounds,
                ptr::null_mut(),
            ),
            GTRUE
        );

        assert_eq!(
            mln_vala_map_handle_jump_to(map, &camera, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_ease_to(map, &camera, &animation, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_move_by(map, 0.0, 0.0, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_scale_by(map, 1.0, ptr::null(), ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_pitch_by(map, 0.0, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_cancel_transitions(map, ptr::null_mut()),
            GTRUE
        );

        let coordinate = LatLng {
            latitude: 0.0,
            longitude: 0.0,
        };
        let mut point = ScreenPoint { x: 0.0, y: 0.0 };
        assert_eq!(
            mln_vala_map_handle_pixel_for_lat_lng(map, &coordinate, &mut point, ptr::null_mut()),
            GTRUE
        );
        let mut round_trip = LatLng {
            latitude: 999.0,
            longitude: 999.0,
        };
        assert_eq!(
            mln_vala_map_handle_lat_lng_for_pixel(map, &point, &mut round_trip, ptr::null_mut()),
            GTRUE
        );
        let coordinates = [coordinate];
        let mut points = [ScreenPoint { x: 0.0, y: 0.0 }];
        assert_eq!(
            mln_vala_map_handle_pixels_for_lat_lngs(
                map,
                coordinates.as_ptr(),
                coordinates.len(),
                points.as_mut_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );
        let mut round_trips = [LatLng {
            latitude: 0.0,
            longitude: 0.0,
        }];
        assert_eq!(
            mln_vala_map_handle_lat_lngs_for_pixels(
                map,
                points.as_ptr(),
                points.len(),
                round_trips.as_mut_ptr(),
                ptr::null_mut(),
            ),
            GTRUE
        );

        assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );

        glib::unref_object(map);
        glib::unref_object(runtime);
    }

    #[test]
    fn map_state_option_descriptors_round_trip() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
        assert!(!map.is_null());

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut viewport: sys::mln_map_viewport_options = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_map_viewport_options_default(&mut viewport, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_get_viewport_options(map, &mut viewport, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_viewport_options(map, &viewport, ptr::null_mut()),
            GTRUE
        );

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut tile: sys::mln_map_tile_options = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_map_tile_options_default(&mut tile, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_get_tile_options(map, &mut tile, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_tile_options(map, &tile, ptr::null_mut()),
            GTRUE
        );

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut bounds: sys::mln_bound_options = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_bound_options_default(&mut bounds, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_get_bounds(map, &mut bounds, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_bounds(map, &bounds, ptr::null_mut()),
            GTRUE
        );

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut free_camera: sys::mln_free_camera_options = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_free_camera_options_default(&mut free_camera, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_get_free_camera_options(map, &mut free_camera, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_free_camera_options(map, &free_camera, ptr::null_mut()),
            GTRUE
        );

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut projection_mode: sys::mln_projection_mode = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_projection_mode_default(&mut projection_mode, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_get_projection_mode(map, &mut projection_mode, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_handle_set_projection_mode(map, &projection_mode, ptr::null_mut()),
            GTRUE
        );

        assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );

        glib::unref_object(map);
        glib::unref_object(runtime);
    }
}

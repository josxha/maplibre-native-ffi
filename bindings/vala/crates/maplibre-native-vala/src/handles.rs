use std::collections::hash_map::DefaultHasher;
use std::ffi::{CStr, CString, c_char, c_void};
use std::hash::{Hash, Hasher};
use std::ptr;
use std::ptr::NonNull;
use std::sync::{Condvar, Mutex};

use maplibre_native_core as core;
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

mod map_exports;
mod offline_exports;
mod option_exports;
mod options_impl;
mod runtime_exports;

pub use map_exports::*;
pub use offline_exports::*;
pub use option_exports::*;
use options_impl::*;
pub use runtime_exports::*;

const RUNTIME_TYPE_NAME: &CStr = c"MlnValaRuntimeHandle";
const MAP_TYPE_NAME: &CStr = c"MlnValaMapHandle";
const OFFLINE_REGION_DEFINITION_TYPE_NAME: &CStr = c"MlnValaOfflineRegionDefinition";
const OFFLINE_REGION_INFO_TYPE_NAME: &CStr = c"MlnValaOfflineRegionInfo";
const OFFLINE_REGION_INFO_LIST_TYPE_NAME: &CStr = c"MlnValaOfflineRegionInfoList";
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

fn gerror_bool(result: error::Result<()>, error_out: *mut *mut GError) -> GBoolean {
    match result {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

fn gerror_pointer<T>(result: error::Result<*mut T>, error_out: *mut *mut GError) -> *mut T {
    match result {
        Ok(pointer) => pointer,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
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
#[derive(Clone, Debug, PartialEq)]
pub struct OfflineRegionDefinition {
    value: core::OfflineRegionDefinition,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct OfflineRegionInfo {
    value: core::OfflineRegionInfo,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct OfflineRegionInfoList {
    regions: Vec<core::OfflineRegionInfo>,
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
    if unsafe { has_custom_geometry_callbacks(handle) } {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "set_style_url is unavailable while custom geometry callbacks are registered; remove the source, replace with inline style JSON, or close the map first",
        ));
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

unsafe fn has_custom_geometry_callbacks(handle: *mut MapHandle) -> bool {
    if handle.is_null() {
        return false;
    }
    let callbacks = unsafe { (*handle).custom_geometry_callbacks };
    if callbacks.is_null() {
        return false;
    }
    !unsafe { &*callbacks }.is_empty()
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
) -> error::Result<Option<core::JsonValue>> {
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
        copy_json_snapshot(snapshot)
    } else {
        Ok(None)
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
) -> error::Result<Option<core::JsonValue>> {
    let map = map_native(handle)?;
    let property_name = string_view_from_c(property_name, "style light property name")?;
    let mut snapshot = ptr::null_mut();
    // SAFETY: `map` is live and output storage is valid and null-initialized.
    error::check(unsafe {
        sys::mln_map_get_style_light_property(map, property_name, &mut snapshot)
    })?;
    copy_nullable_json_snapshot(snapshot)
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
) -> error::Result<Option<core::JsonValue>> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "style layer ID")?;
    let property_name = string_view_from_c(property_name, "style layer property name")?;
    let mut snapshot = ptr::null_mut();
    // SAFETY: `map` is live and output storage is valid and null-initialized.
    error::check(unsafe {
        sys::mln_map_get_layer_property(map, layer_id, property_name, &mut snapshot)
    })?;
    copy_nullable_json_snapshot(snapshot)
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
) -> error::Result<Option<core::JsonValue>> {
    let map = map_native(handle)?;
    let layer_id = string_view_from_c(layer_id, "style layer ID")?;
    let mut snapshot = ptr::null_mut();
    // SAFETY: `map` is live and output storage is valid and null-initialized.
    error::check(unsafe { sys::mln_map_get_layer_filter(map, layer_id, &mut snapshot) })?;
    copy_nullable_json_snapshot(snapshot)
}

fn list_style_source_ids(handle: *mut MapHandle) -> error::Result<StringList> {
    let map = map_native(handle)?;
    let mut native = ptr::null_mut();
    // SAFETY: `map` is live and `native` is null before the call as required.
    error::check(unsafe { sys::mln_map_list_style_source_ids(map, &mut native) })?;
    copy_style_id_list(native)
}

fn list_style_layer_ids(handle: *mut MapHandle) -> error::Result<StringList> {
    let map = map_native(handle)?;
    let mut native = ptr::null_mut();
    // SAFETY: `map` is live and `native` is null before the call as required.
    error::check(unsafe { sys::mln_map_list_style_layer_ids(map, &mut native) })?;
    copy_style_id_list(native)
}

fn copy_style_id_list(native: *mut sys::mln_style_id_list) -> error::Result<StringList> {
    let handle = wrap_style_id_list(native)?;
    let mut count = 0usize;
    let copy_result = (|| {
        style_id_list_count(handle, &mut count)?;
        let mut strings = Vec::with_capacity(count);
        for index in 0..count {
            let id = style_id_list_get(handle, index)?;
            if id.is_null() {
                return Err(Error::invalid_argument("style ID copy is null"));
            }
            // SAFETY: style_id_list_get returns a GLib-allocated NUL-terminated string.
            let bytes = unsafe { CStr::from_ptr(id) }.to_bytes();
            let string = CString::new(bytes)
                .map_err(|_| Error::invalid_argument("style ID contains embedded NUL"));
            // SAFETY: transfer the temporary GLib string copy back to GLib on every exit path.
            unsafe { glib::free(id.cast()) };
            strings.push(string?);
        }
        Ok(StringList::from_strings(strings))
    })();
    close_style_id_list(handle);
    glib::unref_object(handle);
    copy_result
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

fn copy_nullable_json_snapshot(
    native: *mut sys::mln_json_snapshot,
) -> error::Result<Option<core::JsonValue>> {
    if native.is_null() {
        Ok(None)
    } else {
        copy_json_snapshot(native)
    }
}

fn copy_json_snapshot(
    native: *mut sys::mln_json_snapshot,
) -> error::Result<Option<core::JsonValue>> {
    let native = NonNull::new(native)
        .ok_or_else(|| Error::invalid_argument("native JSON snapshot is null"))?;
    // SAFETY: the C API returned an owned JSON snapshot; the core helper copies
    // the JSON value and releases the native snapshot on every exit path.
    unsafe { core::json::copy_json_snapshot(Some(native)) }
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

fn offline_region_definition_ref(
    value: *const OfflineRegionDefinition,
) -> Option<&'static OfflineRegionDefinition> {
    if value.is_null() {
        None
    } else {
        Some(unsafe { &*value })
    }
}

fn offline_region_info_ref(value: *const OfflineRegionInfo) -> Option<&'static OfflineRegionInfo> {
    if value.is_null() {
        None
    } else {
        Some(unsafe { &*value })
    }
}

fn offline_region_info_list_ref(
    value: *const OfflineRegionInfoList,
) -> Option<&'static OfflineRegionInfoList> {
    if value.is_null() {
        None
    } else {
        Some(unsafe { &*value })
    }
}

fn offline_region_info_list_get(
    value: *const OfflineRegionInfoList,
    index: usize,
) -> error::Result<OfflineRegionInfo> {
    let list = offline_region_info_list_ref(value)
        .ok_or_else(|| Error::invalid_argument("offline region info list is null"))?;
    let value = list
        .regions
        .get(index)
        .ok_or_else(|| Error::invalid_argument("offline region info index is out of range"))?
        .clone();
    Ok(OfflineRegionInfo { value })
}

fn offline_region_definition_new_tile_pyramid(
    style_url: *const c_char,
    bounds: *const sys::mln_lat_lng_bounds,
    min_zoom: f64,
    max_zoom: f64,
    pixel_ratio: f32,
    include_ideographs: bool,
) -> error::Result<OfflineRegionDefinition> {
    if bounds.is_null() {
        return Err(Error::invalid_argument(
            "offline tile-pyramid bounds are null",
        ));
    }
    let style_url = copy_input_c_string(style_url)?;
    let bounds = core::values::lat_lng_bounds_from_native(unsafe { *bounds });
    Ok(OfflineRegionDefinition {
        value: core::OfflineRegionDefinition::TilePyramid {
            style_url,
            bounds,
            min_zoom,
            max_zoom,
            pixel_ratio,
            include_ideographs,
        },
    })
}

fn offline_region_definition_new_geometry(
    style_url: *const c_char,
    geometry: *const Geometry,
    min_zoom: f64,
    max_zoom: f64,
    pixel_ratio: f32,
    include_ideographs: bool,
) -> error::Result<OfflineRegionDefinition> {
    let style_url = copy_input_c_string(style_url)?;
    let geometry = values::geometry_ref(geometry)
        .ok_or_else(|| Error::invalid_argument("offline geometry region geometry is null"))?
        .value
        .clone();
    Ok(OfflineRegionDefinition {
        value: core::OfflineRegionDefinition::GeometryRegion {
            style_url,
            geometry,
            min_zoom,
            max_zoom,
            pixel_ratio,
            include_ideographs,
        },
    })
}

fn copy_input_c_string(value: *const c_char) -> error::Result<String> {
    if value.is_null() {
        return Err(Error::invalid_argument("string is null"));
    }
    // SAFETY: The caller supplies a NUL-terminated C string pointer.
    let bytes = unsafe { CStr::from_ptr(value) }.to_bytes();
    Ok(std::str::from_utf8(bytes)
        .map_err(|_| Error::invalid_argument("string is not valid UTF-8"))?
        .to_owned())
}

fn copy_string(value: &str) -> Option<*mut c_char> {
    let c_string = CString::new(value).ok()?;
    glib::copy_string_view(sys::mln_string_view {
        data: c_string.as_ptr(),
        size: c_string.as_bytes().len(),
    })
    .ok()
}

unsafe extern "C" fn offline_region_definition_copy_erased(value: *mut c_void) -> *mut c_void {
    mln_vala_offline_region_definition_copy(value.cast()).cast()
}

unsafe extern "C" fn offline_region_definition_free_erased(value: *mut c_void) {
    mln_vala_offline_region_definition_free(value.cast());
}

unsafe extern "C" fn offline_region_info_copy_erased(value: *mut c_void) -> *mut c_void {
    mln_vala_offline_region_info_copy(value.cast()).cast()
}

unsafe extern "C" fn offline_region_info_free_erased(value: *mut c_void) {
    mln_vala_offline_region_info_free(value.cast());
}

unsafe extern "C" fn offline_region_info_list_copy_erased(value: *mut c_void) -> *mut c_void {
    mln_vala_offline_region_info_list_copy(value.cast()).cast()
}

unsafe extern "C" fn offline_region_info_list_free_erased(value: *mut c_void) {
    mln_vala_offline_region_info_list_free(value.cast());
}

fn copy_offline_region_snapshot(
    native: *mut sys::mln_offline_region_snapshot,
) -> error::Result<OfflineRegionInfo> {
    let native = NonNull::new(native)
        .ok_or_else(|| Error::invalid_argument("native offline region snapshot is null"))?;
    // SAFETY: the C API returned an owned snapshot; the core helper copies all
    // region data and releases the native snapshot on every exit path.
    let value = unsafe { core::runtime::copy_offline_region_snapshot(native) }?;
    Ok(OfflineRegionInfo { value })
}

fn copy_offline_region_list(
    native: *mut sys::mln_offline_region_list,
) -> error::Result<OfflineRegionInfoList> {
    let native = NonNull::new(native)
        .ok_or_else(|| Error::invalid_argument("native offline region list is null"))?;
    // SAFETY: the C API returned an owned list; the core helper copies all
    // region data and releases the native list on every exit path.
    let regions = unsafe { core::runtime::copy_offline_region_list(native) }?;
    Ok(OfflineRegionInfoList { regions })
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
    definition: *const OfflineRegionDefinition,
    metadata: *const u8,
    metadata_size: usize,
    out_operation_id: *mut u64,
) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    let definition = offline_region_definition_ref(definition)
        .ok_or_else(|| Error::invalid_argument("offline region definition is null"))?;
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
    let definition = core::runtime::offline_region_definition_to_native(&definition.value)?;
    let definition = definition.to_raw();
    // SAFETY: `runtime` is live, definition-owned storage is borrowed for this
    // call, and output storage is writable.
    error::check(unsafe {
        sys::mln_runtime_offline_region_create_start(
            runtime,
            &definition,
            metadata,
            metadata_size,
            out_operation_id,
        )
    })
}

#[cfg(test)]
fn initialize_offline_region_definition(definition: &mut sys::mln_offline_region_definition) {
    definition.size = std::mem::size_of::<sys::mln_offline_region_definition>() as u32;
    match definition.type_ {
        sys::MLN_OFFLINE_REGION_DEFINITION_TILE_PYRAMID => {
            definition.data.tile_pyramid.size =
                std::mem::size_of::<sys::mln_offline_tile_pyramid_region_definition>() as u32;
        }
        sys::MLN_OFFLINE_REGION_DEFINITION_GEOMETRY => {
            definition.data.geometry.size =
                std::mem::size_of::<sys::mln_offline_geometry_region_definition>() as u32;
        }
        _ => {}
    }
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
) -> error::Result<OfflineRegionInfo> {
    let runtime = runtime_native(handle)?;
    let mut snapshot = ptr::null_mut();
    // SAFETY: `runtime` is live and output storage is valid. The C API
    // validates operation state.
    error::check(unsafe {
        sys::mln_runtime_offline_region_create_take_result(runtime, operation_id, &mut snapshot)
    })?;
    copy_offline_region_snapshot(snapshot)
}

fn offline_region_get_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    out_found: *mut GBoolean,
) -> error::Result<Option<OfflineRegionInfo>> {
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
        copy_offline_region_snapshot(snapshot).map(Some)
    } else {
        Ok(None)
    }
}

fn offline_regions_list_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
) -> error::Result<OfflineRegionInfoList> {
    let runtime = runtime_native(handle)?;
    let mut list = ptr::null_mut();
    // SAFETY: `runtime` is live and output storage is valid. The C API
    // validates operation state.
    error::check(unsafe {
        sys::mln_runtime_offline_regions_list_take_result(runtime, operation_id, &mut list)
    })?;
    copy_offline_region_list(list)
}

fn offline_regions_merge_database_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
) -> error::Result<OfflineRegionInfoList> {
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
    copy_offline_region_list(list)
}

fn offline_region_update_metadata_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
) -> error::Result<OfflineRegionInfo> {
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
    copy_offline_region_snapshot(snapshot)
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
mod tests;

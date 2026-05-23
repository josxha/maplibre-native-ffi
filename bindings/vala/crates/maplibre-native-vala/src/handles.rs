use std::ffi::{CStr, c_char, c_void};
use std::ptr;
use std::sync::Mutex;

use maplibre_native_core::error::{self, Error};
use maplibre_native_sys as sys;

use crate::events::RuntimeEvent;
use crate::glib::{self, GBoolean, GDestroyNotify, GError, GFALSE, GObject, GTRUE, GType};

const RUNTIME_TYPE_NAME: &CStr = c"MlnValaRuntimeHandle";
const MAP_TYPE_NAME: &CStr = c"MlnValaMapHandle";

#[repr(C)]
pub struct RuntimeHandle {
    parent_instance: GObject,
    native: *mut sys::mln_runtime,
    resource_transform: *mut ResourceTransformState,
}

pub type ResourceTransformCallback =
    unsafe extern "C" fn(kind: u32, url: *const c_char, user_data: *mut c_void) -> *mut c_char;

struct ResourceTransformState {
    callback: ResourceTransformCallback,
    user_data: *mut c_void,
    destroy_notify: GDestroyNotify,
    returned_urls: Mutex<Vec<*mut c_char>>,
}

#[repr(C)]
pub struct MapHandle {
    parent_instance: GObject,
    native: *mut sys::mln_map,
    runtime: *mut RuntimeHandle,
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
    error::check(unsafe { sys::mln_map_set_style_json(map, json) })
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

fn remove_style_source(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_removed: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let mut removed = false;
    // SAFETY: `map` is live, `source_id` borrows a caller string for this call,
    // and `removed` is valid output storage.
    error::check(unsafe { sys::mln_map_remove_style_source(map, source_id, &mut removed) })?;
    glib::clear_optional_out_pointer(out_removed, if removed { GTRUE } else { GFALSE })
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
    unsafe {
        *out_event = if has_event {
            Box::into_raw(Box::new(RuntimeEvent::from_native(&event)))
        } else {
            ptr::null_mut()
        };
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
    }
    Ok(handle)
}

fn close_runtime_handle(handle: *mut RuntimeHandle) -> error::Result<()> {
    let runtime = runtime_native(handle)?;
    // SAFETY: `runtime_native` returns a live native runtime pointer.
    error::check(unsafe { sys::mln_runtime_destroy(runtime) })?;
    // SAFETY: `handle` was checked by `runtime_native`. Successful runtime
    // destruction makes resource transform callbacks unreachable.
    let resource_transform = unsafe {
        (*handle).native = ptr::null_mut();
        let resource_transform = (*handle).resource_transform;
        (*handle).resource_transform = ptr::null_mut();
        resource_transform
    };
    destroy_resource_transform_state(resource_transform);
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
    }
    Ok(handle)
}

fn close_map_handle(handle: *mut MapHandle) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map_native` returns a live native map pointer.
    error::check(unsafe { sys::mln_map_destroy(map) })?;
    // SAFETY: `handle` was checked by `map_native`.
    unsafe {
        (*handle).native = ptr::null_mut();
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TRANSFORM_DESTROY_COUNT: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn passthrough_transform(
        _kind: u32,
        _url: *const c_char,
        _user_data: *mut c_void,
    ) -> *mut c_char {
        ptr::null_mut()
    }

    unsafe extern "C" fn transform_destroy_notify(_user_data: *mut c_void) {
        TRANSFORM_DESTROY_COUNT.fetch_add(1, Ordering::SeqCst);
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
            GFALSE
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
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );

        glib::unref_object(map);
        glib::unref_object(runtime);
    }

    #[test]
    fn resource_transform_replacement_and_clear_run_destroy_notify() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        TRANSFORM_DESTROY_COUNT.store(0, Ordering::SeqCst);

        assert_eq!(
            mln_vala_runtime_handle_set_resource_transform(
                runtime,
                Some(passthrough_transform),
                ptr::null_mut(),
                Some(transform_destroy_notify),
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(TRANSFORM_DESTROY_COUNT.load(Ordering::SeqCst), 0);

        assert_eq!(
            mln_vala_runtime_handle_set_resource_transform(
                runtime,
                Some(passthrough_transform),
                ptr::null_mut(),
                Some(transform_destroy_notify),
                ptr::null_mut(),
            ),
            GTRUE
        );
        assert_eq!(TRANSFORM_DESTROY_COUNT.load(Ordering::SeqCst), 1);

        assert_eq!(
            mln_vala_runtime_handle_clear_resource_transform(runtime, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(TRANSFORM_DESTROY_COUNT.load(Ordering::SeqCst), 2);

        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );
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

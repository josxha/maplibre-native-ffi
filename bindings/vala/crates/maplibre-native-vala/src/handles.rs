use std::ffi::{CStr, c_char, c_void};
use std::ptr;
use std::sync::Mutex;

use maplibre_native_core::error::{self, Error};
use maplibre_native_sys as sys;

use crate::events::RuntimeEvent;
use crate::glib::{self, GBoolean, GDestroyNotify, GError, GFALSE, GObject, GTRUE, GType};

const RUNTIME_TYPE_NAME: &CStr = c"MlnValaRuntimeHandle";
const MAP_TYPE_NAME: &CStr = c"MlnValaMapHandle";
const STYLE_ID_LIST_TYPE_NAME: &CStr = c"MlnValaStyleIdListHandle";

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

#[repr(C)]
pub struct StyleIdListHandle {
    parent_instance: GObject,
    native: *mut sys::mln_style_id_list,
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

fn default_style_tile_source_options(
    out_options: *mut sys::mln_style_tile_source_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_style_tile_source_options_default() };
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

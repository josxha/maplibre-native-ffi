use std::collections::hash_map::DefaultHasher;
use std::ffi::CStr;
use std::hash::{Hash, Hasher};
use std::ptr;

use maplibre_native_core::error::{self, Error};
use maplibre_native_sys as sys;

use crate::glib::{self, GBoolean, GError, GFALSE, GObject, GTRUE, GType};
use crate::handles::{self, JsonSnapshotHandle, MapHandle};
use crate::native_pointer::NativePointer;

const RENDER_SESSION_TYPE_NAME: &CStr = c"MlnValaRenderSessionHandle";
const METAL_FRAME_TYPE_NAME: &CStr = c"MlnValaMetalOwnedTextureFrameHandle";
const VULKAN_FRAME_TYPE_NAME: &CStr = c"MlnValaVulkanOwnedTextureFrameHandle";

fn current_thread_token() -> u64 {
    let mut hasher = DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    hasher.finish()
}

fn render_session_should_finalize_on_owner_thread(handle: *mut RenderSessionHandle) -> bool {
    if handle.is_null() {
        return false;
    }
    // SAFETY: The caller passes a RenderSessionHandle GObject during finalization.
    unsafe { !(*handle).native.is_null() && (*handle).owner_thread == current_thread_token() }
}

fn metal_frame_should_finalize_on_owner_thread(handle: *mut MetalOwnedTextureFrameHandle) -> bool {
    if handle.is_null() {
        return false;
    }
    // SAFETY: The caller passes a MetalOwnedTextureFrameHandle GObject during finalization.
    unsafe { !(*handle).session.is_null() && (*handle).owner_thread == current_thread_token() }
}

fn vulkan_frame_should_finalize_on_owner_thread(
    handle: *mut VulkanOwnedTextureFrameHandle,
) -> bool {
    if handle.is_null() {
        return false;
    }
    // SAFETY: The caller passes a VulkanOwnedTextureFrameHandle GObject during finalization.
    unsafe { !(*handle).session.is_null() && (*handle).owner_thread == current_thread_token() }
}

#[repr(C)]
pub struct RenderSessionHandle {
    parent_instance: GObject,
    native: *mut sys::mln_render_session,
    map: *mut MapHandle,
    owner_thread: u64,
}

#[repr(C)]
pub struct MetalOwnedTextureFrameHandle {
    parent_instance: GObject,
    session: *mut RenderSessionHandle,
    frame: sys::mln_metal_owned_texture_frame,
    owner_thread: u64,
}

#[repr(C)]
pub struct VulkanOwnedTextureFrameHandle {
    parent_instance: GObject,
    session: *mut RenderSessionHandle,
    frame: sys::mln_vulkan_owned_texture_frame,
    owner_thread: u64,
}

impl glib::ObjectFinalize for RenderSessionHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        let handle = object.cast::<RenderSessionHandle>();
        if render_session_should_finalize_on_owner_thread(handle) {
            let _ = close_render_session(handle);
        } else if unsafe { !(*handle).native.is_null() } {
            eprintln!(
                "RenderSessionHandle finalized off its owner thread; call close() on the owner thread to release native state"
            );
        }
    }
}

impl glib::ObjectFinalize for MetalOwnedTextureFrameHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        let handle = object.cast::<MetalOwnedTextureFrameHandle>();
        if metal_frame_should_finalize_on_owner_thread(handle) {
            let _ = release_metal_frame(handle);
        } else if unsafe { !(*handle).session.is_null() } {
            eprintln!(
                "MetalOwnedTextureFrameHandle finalized off its owner thread; call close() on the owner thread to release native frame state"
            );
        }
    }
}

impl glib::ObjectFinalize for VulkanOwnedTextureFrameHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        let handle = object.cast::<VulkanOwnedTextureFrameHandle>();
        if vulkan_frame_should_finalize_on_owner_thread(handle) {
            let _ = release_vulkan_frame(handle);
        } else if unsafe { !(*handle).session.is_null() } {
            eprintln!(
                "VulkanOwnedTextureFrameHandle finalized off its owner thread; call close() on the owner thread to release native frame state"
            );
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_get_type() -> GType {
    glib::register_object_type::<RenderSessionHandle>(RENDER_SESSION_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_owned_texture_frame_handle_get_type() -> GType {
    glib::register_object_type::<MetalOwnedTextureFrameHandle>(METAL_FRAME_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_frame_handle_get_type() -> GType {
    glib::register_object_type::<VulkanOwnedTextureFrameHandle>(VULKAN_FRAME_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_surface_descriptor_default(
    out_descriptor: *mut sys::mln_metal_surface_descriptor,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_metal_surface_descriptor(out_descriptor) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_surface_descriptor_default(
    out_descriptor: *mut sys::mln_vulkan_surface_descriptor,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_vulkan_surface_descriptor(out_descriptor) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_owned_texture_descriptor_default(
    out_descriptor: *mut sys::mln_metal_owned_texture_descriptor,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_metal_owned_texture_descriptor(out_descriptor) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_borrowed_texture_descriptor_default(
    out_descriptor: *mut sys::mln_metal_borrowed_texture_descriptor,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_metal_borrowed_texture_descriptor(out_descriptor) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_descriptor_default(
    out_descriptor: *mut sys::mln_vulkan_owned_texture_descriptor,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_vulkan_owned_texture_descriptor(out_descriptor) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_borrowed_texture_descriptor_default(
    out_descriptor: *mut sys::mln_vulkan_borrowed_texture_descriptor,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_vulkan_borrowed_texture_descriptor(out_descriptor) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_texture_image_info_default(
    out_info: *mut sys::mln_texture_image_info,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_texture_image_info(out_info) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_attach_metal_surface(
    map: *mut MapHandle,
    descriptor: *const sys::mln_metal_surface_descriptor,
    error_out: *mut *mut GError,
) -> *mut RenderSessionHandle {
    match attach_metal_surface(map, descriptor) {
        Ok(handle) => handle,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_attach_vulkan_surface(
    map: *mut MapHandle,
    descriptor: *const sys::mln_vulkan_surface_descriptor,
    error_out: *mut *mut GError,
) -> *mut RenderSessionHandle {
    match attach_vulkan_surface(map, descriptor) {
        Ok(handle) => handle,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_attach_metal_owned_texture(
    map: *mut MapHandle,
    descriptor: *const sys::mln_metal_owned_texture_descriptor,
    error_out: *mut *mut GError,
) -> *mut RenderSessionHandle {
    match attach_metal_owned_texture(map, descriptor) {
        Ok(handle) => handle,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_attach_metal_borrowed_texture(
    map: *mut MapHandle,
    descriptor: *const sys::mln_metal_borrowed_texture_descriptor,
    error_out: *mut *mut GError,
) -> *mut RenderSessionHandle {
    match attach_metal_borrowed_texture(map, descriptor) {
        Ok(handle) => handle,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_attach_vulkan_owned_texture(
    map: *mut MapHandle,
    descriptor: *const sys::mln_vulkan_owned_texture_descriptor,
    error_out: *mut *mut GError,
) -> *mut RenderSessionHandle {
    match attach_vulkan_owned_texture(map, descriptor) {
        Ok(handle) => handle,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_attach_vulkan_borrowed_texture(
    map: *mut MapHandle,
    descriptor: *const sys::mln_vulkan_borrowed_texture_descriptor,
    error_out: *mut *mut GError,
) -> *mut RenderSessionHandle {
    match attach_vulkan_borrowed_texture(map, descriptor) {
        Ok(handle) => handle,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_resize(
    handle: *mut RenderSessionHandle,
    width: u32,
    height: u32,
    scale_factor: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    match resize(handle, width, height, scale_factor) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_render_update(
    handle: *mut RenderSessionHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match session_call(handle, |session| unsafe {
        sys::mln_render_session_render_update(session)
    }) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_set_feature_state(
    handle: *mut RenderSessionHandle,
    selector: *const sys::mln_feature_state_selector,
    state: *const sys::mln_json_value,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_feature_state(handle, selector, state) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_get_feature_state(
    handle: *mut RenderSessionHandle,
    selector: *const sys::mln_feature_state_selector,
    error_out: *mut *mut GError,
) -> *mut JsonSnapshotHandle {
    match get_feature_state(handle, selector) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_remove_feature_state(
    handle: *mut RenderSessionHandle,
    selector: *const sys::mln_feature_state_selector,
    error_out: *mut *mut GError,
) -> GBoolean {
    match remove_feature_state(handle, selector) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_detach(
    handle: *mut RenderSessionHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match session_call(handle, |session| unsafe {
        sys::mln_render_session_detach(session)
    }) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_close(
    handle: *mut RenderSessionHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match close_render_session(handle) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_reduce_memory_use(
    handle: *mut RenderSessionHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match session_call(handle, |session| unsafe {
        sys::mln_render_session_reduce_memory_use(session)
    }) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_clear_data(
    handle: *mut RenderSessionHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match session_call(handle, |session| unsafe {
        sys::mln_render_session_clear_data(session)
    }) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_dump_debug_logs(
    handle: *mut RenderSessionHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match session_call(handle, |session| unsafe {
        sys::mln_render_session_dump_debug_logs(session)
    }) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_acquire_metal_owned_texture_frame(
    handle: *mut RenderSessionHandle,
    error_out: *mut *mut GError,
) -> *mut MetalOwnedTextureFrameHandle {
    match acquire_metal_frame(handle) {
        Ok(frame) => frame,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_acquire_vulkan_owned_texture_frame(
    handle: *mut RenderSessionHandle,
    error_out: *mut *mut GError,
) -> *mut VulkanOwnedTextureFrameHandle {
    match acquire_vulkan_frame(handle) {
        Ok(frame) => frame,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_owned_texture_frame_handle_close(
    handle: *mut MetalOwnedTextureFrameHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match release_metal_frame(handle) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_frame_handle_close(
    handle: *mut VulkanOwnedTextureFrameHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match release_vulkan_frame(handle) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_read_premultiplied_rgba8(
    handle: *mut RenderSessionHandle,
    out_data: *mut u8,
    out_data_capacity: usize,
    out_info: *mut sys::mln_texture_image_info,
    error_out: *mut *mut GError,
) -> GBoolean {
    match read_premultiplied_rgba8(handle, out_data, out_data_capacity, out_info) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_owned_texture_frame_handle_get_texture(
    handle: *mut MetalOwnedTextureFrameHandle,
    error_out: *mut *mut GError,
) -> *mut NativePointer {
    match metal_frame_native_pointer(handle, |frame| frame.texture, "Metal texture") {
        Ok(pointer) => pointer,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_owned_texture_frame_handle_get_device(
    handle: *mut MetalOwnedTextureFrameHandle,
    error_out: *mut *mut GError,
) -> *mut NativePointer {
    match metal_frame_native_pointer(handle, |frame| frame.device, "Metal device") {
        Ok(pointer) => pointer,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_frame_handle_get_image(
    handle: *mut VulkanOwnedTextureFrameHandle,
    error_out: *mut *mut GError,
) -> *mut NativePointer {
    match vulkan_frame_native_pointer(handle, |frame| frame.image, "Vulkan image") {
        Ok(pointer) => pointer,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_frame_handle_get_image_view(
    handle: *mut VulkanOwnedTextureFrameHandle,
    error_out: *mut *mut GError,
) -> *mut NativePointer {
    match vulkan_frame_native_pointer(handle, |frame| frame.image_view, "Vulkan image view") {
        Ok(pointer) => pointer,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_frame_handle_get_device(
    handle: *mut VulkanOwnedTextureFrameHandle,
    error_out: *mut *mut GError,
) -> *mut NativePointer {
    match vulkan_frame_native_pointer(handle, |frame| frame.device, "Vulkan device") {
        Ok(pointer) => pointer,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_owned_texture_frame_handle_get_generation(
    handle: *mut MetalOwnedTextureFrameHandle,
    out_generation: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    metal_frame_u64(handle, out_generation, error_out, |frame| frame.generation)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_owned_texture_frame_handle_get_width(
    handle: *mut MetalOwnedTextureFrameHandle,
    out_width: *mut u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    metal_frame_u32(handle, out_width, error_out, |frame| frame.width)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_owned_texture_frame_handle_get_height(
    handle: *mut MetalOwnedTextureFrameHandle,
    out_height: *mut u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    metal_frame_u32(handle, out_height, error_out, |frame| frame.height)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_owned_texture_frame_handle_get_scale_factor(
    handle: *mut MetalOwnedTextureFrameHandle,
    out_scale_factor: *mut f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    metal_frame_f64(handle, out_scale_factor, error_out, |frame| {
        frame.scale_factor
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_owned_texture_frame_handle_get_frame_id(
    handle: *mut MetalOwnedTextureFrameHandle,
    out_frame_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    metal_frame_u64(handle, out_frame_id, error_out, |frame| frame.frame_id)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_metal_owned_texture_frame_handle_get_pixel_format(
    handle: *mut MetalOwnedTextureFrameHandle,
    out_pixel_format: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    metal_frame_u64(handle, out_pixel_format, error_out, |frame| {
        frame.pixel_format
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_frame_handle_get_generation(
    handle: *mut VulkanOwnedTextureFrameHandle,
    out_generation: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    vulkan_frame_u64(handle, out_generation, error_out, |frame| frame.generation)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_frame_handle_get_width(
    handle: *mut VulkanOwnedTextureFrameHandle,
    out_width: *mut u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    vulkan_frame_u32(handle, out_width, error_out, |frame| frame.width)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_frame_handle_get_height(
    handle: *mut VulkanOwnedTextureFrameHandle,
    out_height: *mut u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    vulkan_frame_u32(handle, out_height, error_out, |frame| frame.height)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_frame_handle_get_scale_factor(
    handle: *mut VulkanOwnedTextureFrameHandle,
    out_scale_factor: *mut f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    vulkan_frame_f64(handle, out_scale_factor, error_out, |frame| {
        frame.scale_factor
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_frame_handle_get_frame_id(
    handle: *mut VulkanOwnedTextureFrameHandle,
    out_frame_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    vulkan_frame_u64(handle, out_frame_id, error_out, |frame| frame.frame_id)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_frame_handle_get_format(
    handle: *mut VulkanOwnedTextureFrameHandle,
    out_format: *mut u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    vulkan_frame_u32(handle, out_format, error_out, |frame| frame.format)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_vulkan_owned_texture_frame_handle_get_layout(
    handle: *mut VulkanOwnedTextureFrameHandle,
    out_layout: *mut u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    vulkan_frame_u32(handle, out_layout, error_out, |frame| frame.layout)
}

fn default_metal_surface_descriptor(
    out_descriptor: *mut sys::mln_metal_surface_descriptor,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let descriptor = unsafe { sys::mln_metal_surface_descriptor_default() };
    glib::clear_optional_out_pointer(out_descriptor, descriptor)
}

fn default_vulkan_surface_descriptor(
    out_descriptor: *mut sys::mln_vulkan_surface_descriptor,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let descriptor = unsafe { sys::mln_vulkan_surface_descriptor_default() };
    glib::clear_optional_out_pointer(out_descriptor, descriptor)
}

fn default_metal_owned_texture_descriptor(
    out_descriptor: *mut sys::mln_metal_owned_texture_descriptor,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let descriptor = unsafe { sys::mln_metal_owned_texture_descriptor_default() };
    glib::clear_optional_out_pointer(out_descriptor, descriptor)
}

fn default_metal_borrowed_texture_descriptor(
    out_descriptor: *mut sys::mln_metal_borrowed_texture_descriptor,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let descriptor = unsafe { sys::mln_metal_borrowed_texture_descriptor_default() };
    glib::clear_optional_out_pointer(out_descriptor, descriptor)
}

fn default_vulkan_owned_texture_descriptor(
    out_descriptor: *mut sys::mln_vulkan_owned_texture_descriptor,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let descriptor = unsafe { sys::mln_vulkan_owned_texture_descriptor_default() };
    glib::clear_optional_out_pointer(out_descriptor, descriptor)
}

fn default_vulkan_borrowed_texture_descriptor(
    out_descriptor: *mut sys::mln_vulkan_borrowed_texture_descriptor,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let descriptor = unsafe { sys::mln_vulkan_borrowed_texture_descriptor_default() };
    glib::clear_optional_out_pointer(out_descriptor, descriptor)
}

fn default_texture_image_info(out_info: *mut sys::mln_texture_image_info) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let info = unsafe { sys::mln_texture_image_info_default() };
    glib::clear_optional_out_pointer(out_info, info)
}

fn attach_metal_surface(
    map: *mut MapHandle,
    descriptor: *const sys::mln_metal_surface_descriptor,
) -> error::Result<*mut RenderSessionHandle> {
    attach_session(map, descriptor, |map, descriptor, out_session| unsafe {
        sys::mln_metal_surface_attach(map, descriptor, out_session)
    })
}

fn attach_vulkan_surface(
    map: *mut MapHandle,
    descriptor: *const sys::mln_vulkan_surface_descriptor,
) -> error::Result<*mut RenderSessionHandle> {
    attach_session(map, descriptor, |map, descriptor, out_session| unsafe {
        sys::mln_vulkan_surface_attach(map, descriptor, out_session)
    })
}

fn attach_metal_owned_texture(
    map: *mut MapHandle,
    descriptor: *const sys::mln_metal_owned_texture_descriptor,
) -> error::Result<*mut RenderSessionHandle> {
    attach_session(map, descriptor, |map, descriptor, out_session| unsafe {
        sys::mln_metal_owned_texture_attach(map, descriptor, out_session)
    })
}

fn attach_metal_borrowed_texture(
    map: *mut MapHandle,
    descriptor: *const sys::mln_metal_borrowed_texture_descriptor,
) -> error::Result<*mut RenderSessionHandle> {
    attach_session(map, descriptor, |map, descriptor, out_session| unsafe {
        sys::mln_metal_borrowed_texture_attach(map, descriptor, out_session)
    })
}

fn attach_vulkan_owned_texture(
    map: *mut MapHandle,
    descriptor: *const sys::mln_vulkan_owned_texture_descriptor,
) -> error::Result<*mut RenderSessionHandle> {
    attach_session(map, descriptor, |map, descriptor, out_session| unsafe {
        sys::mln_vulkan_owned_texture_attach(map, descriptor, out_session)
    })
}

fn attach_vulkan_borrowed_texture(
    map: *mut MapHandle,
    descriptor: *const sys::mln_vulkan_borrowed_texture_descriptor,
) -> error::Result<*mut RenderSessionHandle> {
    attach_session(map, descriptor, |map, descriptor, out_session| unsafe {
        sys::mln_vulkan_borrowed_texture_attach(map, descriptor, out_session)
    })
}

fn attach_session<T>(
    map: *mut MapHandle,
    descriptor: *const T,
    attach: impl FnOnce(
        *mut sys::mln_map,
        *const T,
        *mut *mut sys::mln_render_session,
    ) -> sys::mln_status,
) -> error::Result<*mut RenderSessionHandle> {
    let native_map = handles::map_native(map)?;
    if descriptor.is_null() {
        return Err(Error::invalid_argument("render target descriptor is null"));
    }
    let mut native = ptr::null_mut();
    error::check(attach(native_map, descriptor, &mut native))?;
    wrap_native_session(map, native)
}

fn wrap_native_session(
    map: *mut MapHandle,
    native: *mut sys::mln_render_session,
) -> error::Result<*mut RenderSessionHandle> {
    if native.is_null() {
        return Err(Error::invalid_argument("native render session is null"));
    }
    let handle = glib::new_object::<RenderSessionHandle>(mln_vala_render_session_handle_get_type());
    if handle.is_null() {
        // SAFETY: `native` came from a successful attach operation.
        let _ = error::check(unsafe { sys::mln_render_session_destroy(native) });
        return Err(Error::invalid_argument(
            "failed to allocate RenderSessionHandle",
        ));
    }
    // SAFETY: `handle` points to a newly allocated RenderSessionHandle.
    unsafe {
        (*handle).native = native;
        (*handle).map = glib::ref_object(map);
        (*handle).owner_thread = current_thread_token();
    }
    Ok(handle)
}

pub(crate) fn session_native(
    handle: *mut RenderSessionHandle,
) -> error::Result<*mut sys::mln_render_session> {
    if handle.is_null() {
        return Err(Error::invalid_argument("RenderSessionHandle is null"));
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe { (*handle).native };
    if native.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "RenderSessionHandle is closed",
        ));
    }
    Ok(native)
}

fn session_call(
    handle: *mut RenderSessionHandle,
    call: impl FnOnce(*mut sys::mln_render_session) -> sys::mln_status,
) -> error::Result<()> {
    let session = session_native(handle)?;
    error::check(call(session))
}

fn resize(
    handle: *mut RenderSessionHandle,
    width: u32,
    height: u32,
    scale_factor: f64,
) -> error::Result<()> {
    let session = session_native(handle)?;
    // SAFETY: `session` is live. The C API validates dimensions and scale.
    error::check(unsafe { sys::mln_render_session_resize(session, width, height, scale_factor) })
}

fn set_feature_state(
    handle: *mut RenderSessionHandle,
    selector: *const sys::mln_feature_state_selector,
    state: *const sys::mln_json_value,
) -> error::Result<()> {
    if selector.is_null() {
        return Err(Error::invalid_argument("feature state selector is null"));
    }
    if state.is_null() {
        return Err(Error::invalid_argument("feature state JSON is null"));
    }
    let session = session_native(handle)?;
    // SAFETY: `session` is live; selector and state are borrowed for this call.
    error::check(unsafe { sys::mln_render_session_set_feature_state(session, selector, state) })
}

fn get_feature_state(
    handle: *mut RenderSessionHandle,
    selector: *const sys::mln_feature_state_selector,
) -> error::Result<*mut JsonSnapshotHandle> {
    if selector.is_null() {
        return Err(Error::invalid_argument("feature state selector is null"));
    }
    let session = session_native(handle)?;
    let mut snapshot = ptr::null_mut();
    // SAFETY: `session` is live; selector is borrowed for this call, and output
    // storage is valid and null-initialized.
    error::check(unsafe {
        sys::mln_render_session_get_feature_state(session, selector, &mut snapshot)
    })?;
    handles::wrap_json_snapshot(snapshot)
}

fn remove_feature_state(
    handle: *mut RenderSessionHandle,
    selector: *const sys::mln_feature_state_selector,
) -> error::Result<()> {
    if selector.is_null() {
        return Err(Error::invalid_argument("feature state selector is null"));
    }
    let session = session_native(handle)?;
    // SAFETY: `session` is live; selector is borrowed for this call.
    error::check(unsafe { sys::mln_render_session_remove_feature_state(session, selector) })
}

fn read_premultiplied_rgba8(
    handle: *mut RenderSessionHandle,
    out_data: *mut u8,
    out_data_capacity: usize,
    out_info: *mut sys::mln_texture_image_info,
) -> error::Result<()> {
    let session = session_native(handle)?;
    if out_info.is_null() {
        return Err(Error::invalid_argument("TextureImageInfo output is null"));
    }
    // SAFETY: `out_info` is valid output storage by the null check above.
    unsafe {
        (*out_info).size = std::mem::size_of::<sys::mln_texture_image_info>() as u32;
    }
    // SAFETY: `session` is live. The C API validates the output buffer pointer,
    // capacity, and info storage before writing.
    error::check(unsafe {
        sys::mln_texture_read_premultiplied_rgba8(session, out_data, out_data_capacity, out_info)
    })
}

fn acquire_metal_frame(
    handle: *mut RenderSessionHandle,
) -> error::Result<*mut MetalOwnedTextureFrameHandle> {
    let session = session_native(handle)?;
    // SAFETY: Zeroed storage is immediately initialized by the C API after the
    // public size field is set.
    let mut frame: sys::mln_metal_owned_texture_frame = unsafe { std::mem::zeroed() };
    frame.size = std::mem::size_of::<sys::mln_metal_owned_texture_frame>() as u32;
    // SAFETY: `session` is live and frame points to valid output storage.
    error::check(unsafe { sys::mln_metal_owned_texture_acquire_frame(session, &mut frame) })?;

    let frame_handle = glib::new_object::<MetalOwnedTextureFrameHandle>(
        mln_vala_metal_owned_texture_frame_handle_get_type(),
    );
    if frame_handle.is_null() {
        // SAFETY: `frame` was acquired from `session` and must be released if
        // wrapper allocation fails.
        let _ =
            error::check(unsafe { sys::mln_metal_owned_texture_release_frame(session, &frame) });
        return Err(Error::invalid_argument(
            "failed to allocate MetalOwnedTextureFrameHandle",
        ));
    }
    // SAFETY: `frame_handle` points to a newly allocated frame wrapper.
    unsafe {
        (*frame_handle).session = glib::ref_object(handle);
        (*frame_handle).frame = frame;
        (*frame_handle).owner_thread = current_thread_token();
    }
    Ok(frame_handle)
}

fn acquire_vulkan_frame(
    handle: *mut RenderSessionHandle,
) -> error::Result<*mut VulkanOwnedTextureFrameHandle> {
    let session = session_native(handle)?;
    // SAFETY: Zeroed storage is immediately initialized by the C API after the
    // public size field is set.
    let mut frame: sys::mln_vulkan_owned_texture_frame = unsafe { std::mem::zeroed() };
    frame.size = std::mem::size_of::<sys::mln_vulkan_owned_texture_frame>() as u32;
    // SAFETY: `session` is live and frame points to valid output storage.
    error::check(unsafe { sys::mln_vulkan_owned_texture_acquire_frame(session, &mut frame) })?;

    let frame_handle = glib::new_object::<VulkanOwnedTextureFrameHandle>(
        mln_vala_vulkan_owned_texture_frame_handle_get_type(),
    );
    if frame_handle.is_null() {
        // SAFETY: `frame` was acquired from `session` and must be released if
        // wrapper allocation fails.
        let _ =
            error::check(unsafe { sys::mln_vulkan_owned_texture_release_frame(session, &frame) });
        return Err(Error::invalid_argument(
            "failed to allocate VulkanOwnedTextureFrameHandle",
        ));
    }
    // SAFETY: `frame_handle` points to a newly allocated frame wrapper.
    unsafe {
        (*frame_handle).session = glib::ref_object(handle);
        (*frame_handle).frame = frame;
        (*frame_handle).owner_thread = current_thread_token();
    }
    Ok(frame_handle)
}

fn release_metal_frame(handle: *mut MetalOwnedTextureFrameHandle) -> error::Result<()> {
    if handle.is_null() {
        return Err(Error::invalid_argument(
            "MetalOwnedTextureFrameHandle is null",
        ));
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let session_handle = unsafe { (*handle).session };
    if session_handle.is_null() {
        return Ok(());
    }
    // SAFETY: `handle` is non-null and frame is a copyable C descriptor.
    let frame = unsafe { (*handle).frame };
    if let Ok(session) = session_native(session_handle) {
        // SAFETY: `session` is live and `frame` is the acquired frame identity
        // held by this wrapper.
        error::check(unsafe { sys::mln_metal_owned_texture_release_frame(session, &frame) })?;
    }
    // SAFETY: `handle` was checked above. Clearing the session makes repeated
    // release/finalize calls no-ops.
    unsafe {
        (*handle).session = ptr::null_mut();
        glib::unref_object(session_handle);
    }
    Ok(())
}

fn release_vulkan_frame(handle: *mut VulkanOwnedTextureFrameHandle) -> error::Result<()> {
    if handle.is_null() {
        return Err(Error::invalid_argument(
            "VulkanOwnedTextureFrameHandle is null",
        ));
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let session_handle = unsafe { (*handle).session };
    if session_handle.is_null() {
        return Ok(());
    }
    // SAFETY: `handle` is non-null and frame is a copyable C descriptor.
    let frame = unsafe { (*handle).frame };
    if let Ok(session) = session_native(session_handle) {
        // SAFETY: `session` is live and `frame` is the acquired frame identity
        // held by this wrapper.
        error::check(unsafe { sys::mln_vulkan_owned_texture_release_frame(session, &frame) })?;
    }
    // SAFETY: `handle` was checked above. Clearing the session makes repeated
    // release/finalize calls no-ops.
    unsafe {
        (*handle).session = ptr::null_mut();
        glib::unref_object(session_handle);
    }
    Ok(())
}

fn metal_frame_native_pointer(
    handle: *mut MetalOwnedTextureFrameHandle,
    getter: impl FnOnce(sys::mln_metal_owned_texture_frame) -> *mut std::ffi::c_void,
    name: &str,
) -> error::Result<*mut NativePointer> {
    let (_, frame) = metal_frame_state(handle)?;
    let pointer = NativePointer::from_ptr(getter(frame))
        .ok_or_else(|| Error::invalid_argument(format!("{name} pointer is null")))?;
    Ok(Box::into_raw(Box::new(pointer)))
}

fn vulkan_frame_native_pointer(
    handle: *mut VulkanOwnedTextureFrameHandle,
    getter: impl FnOnce(sys::mln_vulkan_owned_texture_frame) -> *mut std::ffi::c_void,
    name: &str,
) -> error::Result<*mut NativePointer> {
    let (_, frame) = vulkan_frame_state(handle)?;
    let pointer = NativePointer::from_ptr(getter(frame))
        .ok_or_else(|| Error::invalid_argument(format!("{name} pointer is null")))?;
    Ok(Box::into_raw(Box::new(pointer)))
}

fn metal_frame_u32(
    handle: *mut MetalOwnedTextureFrameHandle,
    out: *mut u32,
    error_out: *mut *mut GError,
    getter: impl FnOnce(sys::mln_metal_owned_texture_frame) -> u32,
) -> GBoolean {
    metal_frame_value(handle, out, error_out, getter)
}

fn metal_frame_u64(
    handle: *mut MetalOwnedTextureFrameHandle,
    out: *mut u64,
    error_out: *mut *mut GError,
    getter: impl FnOnce(sys::mln_metal_owned_texture_frame) -> u64,
) -> GBoolean {
    metal_frame_value(handle, out, error_out, getter)
}

fn metal_frame_f64(
    handle: *mut MetalOwnedTextureFrameHandle,
    out: *mut f64,
    error_out: *mut *mut GError,
    getter: impl FnOnce(sys::mln_metal_owned_texture_frame) -> f64,
) -> GBoolean {
    metal_frame_value(handle, out, error_out, getter)
}

fn vulkan_frame_u32(
    handle: *mut VulkanOwnedTextureFrameHandle,
    out: *mut u32,
    error_out: *mut *mut GError,
    getter: impl FnOnce(sys::mln_vulkan_owned_texture_frame) -> u32,
) -> GBoolean {
    vulkan_frame_value(handle, out, error_out, getter)
}

fn vulkan_frame_u64(
    handle: *mut VulkanOwnedTextureFrameHandle,
    out: *mut u64,
    error_out: *mut *mut GError,
    getter: impl FnOnce(sys::mln_vulkan_owned_texture_frame) -> u64,
) -> GBoolean {
    vulkan_frame_value(handle, out, error_out, getter)
}

fn vulkan_frame_f64(
    handle: *mut VulkanOwnedTextureFrameHandle,
    out: *mut f64,
    error_out: *mut *mut GError,
    getter: impl FnOnce(sys::mln_vulkan_owned_texture_frame) -> f64,
) -> GBoolean {
    vulkan_frame_value(handle, out, error_out, getter)
}

fn metal_frame_value<T>(
    handle: *mut MetalOwnedTextureFrameHandle,
    out: *mut T,
    error_out: *mut *mut GError,
    getter: impl FnOnce(sys::mln_metal_owned_texture_frame) -> T,
) -> GBoolean {
    match metal_frame_state(handle)
        .and_then(|(_, frame)| glib::clear_optional_out_pointer(out, getter(frame)))
    {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

fn vulkan_frame_value<T>(
    handle: *mut VulkanOwnedTextureFrameHandle,
    out: *mut T,
    error_out: *mut *mut GError,
    getter: impl FnOnce(sys::mln_vulkan_owned_texture_frame) -> T,
) -> GBoolean {
    match vulkan_frame_state(handle)
        .and_then(|(_, frame)| glib::clear_optional_out_pointer(out, getter(frame)))
    {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

fn metal_frame_state(
    handle: *mut MetalOwnedTextureFrameHandle,
) -> error::Result<(*mut RenderSessionHandle, sys::mln_metal_owned_texture_frame)> {
    if handle.is_null() {
        return Err(Error::invalid_argument(
            "MetalOwnedTextureFrameHandle is null",
        ));
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let session = unsafe { (*handle).session };
    if session.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "MetalOwnedTextureFrameHandle is closed",
        ));
    }
    // SAFETY: `handle` is non-null and frame is a copyable C descriptor.
    let frame = unsafe { (*handle).frame };
    Ok((session, frame))
}

fn vulkan_frame_state(
    handle: *mut VulkanOwnedTextureFrameHandle,
) -> error::Result<(
    *mut RenderSessionHandle,
    sys::mln_vulkan_owned_texture_frame,
)> {
    if handle.is_null() {
        return Err(Error::invalid_argument(
            "VulkanOwnedTextureFrameHandle is null",
        ));
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let session = unsafe { (*handle).session };
    if session.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "VulkanOwnedTextureFrameHandle is closed",
        ));
    }
    // SAFETY: `handle` is non-null and frame is a copyable C descriptor.
    let frame = unsafe { (*handle).frame };
    Ok((session, frame))
}

fn close_render_session(handle: *mut RenderSessionHandle) -> error::Result<()> {
    if handle.is_null() {
        return Err(Error::invalid_argument("RenderSessionHandle is null"));
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let session = unsafe { (*handle).native };
    if session.is_null() {
        return Ok(());
    }
    // SAFETY: `session` is live and owned by this handle.
    error::check(unsafe { sys::mln_render_session_destroy(session) })?;
    // SAFETY: `handle` was checked above.
    unsafe {
        (*handle).native = ptr::null_mut();
        glib::unref_object((*handle).map);
        (*handle).map = ptr::null_mut();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_descriptor_defaults_initialize_sizes() {
        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut metal_surface: sys::mln_metal_surface_descriptor = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_metal_surface_descriptor_default(&mut metal_surface, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(metal_surface.size, 0);

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut vulkan_surface: sys::mln_vulkan_surface_descriptor = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_vulkan_surface_descriptor_default(&mut vulkan_surface, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(vulkan_surface.size, 0);

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut metal_owned: sys::mln_metal_owned_texture_descriptor =
            unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_metal_owned_texture_descriptor_default(&mut metal_owned, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(metal_owned.size, 0);

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut metal_borrowed: sys::mln_metal_borrowed_texture_descriptor =
            unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_metal_borrowed_texture_descriptor_default(
                &mut metal_borrowed,
                ptr::null_mut()
            ),
            GTRUE
        );
        assert_ne!(metal_borrowed.size, 0);

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut vulkan_owned: sys::mln_vulkan_owned_texture_descriptor =
            unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_vulkan_owned_texture_descriptor_default(&mut vulkan_owned, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(vulkan_owned.size, 0);

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut vulkan_borrowed: sys::mln_vulkan_borrowed_texture_descriptor =
            unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_vulkan_borrowed_texture_descriptor_default(
                &mut vulkan_borrowed,
                ptr::null_mut()
            ),
            GTRUE
        );
        assert_ne!(vulkan_borrowed.size, 0);

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut texture_info: sys::mln_texture_image_info = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_texture_image_info_default(&mut texture_info, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(texture_info.size, 0);
    }

    #[test]
    fn null_render_session_reports_binding_error() {
        let mut error = ptr::null_mut();
        assert_eq!(
            mln_vala_render_session_handle_render_update(ptr::null_mut(), &mut error),
            GFALSE
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        let mut info: sys::mln_texture_image_info = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_render_session_handle_read_premultiplied_rgba8(
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                &mut info,
                &mut error,
            ),
            GFALSE
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        assert_eq!(
            mln_vala_render_session_handle_set_feature_state(
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                &mut error,
            ),
            GFALSE
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        assert!(
            mln_vala_render_session_handle_get_feature_state(
                ptr::null_mut(),
                ptr::null(),
                &mut error,
            )
            .is_null()
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        assert_eq!(
            mln_vala_render_session_handle_remove_feature_state(
                ptr::null_mut(),
                ptr::null(),
                &mut error,
            ),
            GFALSE
        );
        assert!(!error.is_null());
    }

    #[test]
    fn null_frame_handles_report_binding_errors() {
        let mut error = ptr::null_mut();
        let mut width = 0;
        assert_eq!(
            mln_vala_metal_owned_texture_frame_handle_get_width(
                ptr::null_mut(),
                &mut width,
                &mut error,
            ),
            GFALSE
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        assert!(
            mln_vala_vulkan_owned_texture_frame_handle_get_image(ptr::null_mut(), &mut error,)
                .is_null()
        );
        assert!(!error.is_null());
    }
}

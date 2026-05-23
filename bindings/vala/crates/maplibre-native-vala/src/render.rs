use std::ffi::CStr;
use std::ptr;

use maplibre_native_core::error::{self, Error};
use maplibre_native_sys as sys;

use crate::glib::{self, GBoolean, GError, GFALSE, GObject, GTRUE, GType};
use crate::handles::{self, MapHandle};

const RENDER_SESSION_TYPE_NAME: &CStr = c"MlnValaRenderSessionHandle";

#[repr(C)]
pub struct RenderSessionHandle {
    parent_instance: GObject,
    native: *mut sys::mln_render_session,
    map: *mut MapHandle,
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_get_type() -> GType {
    glib::register_object_type::<RenderSessionHandle>(RENDER_SESSION_TYPE_NAME)
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
    }
    Ok(handle)
}

fn session_native(handle: *mut RenderSessionHandle) -> error::Result<*mut sys::mln_render_session> {
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

fn close_render_session(handle: *mut RenderSessionHandle) -> error::Result<()> {
    let session = session_native(handle)?;
    // SAFETY: `session` is live.
    error::check(unsafe { sys::mln_render_session_destroy(session) })?;
    // SAFETY: `handle` was checked by `session_native`.
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
        let mut metal: sys::mln_metal_surface_descriptor = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_metal_surface_descriptor_default(&mut metal, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(metal.size, 0);

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
    }
}

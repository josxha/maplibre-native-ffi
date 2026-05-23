use std::ffi::CStr;
use std::ptr;

use maplibre_native_core::error::{self, Error};
use maplibre_native_sys as sys;

use crate::glib::{self, GBoolean, GError, GFALSE, GObject, GTRUE, GType};

const RESOURCE_REQUEST_TYPE_NAME: &CStr = c"MlnValaResourceRequestHandle";

#[repr(C)]
pub struct ResourceRequestHandle {
    parent_instance: GObject,
    native: *mut sys::mln_resource_request_handle,
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_resource_request_handle_get_type() -> GType {
    glib::register_object_type::<ResourceRequestHandle>(RESOURCE_REQUEST_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_resource_response_default(
    out_response: *mut sys::mln_resource_response,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_resource_response(out_response) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_resource_request_handle_complete(
    handle: *mut ResourceRequestHandle,
    response: *const sys::mln_resource_response,
    error_out: *mut *mut GError,
) -> GBoolean {
    match complete_request(handle, response) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_resource_request_handle_is_cancelled(
    handle: *mut ResourceRequestHandle,
    out_cancelled: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    match is_cancelled(handle, out_cancelled) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_resource_request_handle_release(handle: *mut ResourceRequestHandle) {
    release_request(handle);
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_resource_request_handle_close(handle: *mut ResourceRequestHandle) {
    release_request(handle);
}

pub(crate) unsafe fn resource_request_handle_from_native(
    native: *mut sys::mln_resource_request_handle,
) -> *mut ResourceRequestHandle {
    if native.is_null() {
        return ptr::null_mut();
    }

    let handle =
        glib::new_object::<ResourceRequestHandle>(mln_vala_resource_request_handle_get_type());
    if handle.is_null() {
        // SAFETY: The caller transfers a provider-owned request reference to
        // this adapter. If GLib allocation fails, release it to avoid a leak.
        unsafe { sys::mln_resource_request_release(native) };
        return ptr::null_mut();
    }

    // SAFETY: `handle` points to a newly allocated ResourceRequestHandle.
    unsafe {
        (*handle).native = native;
    }
    handle
}

pub(crate) unsafe fn resource_request_handle_disarm(handle: *mut ResourceRequestHandle) {
    if handle.is_null() {
        return;
    }
    // SAFETY: `handle` is non-null and points to this GObject subtype. This
    // makes pass-through callback handles unusable without releasing the native
    // request, because pass-through returns ownership to the C API.
    unsafe {
        (*handle).native = ptr::null_mut();
    }
}

fn default_resource_response(out_response: *mut sys::mln_resource_response) -> error::Result<()> {
    let response = sys::mln_resource_response {
        size: std::mem::size_of::<sys::mln_resource_response>() as u32,
        status: sys::MLN_RESOURCE_RESPONSE_STATUS_OK,
        error_reason: sys::MLN_RESOURCE_ERROR_REASON_NONE,
        bytes: ptr::null(),
        byte_count: 0,
        error_message: ptr::null(),
        must_revalidate: false,
        has_modified: false,
        modified_unix_ms: 0,
        has_expires: false,
        expires_unix_ms: 0,
        etag: ptr::null(),
        has_retry_after: false,
        retry_after_unix_ms: 0,
    };
    glib::clear_optional_out_pointer(out_response, response)
}

fn complete_request(
    handle: *mut ResourceRequestHandle,
    response: *const sys::mln_resource_response,
) -> error::Result<()> {
    let native = request_native(handle)?;
    if response.is_null() {
        return Err(Error::invalid_argument("resource response is null"));
    }

    // SAFETY: `native` is a live provider-owned request handle, and `response`
    // points to caller-owned response data borrowed for this call. The C API
    // copies response contents before returning.
    error::check(unsafe { sys::mln_resource_request_complete(native, response) })
}

fn is_cancelled(
    handle: *mut ResourceRequestHandle,
    out_cancelled: *mut GBoolean,
) -> error::Result<()> {
    let native = request_native(handle)?;
    let mut cancelled = false;
    // SAFETY: `native` is a live provider-owned request handle and `cancelled`
    // is valid output storage.
    error::check(unsafe { sys::mln_resource_request_cancelled(native, &mut cancelled) })?;
    glib::clear_optional_out_pointer(out_cancelled, if cancelled { GTRUE } else { GFALSE })
}

fn release_request(handle: *mut ResourceRequestHandle) {
    if handle.is_null() {
        return;
    }

    // SAFETY: `handle` is non-null and expected to point to this GObject
    // subtype. Swapping the native pointer makes release exactly-once.
    let native = unsafe {
        let native = (*handle).native;
        (*handle).native = ptr::null_mut();
        native
    };
    if native.is_null() {
        return;
    }

    // SAFETY: `native` is the provider-owned request reference held by this
    // wrapper. The C API accepts null and releases non-null exactly once.
    unsafe { sys::mln_resource_request_release(native) };
}

fn request_native(
    handle: *mut ResourceRequestHandle,
) -> error::Result<*mut sys::mln_resource_request_handle> {
    if handle.is_null() {
        return Err(Error::invalid_argument("ResourceRequestHandle is null"));
    }

    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe { (*handle).native };
    if native.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "ResourceRequestHandle is released",
        ));
    }
    Ok(native)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_response_default_initializes_size_and_ok_status() {
        // SAFETY: Zeroed storage is immediately initialized by the public
        // default entry point before use.
        let mut response: sys::mln_resource_response = unsafe { std::mem::zeroed() };

        assert_eq!(
            mln_vala_resource_response_default(&mut response, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            response.size,
            std::mem::size_of::<sys::mln_resource_response>() as u32
        );
        assert_eq!(response.status, sys::MLN_RESOURCE_RESPONSE_STATUS_OK);
        assert_eq!(response.error_reason, sys::MLN_RESOURCE_ERROR_REASON_NONE);
    }

    #[test]
    fn null_resource_request_handle_reports_binding_error() {
        let mut error = ptr::null_mut();
        assert_eq!(
            mln_vala_resource_request_handle_is_cancelled(
                ptr::null_mut(),
                ptr::null_mut(),
                &mut error,
            ),
            GFALSE
        );
        assert!(!error.is_null());
    }

    #[test]
    fn released_resource_request_handle_reports_invalid_state() {
        let handle =
            glib::new_object::<ResourceRequestHandle>(mln_vala_resource_request_handle_get_type());
        assert!(!handle.is_null());

        let mut error = ptr::null_mut();
        assert_eq!(
            mln_vala_resource_request_handle_complete(handle, ptr::null(), &mut error),
            GFALSE
        );
        assert!(!error.is_null());

        mln_vala_resource_request_handle_release(handle);
        glib::unref_object(handle);
    }
}

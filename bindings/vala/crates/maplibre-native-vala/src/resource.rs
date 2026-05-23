#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, c_char};
use std::ptr;
use std::sync::Mutex;

use maplibre_native_core::error::{self, Error, ErrorKind};
use maplibre_native_sys as sys;

use crate::glib::{self, GBoolean, GBytes, GError, GFALSE, GObject, GTRUE, GType};

const RESOURCE_REQUEST_TYPE_NAME: &CStr = c"MlnValaResourceRequestHandle";

#[derive(Debug)]
struct RequestState {
    native: *mut sys::mln_resource_request_handle,
    decision_finalized: bool,
    provider_owned: bool,
    release_accounted_for: bool,
    closed: bool,
    completed: bool,
}

#[repr(C)]
pub struct ResourceRequestHandle {
    parent_instance: GObject,
    state: Mutex<RequestState>,
}

impl glib::ObjectFinalize for ResourceRequestHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        release_request(object.cast::<ResourceRequestHandle>());
    }
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
pub extern "C" fn mln_vala_resource_response_set_bytes(
    response: *mut sys::mln_resource_response,
    bytes: *const u8,
    byte_count: usize,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_response_bytes(response, bytes, byte_count) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_resource_request_get_url(
    request: *const sys::mln_resource_request,
) -> *const c_char {
    if request.is_null() {
        return ptr::null();
    }
    // SAFETY: callback-duration borrowed request; caller uses this during the callback.
    unsafe { (*request).url }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_resource_request_dup_prior_data(
    request: *const sys::mln_resource_request,
) -> *mut GBytes {
    if request.is_null() {
        return ptr::null_mut();
    }
    // SAFETY: callback-duration borrowed request; copy bytes immediately.
    let request = unsafe { &*request };
    if request.prior_data_size == 0 {
        return glib::bytes_new(&[]);
    }
    if request.prior_data.is_null() {
        return ptr::null_mut();
    }
    let bytes = unsafe { std::slice::from_raw_parts(request.prior_data, request.prior_data_size) };
    glib::bytes_new(bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_resource_request_handle_retain_for_async(
    handle: *mut ResourceRequestHandle,
    error_out: *mut *mut GError,
) -> *mut ResourceRequestHandle {
    match validate_request_handle(handle) {
        Ok(()) => glib::ref_object(handle),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
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
        return ptr::null_mut();
    }

    unsafe {
        ptr::write(
            &mut (*handle).state,
            Mutex::new(RequestState {
                native,
                decision_finalized: false,
                provider_owned: false,
                release_accounted_for: false,
                closed: false,
                completed: false,
            }),
        );
    }
    handle
}

#[allow(dead_code)]
pub(crate) unsafe fn resource_request_handle_disarm(handle: *mut ResourceRequestHandle) {
    if handle.is_null() {
        return;
    }
    if let Ok(mut state) = unsafe { &*handle }.state.lock() {
        state.decision_finalized = true;
        state.release_accounted_for = true;
        state.closed = true;
        state.native = ptr::null_mut();
    }
}

pub(crate) unsafe fn resource_request_handle_finish_provider_decision(
    handle: *mut ResourceRequestHandle,
    decision: u32,
) -> u32 {
    if handle.is_null() {
        return sys::MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH;
    }
    let Ok(mut state) = (unsafe { &*handle }).state.lock() else {
        return u32::MAX;
    };
    if state.decision_finalized {
        return if state.provider_owned {
            sys::MLN_RESOURCE_PROVIDER_DECISION_HANDLE
        } else {
            sys::MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH
        };
    }
    if state.completed || decision == sys::MLN_RESOURCE_PROVIDER_DECISION_HANDLE {
        state.decision_finalized = true;
        state.provider_owned = true;
        if state.closed {
            release_if_owned_locked(&mut state);
        }
        return sys::MLN_RESOURCE_PROVIDER_DECISION_HANDLE;
    }

    state.decision_finalized = true;
    state.release_accounted_for = true;
    state.closed = true;
    state.native = ptr::null_mut();
    if decision == sys::MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH {
        sys::MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH
    } else {
        decision
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

fn set_response_bytes(
    response: *mut sys::mln_resource_response,
    bytes: *const u8,
    byte_count: usize,
) -> error::Result<()> {
    if response.is_null() {
        return Err(Error::invalid_argument("ResourceResponse is null"));
    }
    if byte_count != 0 && bytes.is_null() {
        return Err(Error::invalid_argument("ResourceResponse bytes are null"));
    }
    unsafe {
        (*response).bytes = bytes;
        (*response).byte_count = byte_count;
    }
    Ok(())
}

fn validate_request_handle(handle: *mut ResourceRequestHandle) -> error::Result<()> {
    let state = request_state(handle)?;
    if state.closed || state.native.is_null() {
        return Err(Error::new(
            ErrorKind::InvalidState,
            None,
            "ResourceRequestHandle is released",
        ));
    }
    Ok(())
}

fn complete_request(
    handle: *mut ResourceRequestHandle,
    response: *const sys::mln_resource_response,
) -> error::Result<()> {
    if response.is_null() {
        return Err(Error::invalid_argument("resource response is null"));
    }
    let mut state = request_state(handle)?;
    if state.completed {
        return Err(Error::new(
            ErrorKind::InvalidState,
            None,
            "ResourceRequestHandle is already completed",
        ));
    }
    if state.closed || state.native.is_null() {
        return Err(Error::new(
            ErrorKind::InvalidState,
            None,
            "ResourceRequestHandle is released",
        ));
    }
    let native = state.native;
    error::check(unsafe { sys::mln_resource_request_complete(native, response) })?;
    state.completed = true;
    state.closed = true;
    if state.decision_finalized && state.provider_owned {
        release_if_owned_locked(&mut state);
    }
    Ok(())
}

fn is_cancelled(
    handle: *mut ResourceRequestHandle,
    out_cancelled: *mut GBoolean,
) -> error::Result<()> {
    let state = request_state(handle)?;
    if state.closed || state.native.is_null() {
        return Err(Error::new(
            ErrorKind::InvalidState,
            None,
            "ResourceRequestHandle is released",
        ));
    }
    let mut cancelled = false;
    error::check(unsafe { sys::mln_resource_request_cancelled(state.native, &mut cancelled) })?;
    glib::clear_optional_out_pointer(out_cancelled, if cancelled { GTRUE } else { GFALSE })
}

fn release_request(handle: *mut ResourceRequestHandle) {
    if handle.is_null() {
        return;
    }
    let Ok(mut state) = (unsafe { &*handle }).state.lock() else {
        return;
    };
    if state.closed {
        return;
    }
    state.closed = true;
    if state.decision_finalized && state.provider_owned {
        release_if_owned_locked(&mut state);
    }
}

fn request_state(
    handle: *mut ResourceRequestHandle,
) -> error::Result<std::sync::MutexGuard<'static, RequestState>> {
    if handle.is_null() {
        return Err(Error::invalid_argument("ResourceRequestHandle is null"));
    }
    unsafe { &*handle }.state.lock().map_err(|_| {
        Error::new(
            ErrorKind::NativeError,
            None,
            "ResourceRequestHandle lock poisoned",
        )
    })
}

fn release_if_owned_locked(state: &mut RequestState) {
    if state.release_accounted_for || state.native.is_null() {
        return;
    }
    state.release_accounted_for = true;
    let native = state.native;
    state.native = ptr::null_mut();
    unsafe { sys::mln_resource_request_release(native) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_response_default_initializes_size_and_ok_status() {
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
    fn unknown_provider_decision_is_not_collapsed_to_pass_through() {
        let mut handle = ResourceRequestHandle {
            parent_instance: unsafe { std::mem::zeroed() },
            state: Mutex::new(RequestState {
                native: std::ptr::dangling_mut::<sys::mln_resource_request_handle>(),
                decision_finalized: false,
                provider_owned: false,
                release_accounted_for: false,
                closed: false,
                completed: false,
            }),
        };

        let decision =
            unsafe { resource_request_handle_finish_provider_decision(&mut handle, u32::MAX) };

        assert_eq!(decision, u32::MAX);
        assert_ne!(decision, sys::MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH);
        let state = handle.state.lock().expect("request state lock");
        assert!(state.decision_finalized);
        assert!(state.release_accounted_for);
        assert!(state.closed);
        assert!(state.native.is_null());
    }
}

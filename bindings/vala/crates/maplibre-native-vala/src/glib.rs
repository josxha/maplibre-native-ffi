use std::ffi::{CStr, CString, c_char, c_int, c_uint};
use std::ptr;

use maplibre_native_core::error::{Error, ErrorKind};
use maplibre_native_sys as sys;

pub type GBoolean = c_int;
pub type GQuark = c_uint;

pub const GTRUE: GBoolean = 1;
pub const GFALSE: GBoolean = 0;

#[repr(C)]
pub struct GError {
    pub domain: GQuark,
    pub code: c_int,
    pub message: *mut c_char,
}

unsafe extern "C" {
    fn g_quark_from_static_string(string: *const c_char) -> GQuark;
    fn g_error_new_literal(domain: GQuark, code: c_int, message: *const c_char) -> *mut GError;
}

const ERROR_DOMAIN: &CStr = c"maplibre-native-error-quark";
const FALLBACK_DIAGNOSTIC: &CStr = c"MapLibre Native operation failed";

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidArgument = 0,
    InvalidState = 1,
    WrongThread = 2,
    Unsupported = 3,
    NativeError = 4,
    UnknownStatus = 5,
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_error_quark() -> GQuark {
    // SAFETY: The string is statically allocated and NUL-terminated.
    unsafe { g_quark_from_static_string(ERROR_DOMAIN.as_ptr()) }
}

pub(crate) fn set_error(error_out: *mut *mut GError, error: Error) {
    if error_out.is_null() {
        return;
    }

    let message = CString::new(error.diagnostic()).ok();
    let message_ptr = message
        .as_ref()
        .map_or(FALLBACK_DIAGNOSTIC.as_ptr(), |value| value.as_ptr());

    // SAFETY: `error_out` is a caller-provided optional GError**. GLib permits
    // setting it to a newly allocated GError when it is non-null. `message_ptr`
    // remains valid for this call, and GLib copies the literal into the error.
    unsafe {
        *error_out = g_error_new_literal(
            mln_vala_error_quark(),
            error_code(error).code(),
            message_ptr,
        );
    }
}

fn error_code(error: Error) -> ErrorCode {
    if let Some(raw_status) = error.raw_status() {
        return error_code_for_status(raw_status);
    }

    match error.kind() {
        ErrorKind::InvalidArgument | ErrorKind::AbiVersionMismatch => ErrorCode::InvalidArgument,
        ErrorKind::InvalidState => ErrorCode::InvalidState,
        ErrorKind::WrongThread => ErrorCode::WrongThread,
        ErrorKind::Unsupported => ErrorCode::Unsupported,
        ErrorKind::NativeError => ErrorCode::NativeError,
        ErrorKind::UnknownStatus => ErrorCode::UnknownStatus,
        _ => ErrorCode::NativeError,
    }
}

fn error_code_for_status(status: sys::mln_status) -> ErrorCode {
    match status {
        sys::MLN_STATUS_INVALID_ARGUMENT => ErrorCode::InvalidArgument,
        sys::MLN_STATUS_INVALID_STATE => ErrorCode::InvalidState,
        sys::MLN_STATUS_WRONG_THREAD => ErrorCode::WrongThread,
        sys::MLN_STATUS_UNSUPPORTED => ErrorCode::Unsupported,
        sys::MLN_STATUS_NATIVE_ERROR => ErrorCode::NativeError,
        _ => ErrorCode::UnknownStatus,
    }
}

impl ErrorCode {
    fn code(self) -> c_int {
        self as c_int
    }
}

pub(crate) fn clear_optional_out_pointer<T>(out: *mut T, value: T) -> Result<(), Error> {
    if out.is_null() {
        return Err(Error::invalid_argument("output pointer is null"));
    }

    // SAFETY: The null check above proves the caller provided writable storage
    // for one `T` value.
    unsafe {
        ptr::write(out, value);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_domain_is_registered() {
        assert_ne!(mln_vala_error_quark(), 0);
    }

    #[test]
    fn null_out_pointer_returns_binding_error() {
        let error = clear_optional_out_pointer::<u32>(ptr::null_mut(), 1).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidArgument);
        assert_eq!(error.raw_status(), None);
    }
}

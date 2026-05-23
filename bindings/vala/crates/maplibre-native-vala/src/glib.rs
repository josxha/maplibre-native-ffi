use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_void};
use std::mem;
use std::ptr;
use std::sync::OnceLock;

use maplibre_native_core::error::{Error, ErrorKind};
use maplibre_native_sys as sys;

pub type GBoolean = c_int;
pub type GDestroyNotify = Option<unsafe extern "C" fn(*mut c_void)>;
pub type GQuark = c_uint;
pub type GType = usize;

pub const GTRUE: GBoolean = 1;
pub const GFALSE: GBoolean = 0;

#[repr(C)]
pub struct GTypeClass {
    _private: [u8; 0],
}

#[repr(C)]
pub struct GTypeInstance {
    pub g_class: *mut GTypeClass,
}

#[repr(C)]
pub struct GObject {
    pub g_type_instance: GTypeInstance,
    pub ref_count: c_uint,
    pub qdata: *mut c_void,
}

#[repr(C)]
struct GTypeQuery {
    type_: GType,
    type_name: *const c_char,
    class_size: c_uint,
    instance_size: c_uint,
}

#[repr(C)]
pub struct GError {
    pub domain: GQuark,
    pub code: c_int,
    pub message: *mut c_char,
}

unsafe extern "C" {
    fn g_quark_from_static_string(string: *const c_char) -> GQuark;
    fn g_error_new_literal(domain: GQuark, code: c_int, message: *const c_char) -> *mut GError;
    fn g_free(mem: *mut c_void);
    fn g_boxed_type_register_static(
        name: *const c_char,
        boxed_copy: Option<unsafe extern "C" fn(*mut c_void) -> *mut c_void>,
        boxed_free: Option<unsafe extern "C" fn(*mut c_void)>,
    ) -> GType;
    fn g_object_get_type() -> GType;
    fn g_object_new_with_properties(
        object_type: GType,
        n_properties: c_uint,
        names: *const *const c_char,
        values: *const c_void,
    ) -> *mut GObject;
    fn g_object_ref(object: *mut GObject) -> *mut GObject;
    fn g_object_unref(object: *mut GObject);
    fn g_type_query(type_: GType, query: *mut GTypeQuery);
    fn g_type_register_static_simple(
        parent_type: GType,
        type_name: *const c_char,
        class_size: c_uint,
        class_init: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
        instance_size: c_uint,
        instance_init: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
        flags: c_uint,
    ) -> GType;
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

pub fn register_boxed_type(
    type_name: &'static CStr,
    boxed_copy: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    boxed_free: unsafe extern "C" fn(*mut c_void),
) -> GType {
    static BOXED_TYPE_REGISTRY: OnceLock<std::sync::Mutex<Vec<(&'static CStr, GType)>>> =
        OnceLock::new();
    let registry = BOXED_TYPE_REGISTRY.get_or_init(|| std::sync::Mutex::new(Vec::new()));
    let mut registry = registry.lock().expect("GBoxed type registry lock poisoned");

    if let Some((_, gtype)) = registry.iter().find(|(name, _)| *name == type_name) {
        return *gtype;
    }

    // SAFETY: The type name is static and NUL-terminated, and the copy/free
    // functions satisfy GLib boxed ownership conventions for this type.
    let gtype = unsafe {
        g_boxed_type_register_static(type_name.as_ptr(), Some(boxed_copy), Some(boxed_free))
    };
    registry.push((type_name, gtype));
    gtype
}

pub fn register_object_type<T>(type_name: &'static CStr) -> GType {
    static TYPE_REGISTRY: OnceLock<std::sync::Mutex<Vec<(&'static CStr, GType)>>> = OnceLock::new();
    let registry = TYPE_REGISTRY.get_or_init(|| std::sync::Mutex::new(Vec::new()));
    let mut registry = registry
        .lock()
        .expect("GObject type registry lock poisoned");

    if let Some((_, gtype)) = registry.iter().find(|(name, _)| *name == type_name) {
        return *gtype;
    }

    // SAFETY: `g_object_get_type` returns the registered GObject base type, and
    // `g_type_query` initializes the query struct for that type.
    let parent_type = unsafe { g_object_get_type() };
    let mut parent_query = GTypeQuery {
        type_: 0,
        type_name: ptr::null(),
        class_size: 0,
        instance_size: 0,
    };
    // SAFETY: `parent_query` points to writable storage for one query result.
    unsafe { g_type_query(parent_type, &mut parent_query) };

    // SAFETY: The type name is static and NUL-terminated. The instance type `T`
    // starts with `GObject` and has a stable `repr(C)` layout. No class or
    // instance init hooks are needed for these data-only handle wrappers.
    let gtype = unsafe {
        g_type_register_static_simple(
            parent_type,
            type_name.as_ptr(),
            parent_query.class_size,
            None,
            mem::size_of::<T>() as c_uint,
            None,
            0,
        )
    };
    registry.push((type_name, gtype));
    gtype
}

pub fn new_object<T>(gtype: GType) -> *mut T {
    // SAFETY: `gtype` is a registered `GObject` subtype with instance size
    // matching `T`. No construct properties are supplied.
    unsafe { g_object_new_with_properties(gtype, 0, ptr::null(), ptr::null()) as *mut T }
}

pub fn ref_object<T>(object: *mut T) -> *mut T {
    if object.is_null() {
        return ptr::null_mut();
    }

    // SAFETY: The caller supplied a live GObject instance pointer.
    unsafe { g_object_ref(object.cast::<GObject>()).cast::<T>() }
}

pub fn unref_object<T>(object: *mut T) {
    if object.is_null() {
        return;
    }

    // SAFETY: The caller supplied a live GObject instance pointer and releases
    // one reference.
    unsafe { g_object_unref(object.cast::<GObject>()) }
}

/// Releases GLib-allocated memory.
///
/// # Safety
///
/// `mem` must be null or point to memory allocated by GLib-compatible
/// allocation APIs, and ownership must be transferred to this function.
pub unsafe fn free(mem: *mut c_void) {
    if mem.is_null() {
        return;
    }

    // SAFETY: The caller supplied memory allocated by GLib-compatible
    // allocation APIs and transfers it for release.
    unsafe { g_free(mem) }
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

use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr;

use maplibre_native_sys as sys;

use crate::glib::{self, GType};

const RUNTIME_EVENT_TYPE_NAME: &CStr = c"MlnValaRuntimeEvent";

#[repr(C)]
#[derive(Debug)]
pub struct RuntimeEvent {
    pub type_: u32,
    pub source_type: u32,
    pub code: i32,
    pub payload_type: u32,
    pub message: *mut c_char,
    pub message_size: usize,
}

impl RuntimeEvent {
    pub fn from_native(event: &sys::mln_runtime_event) -> Self {
        let message = copy_message(event.message, event.message_size);
        let message_size = message
            .as_ref()
            .map_or(0, |message| message.as_bytes().len());

        Self {
            type_: event.type_,
            source_type: event.source_type,
            code: event.code,
            payload_type: event.payload_type,
            message: message.map_or(ptr::null_mut(), CString::into_raw),
            message_size,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_get_type() -> GType {
    glib::register_boxed_type(
        RUNTIME_EVENT_TYPE_NAME,
        mln_vala_runtime_event_copy_erased,
        mln_vala_runtime_event_free_erased,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_copy(event: *const RuntimeEvent) -> *mut RuntimeEvent {
    runtime_event_copy(event)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_event_free(event: *mut RuntimeEvent) {
    runtime_event_free(event);
}

fn runtime_event_copy(event: *const RuntimeEvent) -> *mut RuntimeEvent {
    if event.is_null() {
        return ptr::null_mut();
    }

    // SAFETY: The caller supplied a non-null pointer to a live RuntimeEvent.
    let event = unsafe { &*event };
    let message = if event.message.is_null() {
        None
    } else {
        // SAFETY: RuntimeEvent owns a NUL-terminated message pointer when it is
        // non-null.
        Some(unsafe { CStr::from_ptr(event.message) }.to_owned())
    };

    Box::into_raw(Box::new(RuntimeEvent {
        type_: event.type_,
        source_type: event.source_type,
        code: event.code,
        payload_type: event.payload_type,
        message: message.map_or(ptr::null_mut(), CString::into_raw),
        message_size: event.message_size,
    }))
}

fn runtime_event_free(event: *mut RuntimeEvent) {
    if event.is_null() {
        return;
    }

    // SAFETY: The caller transfers ownership of a RuntimeEvent allocated by this
    // adapter.
    let event = unsafe { Box::from_raw(event) };
    if !event.message.is_null() {
        // SAFETY: RuntimeEvent owns this CString allocation.
        unsafe {
            drop(CString::from_raw(event.message));
        }
    }
}

unsafe extern "C" fn mln_vala_runtime_event_copy_erased(event: *mut c_void) -> *mut c_void {
    mln_vala_runtime_event_copy(event.cast::<RuntimeEvent>()).cast::<c_void>()
}

unsafe extern "C" fn mln_vala_runtime_event_free_erased(event: *mut c_void) {
    mln_vala_runtime_event_free(event.cast::<RuntimeEvent>());
}

fn copy_message(message: *const c_char, message_size: usize) -> Option<CString> {
    if message.is_null() || message_size == 0 {
        return None;
    }

    // SAFETY: The C API provides `message_size` bytes that are valid until the
    // next poll. This function copies those bytes immediately.
    let bytes = unsafe { std::slice::from_raw_parts(message.cast::<u8>(), message_size) };
    CString::new(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_event_boxed_type_is_registered() {
        assert_ne!(mln_vala_runtime_event_get_type(), 0);
    }

    #[test]
    fn runtime_event_copy_owns_message() {
        let message = CString::new("hello").unwrap();
        let event = RuntimeEvent {
            type_: 1,
            source_type: 2,
            code: 3,
            payload_type: 4,
            message: message.into_raw(),
            message_size: 5,
        };
        let copy = mln_vala_runtime_event_copy(&event);

        assert!(!copy.is_null());
        // SAFETY: copy was just allocated and contains a non-null message.
        unsafe {
            assert_eq!(CStr::from_ptr((*copy).message).to_str().unwrap(), "hello");
        }

        mln_vala_runtime_event_free(copy);
        if !event.message.is_null() {
            // SAFETY: event still owns the original message allocation.
            unsafe {
                drop(CString::from_raw(event.message));
            }
        }
    }
}

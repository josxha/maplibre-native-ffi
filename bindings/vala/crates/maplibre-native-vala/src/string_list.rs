#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr;

use maplibre_native_core::error::Error;
use maplibre_native_sys as sys;

use crate::glib::{self, GError, GType};

const STRING_LIST_TYPE_NAME: &CStr = c"MlnValaStringList";

pub struct StringList {
    strings: Vec<CString>,
    views: Vec<sys::mln_string_view>,
}

// SAFETY: The string views point into immutable CString allocations owned by
// this struct. Moving a list between threads does not invalidate those
// allocations; shared access happens through normal Rust references or locks.
unsafe impl Send for StringList {}

impl Clone for StringList {
    fn clone(&self) -> Self {
        let strings = self
            .strings
            .iter()
            .map(|value| CString::new(value.as_bytes()).expect("CString bytes contain no NUL"))
            .collect();
        Self::from_strings(strings)
    }
}

impl StringList {
    pub(crate) fn from_strings(strings: Vec<CString>) -> Self {
        let views = strings
            .iter()
            .map(|value| sys::mln_string_view {
                data: value.as_ptr(),
                size: value.as_bytes().len(),
            })
            .collect();
        Self { strings, views }
    }

    pub(crate) fn as_ptr(&self) -> *const sys::mln_string_view {
        if self.views.is_empty() {
            ptr::null()
        } else {
            self.views.as_ptr()
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.views.len()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_string_list_get_type() -> GType {
    glib::register_boxed_type(
        STRING_LIST_TYPE_NAME,
        string_list_copy_boxed,
        string_list_free_boxed,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_string_list_new(
    values: *const *const c_char,
    value_count: usize,
    error_out: *mut *mut GError,
) -> *mut StringList {
    match new_string_list(values, value_count) {
        Ok(list) => Box::into_raw(Box::new(list)),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_string_list_copy(list: *const StringList) -> *mut StringList {
    string_list_ref(list)
        .map(|value| Box::into_raw(Box::new(value.clone())))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_string_list_count(list: *const StringList) -> usize {
    string_list_ref(list).map_or(0, StringList::len)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_string_list_get(
    list: *const StringList,
    index: usize,
    error_out: *mut *mut GError,
) -> *mut c_char {
    match string_list_get(list, index) {
        Ok(value) => value,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_string_list_free(list: *mut StringList) {
    if list.is_null() {
        return;
    }
    // SAFETY: GLib/Vala transfers a pointer allocated by `Box::into_raw` in
    // this module.
    unsafe { drop(Box::from_raw(list)) };
}

unsafe extern "C" fn string_list_copy_boxed(list: *mut c_void) -> *mut c_void {
    mln_vala_string_list_copy(list.cast::<StringList>()).cast::<c_void>()
}

unsafe extern "C" fn string_list_free_boxed(list: *mut c_void) {
    mln_vala_string_list_free(list.cast::<StringList>())
}

fn string_list_get(list: *const StringList, index: usize) -> Result<*mut c_char, Error> {
    let list =
        string_list_ref(list).ok_or_else(|| Error::invalid_argument("string list is null"))?;
    let view = list
        .views
        .get(index)
        .copied()
        .ok_or_else(|| Error::invalid_argument("string list index is out of range"))?;
    glib::copy_string_view(view)
}

fn new_string_list(values: *const *const c_char, value_count: usize) -> Result<StringList, Error> {
    if value_count != 0 && values.is_null() {
        return Err(Error::invalid_argument("string list values are null"));
    }

    let mut strings = Vec::with_capacity(value_count);
    for index in 0..value_count {
        // SAFETY: `values` points to `value_count` readable C string pointers.
        let value = unsafe { *values.add(index) };
        if value.is_null() {
            return Err(Error::invalid_argument(
                "string list contains a null string",
            ));
        }
        // SAFETY: Each value is a non-null NUL-terminated UTF-8 string supplied
        // by GLib/Vala for this call.
        let bytes = unsafe { CStr::from_ptr(value) }.to_bytes();
        strings.push(
            CString::new(bytes)
                .map_err(|_| Error::invalid_argument("string list contains embedded NUL"))?,
        );
    }
    Ok(StringList::from_strings(strings))
}

pub(crate) fn string_list_ref(list: *const StringList) -> Option<&'static StringList> {
    if list.is_null() {
        return None;
    }
    // SAFETY: The caller supplies a live boxed `StringList` pointer. Public
    // adapter calls use the returned borrow only for the current call unless
    // they explicitly copy it into descriptor-owned storage.
    Some(unsafe { &*list })
}

use std::ffi::{CStr, c_void};
use std::num::NonZeroUsize;
use std::ptr;

use crate::glib::{self, GBoolean, GError, GFALSE, GTRUE, GType};

const NATIVE_POINTER_TYPE_NAME: &CStr = c"MlnValaNativePointer";

/// Borrowed non-null backend-native address value.
///
/// This mirrors the public `NativePointer` value planned for the GLib/Vala API.
/// It never owns the pointee and grants no memory access by itself. Nullable API
/// positions should use `Option<NativePointer>` rather than treating null as a
/// valid pointer value.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NativePointer {
    bits: NonZeroUsize,
}

impl NativePointer {
    pub fn from_ptr(ptr: *mut c_void) -> Option<Self> {
        NonZeroUsize::new(ptr as usize).map(|bits| Self { bits })
    }

    pub fn as_ptr(self) -> *mut c_void {
        self.bits.get() as *mut c_void
    }

    pub fn bits(self) -> usize {
        self.bits.get()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_native_pointer_get_type() -> GType {
    glib::register_boxed_type(
        NATIVE_POINTER_TYPE_NAME,
        mln_vala_native_pointer_copy_erased,
        mln_vala_native_pointer_free_erased,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_native_pointer_new(
    bits: usize,
    out_pointer: *mut *mut NativePointer,
    error_out: *mut *mut GError,
) -> GBoolean {
    match native_pointer_new(bits, out_pointer) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_native_pointer_get_bits(pointer: *const NativePointer) -> usize {
    native_pointer_ref(pointer).map_or(0, NativePointer::bits)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_native_pointer_copy(
    pointer: *const NativePointer,
) -> *mut NativePointer {
    native_pointer_copy(pointer)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_native_pointer_free(pointer: *mut NativePointer) {
    native_pointer_free(pointer);
}

fn native_pointer_new(
    bits: usize,
    out_pointer: *mut *mut NativePointer,
) -> maplibre_native_core::error::Result<()> {
    if out_pointer.is_null() {
        return Err(maplibre_native_core::error::Error::invalid_argument(
            "native pointer output pointer is null",
        ));
    }
    let Some(bits) = NonZeroUsize::new(bits) else {
        return Err(maplibre_native_core::error::Error::invalid_argument(
            "NativePointer address is null",
        ));
    };
    let pointer = Box::into_raw(Box::new(NativePointer { bits }));
    // SAFETY: `out_pointer` was null-checked and points to writable storage for
    // one pointer-sized value.
    unsafe {
        *out_pointer = pointer;
    }
    Ok(())
}

fn native_pointer_ref(pointer: *const NativePointer) -> Option<NativePointer> {
    if pointer.is_null() {
        return None;
    }

    // SAFETY: The caller supplied a non-null pointer to a live NativePointer.
    Some(unsafe { *pointer })
}

fn native_pointer_copy(pointer: *const NativePointer) -> *mut NativePointer {
    native_pointer_ref(pointer).map_or(ptr::null_mut(), |pointer| Box::into_raw(Box::new(pointer)))
}

fn native_pointer_free(pointer: *mut NativePointer) {
    if pointer.is_null() {
        return;
    }

    // SAFETY: The caller transfers ownership of a NativePointer allocated by
    // this adapter.
    unsafe {
        drop(Box::from_raw(pointer));
    }
}

unsafe extern "C" fn mln_vala_native_pointer_copy_erased(pointer: *mut c_void) -> *mut c_void {
    native_pointer_copy(pointer.cast::<NativePointer>()).cast::<c_void>()
}

unsafe extern "C" fn mln_vala_native_pointer_free_erased(pointer: *mut c_void) {
    native_pointer_free(pointer.cast::<NativePointer>());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_pointer_is_not_a_native_pointer_value() {
        assert_eq!(NativePointer::from_ptr(std::ptr::null_mut()), None);
    }

    #[test]
    fn non_null_pointer_round_trips_bits() {
        let ptr = std::ptr::dangling_mut::<c_void>();
        let native_pointer = NativePointer::from_ptr(ptr).expect("dangling pointer is non-null");

        assert_eq!(native_pointer.as_ptr(), ptr);
        assert_eq!(native_pointer.bits(), ptr as usize);
    }

    #[test]
    fn native_pointer_constructor_rejects_zero_bits() {
        let mut pointer = ptr::null_mut();
        let mut error = ptr::null_mut();

        assert_eq!(
            mln_vala_native_pointer_new(0, &mut pointer, &mut error),
            GFALSE
        );
        assert!(pointer.is_null());
        assert!(!error.is_null());
    }

    #[test]
    fn native_pointer_boxed_copy_preserves_bits() {
        let mut pointer = ptr::null_mut();
        assert_eq!(
            mln_vala_native_pointer_new(0x1234, &mut pointer, ptr::null_mut()),
            GTRUE
        );
        let copy = mln_vala_native_pointer_copy(pointer);

        assert!(!copy.is_null());
        assert_eq!(mln_vala_native_pointer_get_bits(copy), 0x1234);

        mln_vala_native_pointer_free(copy);
        mln_vala_native_pointer_free(pointer);
    }
}

use std::ffi::c_void;
use std::num::NonZeroUsize;

/// Borrowed non-null backend-native address value.
///
/// This mirrors the public `NativePointer` value planned for the GLib/Vala API.
/// It never owns the pointee and grants no memory access by itself. Nullable API
/// positions should use `Option<NativePointer>` rather than treating null as a
/// valid pointer value.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NativePointer(NonZeroUsize);

impl NativePointer {
    pub fn from_ptr(ptr: *mut c_void) -> Option<Self> {
        NonZeroUsize::new(ptr as usize).map(Self)
    }

    pub fn as_ptr(self) -> *mut c_void {
        self.0.get() as *mut c_void
    }

    pub fn bits(self) -> usize {
        self.0.get()
    }
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
}

use maplibre_native_core::error;
use maplibre_native_sys as sys;

use crate::glib::{self, GBoolean, GError, GFALSE, GTRUE};

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_log_clear_callback(error_out: *mut *mut GError) -> GBoolean {
    // SAFETY: The C API takes no input pointers and clears process-global state.
    match error::check(unsafe { sys::mln_log_clear_callback() }) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_log_set_async_severity_mask(
    mask: u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    // SAFETY: The C API validates mask domain bits.
    match error::check(unsafe { sys::mln_log_set_async_severity_mask(mask) }) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn async_severity_mask_accepts_default_and_rejects_unknown_bits() {
        assert_eq!(
            mln_vala_log_set_async_severity_mask(
                sys::MLN_LOG_SEVERITY_MASK_DEFAULT,
                std::ptr::null_mut(),
            ),
            GTRUE
        );

        let mut error = std::ptr::null_mut();
        assert_eq!(
            mln_vala_log_set_async_severity_mask(1 << 31, &mut error),
            GFALSE
        );
        assert!(!error.is_null());
    }

    #[test]
    fn clear_callback_succeeds_without_callback() {
        assert_eq!(mln_vala_log_clear_callback(std::ptr::null_mut()), GTRUE);
    }
}

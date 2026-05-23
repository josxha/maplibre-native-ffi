use std::ffi::{c_char, c_void};
use std::sync::Mutex;

use maplibre_native_core::error;
use maplibre_native_sys as sys;

use crate::glib::{self, GBoolean, GDestroyNotify, GError, GFALSE, GTRUE};

type LogCallback = unsafe extern "C" fn(
    severity: u32,
    event: u32,
    code: i64,
    message: *const c_char,
    user_data: *mut c_void,
) -> GBoolean;

#[derive(Clone, Copy)]
struct LogCallbackState {
    callback: LogCallback,
    user_data: usize,
    destroy_notify: GDestroyNotify,
}

static LOG_CALLBACK_STATE: Mutex<Option<LogCallbackState>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_log_set_callback(
    callback: Option<LogCallback>,
    user_data: *mut c_void,
    destroy_notify: GDestroyNotify,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_callback(callback, user_data, destroy_notify) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_log_clear_callback(error_out: *mut *mut GError) -> GBoolean {
    match clear_callback() {
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

fn set_callback(
    callback: Option<LogCallback>,
    user_data: *mut c_void,
    destroy_notify: GDestroyNotify,
) -> error::Result<()> {
    let Some(callback) = callback else {
        return clear_callback();
    };

    let replacement = LogCallbackState {
        callback,
        user_data: user_data as usize,
        destroy_notify,
    };

    // SAFETY: The native logger stores the trampoline and user data by
    // reference. The actual closure state lives in `LOG_CALLBACK_STATE` until
    // replacement or clear.
    error::check(unsafe {
        sys::mln_log_set_callback(Some(log_callback_trampoline), std::ptr::null_mut())
    })?;

    let old_state = {
        let mut state = LOG_CALLBACK_STATE
            .lock()
            .expect("log callback state lock poisoned");
        state.replace(replacement)
    };
    destroy_state(old_state);
    Ok(())
}

fn clear_callback() -> error::Result<()> {
    // SAFETY: The C API takes no input pointers and clears process-global state.
    error::check(unsafe { sys::mln_log_clear_callback() })?;
    let old_state = {
        let mut state = LOG_CALLBACK_STATE
            .lock()
            .expect("log callback state lock poisoned");
        state.take()
    };
    destroy_state(old_state);
    Ok(())
}

fn destroy_state(state: Option<LogCallbackState>) {
    let Some(state) = state else {
        return;
    };
    let Some(destroy_notify) = state.destroy_notify else {
        return;
    };

    // SAFETY: The destroy notify belongs to the user data stored with this
    // callback state and is called exactly once when that state is replaced or
    // cleared.
    unsafe {
        destroy_notify(state.user_data as *mut c_void);
    }
}

unsafe extern "C" fn log_callback_trampoline(
    _user_data: *mut c_void,
    severity: u32,
    event: u32,
    code: i64,
    message: *const c_char,
) -> u32 {
    let state = LOG_CALLBACK_STATE
        .lock()
        .expect("log callback state lock poisoned");
    let Some(state) = *state else {
        return 0;
    };

    // SAFETY: The callback and user data are kept live in LOG_CALLBACK_STATE
    // until replacement or clear. The message pointer is borrowed from native
    // logging for this callback duration.
    unsafe {
        if (state.callback)(
            severity,
            event,
            code,
            message,
            state.user_data as *mut c_void,
        ) != GFALSE
        {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DESTROY_COUNT: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn test_callback(
        _severity: u32,
        _event: u32,
        _code: i64,
        _message: *const c_char,
        _user_data: *mut c_void,
    ) -> GBoolean {
        GFALSE
    }

    unsafe extern "C" fn destroy_notify(_user_data: *mut c_void) {
        DESTROY_COUNT.fetch_add(1, Ordering::SeqCst);
    }

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

    #[test]
    fn replacing_and_clearing_callback_runs_destroy_notify() {
        assert_eq!(mln_vala_log_clear_callback(std::ptr::null_mut()), GTRUE);
        DESTROY_COUNT.store(0, Ordering::SeqCst);

        assert_eq!(
            mln_vala_log_set_callback(
                Some(test_callback),
                std::ptr::null_mut(),
                Some(destroy_notify),
                std::ptr::null_mut()
            ),
            GTRUE
        );
        assert_eq!(DESTROY_COUNT.load(Ordering::SeqCst), 0);

        assert_eq!(
            mln_vala_log_set_callback(
                Some(test_callback),
                std::ptr::null_mut(),
                Some(destroy_notify),
                std::ptr::null_mut()
            ),
            GTRUE
        );
        assert_eq!(DESTROY_COUNT.load(Ordering::SeqCst), 1);

        assert_eq!(mln_vala_log_clear_callback(std::ptr::null_mut()), GTRUE);
        assert_eq!(DESTROY_COUNT.load(Ordering::SeqCst), 2);
    }
}

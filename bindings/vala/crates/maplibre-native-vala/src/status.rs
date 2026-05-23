use maplibre_native_core::error::{Error, ErrorKind};
use maplibre_native_sys as sys;

const DIAGNOSTIC_CAPACITY: usize = 512;

/// Status-and-value result used by the scaffold's C-callable proof slice.
///
/// Final GObject entry points will translate failures into `GError`. This
/// temporary shape keeps tests allocation-free and records the same diagnostic
/// that a future `GError` message will carry.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatusResult {
    pub status: i32,
    pub value: u32,
    pub diagnostic_len: usize,
    pub diagnostic: [u8; DIAGNOSTIC_CAPACITY],
}

impl StatusResult {
    pub fn ok(value: u32) -> Self {
        Self {
            status: 0,
            value,
            diagnostic_len: 0,
            diagnostic: [0; DIAGNOSTIC_CAPACITY],
        }
    }

    pub fn from_error(error: Error) -> Self {
        let mut result = Self {
            status: status_for_error(&error),
            value: 0,
            diagnostic_len: 0,
            diagnostic: [0; DIAGNOSTIC_CAPACITY],
        };
        result.copy_diagnostic(error.diagnostic());
        result
    }

    pub fn diagnostic(&self) -> &str {
        let len = self.diagnostic_len.min(DIAGNOSTIC_CAPACITY);
        std::str::from_utf8(&self.diagnostic[..len]).unwrap_or("")
    }

    fn copy_diagnostic(&mut self, diagnostic: &str) {
        let bytes = diagnostic.as_bytes();
        let len = bytes.len().min(DIAGNOSTIC_CAPACITY);
        self.diagnostic[..len].copy_from_slice(&bytes[..len]);
        self.diagnostic_len = len;
    }
}

fn status_for_error(error: &Error) -> i32 {
    if let Some(status) = error.raw_status() {
        return status;
    }

    match error.kind() {
        ErrorKind::InvalidArgument | ErrorKind::AbiVersionMismatch => {
            sys::MLN_STATUS_INVALID_ARGUMENT
        }
        ErrorKind::InvalidState => sys::MLN_STATUS_INVALID_STATE,
        ErrorKind::WrongThread => sys::MLN_STATUS_WRONG_THREAD,
        ErrorKind::Unsupported => sys::MLN_STATUS_UNSUPPORTED,
        ErrorKind::NativeError | ErrorKind::UnknownStatus => sys::MLN_STATUS_NATIVE_ERROR,
        _ => sys::MLN_STATUS_NATIVE_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplibre_native_core::error::{Error, ErrorKind};

    #[test]
    fn status_result_copies_diagnostic_bytes() {
        let result = StatusResult::from_error(Error::new(
            ErrorKind::InvalidArgument,
            Some(sys::MLN_STATUS_INVALID_ARGUMENT),
            "invalid network status",
        ));

        assert_eq!(result.status, sys::MLN_STATUS_INVALID_ARGUMENT);
        assert_eq!(result.diagnostic(), "invalid network status");
    }

    #[test]
    fn status_result_maps_binding_owned_errors_to_non_ok_status() {
        let result = StatusResult::from_error(Error::invalid_argument("closed handle"));

        assert_eq!(result.status, sys::MLN_STATUS_INVALID_ARGUMENT);
        assert_eq!(result.diagnostic(), "closed handle");
    }
}

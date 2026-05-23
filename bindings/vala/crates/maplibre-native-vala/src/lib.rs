//! Rust adapter seed for the Vala/GLib binding.
//!
//! The final crate exports a GObject Introspection-friendly ABI implemented in
//! Rust. The initial scaffold keeps the first native calls in one place so the
//! generated GIR/VAPI layer can grow from real C ABI behavior.

pub mod native_pointer;
pub mod status;

use maplibre_native_sys as sys;
use status::StatusResult;

/// Returns the MapLibre Native C ABI version reported by the loaded library.
pub fn c_version() -> u32 {
    // SAFETY: `mln_c_version` takes no arguments and returns a plain value.
    unsafe { sys::mln_c_version() }
}

/// Returns the backend mask reported by the loaded library.
pub fn supported_render_backend_mask() -> u32 {
    // SAFETY: `mln_supported_render_backend_mask` takes no arguments and returns
    // a plain bit mask.
    unsafe { sys::mln_supported_render_backend_mask() }
}

/// Reads the process-global network status through the public C ABI.
pub fn network_status() -> maplibre_native_core::error::Result<u32> {
    let mut raw_status = 0;
    // SAFETY: The output pointer is valid for this call.
    maplibre_native_core::error::check(unsafe { sys::mln_network_status_get(&mut raw_status) })?;
    Ok(raw_status)
}

/// Sets the process-global network status through the public C ABI.
pub fn set_network_status(raw_status: u32) -> maplibre_native_core::error::Result<()> {
    // SAFETY: The C API validates the enum-domain value.
    maplibre_native_core::error::check(unsafe { sys::mln_network_status_set(raw_status) })
}

/// C-callable proof-slice entry point used by future GIR scanner fixtures.
#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_c_version() -> u32 {
    c_version()
}

/// C-callable proof-slice entry point used by future GIR scanner fixtures.
#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_supported_render_backend_mask() -> u32 {
    supported_render_backend_mask()
}

/// C-callable proof-slice entry point that reports status without allocating a
/// GLib error. The generated GObject ABI will wrap this in `GError`-returning
/// entry points.
#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_network_status_get() -> StatusResult {
    match network_status() {
        Ok(value) => StatusResult::ok(value),
        Err(error) => StatusResult::from_error(error),
    }
}

/// C-callable proof-slice entry point that reports status without allocating a
/// GLib error. The generated GObject ABI will wrap this in `GError`-returning
/// entry points.
#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_network_status_set(raw_status: u32) -> StatusResult {
    match set_network_status(raw_status) {
        Ok(()) => StatusResult::ok(0),
        Err(error) => StatusResult::from_error(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_version_is_readable() {
        assert_eq!(
            c_version(),
            maplibre_native_core::abi::EXPECTED_C_ABI_VERSION
        );
    }

    #[test]
    fn supported_backends_preserve_mask_bits() {
        assert_ne!(supported_render_backend_mask(), 0);
    }

    #[test]
    fn network_status_round_trips_through_native() {
        let original = network_status().expect("network status is readable");
        set_network_status(sys::MLN_NETWORK_STATUS_OFFLINE).expect("offline status is accepted");
        assert_eq!(
            network_status().expect("network status is readable"),
            sys::MLN_NETWORK_STATUS_OFFLINE
        );
        set_network_status(original).expect("original status is restored");
    }

    #[test]
    fn invalid_network_status_preserves_diagnostic() {
        let result = mln_vala_network_status_set(999_999);
        assert_ne!(result.status, 0);
        assert_ne!(result.diagnostic_len, 0);
    }
}

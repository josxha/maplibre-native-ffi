use napi::bindgen_prelude::Result;
use napi_derive::napi;

use crate::error;

#[napi(object)]
pub struct RenderBackends {
    pub raw_mask: u32,
    pub metal: bool,
    pub vulkan: bool,
}

#[napi(object)]
pub struct NetworkStatusValue {
    pub kind: String,
    pub raw: u32,
}

#[napi(js_name = "cVersion")]
pub fn c_version() -> u32 {
    // SAFETY: mln_c_version takes no arguments and returns the process-global C
    // ABI version for the linked native library.
    unsafe { maplibre_native_sys::mln_c_version() }
}

#[napi(js_name = "supportedRenderBackends")]
pub fn supported_render_backends() -> RenderBackends {
    // SAFETY: mln_supported_render_backend_mask takes no arguments and returns a
    // value mask. Unknown future bits are preserved in raw_mask.
    let raw_mask = unsafe { maplibre_native_sys::mln_supported_render_backend_mask() };
    RenderBackends {
        raw_mask,
        metal: raw_mask & maplibre_native_sys::MLN_RENDER_BACKEND_FLAG_METAL != 0,
        vulkan: raw_mask & maplibre_native_sys::MLN_RENDER_BACKEND_FLAG_VULKAN != 0,
    }
}

#[napi(js_name = "networkStatus")]
pub fn network_status() -> Result<NetworkStatusValue> {
    let mut raw_status = 0;
    // SAFETY: raw_status points to valid writable storage for one u32.
    maplibre_native_core::check(unsafe {
        maplibre_native_sys::mln_network_status_get(&mut raw_status)
    })
    .map_err(error::from_core)?;
    Ok(network_status_from_raw(raw_status))
}

#[napi(js_name = "setNetworkStatus")]
pub fn set_network_status(status: String) -> Result<()> {
    let raw_status = match status.as_str() {
        "online" => maplibre_native_sys::MLN_NETWORK_STATUS_ONLINE,
        "offline" => maplibre_native_sys::MLN_NETWORK_STATUS_OFFLINE,
        other => {
            return Err(error::invalid_argument(format!(
                "network status must be 'online' or 'offline', got '{other}'"
            )));
        }
    };

    // SAFETY: The raw enum value is passed by value. The C API validates the
    // enum domain and reports invalid values as MLN_STATUS_INVALID_ARGUMENT.
    maplibre_native_core::check(unsafe { maplibre_native_sys::mln_network_status_set(raw_status) })
        .map_err(error::from_core)
}

fn network_status_from_raw(raw: u32) -> NetworkStatusValue {
    let kind = match raw {
        maplibre_native_sys::MLN_NETWORK_STATUS_ONLINE => "online".to_owned(),
        maplibre_native_sys::MLN_NETWORK_STATUS_OFFLINE => "offline".to_owned(),
        _ => "unknown".to_owned(),
    };
    NetworkStatusValue { kind, raw }
}

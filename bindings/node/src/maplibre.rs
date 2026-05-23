use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::sync::{Arc, Mutex, OnceLock};

use napi::bindgen_prelude::Result;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
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

#[napi(object)]
pub struct LogRecord {
    pub severity: String,
    pub raw_severity: u32,
    pub event: String,
    pub raw_event: u32,
    pub code: i64,
    pub message: String,
}

static LOG_CALLBACK: OnceLock<Mutex<Option<Arc<ThreadsafeFunction<LogRecord>>>>> = OnceLock::new();

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

#[napi(js_name = "nativeSetLogCallback")]
pub fn native_set_log_callback(callback: ThreadsafeFunction<LogRecord>) -> Result<()> {
    *log_callback_slot()
        .lock()
        .expect("log callback mutex poisoned") = Some(Arc::new(callback));
    maplibre_native_core::check(unsafe {
        maplibre_native_sys::mln_log_set_callback(Some(log_trampoline), std::ptr::null_mut())
    })
    .map_err(error::from_core)
}

#[napi(js_name = "nativeClearLogCallback")]
pub fn native_clear_log_callback() -> Result<()> {
    maplibre_native_core::check(unsafe { maplibre_native_sys::mln_log_clear_callback() })
        .map_err(error::from_core)?;
    *log_callback_slot()
        .lock()
        .expect("log callback mutex poisoned") = None;
    Ok(())
}

#[napi(js_name = "nativeSetAsyncLogSeverityMask")]
pub fn native_set_async_log_severity_mask(mask: u32) -> Result<()> {
    maplibre_native_core::check(unsafe {
        maplibre_native_sys::mln_log_set_async_severity_mask(mask)
    })
    .map_err(error::from_core)
}

#[napi(js_name = "nativeDefaultAsyncLogSeverityMask")]
pub fn native_default_async_log_severity_mask() -> u32 {
    maplibre_native_sys::MLN_LOG_SEVERITY_MASK_DEFAULT
}

#[napi(js_name = "nativeLogSeverityMaskBit")]
pub fn native_log_severity_mask_bit(severity: String) -> Result<u32> {
    match severity.as_str() {
        "info" => Ok(maplibre_native_sys::MLN_LOG_SEVERITY_MASK_INFO),
        "warning" => Ok(maplibre_native_sys::MLN_LOG_SEVERITY_MASK_WARNING),
        "error" => Ok(maplibre_native_sys::MLN_LOG_SEVERITY_MASK_ERROR),
        other => Err(error::invalid_argument(format!(
            "log severity must be 'info', 'warning', or 'error', got '{other}'"
        ))),
    }
}

extern "C" fn log_trampoline(
    _user_data: *mut c_void,
    severity: u32,
    event: u32,
    code: i64,
    message: *const c_char,
) -> u32 {
    let callback = log_callback_slot()
        .lock()
        .expect("log callback mutex poisoned")
        .clone();
    if let Some(callback) = callback {
        callback.call(
            Ok(LogRecord {
                severity: log_severity_name(severity).to_owned(),
                raw_severity: severity,
                event: log_event_name(event).to_owned(),
                raw_event: event,
                code,
                message: copy_log_message(message),
            }),
            ThreadsafeFunctionCallMode::NonBlocking,
        );
    }
    0
}

fn log_callback_slot() -> &'static Mutex<Option<Arc<ThreadsafeFunction<LogRecord>>>> {
    LOG_CALLBACK.get_or_init(|| Mutex::new(None))
}

fn copy_log_message(message: *const c_char) -> String {
    if message.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned()
    }
}

fn log_severity_name(severity: u32) -> &'static str {
    match severity {
        maplibre_native_sys::MLN_LOG_SEVERITY_INFO => "info",
        maplibre_native_sys::MLN_LOG_SEVERITY_WARNING => "warning",
        maplibre_native_sys::MLN_LOG_SEVERITY_ERROR => "error",
        _ => "unknown",
    }
}

fn log_event_name(event: u32) -> &'static str {
    match event {
        maplibre_native_sys::MLN_LOG_EVENT_GENERAL => "general",
        maplibre_native_sys::MLN_LOG_EVENT_SETUP => "setup",
        maplibre_native_sys::MLN_LOG_EVENT_SHADER => "shader",
        maplibre_native_sys::MLN_LOG_EVENT_PARSE_STYLE => "parseStyle",
        maplibre_native_sys::MLN_LOG_EVENT_PARSE_TILE => "parseTile",
        maplibre_native_sys::MLN_LOG_EVENT_RENDER => "render",
        maplibre_native_sys::MLN_LOG_EVENT_STYLE => "style",
        maplibre_native_sys::MLN_LOG_EVENT_DATABASE => "database",
        maplibre_native_sys::MLN_LOG_EVENT_HTTP_REQUEST => "httpRequest",
        maplibre_native_sys::MLN_LOG_EVENT_SPRITE => "sprite",
        maplibre_native_sys::MLN_LOG_EVENT_IMAGE => "image",
        maplibre_native_sys::MLN_LOG_EVENT_OPENGL => "openGl",
        maplibre_native_sys::MLN_LOG_EVENT_JNI => "jni",
        maplibre_native_sys::MLN_LOG_EVENT_ANDROID => "android",
        maplibre_native_sys::MLN_LOG_EVENT_CRASH => "crash",
        maplibre_native_sys::MLN_LOG_EVENT_GLYPH => "glyph",
        maplibre_native_sys::MLN_LOG_EVENT_TIMING => "timing",
        _ => "unknown",
    }
}

fn network_status_from_raw(raw: u32) -> NetworkStatusValue {
    let kind = match raw {
        maplibre_native_sys::MLN_NETWORK_STATUS_ONLINE => "online".to_owned(),
        maplibre_native_sys::MLN_NETWORK_STATUS_OFFLINE => "offline".to_owned(),
        _ => "unknown".to_owned(),
    };
    NetworkStatusValue { kind, raw }
}

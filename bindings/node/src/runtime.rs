use maplibre_native_core::{self as core, handle::NativeHandleState};
use maplibre_native_sys as sys;
use napi::bindgen_prelude::{BigInt, Result};
use napi_derive::napi;

use crate::error;

#[napi(object)]
pub struct RuntimeOptions {
    pub asset_path: Option<String>,
    pub cache_path: Option<String>,
    pub maximum_cache_size: Option<BigInt>,
}

#[napi(js_name = "NativeRuntimeHandle")]
pub struct NativeRuntimeHandle {
    state: NativeHandleState<sys::mln_runtime>,
}

#[napi(js_name = "createNativeRuntimeHandle")]
pub fn create_native_runtime_handle(
    options: Option<RuntimeOptions>,
) -> Result<NativeRuntimeHandle> {
    let options = options.unwrap_or_default().into_core()?;
    let native_options =
        core::runtime::runtime_options_to_native(&options).map_err(error::from_core)?;
    let mut runtime = std::ptr::null_mut();

    core::check(unsafe { sys::mln_runtime_create(&native_options.to_raw(), &mut runtime) })
        .map_err(error::from_core)?;
    let state = unsafe { NativeHandleState::from_raw_ptr(runtime, "RuntimeHandle") }
        .map_err(error::from_core)?;
    Ok(NativeRuntimeHandle { state })
}

#[napi]
impl NativeRuntimeHandle {
    #[napi]
    pub fn close(&self) -> Result<()> {
        unsafe { self.state.close_status(sys::mln_runtime_destroy) }.map_err(error::from_core)
    }

    #[napi(getter)]
    pub fn closed(&self) -> bool {
        self.state.is_closed()
    }

    #[napi(js_name = "runOnce")]
    pub fn run_once(&self) -> Result<()> {
        core::check(unsafe { sys::mln_runtime_run_once(self.state.as_ptr()) })
            .map_err(error::from_core)
    }
}

impl Drop for NativeRuntimeHandle {
    fn drop(&mut self) {
        let _ = self.state.leak_for_report();
    }
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            asset_path: None,
            cache_path: None,
            maximum_cache_size: None,
        }
    }
}

impl RuntimeOptions {
    fn into_core(self) -> Result<core::RuntimeOptions> {
        let mut options = core::RuntimeOptions::new();
        if let Some(asset_path) = self.asset_path {
            options = options.with_asset_path(asset_path);
        }
        if let Some(cache_path) = self.cache_path {
            options = options.with_cache_path(cache_path);
        }
        if let Some(maximum_cache_size) = self.maximum_cache_size {
            options =
                options.with_maximum_cache_size(maximum_cache_size_to_u64(maximum_cache_size)?);
        }
        Ok(options)
    }
}

fn maximum_cache_size_to_u64(value: BigInt) -> Result<u64> {
    let (signed, value, lossless) = value.get_u64();
    if signed || !lossless {
        return Err(error::invalid_argument(
            "maximumCacheSize must be a non-negative 64-bit bigint",
        ));
    }
    Ok(value)
}

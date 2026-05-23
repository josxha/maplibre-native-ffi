use std::ffi::c_void;

use maplibre_native_core::{self as core, handle::NativeHandleState};
use maplibre_native_sys as sys;
use napi::bindgen_prelude::{BigInt, Result, Uint8Array};
use napi_derive::napi;

use crate::{error, map::NativeMapHandle};

#[napi(object)]
pub struct RenderTargetExtent {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

#[napi(object)]
pub struct MetalContextDescriptor {
    pub device_address: Option<BigInt>,
}

#[napi(object)]
pub struct MetalOwnedTextureDescriptor {
    pub extent: RenderTargetExtent,
    pub context: MetalContextDescriptor,
}

#[napi(object)]
pub struct TextureImageInfo {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub byte_length: i64,
}

#[napi(object)]
pub struct TextureReadback {
    pub info: TextureImageInfo,
    pub pixels: Uint8Array,
}

#[napi(js_name = "NativeRenderSessionHandle")]
pub struct NativeRenderSessionHandle {
    state: NativeHandleState<sys::mln_render_session>,
}

#[napi(js_name = "createMetalOwnedTextureRenderSession")]
pub fn create_metal_owned_texture_render_session(
    map: &NativeMapHandle,
    descriptor: MetalOwnedTextureDescriptor,
) -> Result<NativeRenderSessionHandle> {
    let mut raw_descriptor = unsafe { sys::mln_metal_owned_texture_descriptor_default() };
    raw_descriptor.extent = descriptor.extent.into_native();
    raw_descriptor.context.device = descriptor.context.device_ptr()?;
    let mut session = std::ptr::null_mut();
    core::check(unsafe {
        sys::mln_metal_owned_texture_attach(map.as_ptr(), &raw_descriptor, &mut session)
    })
    .map_err(error::from_core)?;
    let state = unsafe { NativeHandleState::from_raw_ptr(session, "RenderSessionHandle") }
        .map_err(error::from_core)?;
    Ok(NativeRenderSessionHandle { state })
}

#[napi]
impl NativeRenderSessionHandle {
    #[napi]
    pub fn close(&self) -> Result<()> {
        unsafe { self.state.close_status(sys::mln_render_session_destroy) }
            .map_err(error::from_core)
    }

    #[napi(getter)]
    pub fn closed(&self) -> bool {
        self.state.is_closed()
    }

    #[napi]
    pub fn resize(&self, width: u32, height: u32, scale_factor: f64) -> Result<()> {
        core::check(unsafe {
            sys::mln_render_session_resize(self.state.as_ptr(), width, height, scale_factor)
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "renderUpdate")]
    pub fn render_update(&self) -> Result<()> {
        core::check(unsafe { sys::mln_render_session_render_update(self.state.as_ptr()) })
            .map_err(error::from_core)
    }

    #[napi]
    pub fn detach(&self) -> Result<()> {
        core::check(unsafe { sys::mln_render_session_detach(self.state.as_ptr()) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "reduceMemoryUse")]
    pub fn reduce_memory_use(&self) -> Result<()> {
        core::check(unsafe { sys::mln_render_session_reduce_memory_use(self.state.as_ptr()) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "clearData")]
    pub fn clear_data(&self) -> Result<()> {
        core::check(unsafe { sys::mln_render_session_clear_data(self.state.as_ptr()) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "dumpDebugLogs")]
    pub fn dump_debug_logs(&self) -> Result<()> {
        core::check(unsafe { sys::mln_render_session_dump_debug_logs(self.state.as_ptr()) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "readPremultipliedRgba8")]
    pub fn read_premultiplied_rgba8(&self) -> Result<TextureReadback> {
        let mut info = unsafe { sys::mln_texture_image_info_default() };
        let first_status = unsafe {
            sys::mln_texture_read_premultiplied_rgba8(
                self.state.as_ptr(),
                std::ptr::null_mut(),
                0,
                &mut info,
            )
        };
        if first_status != sys::MLN_STATUS_INVALID_ARGUMENT {
            core::check(first_status).map_err(error::from_core)?;
        }
        let mut pixels = vec![0; info.byte_length];
        core::check(unsafe {
            sys::mln_texture_read_premultiplied_rgba8(
                self.state.as_ptr(),
                pixels.as_mut_ptr(),
                pixels.len(),
                &mut info,
            )
        })
        .map_err(error::from_core)?;
        Ok(TextureReadback {
            info: TextureImageInfo::from_native(info),
            pixels: Uint8Array::from(pixels),
        })
    }
}

impl RenderTargetExtent {
    fn into_native(self) -> sys::mln_render_target_extent {
        sys::mln_render_target_extent {
            size: std::mem::size_of::<sys::mln_render_target_extent>() as u32,
            width: self.width,
            height: self.height,
            scale_factor: self.scale_factor,
        }
    }
}

impl MetalContextDescriptor {
    fn device_ptr(&self) -> Result<*mut c_void> {
        self.device_address
            .as_ref()
            .map(|value| bigint_to_ptr(value, "deviceAddress"))
            .transpose()
            .map(|value| value.unwrap_or(std::ptr::null_mut()))
    }
}

impl TextureImageInfo {
    fn from_native(raw: sys::mln_texture_image_info) -> Self {
        Self {
            width: raw.width,
            height: raw.height,
            stride: raw.stride,
            byte_length: raw.byte_length as i64,
        }
    }
}

fn bigint_to_ptr(value: &BigInt, field_name: &str) -> Result<*mut c_void> {
    let (signed, value, lossless) = value.get_u64();
    if signed || !lossless {
        return Err(error::invalid_argument(format!(
            "{field_name} must be an unsigned pointer-sized bigint"
        )));
    }
    Ok(value as usize as *mut c_void)
}

impl Drop for NativeRenderSessionHandle {
    fn drop(&mut self) {
        let _ = self.state.leak_for_report();
    }
}

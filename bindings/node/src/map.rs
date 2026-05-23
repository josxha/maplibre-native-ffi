use std::ffi::CString;

use maplibre_native_core::{self as core, handle::NativeHandleState};
use maplibre_native_sys as sys;
use napi::bindgen_prelude::Result;
use napi_derive::napi;

use crate::{
    error,
    runtime::NativeRuntimeHandle,
    values::{LatLng, ScreenPoint},
};

#[napi(object)]
pub struct MapOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub scale_factor: Option<f64>,
    pub map_mode: Option<String>,
}

#[napi(object)]
pub struct CameraOptions {
    pub center: Option<LatLng>,
    pub zoom: Option<f64>,
    pub bearing: Option<f64>,
    pub pitch: Option<f64>,
}

#[napi(js_name = "NativeMapHandle")]
pub struct NativeMapHandle {
    state: NativeHandleState<sys::mln_map>,
}

#[napi(js_name = "createNativeMapHandle")]
pub fn create_native_map_handle(
    runtime: &NativeRuntimeHandle,
    options: Option<MapOptions>,
) -> Result<NativeMapHandle> {
    let options = options.unwrap_or_default().into_core()?;
    let native_options = core::options::map_options_to_native(&options);
    let mut map = std::ptr::null_mut();

    core::check(unsafe { sys::mln_map_create(runtime.as_ptr(), &native_options, &mut map) })
        .map_err(error::from_core)?;
    let state =
        unsafe { NativeHandleState::from_raw_ptr(map, "MapHandle") }.map_err(error::from_core)?;
    Ok(NativeMapHandle { state })
}

#[napi]
impl NativeMapHandle {
    #[napi]
    pub fn close(&self) -> Result<()> {
        unsafe { self.state.close_status(sys::mln_map_destroy) }.map_err(error::from_core)
    }

    #[napi(getter)]
    pub fn closed(&self) -> bool {
        self.state.is_closed()
    }

    #[napi(js_name = "requestRepaint")]
    pub fn request_repaint(&self) -> Result<()> {
        core::check(unsafe { sys::mln_map_request_repaint(self.state.as_ptr()) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "requestStillImage")]
    pub fn request_still_image(&self) -> Result<()> {
        core::check(unsafe { sys::mln_map_request_still_image(self.state.as_ptr()) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "isFullyLoaded")]
    pub fn is_fully_loaded(&self) -> Result<bool> {
        let mut loaded = false;
        core::check(unsafe { sys::mln_map_is_fully_loaded(self.state.as_ptr(), &mut loaded) })
            .map_err(error::from_core)?;
        Ok(loaded)
    }

    #[napi(js_name = "dumpDebugLogs")]
    pub fn dump_debug_logs(&self) -> Result<()> {
        core::check(unsafe { sys::mln_map_dump_debug_logs(self.state.as_ptr()) })
            .map_err(error::from_core)
    }

    #[napi(getter, js_name = "renderingStatsViewEnabled")]
    pub fn rendering_stats_view_enabled(&self) -> Result<bool> {
        let mut enabled = false;
        core::check(unsafe {
            sys::mln_map_get_rendering_stats_view_enabled(self.state.as_ptr(), &mut enabled)
        })
        .map_err(error::from_core)?;
        Ok(enabled)
    }

    #[napi(setter, js_name = "renderingStatsViewEnabled")]
    pub fn set_rendering_stats_view_enabled(&self, enabled: bool) -> Result<()> {
        core::check(unsafe {
            sys::mln_map_set_rendering_stats_view_enabled(self.state.as_ptr(), enabled)
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "getCamera")]
    pub fn get_camera(&self) -> Result<CameraOptions> {
        let mut raw = unsafe { sys::mln_camera_options_default() };
        core::check(unsafe { sys::mln_map_get_camera(self.state.as_ptr(), &mut raw) })
            .map_err(error::from_core)?;
        Ok(CameraOptions::from_core(
            core::camera::camera_options_from_native(raw),
        ))
    }

    #[napi(js_name = "jumpTo")]
    pub fn jump_to(&self, camera: CameraOptions) -> Result<()> {
        let camera = core::camera::camera_options_to_native(&camera.into_core());
        core::check(unsafe { sys::mln_map_jump_to(self.state.as_ptr(), &camera) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "pixelForLatLng")]
    pub fn pixel_for_lat_lng(&self, coordinate: LatLng) -> Result<ScreenPoint> {
        let mut raw_point = sys::mln_screen_point { x: 0.0, y: 0.0 };
        core::check(unsafe {
            sys::mln_map_pixel_for_lat_lng(
                self.state.as_ptr(),
                coordinate.into_native(),
                &mut raw_point,
            )
        })
        .map_err(error::from_core)?;
        Ok(ScreenPoint::from_native(raw_point))
    }

    #[napi(js_name = "latLngForPixel")]
    pub fn lat_lng_for_pixel(&self, point: ScreenPoint) -> Result<LatLng> {
        let mut raw_coordinate = sys::mln_lat_lng {
            latitude: 0.0,
            longitude: 0.0,
        };
        core::check(unsafe {
            sys::mln_map_lat_lng_for_pixel(
                self.state.as_ptr(),
                point.into_native(),
                &mut raw_coordinate,
            )
        })
        .map_err(error::from_core)?;
        Ok(LatLng::from_native(raw_coordinate))
    }

    #[napi(js_name = "getDebugOptionsRaw")]
    pub fn get_debug_options_raw(&self) -> Result<u32> {
        let mut options = 0;
        core::check(unsafe { sys::mln_map_get_debug_options(self.state.as_ptr(), &mut options) })
            .map_err(error::from_core)?;
        Ok(options)
    }

    #[napi(js_name = "setDebugOptionsRaw")]
    pub fn set_debug_options_raw(&self, options: u32) -> Result<()> {
        core::check(unsafe { sys::mln_map_set_debug_options(self.state.as_ptr(), options) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "setStyleJson")]
    pub fn set_style_json(&self, json: String) -> Result<()> {
        let json = c_string(json, "style JSON")?;
        core::check(unsafe { sys::mln_map_set_style_json(self.state.as_ptr(), json.as_ptr()) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "setStyleUrl")]
    pub fn set_style_url(&self, url: String) -> Result<()> {
        let url = c_string(url, "style URL")?;
        core::check(unsafe { sys::mln_map_set_style_url(self.state.as_ptr(), url.as_ptr()) })
            .map_err(error::from_core)
    }
}

impl Drop for NativeMapHandle {
    fn drop(&mut self) {
        let _ = self.state.leak_for_report();
    }
}

impl Default for MapOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            scale_factor: None,
            map_mode: None,
        }
    }
}

impl CameraOptions {
    fn into_core(self) -> core::CameraOptions {
        let mut camera = core::CameraOptions::new();
        if let Some(center) = self.center {
            camera = camera.with_center(center.into_core());
        }
        if let Some(zoom) = self.zoom {
            camera = camera.with_zoom(zoom);
        }
        if let Some(bearing) = self.bearing {
            camera = camera.with_bearing(bearing);
        }
        if let Some(pitch) = self.pitch {
            camera = camera.with_pitch(pitch);
        }
        camera
    }

    fn from_core(camera: core::CameraOptions) -> Self {
        Self {
            center: camera.center.map(LatLng::from_core),
            zoom: camera.zoom,
            bearing: camera.bearing,
            pitch: camera.pitch,
        }
    }
}

impl MapOptions {
    fn into_core(self) -> Result<core::MapOptions> {
        let defaults = core::MapOptions::default();
        let mut options = core::MapOptions::new(
            self.width.unwrap_or(defaults.width),
            self.height.unwrap_or(defaults.height),
            self.scale_factor.unwrap_or(defaults.scale_factor),
        );
        if let Some(map_mode) = self.map_mode {
            options = options.with_mode(map_mode_from_string(&map_mode)?);
        }
        Ok(options)
    }
}

fn map_mode_from_string(map_mode: &str) -> Result<core::MapMode> {
    match map_mode {
        "continuous" => Ok(core::MapMode::Continuous),
        "static" => Ok(core::MapMode::Static),
        "tile" => Ok(core::MapMode::Tile),
        other => Err(error::invalid_argument(format!(
            "mapMode must be 'continuous', 'static', or 'tile', got '{other}'"
        ))),
    }
}

#[napi(js_name = "nativeMapDebugOptionMaskBit")]
pub fn native_map_debug_option_mask_bit(option: String) -> Result<u32> {
    match option.as_str() {
        "tileBorders" => Ok(sys::MLN_MAP_DEBUG_TILE_BORDERS),
        "parseStatus" => Ok(sys::MLN_MAP_DEBUG_PARSE_STATUS),
        "timestamps" => Ok(sys::MLN_MAP_DEBUG_TIMESTAMPS),
        "collision" => Ok(sys::MLN_MAP_DEBUG_COLLISION),
        "overdraw" => Ok(sys::MLN_MAP_DEBUG_OVERDRAW),
        "stencilClip" => Ok(sys::MLN_MAP_DEBUG_STENCIL_CLIP),
        "depthBuffer" => Ok(sys::MLN_MAP_DEBUG_DEPTH_BUFFER),
        other => Err(error::invalid_argument(format!(
            "debug option must be a known MapDebugOption, got '{other}'"
        ))),
    }
}

fn c_string(value: String, field_name: &str) -> Result<CString> {
    CString::new(value)
        .map_err(|_| error::invalid_argument(format!("{field_name} contains an embedded NUL byte")))
}

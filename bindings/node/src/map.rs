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

#[napi(object)]
pub struct StyleSourceInfo {
    pub source_type: String,
    pub raw_type: u32,
    pub id_size: i64,
    pub is_volatile: bool,
    pub has_attribution: bool,
    pub attribution_size: i64,
    pub attribution: Option<String>,
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
    pub(crate) fn as_ptr(&self) -> *mut sys::mln_map {
        self.state.as_ptr()
    }

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

    #[napi(js_name = "moveBy")]
    pub fn move_by(&self, delta_x: f64, delta_y: f64) -> Result<()> {
        core::check(unsafe { sys::mln_map_move_by(self.state.as_ptr(), delta_x, delta_y) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "scaleBy")]
    pub fn scale_by(&self, scale: f64, anchor: Option<ScreenPoint>) -> Result<()> {
        let anchor = anchor.map(ScreenPoint::into_native);
        let anchor_ptr = anchor.as_ref().map_or(std::ptr::null(), |anchor| {
            anchor as *const sys::mln_screen_point
        });
        core::check(unsafe { sys::mln_map_scale_by(self.state.as_ptr(), scale, anchor_ptr) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "rotateBy")]
    pub fn rotate_by(&self, first: ScreenPoint, second: ScreenPoint) -> Result<()> {
        core::check(unsafe {
            sys::mln_map_rotate_by(
                self.state.as_ptr(),
                first.into_native(),
                second.into_native(),
            )
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "pitchBy")]
    pub fn pitch_by(&self, pitch: f64) -> Result<()> {
        core::check(unsafe { sys::mln_map_pitch_by(self.state.as_ptr(), pitch) })
            .map_err(error::from_core)
    }

    #[napi(js_name = "cancelTransitions")]
    pub fn cancel_transitions(&self) -> Result<()> {
        core::check(unsafe { sys::mln_map_cancel_transitions(self.state.as_ptr()) })
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

    #[napi(js_name = "addStyleSourceJson")]
    pub fn add_style_source_json(&self, source_id: String, source_json: String) -> Result<()> {
        let source_id = core::string::string_view(&source_id);
        let source_json = parse_json_value(source_json)?;
        let native_source_json =
            core::json::json_value_try_to_native(&source_json).map_err(error::from_core)?;
        core::check(unsafe {
            sys::mln_map_add_style_source_json(
                self.state.as_ptr(),
                source_id.raw(),
                native_source_json.as_ptr(),
            )
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "styleSourceExists")]
    pub fn style_source_exists(&self, source_id: String) -> Result<bool> {
        let source_id = core::string::string_view(&source_id);
        let mut exists = false;
        core::check(unsafe {
            sys::mln_map_style_source_exists(self.state.as_ptr(), source_id.raw(), &mut exists)
        })
        .map_err(error::from_core)?;
        Ok(exists)
    }

    #[napi(js_name = "removeStyleSource")]
    pub fn remove_style_source(&self, source_id: String) -> Result<bool> {
        let source_id = core::string::string_view(&source_id);
        let mut removed = false;
        core::check(unsafe {
            sys::mln_map_remove_style_source(self.state.as_ptr(), source_id.raw(), &mut removed)
        })
        .map_err(error::from_core)?;
        Ok(removed)
    }

    #[napi(js_name = "listStyleSourceIds")]
    pub fn list_style_source_ids(&self) -> Result<Vec<String>> {
        let mut list = std::ptr::null_mut();
        core::check(unsafe { sys::mln_map_list_style_source_ids(self.state.as_ptr(), &mut list) })
            .map_err(error::from_core)?;
        copy_style_id_list(list).map_err(error::from_core)
    }

    #[napi(js_name = "getStyleSourceType")]
    pub fn get_style_source_type(&self, source_id: String) -> Result<Option<String>> {
        let source_id = core::string::string_view(&source_id);
        let mut raw_type = 0;
        let mut found = false;
        core::check(unsafe {
            sys::mln_map_get_style_source_type(
                self.state.as_ptr(),
                source_id.raw(),
                &mut raw_type,
                &mut found,
            )
        })
        .map_err(error::from_core)?;
        Ok(found.then(|| style_source_type_name(raw_type).to_owned()))
    }

    #[napi(js_name = "getStyleSourceInfo")]
    pub fn get_style_source_info(&self, source_id: String) -> Result<Option<StyleSourceInfo>> {
        let source_id_view = core::string::string_view(&source_id);
        let mut raw_info = sys::mln_style_source_info {
            size: std::mem::size_of::<sys::mln_style_source_info>() as u32,
            type_: 0,
            id_size: 0,
            is_volatile: false,
            has_attribution: false,
            attribution_size: 0,
        };
        let mut found = false;
        core::check(unsafe {
            sys::mln_map_get_style_source_info(
                self.state.as_ptr(),
                source_id_view.raw(),
                &mut raw_info,
                &mut found,
            )
        })
        .map_err(error::from_core)?;
        if !found {
            return Ok(None);
        }
        let attribution = copy_style_source_attribution(
            self.state.as_ptr(),
            source_id_view.raw(),
            raw_info.attribution_size,
        )
        .map_err(error::from_core)?;
        Ok(Some(StyleSourceInfo {
            source_type: style_source_type_name(raw_info.type_).to_owned(),
            raw_type: raw_info.type_,
            id_size: raw_info.id_size as i64,
            is_volatile: raw_info.is_volatile,
            has_attribution: raw_info.has_attribution,
            attribution_size: raw_info.attribution_size as i64,
            attribution,
        }))
    }

    #[napi(js_name = "addStyleLayerJson")]
    pub fn add_style_layer_json(
        &self,
        layer_json: String,
        before_layer_id: Option<String>,
    ) -> Result<()> {
        let layer_json = parse_json_value(layer_json)?;
        let native_layer_json =
            core::json::json_value_try_to_native(&layer_json).map_err(error::from_core)?;
        let before_layer_id = before_layer_id.unwrap_or_default();
        let before_layer_id = core::string::string_view(&before_layer_id);
        core::check(unsafe {
            sys::mln_map_add_style_layer_json(
                self.state.as_ptr(),
                native_layer_json.as_ptr(),
                before_layer_id.raw(),
            )
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "styleLayerExists")]
    pub fn style_layer_exists(&self, layer_id: String) -> Result<bool> {
        let layer_id = core::string::string_view(&layer_id);
        let mut exists = false;
        core::check(unsafe {
            sys::mln_map_style_layer_exists(self.state.as_ptr(), layer_id.raw(), &mut exists)
        })
        .map_err(error::from_core)?;
        Ok(exists)
    }

    #[napi(js_name = "removeStyleLayer")]
    pub fn remove_style_layer(&self, layer_id: String) -> Result<bool> {
        let layer_id = core::string::string_view(&layer_id);
        let mut removed = false;
        core::check(unsafe {
            sys::mln_map_remove_style_layer(self.state.as_ptr(), layer_id.raw(), &mut removed)
        })
        .map_err(error::from_core)?;
        Ok(removed)
    }

    #[napi(js_name = "listStyleLayerIds")]
    pub fn list_style_layer_ids(&self) -> Result<Vec<String>> {
        let mut list = std::ptr::null_mut();
        core::check(unsafe { sys::mln_map_list_style_layer_ids(self.state.as_ptr(), &mut list) })
            .map_err(error::from_core)?;
        copy_style_id_list(list).map_err(error::from_core)
    }

    #[napi(js_name = "getStyleLayerType")]
    pub fn get_style_layer_type(&self, layer_id: String) -> Result<Option<String>> {
        let layer_id = core::string::string_view(&layer_id);
        let mut raw_type = sys::mln_string_view {
            data: std::ptr::null(),
            size: 0,
        };
        let mut found = false;
        core::check(unsafe {
            sys::mln_map_get_style_layer_type(
                self.state.as_ptr(),
                layer_id.raw(),
                &mut raw_type,
                &mut found,
            )
        })
        .map_err(error::from_core)?;
        if found {
            Ok(Some(
                unsafe { core::string::copy_string_view(raw_type) }.map_err(error::from_core)?,
            ))
        } else {
            Ok(None)
        }
    }

    #[napi(js_name = "getStyleLayerJson")]
    pub fn get_style_layer_json(&self, layer_id: String) -> Result<Option<String>> {
        let layer_id = core::string::string_view(&layer_id);
        let mut snapshot = std::ptr::null_mut();
        let mut found = false;
        core::check(unsafe {
            sys::mln_map_get_style_layer_json(
                self.state.as_ptr(),
                layer_id.raw(),
                &mut snapshot,
                &mut found,
            )
        })
        .map_err(error::from_core)?;
        if !found {
            return Ok(None);
        }
        json_snapshot_to_string(snapshot)
    }

    #[napi(js_name = "moveStyleLayer")]
    pub fn move_style_layer(
        &self,
        layer_id: String,
        before_layer_id: Option<String>,
    ) -> Result<()> {
        let layer_id = core::string::string_view(&layer_id);
        let before_layer_id = before_layer_id.unwrap_or_default();
        let before_layer_id = core::string::string_view(&before_layer_id);
        core::check(unsafe {
            sys::mln_map_move_style_layer(
                self.state.as_ptr(),
                layer_id.raw(),
                before_layer_id.raw(),
            )
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "setLayerPropertyJson")]
    pub fn set_layer_property_json(
        &self,
        layer_id: String,
        property_name: String,
        value_json: String,
    ) -> Result<()> {
        let layer_id = core::string::string_view(&layer_id);
        let property_name = core::string::string_view(&property_name);
        let value = parse_json_value(value_json)?;
        let native_value =
            core::json::json_value_try_to_native(&value).map_err(error::from_core)?;
        core::check(unsafe {
            sys::mln_map_set_layer_property(
                self.state.as_ptr(),
                layer_id.raw(),
                property_name.raw(),
                native_value.as_ptr(),
            )
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "getLayerPropertyJson")]
    pub fn get_layer_property_json(
        &self,
        layer_id: String,
        property_name: String,
    ) -> Result<Option<String>> {
        let layer_id = core::string::string_view(&layer_id);
        let property_name = core::string::string_view(&property_name);
        let mut snapshot = std::ptr::null_mut();
        core::check(unsafe {
            sys::mln_map_get_layer_property(
                self.state.as_ptr(),
                layer_id.raw(),
                property_name.raw(),
                &mut snapshot,
            )
        })
        .map_err(error::from_core)?;
        json_snapshot_to_string(snapshot)
    }

    #[napi(js_name = "setLayerFilterJson")]
    pub fn set_layer_filter_json(
        &self,
        layer_id: String,
        filter_json: Option<String>,
    ) -> Result<()> {
        let layer_id = core::string::string_view(&layer_id);
        let filter = filter_json.map(parse_json_value).transpose()?;
        let native_filter = filter
            .as_ref()
            .map(core::json::json_value_try_to_native)
            .transpose()
            .map_err(error::from_core)?;
        let filter_ptr = native_filter
            .as_ref()
            .map_or(std::ptr::null(), |filter| filter.as_ptr());
        core::check(unsafe {
            sys::mln_map_set_layer_filter(self.state.as_ptr(), layer_id.raw(), filter_ptr)
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "getLayerFilterJson")]
    pub fn get_layer_filter_json(&self, layer_id: String) -> Result<Option<String>> {
        let layer_id = core::string::string_view(&layer_id);
        let mut snapshot = std::ptr::null_mut();
        core::check(unsafe {
            sys::mln_map_get_layer_filter(self.state.as_ptr(), layer_id.raw(), &mut snapshot)
        })
        .map_err(error::from_core)?;
        json_snapshot_to_string(snapshot)
    }

    #[napi(js_name = "setStyleLightJson")]
    pub fn set_style_light_json(&self, light_json: String) -> Result<()> {
        let light = parse_json_value(light_json)?;
        let native_light =
            core::json::json_value_try_to_native(&light).map_err(error::from_core)?;
        core::check(unsafe {
            sys::mln_map_set_style_light_json(self.state.as_ptr(), native_light.as_ptr())
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "setStyleLightPropertyJson")]
    pub fn set_style_light_property_json(
        &self,
        property_name: String,
        value_json: String,
    ) -> Result<()> {
        let property_name = core::string::string_view(&property_name);
        let value = parse_json_value(value_json)?;
        let native_value =
            core::json::json_value_try_to_native(&value).map_err(error::from_core)?;
        core::check(unsafe {
            sys::mln_map_set_style_light_property(
                self.state.as_ptr(),
                property_name.raw(),
                native_value.as_ptr(),
            )
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "getStyleLightPropertyJson")]
    pub fn get_style_light_property_json(&self, property_name: String) -> Result<Option<String>> {
        let property_name = core::string::string_view(&property_name);
        let mut snapshot = std::ptr::null_mut();
        core::check(unsafe {
            sys::mln_map_get_style_light_property(
                self.state.as_ptr(),
                property_name.raw(),
                &mut snapshot,
            )
        })
        .map_err(error::from_core)?;
        json_snapshot_to_string(snapshot)
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
    pub(crate) fn into_core(self) -> core::CameraOptions {
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

    pub(crate) fn from_core(camera: core::CameraOptions) -> Self {
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

fn json_snapshot_to_string(snapshot: *mut sys::mln_json_snapshot) -> Result<Option<String>> {
    let value = unsafe { core::json::copy_json_snapshot(std::ptr::NonNull::new(snapshot)) }
        .map_err(error::from_core)?;
    value.map(json_value_to_string).transpose()
}

fn copy_style_source_attribution(
    map: *mut sys::mln_map,
    source_id: sys::mln_string_view,
    attribution_size: usize,
) -> core::Result<Option<String>> {
    if attribution_size == 0 {
        return Ok(None);
    }
    let mut buffer = vec![0; attribution_size];
    let mut copied_size = 0;
    let mut found = false;
    core::check(unsafe {
        sys::mln_map_copy_style_source_attribution(
            map,
            source_id,
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut copied_size,
            &mut found,
        )
    })?;
    if !found || copied_size == 0 {
        return Ok(None);
    }
    buffer.truncate(copied_size);
    String::from_utf8(buffer).map(Some).map_err(|error| {
        core::Error::invalid_argument(format!("style source attribution was not UTF-8: {error}"))
    })
}

fn copy_style_id_list(list: *mut sys::mln_style_id_list) -> core::Result<Vec<String>> {
    struct StyleIdListGuard(*mut sys::mln_style_id_list);

    impl Drop for StyleIdListGuard {
        fn drop(&mut self) {
            unsafe { sys::mln_style_id_list_destroy(self.0) };
        }
    }

    let list = StyleIdListGuard(list);
    let mut count = 0;
    core::check(unsafe { sys::mln_style_id_list_count(list.0, &mut count) })?;
    let mut ids = Vec::with_capacity(count);
    for index in 0..count {
        let mut raw_id = sys::mln_string_view {
            data: std::ptr::null(),
            size: 0,
        };
        core::check(unsafe { sys::mln_style_id_list_get(list.0, index, &mut raw_id) })?;
        ids.push(unsafe { core::string::copy_string_view(raw_id) }?);
    }
    Ok(ids)
}

fn style_source_type_name(raw_type: u32) -> &'static str {
    match raw_type {
        sys::MLN_STYLE_SOURCE_TYPE_VECTOR => "vector",
        sys::MLN_STYLE_SOURCE_TYPE_RASTER => "raster",
        sys::MLN_STYLE_SOURCE_TYPE_RASTER_DEM => "raster-dem",
        sys::MLN_STYLE_SOURCE_TYPE_GEOJSON => "geojson",
        sys::MLN_STYLE_SOURCE_TYPE_IMAGE => "image",
        sys::MLN_STYLE_SOURCE_TYPE_VIDEO => "video",
        sys::MLN_STYLE_SOURCE_TYPE_ANNOTATIONS => "annotations",
        sys::MLN_STYLE_SOURCE_TYPE_CUSTOM_VECTOR => "custom-vector",
        _ => "unknown",
    }
}

fn parse_json_value(value: String) -> Result<core::JsonValue> {
    let value: serde_json::Value = serde_json::from_str(&value).map_err(|parse_error| {
        error::invalid_argument(format!("JSON input is invalid: {parse_error}"))
    })?;
    json_value_from_serde(value)
}

fn json_value_to_string(value: core::JsonValue) -> Result<String> {
    serde_json::to_string(&json_value_to_serde(value)).map_err(|serialize_error| {
        error::invalid_argument(format!(
            "JSON value could not be serialized: {serialize_error}"
        ))
    })
}

fn json_value_to_serde(value: core::JsonValue) -> serde_json::Value {
    match value {
        core::JsonValue::Null => serde_json::Value::Null,
        core::JsonValue::Bool(value) => serde_json::Value::Bool(value),
        core::JsonValue::UInt(value) => serde_json::Value::Number(value.into()),
        core::JsonValue::Int(value) => serde_json::Value::Number(value.into()),
        core::JsonValue::Double(value) => serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        core::JsonValue::String(value) => serde_json::Value::String(value),
        core::JsonValue::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(json_value_to_serde).collect())
        }
        core::JsonValue::Object(members) => serde_json::Value::Object(
            members
                .into_iter()
                .map(|member| (member.key, json_value_to_serde(member.value)))
                .collect(),
        ),
        _ => serde_json::Value::Null,
    }
}

fn json_value_from_serde(value: serde_json::Value) -> Result<core::JsonValue> {
    match value {
        serde_json::Value::Null => Ok(core::JsonValue::Null),
        serde_json::Value::Bool(value) => Ok(core::JsonValue::Bool(value)),
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_u64() {
                Ok(core::JsonValue::UInt(value))
            } else if let Some(value) = value.as_i64() {
                Ok(core::JsonValue::Int(value))
            } else if let Some(value) = value.as_f64() {
                Ok(core::JsonValue::Double(value))
            } else {
                Err(error::invalid_argument(
                    "JSON number could not be represented",
                ))
            }
        }
        serde_json::Value::String(value) => Ok(core::JsonValue::String(value)),
        serde_json::Value::Array(values) => values
            .into_iter()
            .map(json_value_from_serde)
            .collect::<Result<Vec<_>>>()
            .map(core::JsonValue::Array),
        serde_json::Value::Object(members) => members
            .into_iter()
            .map(|(key, value)| Ok(core::JsonMember::new(key, json_value_from_serde(value)?)))
            .collect::<Result<Vec<_>>>()
            .map(core::JsonValue::Object),
    }
}

fn c_string(value: String, field_name: &str) -> Result<CString> {
    CString::new(value)
        .map_err(|_| error::invalid_argument(format!("{field_name} contains an embedded NUL byte")))
}

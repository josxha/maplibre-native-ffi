use std::ffi::c_void;

use maplibre_native_core::{self as core, handle::NativeHandleState};
use maplibre_native_sys as sys;
use napi::bindgen_prelude::{BigInt, Result, Uint8Array};
use napi_derive::napi;

use crate::{
    error,
    map::{NativeMapHandle, json_value_to_serde, json_value_to_string, parse_json_value},
    values::ScreenPoint,
};

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
pub struct MetalBorrowedTextureDescriptor {
    pub extent: RenderTargetExtent,
    pub texture_address: BigInt,
}

#[napi(object)]
pub struct MetalSurfaceDescriptor {
    pub extent: RenderTargetExtent,
    pub context: MetalContextDescriptor,
    pub layer_address: BigInt,
}

#[napi(object)]
pub struct VulkanContextDescriptor {
    pub instance_address: BigInt,
    pub physical_device_address: BigInt,
    pub device_address: BigInt,
    pub graphics_queue_address: BigInt,
    pub graphics_queue_family_index: u32,
}

#[napi(object)]
pub struct VulkanOwnedTextureDescriptor {
    pub extent: RenderTargetExtent,
    pub context: VulkanContextDescriptor,
}

#[napi(object)]
pub struct VulkanBorrowedTextureDescriptor {
    pub extent: RenderTargetExtent,
    pub context: VulkanContextDescriptor,
    pub image_address: BigInt,
    pub image_view_address: BigInt,
    pub format: u32,
    pub initial_layout: u32,
    pub final_layout: u32,
}

#[napi(object)]
pub struct VulkanSurfaceDescriptor {
    pub extent: RenderTargetExtent,
    pub context: VulkanContextDescriptor,
    pub surface_address: BigInt,
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

#[napi(object)]
pub struct FeatureStateSelectorInput {
    pub source_id: String,
    pub source_layer_id: Option<String>,
    pub feature_id: Option<String>,
    pub state_key: Option<String>,
}

#[napi(object)]
pub struct ScreenBoxInput {
    pub min: ScreenPoint,
    pub max: ScreenPoint,
}

#[napi(object)]
pub struct RenderedQueryGeometryInput {
    pub kind: String,
    pub point: Option<ScreenPoint>,
    pub box_: Option<ScreenBoxInput>,
    pub points: Option<Vec<ScreenPoint>>,
}

#[napi(object)]
pub struct RenderedFeatureQueryOptionsInput {
    pub layer_ids: Option<Vec<String>>,
    pub filter: Option<String>,
}

#[napi(object)]
pub struct SourceFeatureQueryOptionsInput {
    pub source_layer_ids: Option<Vec<String>>,
    pub filter: Option<String>,
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
    attach_render_session(map, |out_session| unsafe {
        sys::mln_metal_owned_texture_attach(map.as_ptr(), &raw_descriptor, out_session)
    })
}

#[napi(js_name = "createMetalBorrowedTextureRenderSession")]
pub fn create_metal_borrowed_texture_render_session(
    map: &NativeMapHandle,
    descriptor: MetalBorrowedTextureDescriptor,
) -> Result<NativeRenderSessionHandle> {
    let mut raw_descriptor = unsafe { sys::mln_metal_borrowed_texture_descriptor_default() };
    raw_descriptor.extent = descriptor.extent.into_native();
    raw_descriptor.texture = bigint_to_ptr(&descriptor.texture_address, "textureAddress")?;
    attach_render_session(map, |out_session| unsafe {
        sys::mln_metal_borrowed_texture_attach(map.as_ptr(), &raw_descriptor, out_session)
    })
}

#[napi(js_name = "createMetalSurfaceRenderSession")]
pub fn create_metal_surface_render_session(
    map: &NativeMapHandle,
    descriptor: MetalSurfaceDescriptor,
) -> Result<NativeRenderSessionHandle> {
    let mut raw_descriptor = unsafe { sys::mln_metal_surface_descriptor_default() };
    raw_descriptor.extent = descriptor.extent.into_native();
    raw_descriptor.context.device = descriptor.context.device_ptr()?;
    raw_descriptor.layer = bigint_to_ptr(&descriptor.layer_address, "layerAddress")?;
    attach_render_session(map, |out_session| unsafe {
        sys::mln_metal_surface_attach(map.as_ptr(), &raw_descriptor, out_session)
    })
}

#[napi(js_name = "createVulkanOwnedTextureRenderSession")]
pub fn create_vulkan_owned_texture_render_session(
    map: &NativeMapHandle,
    descriptor: VulkanOwnedTextureDescriptor,
) -> Result<NativeRenderSessionHandle> {
    let mut raw_descriptor = unsafe { sys::mln_vulkan_owned_texture_descriptor_default() };
    raw_descriptor.extent = descriptor.extent.into_native();
    raw_descriptor.context = descriptor.context.into_native()?;
    attach_render_session(map, |out_session| unsafe {
        sys::mln_vulkan_owned_texture_attach(map.as_ptr(), &raw_descriptor, out_session)
    })
}

#[napi(js_name = "createVulkanBorrowedTextureRenderSession")]
pub fn create_vulkan_borrowed_texture_render_session(
    map: &NativeMapHandle,
    descriptor: VulkanBorrowedTextureDescriptor,
) -> Result<NativeRenderSessionHandle> {
    let mut raw_descriptor = unsafe { sys::mln_vulkan_borrowed_texture_descriptor_default() };
    raw_descriptor.extent = descriptor.extent.into_native();
    raw_descriptor.context = descriptor.context.into_native()?;
    raw_descriptor.image = bigint_to_ptr(&descriptor.image_address, "imageAddress")?;
    raw_descriptor.image_view = bigint_to_ptr(&descriptor.image_view_address, "imageViewAddress")?;
    raw_descriptor.format = descriptor.format;
    raw_descriptor.initial_layout = descriptor.initial_layout;
    raw_descriptor.final_layout = descriptor.final_layout;
    attach_render_session(map, |out_session| unsafe {
        sys::mln_vulkan_borrowed_texture_attach(map.as_ptr(), &raw_descriptor, out_session)
    })
}

#[napi(js_name = "createVulkanSurfaceRenderSession")]
pub fn create_vulkan_surface_render_session(
    map: &NativeMapHandle,
    descriptor: VulkanSurfaceDescriptor,
) -> Result<NativeRenderSessionHandle> {
    let mut raw_descriptor = unsafe { sys::mln_vulkan_surface_descriptor_default() };
    raw_descriptor.extent = descriptor.extent.into_native();
    raw_descriptor.context = descriptor.context.into_native()?;
    raw_descriptor.surface = bigint_to_ptr(&descriptor.surface_address, "surfaceAddress")?;
    attach_render_session(map, |out_session| unsafe {
        sys::mln_vulkan_surface_attach(map.as_ptr(), &raw_descriptor, out_session)
    })
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

    #[napi(js_name = "setFeatureState")]
    pub fn set_feature_state(
        &self,
        selector: FeatureStateSelectorInput,
        state: String,
    ) -> Result<()> {
        let selector = selector.into_core()?;
        let native_selector = core::query::feature_state_selector_to_native(&selector);
        let state = parse_json_value(state)?;
        let native_state =
            core::json::json_value_try_to_native(&state).map_err(error::from_core)?;
        core::check(unsafe {
            sys::mln_render_session_set_feature_state(
                self.state.as_ptr(),
                native_selector.as_ptr(),
                native_state.as_ref(),
            )
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "getFeatureState")]
    pub fn get_feature_state(&self, selector: FeatureStateSelectorInput) -> Result<String> {
        let selector = selector.into_core()?;
        let native_selector = core::query::feature_state_selector_to_native(&selector);
        let mut snapshot = std::ptr::null_mut();
        core::check(unsafe {
            sys::mln_render_session_get_feature_state(
                self.state.as_ptr(),
                native_selector.as_ptr(),
                &mut snapshot,
            )
        })
        .map_err(error::from_core)?;
        let snapshot = std::ptr::NonNull::new(snapshot)
            .ok_or_else(|| error::invalid_argument("feature state snapshot result was null"))?;
        let value = unsafe { core::json::copy_json_snapshot(Some(snapshot)) }
            .map_err(error::from_core)?
            .unwrap_or(core::JsonValue::Null);
        json_value_to_string(value)
    }

    #[napi(js_name = "removeFeatureState")]
    pub fn remove_feature_state(&self, selector: FeatureStateSelectorInput) -> Result<()> {
        let selector = selector.into_core()?;
        let native_selector = core::query::feature_state_selector_to_native(&selector);
        core::check(unsafe {
            sys::mln_render_session_remove_feature_state(
                self.state.as_ptr(),
                native_selector.as_ptr(),
            )
        })
        .map_err(error::from_core)
    }

    #[napi(js_name = "queryRenderedFeatures")]
    pub fn query_rendered_features(
        &self,
        geometry: RenderedQueryGeometryInput,
        options: Option<RenderedFeatureQueryOptionsInput>,
    ) -> Result<String> {
        let geometry = geometry.into_core()?;
        let native_geometry = core::query::rendered_query_geometry_to_native(&geometry);
        let options = options.unwrap_or_default().into_core()?;
        let native_options = core::query::rendered_feature_query_options_to_native(&options)
            .map_err(error::from_core)?;
        let mut result = std::ptr::null_mut();
        core::check(unsafe {
            sys::mln_render_session_query_rendered_features(
                self.state.as_ptr(),
                native_geometry.as_ptr(),
                native_options.as_ptr(),
                &mut result,
            )
        })
        .map_err(error::from_core)?;
        queried_features_to_string(result)
    }

    #[napi(js_name = "querySourceFeatures")]
    pub fn query_source_features(
        &self,
        source_id: String,
        options: Option<SourceFeatureQueryOptionsInput>,
    ) -> Result<String> {
        let source_id = core::string::string_view(&source_id);
        let options = options.unwrap_or_default().into_core()?;
        let native_options = core::query::source_feature_query_options_to_native(&options)
            .map_err(error::from_core)?;
        let mut result = std::ptr::null_mut();
        core::check(unsafe {
            sys::mln_render_session_query_source_features(
                self.state.as_ptr(),
                source_id.raw(),
                native_options.as_ptr(),
                &mut result,
            )
        })
        .map_err(error::from_core)?;
        queried_features_to_string(result)
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

impl Default for RenderedFeatureQueryOptionsInput {
    fn default() -> Self {
        Self {
            layer_ids: None,
            filter: None,
        }
    }
}

impl Default for SourceFeatureQueryOptionsInput {
    fn default() -> Self {
        Self {
            source_layer_ids: None,
            filter: None,
        }
    }
}

impl RenderedQueryGeometryInput {
    fn into_core(self) -> Result<core::query::RenderedQueryGeometry> {
        match self.kind.as_str() {
            "point" => Ok(core::query::RenderedQueryGeometry::point(
                self.point
                    .ok_or_else(|| error::invalid_argument("point query geometry requires point"))?
                    .into_core(),
            )),
            "box" => {
                let box_ = self
                    .box_
                    .ok_or_else(|| error::invalid_argument("box query geometry requires box"))?;
                Ok(core::query::RenderedQueryGeometry::box_(
                    core::values::ScreenBox {
                        min: box_.min.into_core(),
                        max: box_.max.into_core(),
                    },
                ))
            }
            "lineString" => Ok(core::query::RenderedQueryGeometry::line_string(
                self.points
                    .ok_or_else(|| {
                        error::invalid_argument("lineString query geometry requires points")
                    })?
                    .into_iter()
                    .map(ScreenPoint::into_core)
                    .collect(),
            )),
            other => Err(error::invalid_argument(format!(
                "query geometry kind must be 'point', 'box', or 'lineString', got '{other}'"
            ))),
        }
    }
}

impl RenderedFeatureQueryOptionsInput {
    fn into_core(self) -> Result<core::query::RenderedFeatureQueryOptions> {
        let mut options = core::query::RenderedFeatureQueryOptions::new();
        if let Some(layer_ids) = self.layer_ids {
            options = options.with_layer_ids(layer_ids);
        }
        if let Some(filter) = self.filter {
            options = options.with_filter(parse_json_value(filter)?);
        }
        Ok(options)
    }
}

impl SourceFeatureQueryOptionsInput {
    fn into_core(self) -> Result<core::query::SourceFeatureQueryOptions> {
        let mut options = core::query::SourceFeatureQueryOptions::new();
        if let Some(source_layer_ids) = self.source_layer_ids {
            options = options.with_source_layer_ids(source_layer_ids);
        }
        if let Some(filter) = self.filter {
            options = options.with_filter(parse_json_value(filter)?);
        }
        Ok(options)
    }
}

impl FeatureStateSelectorInput {
    fn into_core(self) -> Result<core::query::FeatureStateSelector> {
        let mut selector = core::query::FeatureStateSelector::new(self.source_id);
        if let Some(source_layer_id) = self.source_layer_id {
            selector = selector.with_source_layer_id(source_layer_id);
        }
        if let Some(feature_id) = self.feature_id {
            selector = selector.with_feature_id(feature_id);
        }
        if let Some(state_key) = self.state_key {
            selector = selector
                .with_state_key(state_key)
                .map_err(error::from_core)?;
        }
        Ok(selector)
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

impl VulkanContextDescriptor {
    fn into_native(self) -> Result<sys::mln_vulkan_context_descriptor> {
        Ok(sys::mln_vulkan_context_descriptor {
            size: std::mem::size_of::<sys::mln_vulkan_context_descriptor>() as u32,
            instance: bigint_to_ptr(&self.instance_address, "instanceAddress")?,
            physical_device: bigint_to_ptr(&self.physical_device_address, "physicalDeviceAddress")?,
            device: bigint_to_ptr(&self.device_address, "deviceAddress")?,
            graphics_queue: bigint_to_ptr(&self.graphics_queue_address, "graphicsQueueAddress")?,
            graphics_queue_family_index: self.graphics_queue_family_index,
        })
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

fn queried_features_to_string(result: *mut sys::mln_feature_query_result) -> Result<String> {
    let result = std::ptr::NonNull::new(result)
        .ok_or_else(|| error::invalid_argument("feature query result was null"))?;
    let features =
        unsafe { core::query::copy_feature_query_result(result) }.map_err(error::from_core)?;
    let value =
        serde_json::Value::Array(features.into_iter().map(queried_feature_to_serde).collect());
    serde_json::to_string(&value).map_err(|serialize_error| {
        error::invalid_argument(format!(
            "query result could not be serialized: {serialize_error}"
        ))
    })
}

fn queried_feature_to_serde(feature: core::query::QueriedFeature) -> serde_json::Value {
    serde_json::json!({
        "feature": feature_to_serde(feature.feature),
        "sourceId": feature.source_id,
        "sourceLayerId": feature.source_layer_id,
        "state": feature.state.map(json_value_to_serde),
    })
}

fn feature_to_serde(feature: core::Feature) -> serde_json::Value {
    let properties = feature
        .properties
        .into_iter()
        .map(|member| (member.key, json_value_to_serde(member.value)))
        .collect::<serde_json::Map<_, _>>();
    let mut value = serde_json::json!({
        "type": "Feature",
        "geometry": geometry_to_serde(feature.geometry),
        "properties": properties,
    });
    if !matches!(feature.identifier, core::FeatureIdentifier::Null) {
        value["id"] = feature_identifier_to_serde(feature.identifier);
    }
    value
}

fn feature_identifier_to_serde(identifier: core::FeatureIdentifier) -> serde_json::Value {
    match identifier {
        core::FeatureIdentifier::Null => serde_json::Value::Null,
        core::FeatureIdentifier::UInt(value) => serde_json::Value::Number(value.into()),
        core::FeatureIdentifier::Int(value) => serde_json::Value::Number(value.into()),
        core::FeatureIdentifier::Double(value) => serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        core::FeatureIdentifier::String(value) => serde_json::Value::String(value),
        _ => serde_json::Value::Null,
    }
}

fn geometry_to_serde(geometry: core::Geometry) -> serde_json::Value {
    match geometry {
        core::Geometry::Empty => serde_json::Value::Null,
        core::Geometry::Point(coordinate) => serde_json::json!({
            "type": "Point",
            "coordinates": [coordinate.longitude, coordinate.latitude]
        }),
        core::Geometry::LineString(coordinates) => serde_json::json!({
            "type": "LineString",
            "coordinates": coordinates_to_serde(coordinates)
        }),
        core::Geometry::Polygon(rings) => serde_json::json!({
            "type": "Polygon",
            "coordinates": rings.into_iter().map(coordinates_to_serde).collect::<Vec<_>>()
        }),
        core::Geometry::MultiPoint(coordinates) => serde_json::json!({
            "type": "MultiPoint",
            "coordinates": coordinates_to_serde(coordinates)
        }),
        core::Geometry::MultiLineString(lines) => serde_json::json!({
            "type": "MultiLineString",
            "coordinates": lines.into_iter().map(coordinates_to_serde).collect::<Vec<_>>()
        }),
        core::Geometry::MultiPolygon(polygons) => serde_json::json!({
            "type": "MultiPolygon",
            "coordinates": polygons
                .into_iter()
                .map(|rings| rings.into_iter().map(coordinates_to_serde).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        }),
        core::Geometry::GeometryCollection(geometries) => serde_json::json!({
            "type": "GeometryCollection",
            "geometries": geometries.into_iter().map(geometry_to_serde).collect::<Vec<_>>()
        }),
        _ => serde_json::Value::Null,
    }
}

fn coordinates_to_serde(coordinates: Vec<core::LatLng>) -> serde_json::Value {
    serde_json::Value::Array(
        coordinates
            .into_iter()
            .map(|coordinate| serde_json::json!([coordinate.longitude, coordinate.latitude]))
            .collect(),
    )
}

fn attach_render_session(
    map: &NativeMapHandle,
    attach: impl FnOnce(*mut *mut sys::mln_render_session) -> sys::mln_status,
) -> Result<NativeRenderSessionHandle> {
    let mut session = std::ptr::null_mut();
    core::check(attach(&mut session)).map_err(error::from_core)?;
    let state = unsafe { NativeHandleState::from_raw_ptr(session, "RenderSessionHandle") }
        .map_err(error::from_core)?;
    let _ = map;
    Ok(NativeRenderSessionHandle { state })
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

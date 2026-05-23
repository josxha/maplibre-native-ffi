#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, c_char, c_void};
use std::ptr;

use maplibre_native_core as core;
use maplibre_native_core::error::{Error, Result};
#[cfg(test)]
use maplibre_native_sys as sys;

use crate::geo::LatLng;
use crate::glib::{self, GError, GType};

const JSON_VALUE_TYPE_NAME: &CStr = c"MlnValaJsonValue";
const GEOMETRY_TYPE_NAME: &CStr = c"MlnValaGeometry";
const GEOJSON_TYPE_NAME: &CStr = c"MlnValaGeoJson";

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct JsonValue {
    pub(crate) value: core::JsonValue,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    pub(crate) value: core::Geometry,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Feature {
    pub(crate) value: core::Feature,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct GeoJson {
    pub(crate) value: core::GeoJson,
}

impl JsonValue {
    pub(crate) fn materialize(&self) -> Result<core::json::NativeJsonValue> {
        core::json::json_value_try_to_native(&self.value)
    }
}

impl Geometry {
    pub(crate) fn materialize(&self) -> Result<core::geometry::NativeGeometry> {
        core::geometry::geometry_try_to_native(&self.value)
    }
}

impl GeoJson {
    pub(crate) fn materialize(&self) -> Result<core::geojson::NativeGeoJson> {
        core::geojson::geojson_try_to_native(&self.value)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_value_get_type() -> GType {
    glib::register_boxed_type(JSON_VALUE_TYPE_NAME, json_copy_erased, json_free_erased)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_geometry_get_type() -> GType {
    glib::register_boxed_type(
        GEOMETRY_TYPE_NAME,
        geometry_copy_erased,
        geometry_free_erased,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_geo_json_get_type() -> GType {
    glib::register_boxed_type(GEOJSON_TYPE_NAME, geojson_copy_erased, geojson_free_erased)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_value_copy(value: *const JsonValue) -> *mut JsonValue {
    json_ref(value).map_or(ptr::null_mut(), |value| {
        Box::into_raw(Box::new(value.clone()))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_value_free(value: *mut JsonValue) {
    if !value.is_null() {
        unsafe {
            drop(Box::from_raw(value));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_geometry_copy(value: *const Geometry) -> *mut Geometry {
    geometry_ref(value).map_or(ptr::null_mut(), |value| {
        Box::into_raw(Box::new(value.clone()))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_geometry_free(value: *mut Geometry) {
    if !value.is_null() {
        unsafe {
            drop(Box::from_raw(value));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_geo_json_copy(value: *const GeoJson) -> *mut GeoJson {
    geojson_ref(value).map_or(ptr::null_mut(), |value| {
        Box::into_raw(Box::new(value.clone()))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_geo_json_free(value: *mut GeoJson) {
    if !value.is_null() {
        unsafe {
            drop(Box::from_raw(value));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_value_new_null() -> *mut JsonValue {
    Box::into_raw(Box::new(JsonValue {
        value: core::JsonValue::Null,
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_value_new_bool(value: bool) -> *mut JsonValue {
    Box::into_raw(Box::new(JsonValue {
        value: core::JsonValue::Bool(value),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_value_new_uint(value: u64) -> *mut JsonValue {
    Box::into_raw(Box::new(JsonValue {
        value: core::JsonValue::UInt(value),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_value_new_int(value: i64) -> *mut JsonValue {
    Box::into_raw(Box::new(JsonValue {
        value: core::JsonValue::Int(value),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_value_new_double(
    value: f64,
    error_out: *mut *mut GError,
) -> *mut JsonValue {
    if !value.is_finite() {
        glib::set_error(
            error_out,
            Error::invalid_argument("JSON double values must be finite"),
        );
        return ptr::null_mut();
    }
    Box::into_raw(Box::new(JsonValue {
        value: core::JsonValue::Double(value),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_value_new_string(
    value: *const c_char,
    error_out: *mut *mut GError,
) -> *mut JsonValue {
    match copy_c_string(value) {
        Ok(value) => Box::into_raw(Box::new(JsonValue {
            value: core::JsonValue::String(value),
        })),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_value_new_array(
    values: *const *const JsonValue,
    value_count: usize,
    error_out: *mut *mut GError,
) -> *mut JsonValue {
    match json_array(values, value_count) {
        Ok(value) => Box::into_raw(Box::new(JsonValue { value })),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_value_new_object(
    keys: *const *const c_char,
    values: *const *const JsonValue,
    member_count: usize,
    error_out: *mut *mut GError,
) -> *mut JsonValue {
    match json_object(keys, values, member_count) {
        Ok(value) => Box::into_raw(Box::new(JsonValue { value })),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_geometry_new_empty() -> *mut Geometry {
    Box::into_raw(Box::new(Geometry {
        value: core::Geometry::Empty,
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_geometry_new_point(
    point: *const LatLng,
    error_out: *mut *mut GError,
) -> *mut Geometry {
    if point.is_null() {
        glib::set_error(error_out, Error::invalid_argument("point is null"));
        return ptr::null_mut();
    }
    let point = unsafe { *point };
    Box::into_raw(Box::new(Geometry {
        value: core::Geometry::Point(to_core_lat_lng(point)),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_geometry_new_line_string(
    coordinates: *const LatLng,
    coordinate_count: usize,
    error_out: *mut *mut GError,
) -> *mut Geometry {
    match coordinates_slice(coordinates, coordinate_count) {
        Ok(coordinates) => Box::into_raw(Box::new(Geometry {
            value: core::Geometry::LineString(
                coordinates.iter().copied().map(to_core_lat_lng).collect(),
            ),
        })),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_geo_json_new_geometry(
    geometry: *const Geometry,
    error_out: *mut *mut GError,
) -> *mut GeoJson {
    let Some(geometry) = geometry_ref(geometry) else {
        glib::set_error(
            error_out,
            Error::invalid_argument("GeoJSON geometry is null"),
        );
        return ptr::null_mut();
    };
    Box::into_raw(Box::new(GeoJson {
        value: core::GeoJson::Geometry(geometry.value.clone()),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_geo_json_new_feature(
    geometry: *const Geometry,
    error_out: *mut *mut GError,
) -> *mut GeoJson {
    let Some(geometry) = geometry_ref(geometry) else {
        glib::set_error(
            error_out,
            Error::invalid_argument("GeoJSON feature geometry is null"),
        );
        return ptr::null_mut();
    };
    let feature = core::Feature::new(geometry.value.clone(), Vec::new());
    Box::into_raw(Box::new(GeoJson {
        value: core::GeoJson::Feature(feature),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_geo_json_new_feature_collection(
    geometries: *const *const Geometry,
    geometry_count: usize,
    error_out: *mut *mut GError,
) -> *mut GeoJson {
    match geometry_feature_array(geometries, geometry_count) {
        Ok(features) => Box::into_raw(Box::new(GeoJson {
            value: core::GeoJson::FeatureCollection(features),
        })),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

pub(crate) fn json_ref(value: *const JsonValue) -> Option<&'static JsonValue> {
    if value.is_null() {
        None
    } else {
        Some(unsafe { &*value })
    }
}

pub(crate) fn geometry_ref(value: *const Geometry) -> Option<&'static Geometry> {
    if value.is_null() {
        None
    } else {
        Some(unsafe { &*value })
    }
}

pub(crate) fn feature_ref(value: *const Feature) -> Option<&'static Feature> {
    if value.is_null() {
        None
    } else {
        Some(unsafe { &*value })
    }
}

pub(crate) fn geojson_ref(value: *const GeoJson) -> Option<&'static GeoJson> {
    if value.is_null() {
        None
    } else {
        Some(unsafe { &*value })
    }
}

unsafe extern "C" fn json_copy_erased(value: *mut c_void) -> *mut c_void {
    mln_vala_json_value_copy(value.cast()).cast()
}
unsafe extern "C" fn json_free_erased(value: *mut c_void) {
    mln_vala_json_value_free(value.cast());
}
unsafe extern "C" fn geometry_copy_erased(value: *mut c_void) -> *mut c_void {
    mln_vala_geometry_copy(value.cast()).cast()
}
unsafe extern "C" fn geometry_free_erased(value: *mut c_void) {
    mln_vala_geometry_free(value.cast());
}
unsafe extern "C" fn geojson_copy_erased(value: *mut c_void) -> *mut c_void {
    mln_vala_geo_json_copy(value.cast()).cast()
}
unsafe extern "C" fn geojson_free_erased(value: *mut c_void) {
    mln_vala_geo_json_free(value.cast());
}

fn json_array(values: *const *const JsonValue, value_count: usize) -> Result<core::JsonValue> {
    let values = pointer_slice(values, value_count, "JSON array values")?;
    let mut copied = Vec::with_capacity(values.len());
    for value in values {
        let Some(value) = json_ref(*value) else {
            return Err(Error::invalid_argument("JSON array value is null"));
        };
        copied.push(value.value.clone());
    }
    Ok(core::JsonValue::Array(copied))
}

fn json_object(
    keys: *const *const c_char,
    values: *const *const JsonValue,
    member_count: usize,
) -> Result<core::JsonValue> {
    let keys = pointer_slice(keys, member_count, "JSON object keys")?;
    let values = pointer_slice(values, member_count, "JSON object values")?;
    let mut members = Vec::with_capacity(member_count);
    for index in 0..member_count {
        let key = copy_c_string(keys[index])?;
        let Some(value) = json_ref(values[index]) else {
            return Err(Error::invalid_argument("JSON object value is null"));
        };
        members.push(core::JsonMember::new(key, value.value.clone()));
    }
    Ok(core::JsonValue::Object(members))
}

fn geometry_feature_array(
    geometries: *const *const Geometry,
    geometry_count: usize,
) -> Result<Vec<core::Feature>> {
    let geometries = pointer_slice(geometries, geometry_count, "GeoJSON feature geometries")?;
    let mut copied = Vec::with_capacity(geometries.len());
    for geometry in geometries {
        let Some(geometry) = geometry_ref(*geometry) else {
            return Err(Error::invalid_argument("GeoJSON feature geometry is null"));
        };
        copied.push(core::Feature::new(geometry.value.clone(), Vec::new()));
    }
    Ok(copied)
}

fn pointer_slice<'a, T>(
    ptr: *const *const T,
    count: usize,
    name: &'static str,
) -> Result<&'a [*const T]> {
    if count == 0 {
        return Ok(&[]);
    }
    if ptr.is_null() {
        return Err(Error::invalid_argument(format!("{name} pointer is null")));
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr, count) })
}

fn coordinates_slice<'a>(ptr: *const LatLng, count: usize) -> Result<&'a [LatLng]> {
    if count == 0 {
        return Ok(&[]);
    }
    if ptr.is_null() {
        return Err(Error::invalid_argument("coordinates pointer is null"));
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr, count) })
}

fn copy_c_string(value: *const c_char) -> Result<String> {
    if value.is_null() {
        return Err(Error::invalid_argument("string is null"));
    }
    unsafe { CStr::from_ptr(value) }
        .to_str()
        .map(|value| value.to_owned())
        .map_err(|_| Error::invalid_argument("string is not valid UTF-8"))
}

fn to_core_lat_lng(value: LatLng) -> core::LatLng {
    core::LatLng::new(value.latitude, value.longitude)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_string_materializes() {
        let value = mln_vala_json_value_new_string(c"park".as_ptr(), ptr::null_mut());
        assert!(!value.is_null());
        let native = json_ref(value).unwrap().materialize().unwrap();
        assert_eq!(native.as_ref().type_, sys::MLN_JSON_VALUE_TYPE_STRING);
        mln_vala_json_value_free(value);
    }

    #[test]
    fn geometry_line_string_materializes() {
        let coordinates = [LatLng {
            latitude: 1.0,
            longitude: 2.0,
        }];
        let value = mln_vala_geometry_new_line_string(
            coordinates.as_ptr(),
            coordinates.len(),
            ptr::null_mut(),
        );
        assert!(!value.is_null());
        let native = geometry_ref(value).unwrap().materialize().unwrap();
        assert_eq!(native.as_ref().type_, sys::MLN_GEOMETRY_TYPE_LINE_STRING);
        mln_vala_geometry_free(value);
    }
}

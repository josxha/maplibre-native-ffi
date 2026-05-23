use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr;
use std::ptr::NonNull;

use maplibre_native_core as core;
use maplibre_native_core::error::{self, Error};
use maplibre_native_sys as sys;

use crate::glib::{self, GBoolean, GError, GFALSE, GObject, GTRUE, GType};
use crate::render::{self, RenderSessionHandle};
use crate::values::{self, Feature, GeoJson, JsonValue};

const QUERIED_FEATURE_TYPE_NAME: &CStr = c"MlnValaQueriedFeature";
const QUERIED_FEATURE_LIST_TYPE_NAME: &CStr = c"MlnValaQueriedFeatureList";
const FEATURE_EXTENSION_RESULT_TYPE_NAME: &CStr = c"MlnValaFeatureExtensionResult";
const FEATURE_QUERY_RESULT_TYPE_NAME: &CStr = c"MlnValaFeatureQueryResultHandle";
const FEATURE_EXTENSION_RESULT_HANDLE_TYPE_NAME: &CStr = c"MlnValaFeatureExtensionResultHandle";

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct QueriedFeature {
    value: core::query::QueriedFeature,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct QueriedFeatureList {
    features: Vec<core::query::QueriedFeature>,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct FeatureExtensionResult {
    value: core::query::FeatureExtensionResult,
}

#[repr(C)]
pub struct FeatureQueryResultHandle {
    parent_instance: GObject,
    native: *mut sys::mln_feature_query_result,
}

#[repr(C)]
pub struct FeatureExtensionResultHandle {
    parent_instance: GObject,
    native: *mut sys::mln_feature_extension_result,
}

impl glib::ObjectFinalize for FeatureQueryResultHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        close_feature_query_result(object.cast::<FeatureQueryResultHandle>());
    }
}

impl glib::ObjectFinalize for FeatureExtensionResultHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        close_feature_extension_result(object.cast::<FeatureExtensionResultHandle>());
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_query_result_handle_get_type() -> GType {
    glib::register_object_type::<FeatureQueryResultHandle>(FEATURE_QUERY_RESULT_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_extension_result_handle_get_type() -> GType {
    glib::register_object_type::<FeatureExtensionResultHandle>(
        FEATURE_EXTENSION_RESULT_HANDLE_TYPE_NAME,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_queried_feature_get_type() -> GType {
    glib::register_boxed_type(
        QUERIED_FEATURE_TYPE_NAME,
        queried_feature_copy_erased,
        queried_feature_free_erased,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_queried_feature_list_get_type() -> GType {
    glib::register_boxed_type(
        QUERIED_FEATURE_LIST_TYPE_NAME,
        queried_feature_list_copy_erased,
        queried_feature_list_free_erased,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_extension_result_get_type() -> GType {
    glib::register_boxed_type(
        FEATURE_EXTENSION_RESULT_TYPE_NAME,
        feature_extension_result_copy_erased,
        feature_extension_result_free_erased,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_queried_feature_copy(
    value: *const QueriedFeature,
) -> *mut QueriedFeature {
    queried_feature_ref(value)
        .map(|value| Box::into_raw(Box::new(value.clone())))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn mln_vala_queried_feature_free(value: *mut QueriedFeature) {
    if !value.is_null() {
        unsafe { drop(Box::from_raw(value)) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_queried_feature_list_copy(
    value: *const QueriedFeatureList,
) -> *mut QueriedFeatureList {
    queried_feature_list_ref(value)
        .map(|value| Box::into_raw(Box::new(value.clone())))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn mln_vala_queried_feature_list_free(value: *mut QueriedFeatureList) {
    if !value.is_null() {
        unsafe { drop(Box::from_raw(value)) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_extension_result_copy(
    value: *const FeatureExtensionResult,
) -> *mut FeatureExtensionResult {
    feature_extension_result_ref(value)
        .map(|value| Box::into_raw(Box::new(value.clone())))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn mln_vala_feature_extension_result_free(value: *mut FeatureExtensionResult) {
    if !value.is_null() {
        unsafe { drop(Box::from_raw(value)) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_rendered_feature_query_options_default(
    out_options: *mut sys::mln_rendered_feature_query_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_rendered_feature_query_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_source_feature_query_options_default(
    out_options: *mut sys::mln_source_feature_query_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match default_source_feature_query_options(out_options) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_rendered_query_geometry_point(
    point: *const sys::mln_screen_point,
    out_geometry: *mut sys::mln_rendered_query_geometry,
    error_out: *mut *mut GError,
) -> GBoolean {
    match rendered_query_geometry_point(point, out_geometry) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_rendered_query_geometry_box(
    box_: *const sys::mln_screen_box,
    out_geometry: *mut sys::mln_rendered_query_geometry,
    error_out: *mut *mut GError,
) -> GBoolean {
    match rendered_query_geometry_box(box_, out_geometry) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_rendered_query_geometry_line_string(
    points: *const sys::mln_screen_point,
    point_count: usize,
    out_geometry: *mut sys::mln_rendered_query_geometry,
    error_out: *mut *mut GError,
) -> GBoolean {
    match rendered_query_geometry_line_string(points, point_count, out_geometry) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_query_rendered_features(
    session: *mut RenderSessionHandle,
    geometry: *const sys::mln_rendered_query_geometry,
    options: *const sys::mln_rendered_feature_query_options,
    error_out: *mut *mut GError,
) -> *mut QueriedFeatureList {
    match query_rendered_features(session, geometry, options) {
        Ok(features) => Box::into_raw(Box::new(QueriedFeatureList { features })),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_query_source_features(
    session: *mut RenderSessionHandle,
    source_id: *const c_char,
    options: *const sys::mln_source_feature_query_options,
    error_out: *mut *mut GError,
) -> *mut QueriedFeatureList {
    match query_source_features(session, source_id, options) {
        Ok(features) => Box::into_raw(Box::new(QueriedFeatureList { features })),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_render_session_handle_query_feature_extension(
    session: *mut RenderSessionHandle,
    source_id: *const c_char,
    feature: *const Feature,
    extension: *const c_char,
    extension_field: *const c_char,
    arguments: *const JsonValue,
    error_out: *mut *mut GError,
) -> *mut FeatureExtensionResult {
    match query_feature_extension(
        session,
        source_id,
        feature,
        extension,
        extension_field,
        arguments,
    ) {
        Ok(value) => Box::into_raw(Box::new(FeatureExtensionResult { value })),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_queried_feature_list_count(value: *const QueriedFeatureList) -> usize {
    queried_feature_list_ref(value).map_or(0, |value| value.features.len())
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_queried_feature_list_get(
    value: *const QueriedFeatureList,
    index: usize,
    error_out: *mut *mut GError,
) -> *mut QueriedFeature {
    match queried_feature_list_get(value, index) {
        Ok(feature) => Box::into_raw(Box::new(feature)),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_queried_feature_get_feature(
    value: *const QueriedFeature,
) -> *mut Feature {
    queried_feature_ref(value).map_or(ptr::null_mut(), |value| {
        Box::into_raw(Box::new(Feature {
            value: value.value.feature.clone(),
        }))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_queried_feature_dup_source_id(
    value: *const QueriedFeature,
) -> *mut c_char {
    queried_feature_ref(value)
        .and_then(|value| value.value.source_id.as_deref())
        .and_then(copy_string)
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_queried_feature_dup_source_layer_id(
    value: *const QueriedFeature,
) -> *mut c_char {
    queried_feature_ref(value)
        .and_then(|value| value.value.source_layer_id.as_deref())
        .and_then(copy_string)
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_queried_feature_get_state(
    value: *const QueriedFeature,
) -> *mut JsonValue {
    queried_feature_ref(value)
        .and_then(|value| value.value.state.clone())
        .map(|value| Box::into_raw(Box::new(JsonValue { value })))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_extension_result_get_result_type(
    value: *const FeatureExtensionResult,
) -> sys::mln_feature_extension_result_type {
    match feature_extension_result_ref(value).map(|value| &value.value) {
        Some(core::query::FeatureExtensionResult::Value(_)) => {
            sys::MLN_FEATURE_EXTENSION_RESULT_TYPE_VALUE
        }
        Some(core::query::FeatureExtensionResult::FeatureCollection(_)) => {
            sys::MLN_FEATURE_EXTENSION_RESULT_TYPE_FEATURE_COLLECTION
        }
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_extension_result_get_value(
    value: *const FeatureExtensionResult,
) -> *mut JsonValue {
    match feature_extension_result_ref(value).map(|value| &value.value) {
        Some(core::query::FeatureExtensionResult::Value(value)) => {
            Box::into_raw(Box::new(JsonValue {
                value: value.clone(),
            }))
        }
        _ => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_extension_result_get_feature_collection(
    value: *const FeatureExtensionResult,
) -> *mut GeoJson {
    match feature_extension_result_ref(value).map(|value| &value.value) {
        Some(core::query::FeatureExtensionResult::FeatureCollection(features)) => {
            Box::into_raw(Box::new(GeoJson {
                value: core::GeoJson::FeatureCollection(features.clone()),
            }))
        }
        _ => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_query_result_handle_count(
    handle: *mut FeatureQueryResultHandle,
    out_count: *mut usize,
    error_out: *mut *mut GError,
) -> GBoolean {
    match feature_query_result_count(handle, out_count) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_query_result_handle_get(
    handle: *mut FeatureQueryResultHandle,
    index: usize,
    out_feature: *mut sys::mln_queried_feature,
    error_out: *mut *mut GError,
) -> GBoolean {
    match feature_query_result_get(handle, index, out_feature) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_query_result_handle_close(
    handle: *mut FeatureQueryResultHandle,
) {
    close_feature_query_result(handle);
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_extension_result_handle_get(
    handle: *mut FeatureExtensionResultHandle,
    out_info: *mut sys::mln_feature_extension_result_info,
    error_out: *mut *mut GError,
) -> GBoolean {
    match feature_extension_result_get(handle, out_info) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_extension_result_handle_close(
    handle: *mut FeatureExtensionResultHandle,
) {
    close_feature_extension_result(handle);
}

fn default_rendered_feature_query_options(
    out_options: *mut sys::mln_rendered_feature_query_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_rendered_feature_query_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn default_source_feature_query_options(
    out_options: *mut sys::mln_source_feature_query_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_source_feature_query_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

fn rendered_query_geometry_point(
    point: *const sys::mln_screen_point,
    out_geometry: *mut sys::mln_rendered_query_geometry,
) -> error::Result<()> {
    if point.is_null() {
        return Err(maplibre_native_core::error::Error::invalid_argument(
            "query point is null",
        ));
    }
    // SAFETY: `point` was checked non-null and constructor returns an initialized value.
    let geometry = unsafe { sys::mln_rendered_query_geometry_point(*point) };
    glib::clear_optional_out_pointer(out_geometry, geometry)
}

fn rendered_query_geometry_box(
    box_: *const sys::mln_screen_box,
    out_geometry: *mut sys::mln_rendered_query_geometry,
) -> error::Result<()> {
    if box_.is_null() {
        return Err(maplibre_native_core::error::Error::invalid_argument(
            "query box is null",
        ));
    }
    // SAFETY: `box_` was checked non-null and constructor returns an initialized value.
    let geometry = unsafe { sys::mln_rendered_query_geometry_box(*box_) };
    glib::clear_optional_out_pointer(out_geometry, geometry)
}

fn rendered_query_geometry_line_string(
    points: *const sys::mln_screen_point,
    point_count: usize,
    out_geometry: *mut sys::mln_rendered_query_geometry,
) -> error::Result<()> {
    // SAFETY: Constructor borrows caller points for the descriptor and returns
    // an initialized value. The C API validates null/count combinations when a
    // query consumes the descriptor.
    let geometry = unsafe { sys::mln_rendered_query_geometry_line_string(points, point_count) };
    glib::clear_optional_out_pointer(out_geometry, geometry)
}

fn query_rendered_features(
    session: *mut RenderSessionHandle,
    geometry: *const sys::mln_rendered_query_geometry,
    options: *const sys::mln_rendered_feature_query_options,
) -> error::Result<Vec<core::query::QueriedFeature>> {
    if geometry.is_null() {
        return Err(Error::invalid_argument("rendered query geometry is null"));
    }
    if options.is_null() {
        return Err(Error::invalid_argument(
            "rendered feature query options are null",
        ));
    }
    let session = render::session_native(session)?;
    let mut result = ptr::null_mut();
    // SAFETY: `session` is live, descriptors are borrowed for this call, and
    // result output storage is valid.
    error::check(unsafe {
        sys::mln_render_session_query_rendered_features(session, geometry, options, &mut result)
    })?;
    copy_feature_query_result(result)
}

fn query_source_features(
    session: *mut RenderSessionHandle,
    source_id: *const c_char,
    options: *const sys::mln_source_feature_query_options,
) -> error::Result<Vec<core::query::QueriedFeature>> {
    if options.is_null() {
        return Err(Error::invalid_argument(
            "source feature query options are null",
        ));
    }
    let session = render::session_native(session)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let mut result = ptr::null_mut();
    // SAFETY: `session` is live, source ID and options are borrowed for this
    // call, and result output storage is valid.
    error::check(unsafe {
        sys::mln_render_session_query_source_features(session, source_id, options, &mut result)
    })?;
    copy_feature_query_result(result)
}

fn query_feature_extension(
    session: *mut RenderSessionHandle,
    source_id: *const c_char,
    feature: *const Feature,
    extension: *const c_char,
    extension_field: *const c_char,
    arguments: *const JsonValue,
) -> error::Result<core::query::FeatureExtensionResult> {
    let feature = values::feature_ref(feature)
        .ok_or_else(|| Error::invalid_argument("feature descriptor is null"))?;
    let feature = core::geojson::feature_try_to_native(&feature.value, 0)?;
    let arguments = if arguments.is_null() {
        None
    } else {
        Some(
            values::json_ref(arguments)
                .ok_or_else(|| Error::invalid_argument("feature extension arguments are null"))?
                .materialize()?,
        )
    };
    let session = render::session_native(session)?;
    let source_id = string_view_from_c(source_id, "source ID")?;
    let extension = string_view_from_c(extension, "feature extension name")?;
    let extension_field = string_view_from_c(extension_field, "feature extension field")?;
    let mut result = ptr::null_mut();
    // SAFETY: `session` is live, materializers own descriptor storage for this
    // call, and result output storage is valid.
    error::check(unsafe {
        sys::mln_render_session_query_feature_extensions(
            session,
            source_id,
            feature.as_ref(),
            extension,
            extension_field,
            arguments
                .as_ref()
                .map_or(ptr::null(), |arguments| arguments.as_ptr()),
            &mut result,
        )
    })?;
    copy_feature_extension_result(result)
}

fn queried_feature_ref(value: *const QueriedFeature) -> Option<&'static QueriedFeature> {
    if value.is_null() {
        None
    } else {
        Some(unsafe { &*value })
    }
}

fn queried_feature_list_ref(
    value: *const QueriedFeatureList,
) -> Option<&'static QueriedFeatureList> {
    if value.is_null() {
        None
    } else {
        Some(unsafe { &*value })
    }
}

fn feature_extension_result_ref(
    value: *const FeatureExtensionResult,
) -> Option<&'static FeatureExtensionResult> {
    if value.is_null() {
        None
    } else {
        Some(unsafe { &*value })
    }
}

fn queried_feature_list_get(
    value: *const QueriedFeatureList,
    index: usize,
) -> error::Result<QueriedFeature> {
    let list = queried_feature_list_ref(value)
        .ok_or_else(|| Error::invalid_argument("queried feature list is null"))?;
    let value = list
        .features
        .get(index)
        .ok_or_else(|| Error::invalid_argument("queried feature index is out of range"))?
        .clone();
    Ok(QueriedFeature { value })
}

fn copy_string(value: &str) -> Option<*mut c_char> {
    let c_string = CString::new(value).ok()?;
    glib::copy_string_view(sys::mln_string_view {
        data: c_string.as_ptr(),
        size: c_string.as_bytes().len(),
    })
    .ok()
}

unsafe extern "C" fn queried_feature_copy_erased(value: *mut c_void) -> *mut c_void {
    mln_vala_queried_feature_copy(value.cast()).cast()
}

unsafe extern "C" fn queried_feature_free_erased(value: *mut c_void) {
    mln_vala_queried_feature_free(value.cast());
}

unsafe extern "C" fn queried_feature_list_copy_erased(value: *mut c_void) -> *mut c_void {
    mln_vala_queried_feature_list_copy(value.cast()).cast()
}

unsafe extern "C" fn queried_feature_list_free_erased(value: *mut c_void) {
    mln_vala_queried_feature_list_free(value.cast());
}

unsafe extern "C" fn feature_extension_result_copy_erased(value: *mut c_void) -> *mut c_void {
    mln_vala_feature_extension_result_copy(value.cast()).cast()
}

unsafe extern "C" fn feature_extension_result_free_erased(value: *mut c_void) {
    mln_vala_feature_extension_result_free(value.cast());
}

fn copy_feature_query_result(
    native: *mut sys::mln_feature_query_result,
) -> error::Result<Vec<core::query::QueriedFeature>> {
    let native = NonNull::new(native)
        .ok_or_else(|| Error::invalid_argument("native feature query result is null"))?;
    // SAFETY: the C API returned an owned query result handle; the core helper
    // copies all result data and releases the native handle on every exit path.
    unsafe { core::query::copy_feature_query_result(native) }
}

fn copy_feature_extension_result(
    native: *mut sys::mln_feature_extension_result,
) -> error::Result<core::query::FeatureExtensionResult> {
    let native = NonNull::new(native)
        .ok_or_else(|| Error::invalid_argument("native feature extension result is null"))?;
    // SAFETY: the C API returned an owned extension result handle; the core
    // helper copies all result data and releases the native handle on every exit path.
    unsafe { core::query::copy_feature_extension_result(native) }
}

fn feature_query_result_native(
    handle: *mut FeatureQueryResultHandle,
) -> error::Result<*mut sys::mln_feature_query_result> {
    if handle.is_null() {
        return Err(Error::invalid_argument("FeatureQueryResultHandle is null"));
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe { (*handle).native };
    if native.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "FeatureQueryResultHandle is closed",
        ));
    }
    Ok(native)
}

fn feature_query_result_count(
    handle: *mut FeatureQueryResultHandle,
    out_count: *mut usize,
) -> error::Result<()> {
    let native = feature_query_result_native(handle)?;
    let mut count = 0;
    // SAFETY: `native` is live and output storage is valid.
    error::check(unsafe { sys::mln_feature_query_result_count(native, &mut count) })?;
    glib::clear_optional_out_pointer(out_count, count)
}

fn feature_query_result_get(
    handle: *mut FeatureQueryResultHandle,
    index: usize,
    out_feature: *mut sys::mln_queried_feature,
) -> error::Result<()> {
    let native = feature_query_result_native(handle)?;
    if out_feature.is_null() {
        return Err(Error::invalid_argument(
            "queried feature output pointer is null",
        ));
    }
    // SAFETY: `out_feature` is valid output storage by the null check above.
    unsafe {
        (*out_feature).size = std::mem::size_of::<sys::mln_queried_feature>() as u32;
    }
    // SAFETY: `native` is live and output storage has the required size field.
    error::check(unsafe { sys::mln_feature_query_result_get(native, index, out_feature) })
}

fn close_feature_query_result(handle: *mut FeatureQueryResultHandle) {
    if handle.is_null() {
        return;
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe {
        let native = (*handle).native;
        (*handle).native = ptr::null_mut();
        native
    };
    if !native.is_null() {
        // SAFETY: This wrapper owns the native query result and closes it exactly once.
        unsafe { sys::mln_feature_query_result_destroy(native) };
    }
}

fn feature_extension_result_native(
    handle: *mut FeatureExtensionResultHandle,
) -> error::Result<*mut sys::mln_feature_extension_result> {
    if handle.is_null() {
        return Err(Error::invalid_argument(
            "FeatureExtensionResultHandle is null",
        ));
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe { (*handle).native };
    if native.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "FeatureExtensionResultHandle is closed",
        ));
    }
    Ok(native)
}

fn feature_extension_result_get(
    handle: *mut FeatureExtensionResultHandle,
    out_info: *mut sys::mln_feature_extension_result_info,
) -> error::Result<()> {
    let native = feature_extension_result_native(handle)?;
    if out_info.is_null() {
        return Err(Error::invalid_argument(
            "feature extension result info output pointer is null",
        ));
    }
    // SAFETY: `out_info` is valid output storage by the null check above.
    unsafe {
        (*out_info).size = std::mem::size_of::<sys::mln_feature_extension_result_info>() as u32;
    }
    // SAFETY: `native` is live and output storage has the required size field.
    error::check(unsafe { sys::mln_feature_extension_result_get(native, out_info) })
}

fn close_feature_extension_result(handle: *mut FeatureExtensionResultHandle) {
    if handle.is_null() {
        return;
    }
    // SAFETY: `handle` is non-null and expected to point to this type.
    let native = unsafe {
        let native = (*handle).native;
        (*handle).native = ptr::null_mut();
        native
    };
    if !native.is_null() {
        // SAFETY: This wrapper owns the native extension result and closes it exactly once.
        unsafe { sys::mln_feature_extension_result_destroy(native) };
    }
}

fn string_view_from_c(value: *const c_char, label: &str) -> error::Result<sys::mln_string_view> {
    if value.is_null() {
        return Err(Error::invalid_argument(format!("{label} is null")));
    }
    // SAFETY: The caller supplies a NUL-terminated C string pointer.
    let bytes = unsafe { CStr::from_ptr(value) }.to_bytes();
    Ok(sys::mln_string_view {
        data: value,
        size: bytes.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn query_defaults_and_geometry_constructors_initialize_sizes() {
        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut rendered_options: sys::mln_rendered_feature_query_options =
            unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_rendered_feature_query_options_default(&mut rendered_options, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(rendered_options.size, 0);

        // SAFETY: Zeroed storage is immediately initialized by default helpers.
        let mut source_options: sys::mln_source_feature_query_options =
            unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_source_feature_query_options_default(&mut source_options, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(source_options.size, 0);

        let point = sys::mln_screen_point { x: 1.0, y: 2.0 };
        // SAFETY: Zeroed storage is immediately initialized by constructor.
        let mut geometry: sys::mln_rendered_query_geometry = unsafe { std::mem::zeroed() };
        assert_eq!(
            mln_vala_rendered_query_geometry_point(&point, &mut geometry, ptr::null_mut()),
            GTRUE
        );
        assert_ne!(geometry.size, 0);
        assert_eq!(geometry.type_, sys::MLN_RENDERED_QUERY_GEOMETRY_TYPE_POINT);

        let mut error = ptr::null_mut();
        assert!(
            mln_vala_render_session_handle_query_rendered_features(
                ptr::null_mut(),
                &geometry,
                &rendered_options,
                &mut error,
            )
            .is_null()
        );
        assert!(!error.is_null());

        error = ptr::null_mut();
        assert_eq!(mln_vala_queried_feature_list_count(ptr::null()), 0);
        assert!(mln_vala_queried_feature_list_get(ptr::null(), 0, &mut error).is_null());
        assert!(!error.is_null());
        assert!(mln_vala_queried_feature_get_feature(ptr::null()).is_null());
        assert!(mln_vala_queried_feature_dup_source_id(ptr::null()).is_null());
        assert!(mln_vala_queried_feature_dup_source_layer_id(ptr::null()).is_null());
        assert!(mln_vala_queried_feature_get_state(ptr::null()).is_null());
        assert_eq!(
            mln_vala_feature_extension_result_get_result_type(ptr::null()),
            0
        );
        assert!(mln_vala_feature_extension_result_get_value(ptr::null()).is_null());
        assert!(mln_vala_feature_extension_result_get_feature_collection(ptr::null()).is_null());

        error = ptr::null_mut();
        assert_eq!(
            mln_vala_feature_query_result_handle_count(
                ptr::null_mut(),
                ptr::null_mut(),
                &mut error
            ),
            GFALSE
        );
        assert!(!error.is_null());
    }
}

use std::ffi::{CStr, c_char};
use std::ptr;

use maplibre_native_core::error::{self, Error};
use maplibre_native_sys as sys;

use crate::glib::{self, GBoolean, GError, GFALSE, GObject, GTRUE, GType};
use crate::render::{self, RenderSessionHandle};

const FEATURE_QUERY_RESULT_TYPE_NAME: &CStr = c"MlnValaFeatureQueryResultHandle";

#[repr(C)]
pub struct FeatureQueryResultHandle {
    parent_instance: GObject,
    native: *mut sys::mln_feature_query_result,
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_feature_query_result_handle_get_type() -> GType {
    glib::register_object_type::<FeatureQueryResultHandle>(FEATURE_QUERY_RESULT_TYPE_NAME)
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
) -> *mut FeatureQueryResultHandle {
    match query_rendered_features(session, geometry, options) {
        Ok(handle) => handle,
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
) -> *mut FeatureQueryResultHandle {
    match query_source_features(session, source_id, options) {
        Ok(handle) => handle,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
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
) -> error::Result<*mut FeatureQueryResultHandle> {
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
    wrap_feature_query_result(result)
}

fn query_source_features(
    session: *mut RenderSessionHandle,
    source_id: *const c_char,
    options: *const sys::mln_source_feature_query_options,
) -> error::Result<*mut FeatureQueryResultHandle> {
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
    wrap_feature_query_result(result)
}

fn wrap_feature_query_result(
    native: *mut sys::mln_feature_query_result,
) -> error::Result<*mut FeatureQueryResultHandle> {
    if native.is_null() {
        return Err(Error::invalid_argument(
            "native feature query result is null",
        ));
    }
    let handle = glib::new_object::<FeatureQueryResultHandle>(
        mln_vala_feature_query_result_handle_get_type(),
    );
    if handle.is_null() {
        // SAFETY: `native` came from a successful query operation.
        unsafe { sys::mln_feature_query_result_destroy(native) };
        return Err(Error::invalid_argument(
            "failed to allocate FeatureQueryResultHandle",
        ));
    }
    // SAFETY: `handle` points to a newly allocated query result wrapper.
    unsafe {
        (*handle).native = native;
    }
    Ok(handle)
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

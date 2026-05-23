use maplibre_native_core::error;
use maplibre_native_sys as sys;

use crate::glib::{self, GBoolean, GError, GFALSE, GTRUE};

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
    }
}

use std::ffi::CStr;
use std::ptr;

use maplibre_native_core::error::{self, Error};
use maplibre_native_sys as sys;

use crate::geo::{LatLng, ScreenPoint};
use crate::glib::{self, GBoolean, GError, GFALSE, GObject, GTRUE, GType};
use crate::handles::{self, MapHandle};

const PROJECTION_TYPE_NAME: &CStr = c"MlnValaMapProjectionHandle";

#[repr(C)]
pub struct MapProjectionHandle {
    parent_instance: GObject,
    native: *mut sys::mln_map_projection,
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_projection_handle_get_type() -> GType {
    glib::register_object_type::<MapProjectionHandle>(PROJECTION_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_projection_handle_new(
    map: *mut MapHandle,
    error_out: *mut *mut GError,
) -> *mut MapProjectionHandle {
    match create_projection_handle(map) {
        Ok(handle) => handle,
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_projection_handle_close(
    handle: *mut MapProjectionHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    match close_projection_handle(handle) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_projection_handle_pixel_for_lat_lng(
    handle: *mut MapProjectionHandle,
    coordinate: *const LatLng,
    out_point: *mut ScreenPoint,
    error_out: *mut *mut GError,
) -> GBoolean {
    match pixel_for_lat_lng(handle, coordinate, out_point) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_projection_handle_lat_lng_for_pixel(
    handle: *mut MapProjectionHandle,
    point: *const ScreenPoint,
    out_coordinate: *mut LatLng,
    error_out: *mut *mut GError,
) -> GBoolean {
    match lat_lng_for_pixel(handle, point, out_coordinate) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

fn create_projection_handle(map: *mut MapHandle) -> error::Result<*mut MapProjectionHandle> {
    let native_map = handles::map_native(map)?;
    let mut native = ptr::null_mut();
    // SAFETY: `native_map` is live and the out pointer is valid for this call.
    error::check(unsafe { sys::mln_map_projection_create(native_map, &mut native) })?;

    let handle = glib::new_object::<MapProjectionHandle>(mln_vala_map_projection_handle_get_type());
    if handle.is_null() {
        // SAFETY: `native` came from successful projection creation.
        let _ = error::check(unsafe { sys::mln_map_projection_destroy(native) });
        return Err(Error::invalid_argument(
            "failed to allocate MapProjectionHandle",
        ));
    }

    // SAFETY: `handle` points to an instance allocated with space for
    // `MapProjectionHandle`.
    unsafe {
        (*handle).native = native;
    }
    Ok(handle)
}

fn close_projection_handle(handle: *mut MapProjectionHandle) -> error::Result<()> {
    let projection = projection_native(handle)?;
    // SAFETY: `projection_native` returns a live native projection pointer.
    error::check(unsafe { sys::mln_map_projection_destroy(projection) })?;
    // SAFETY: `handle` was checked by `projection_native`.
    unsafe {
        (*handle).native = ptr::null_mut();
    }
    Ok(())
}

fn projection_native(
    handle: *mut MapProjectionHandle,
) -> error::Result<*mut sys::mln_map_projection> {
    if handle.is_null() {
        return Err(Error::invalid_argument("MapProjectionHandle is null"));
    }

    // SAFETY: `handle` is non-null and expected to point to
    // `MapProjectionHandle`.
    let native = unsafe { (*handle).native };
    if native.is_null() {
        return Err(Error::new(
            maplibre_native_core::error::ErrorKind::InvalidState,
            None,
            "MapProjectionHandle is closed",
        ));
    }
    Ok(native)
}

fn pixel_for_lat_lng(
    handle: *mut MapProjectionHandle,
    coordinate: *const LatLng,
    out_point: *mut ScreenPoint,
) -> error::Result<()> {
    if coordinate.is_null() {
        return Err(Error::invalid_argument("coordinate is null"));
    }
    let projection = projection_native(handle)?;
    // SAFETY: `coordinate` was null-checked and points to one LatLng.
    let coordinate = unsafe { *coordinate };
    let mut native_point = sys::mln_screen_point { x: 0.0, y: 0.0 };
    // SAFETY: `projection` is live and `native_point` out pointer is valid.
    error::check(unsafe {
        sys::mln_map_projection_pixel_for_lat_lng(projection, coordinate.into(), &mut native_point)
    })?;
    glib::clear_optional_out_pointer(out_point, native_point.into())
}

fn lat_lng_for_pixel(
    handle: *mut MapProjectionHandle,
    point: *const ScreenPoint,
    out_coordinate: *mut LatLng,
) -> error::Result<()> {
    if point.is_null() {
        return Err(Error::invalid_argument("screen point is null"));
    }
    let projection = projection_native(handle)?;
    // SAFETY: `point` was null-checked and points to one ScreenPoint.
    let point = unsafe { *point };
    let mut native_coordinate = sys::mln_lat_lng {
        latitude: 0.0,
        longitude: 0.0,
    };
    // SAFETY: `projection` is live and `native_coordinate` out pointer is valid.
    error::check(unsafe {
        sys::mln_map_projection_lat_lng_for_pixel(projection, point.into(), &mut native_coordinate)
    })?;
    glib::clear_optional_out_pointer(out_coordinate, native_coordinate.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handles::{
        mln_vala_map_handle_close, mln_vala_map_handle_new, mln_vala_runtime_handle_close,
        mln_vala_runtime_handle_new,
    };

    #[test]
    fn projection_converts_coordinate_and_pixel() {
        let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
        assert!(!runtime.is_null());
        let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
        assert!(!map.is_null());
        let projection = mln_vala_map_projection_handle_new(map, ptr::null_mut());
        assert!(!projection.is_null());

        let coordinate = LatLng {
            latitude: 0.0,
            longitude: 0.0,
        };
        let mut point = ScreenPoint { x: 0.0, y: 0.0 };
        assert_eq!(
            mln_vala_map_projection_handle_pixel_for_lat_lng(
                projection,
                &coordinate,
                &mut point,
                ptr::null_mut(),
            ),
            GTRUE
        );
        let mut round_trip = LatLng {
            latitude: 999.0,
            longitude: 999.0,
        };
        assert_eq!(
            mln_vala_map_projection_handle_lat_lng_for_pixel(
                projection,
                &point,
                &mut round_trip,
                ptr::null_mut(),
            ),
            GTRUE
        );

        assert_eq!(
            mln_vala_map_projection_handle_close(projection, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
        assert_eq!(
            mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
            GTRUE
        );

        glib::unref_object(projection);
        glib::unref_object(map);
        glib::unref_object(runtime);
    }
}

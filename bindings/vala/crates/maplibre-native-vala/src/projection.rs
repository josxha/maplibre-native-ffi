use std::collections::hash_map::DefaultHasher;
use std::ffi::CStr;
use std::hash::{Hash, Hasher};
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
    owner_thread: u64,
}

fn current_thread_token() -> u64 {
    let mut hasher = DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    hasher.finish()
}

fn projection_should_finalize_on_owner_thread(handle: *mut MapProjectionHandle) -> bool {
    if handle.is_null() {
        return false;
    }
    // SAFETY: The caller passes a MapProjectionHandle GObject during finalization.
    unsafe { !(*handle).native.is_null() && (*handle).owner_thread == current_thread_token() }
}

impl glib::ObjectFinalize for MapProjectionHandle {
    unsafe extern "C" fn finalize(object: *mut GObject) {
        let handle = object.cast::<MapProjectionHandle>();
        if projection_should_finalize_on_owner_thread(handle) {
            let _ = close_projection_handle(handle);
        } else if unsafe { !(*handle).native.is_null() } {
            eprintln!(
                "MapProjectionHandle finalized off its owner thread; call close() on the owner thread to release native state"
            );
        }
    }
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
pub extern "C" fn mln_vala_map_projection_handle_get_camera(
    handle: *mut MapProjectionHandle,
    out_camera: *mut sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match get_camera(handle, out_camera) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_projection_handle_set_camera(
    handle: *mut MapProjectionHandle,
    camera: *const sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_camera(handle, camera) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_projection_handle_set_visible_coordinates(
    handle: *mut MapProjectionHandle,
    coordinates: *const LatLng,
    coordinate_count: usize,
    padding: *const sys::mln_edge_insets,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_visible_coordinates(handle, coordinates, coordinate_count, padding) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_projection_handle_set_visible_geometry(
    handle: *mut MapProjectionHandle,
    geometry: *const sys::mln_geometry,
    padding: *const sys::mln_edge_insets,
    error_out: *mut *mut GError,
) -> GBoolean {
    match set_visible_geometry(handle, geometry, padding) {
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
        (*handle).owner_thread = current_thread_token();
    }
    Ok(handle)
}

fn close_projection_handle(handle: *mut MapProjectionHandle) -> error::Result<()> {
    if handle.is_null() {
        return Err(Error::invalid_argument("MapProjectionHandle is null"));
    }
    // SAFETY: `handle` is non-null and expected to point to MapProjectionHandle.
    let projection = unsafe { (*handle).native };
    if projection.is_null() {
        return Ok(());
    }
    // SAFETY: `projection` is live and owned by this handle.
    error::check(unsafe { sys::mln_map_projection_destroy(projection) })?;
    // SAFETY: `handle` was checked above.
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

fn get_camera(
    handle: *mut MapProjectionHandle,
    out_camera: *mut sys::mln_camera_options,
) -> error::Result<()> {
    if out_camera.is_null() {
        return Err(Error::invalid_argument("camera output pointer is null"));
    }
    let projection = projection_native(handle)?;
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mut camera = unsafe { sys::mln_camera_options_default() };
    // SAFETY: `projection` is live and `camera` is writable output storage.
    error::check(unsafe { sys::mln_map_projection_get_camera(projection, &mut camera) })?;
    glib::clear_optional_out_pointer(out_camera, camera)
}

fn set_camera(
    handle: *mut MapProjectionHandle,
    camera: *const sys::mln_camera_options,
) -> error::Result<()> {
    if camera.is_null() {
        return Err(Error::invalid_argument("camera options are null"));
    }
    let projection = projection_native(handle)?;
    // SAFETY: `projection` is live and `camera` points to a borrowed descriptor
    // for this call.
    error::check(unsafe { sys::mln_map_projection_set_camera(projection, camera) })
}

fn set_visible_coordinates(
    handle: *mut MapProjectionHandle,
    coordinates: *const LatLng,
    coordinate_count: usize,
    padding: *const sys::mln_edge_insets,
) -> error::Result<()> {
    if coordinates.is_null() {
        return Err(Error::invalid_argument("visible coordinates are null"));
    }
    if padding.is_null() {
        return Err(Error::invalid_argument(
            "visible coordinate padding is null",
        ));
    }
    let projection = projection_native(handle)?;
    // SAFETY: `LatLng` is repr-compatible with `mln_lat_lng`; all pointers are
    // borrowed for this call and validated by the C API.
    error::check(unsafe {
        sys::mln_map_projection_set_visible_coordinates(
            projection,
            coordinates.cast::<sys::mln_lat_lng>(),
            coordinate_count,
            *padding,
        )
    })
}

fn set_visible_geometry(
    handle: *mut MapProjectionHandle,
    geometry: *const sys::mln_geometry,
    padding: *const sys::mln_edge_insets,
) -> error::Result<()> {
    if geometry.is_null() {
        return Err(Error::invalid_argument("geometry is null"));
    }
    if padding.is_null() {
        return Err(Error::invalid_argument("padding is null"));
    }
    let projection = projection_native(handle)?;
    // SAFETY: `projection` is live, geometry and padding are borrowed for this call.
    let padding = unsafe { *padding };
    error::check(unsafe {
        sys::mln_map_projection_set_visible_geometry(projection, geometry, padding)
    })
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

        // SAFETY: Zeroed storage is immediately initialized by default helper.
        let mut camera: sys::mln_camera_options = unsafe { std::mem::zeroed() };
        assert_eq!(
            crate::handles::mln_vala_camera_options_default(&mut camera, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_projection_handle_get_camera(projection, &mut camera, ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_map_projection_handle_set_camera(projection, &camera, ptr::null_mut()),
            GTRUE
        );

        let coordinate = LatLng {
            latitude: 0.0,
            longitude: 0.0,
        };
        let visible_coordinates = [coordinate];
        let padding = sys::mln_edge_insets {
            top: 0.0,
            left: 0.0,
            bottom: 0.0,
            right: 0.0,
        };
        assert_eq!(
            mln_vala_map_projection_handle_set_visible_coordinates(
                projection,
                visible_coordinates.as_ptr(),
                visible_coordinates.len(),
                &padding,
                ptr::null_mut(),
            ),
            GTRUE
        );
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

use maplibre_native_core::error;
use maplibre_native_sys as sys;

use crate::glib::{self, GBoolean, GError, GFALSE, GTRUE};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LatLng {
    pub latitude: f64,
    pub longitude: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectedMeters {
    pub northing: f64,
    pub easting: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}

impl From<LatLng> for sys::mln_lat_lng {
    fn from(value: LatLng) -> Self {
        Self {
            latitude: value.latitude,
            longitude: value.longitude,
        }
    }
}

impl From<sys::mln_lat_lng> for LatLng {
    fn from(value: sys::mln_lat_lng) -> Self {
        Self {
            latitude: value.latitude,
            longitude: value.longitude,
        }
    }
}

impl From<ProjectedMeters> for sys::mln_projected_meters {
    fn from(value: ProjectedMeters) -> Self {
        Self {
            northing: value.northing,
            easting: value.easting,
        }
    }
}

impl From<sys::mln_projected_meters> for ProjectedMeters {
    fn from(value: sys::mln_projected_meters) -> Self {
        Self {
            northing: value.northing,
            easting: value.easting,
        }
    }
}

impl From<ScreenPoint> for sys::mln_screen_point {
    fn from(value: ScreenPoint) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<sys::mln_screen_point> for ScreenPoint {
    fn from(value: sys::mln_screen_point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_projected_meters_for_lat_lng(
    coordinate: *const LatLng,
    out_meters: *mut ProjectedMeters,
    error_out: *mut *mut GError,
) -> GBoolean {
    match projected_meters_for_lat_lng(coordinate, out_meters) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_lat_lng_for_projected_meters(
    meters: *const ProjectedMeters,
    out_coordinate: *mut LatLng,
    error_out: *mut *mut GError,
) -> GBoolean {
    match lat_lng_for_projected_meters(meters, out_coordinate) {
        Ok(()) => GTRUE,
        Err(error) => {
            glib::set_error(error_out, error);
            GFALSE
        }
    }
}

fn projected_meters_for_lat_lng(
    coordinate: *const LatLng,
    out_meters: *mut ProjectedMeters,
) -> error::Result<()> {
    if coordinate.is_null() {
        return Err(maplibre_native_core::error::Error::invalid_argument(
            "coordinate is null",
        ));
    }
    // SAFETY: The null check above proves `coordinate` points to one LatLng for
    // the duration of this call.
    let coordinate = unsafe { *coordinate };
    let mut native_meters = sys::mln_projected_meters {
        northing: 0.0,
        easting: 0.0,
    };
    // SAFETY: `native_meters` out pointer is valid for this call; C validates
    // coordinate domain rules.
    error::check(unsafe {
        sys::mln_projected_meters_for_lat_lng(coordinate.into(), &mut native_meters)
    })?;
    glib::clear_optional_out_pointer(out_meters, native_meters.into())
}

fn lat_lng_for_projected_meters(
    meters: *const ProjectedMeters,
    out_coordinate: *mut LatLng,
) -> error::Result<()> {
    if meters.is_null() {
        return Err(maplibre_native_core::error::Error::invalid_argument(
            "projected meters is null",
        ));
    }
    // SAFETY: The null check above proves `meters` points to one
    // ProjectedMeters for the duration of this call.
    let meters = unsafe { *meters };
    let mut native_coordinate = sys::mln_lat_lng {
        latitude: 0.0,
        longitude: 0.0,
    };
    // SAFETY: `native_coordinate` out pointer is valid for this call; C
    // validates projected meter domain rules.
    error::check(unsafe {
        sys::mln_lat_lng_for_projected_meters(meters.into(), &mut native_coordinate)
    })?;
    glib::clear_optional_out_pointer(out_coordinate, native_coordinate.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projected_meters_round_trip() {
        let coordinate = LatLng {
            latitude: 37.7749,
            longitude: -122.4194,
        };
        let mut meters = ProjectedMeters {
            northing: 0.0,
            easting: 0.0,
        };
        let mut round_trip = LatLng {
            latitude: 0.0,
            longitude: 0.0,
        };

        assert_eq!(
            mln_vala_projected_meters_for_lat_lng(&coordinate, &mut meters, std::ptr::null_mut()),
            GTRUE
        );
        assert_eq!(
            mln_vala_lat_lng_for_projected_meters(&meters, &mut round_trip, std::ptr::null_mut()),
            GTRUE
        );
        assert!((round_trip.latitude - coordinate.latitude).abs() < 0.000_001);
        assert!((round_trip.longitude - coordinate.longitude).abs() < 0.000_001);
    }

    #[test]
    fn invalid_coordinate_reports_gerror() {
        let coordinate = LatLng {
            latitude: 100.0,
            longitude: 0.0,
        };
        let mut meters = ProjectedMeters {
            northing: 0.0,
            easting: 0.0,
        };
        let mut error = std::ptr::null_mut();

        assert_eq!(
            mln_vala_projected_meters_for_lat_lng(&coordinate, &mut meters, &mut error),
            GFALSE
        );
        assert!(!error.is_null());
    }
}

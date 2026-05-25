use super::*;

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_definition_get_type() -> GType {
    glib::register_boxed_type(
        OFFLINE_REGION_DEFINITION_TYPE_NAME,
        offline_region_definition_copy_erased,
        offline_region_definition_free_erased,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_info_get_type() -> GType {
    glib::register_boxed_type(
        OFFLINE_REGION_INFO_TYPE_NAME,
        offline_region_info_copy_erased,
        offline_region_info_free_erased,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_info_list_get_type() -> GType {
    glib::register_boxed_type(
        OFFLINE_REGION_INFO_LIST_TYPE_NAME,
        offline_region_info_list_copy_erased,
        offline_region_info_list_free_erased,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_id_list_handle_get_type() -> GType {
    glib::register_object_type::<StyleIdListHandle>(STYLE_ID_LIST_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_snapshot_handle_get_type() -> GType {
    glib::register_object_type::<JsonSnapshotHandle>(JSON_SNAPSHOT_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_snapshot_handle_get_type() -> GType {
    glib::register_object_type::<OfflineRegionSnapshotHandle>(OFFLINE_REGION_SNAPSHOT_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_list_handle_get_type() -> GType {
    glib::register_object_type::<OfflineRegionListHandle>(OFFLINE_REGION_LIST_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_definition_copy(
    value: *const OfflineRegionDefinition,
) -> *mut OfflineRegionDefinition {
    offline_region_definition_ref(value)
        .map(|value| Box::into_raw(Box::new(value.clone())))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn mln_vala_offline_region_definition_free(value: *mut OfflineRegionDefinition) {
    if !value.is_null() {
        unsafe { drop(Box::from_raw(value)) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_info_copy(
    value: *const OfflineRegionInfo,
) -> *mut OfflineRegionInfo {
    offline_region_info_ref(value)
        .map(|value| Box::into_raw(Box::new(value.clone())))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn mln_vala_offline_region_info_free(value: *mut OfflineRegionInfo) {
    if !value.is_null() {
        unsafe { drop(Box::from_raw(value)) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_info_list_copy(
    value: *const OfflineRegionInfoList,
) -> *mut OfflineRegionInfoList {
    offline_region_info_list_ref(value)
        .map(|value| Box::into_raw(Box::new(value.clone())))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn mln_vala_offline_region_info_list_free(value: *mut OfflineRegionInfoList) {
    if !value.is_null() {
        unsafe { drop(Box::from_raw(value)) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_definition_new_tile_pyramid(
    style_url: *const c_char,
    bounds: *const sys::mln_lat_lng_bounds,
    min_zoom: f64,
    max_zoom: f64,
    pixel_ratio: f32,
    include_ideographs: bool,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionDefinition {
    match offline_region_definition_new_tile_pyramid(
        style_url,
        bounds,
        min_zoom,
        max_zoom,
        pixel_ratio,
        include_ideographs,
    ) {
        Ok(value) => Box::into_raw(Box::new(value)),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_definition_new_geometry(
    style_url: *const c_char,
    geometry: *const Geometry,
    min_zoom: f64,
    max_zoom: f64,
    pixel_ratio: f32,
    include_ideographs: bool,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionDefinition {
    match offline_region_definition_new_geometry(
        style_url,
        geometry,
        min_zoom,
        max_zoom,
        pixel_ratio,
        include_ideographs,
    ) {
        Ok(value) => Box::into_raw(Box::new(value)),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_definition_get_definition_type(
    value: *const OfflineRegionDefinition,
) -> sys::mln_offline_region_definition_type {
    match offline_region_definition_ref(value).map(|value| &value.value) {
        Some(core::OfflineRegionDefinition::TilePyramid { .. }) => {
            sys::MLN_OFFLINE_REGION_DEFINITION_TILE_PYRAMID
        }
        Some(core::OfflineRegionDefinition::GeometryRegion { .. }) => {
            sys::MLN_OFFLINE_REGION_DEFINITION_GEOMETRY
        }
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_definition_dup_style_url(
    value: *const OfflineRegionDefinition,
) -> *mut c_char {
    offline_region_definition_ref(value)
        .and_then(|value| match &value.value {
            core::OfflineRegionDefinition::TilePyramid { style_url, .. }
            | core::OfflineRegionDefinition::GeometryRegion { style_url, .. } => {
                Some(style_url.as_str())
            }
            _ => None,
        })
        .and_then(copy_string)
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_definition_get_bounds(
    value: *const OfflineRegionDefinition,
    out_bounds: *mut sys::mln_lat_lng_bounds,
) -> GBoolean {
    let Some(value) = offline_region_definition_ref(value) else {
        return GFALSE;
    };
    let core::OfflineRegionDefinition::TilePyramid { bounds, .. } = &value.value else {
        return GFALSE;
    };
    glib::clear_optional_out_pointer(out_bounds, core::values::lat_lng_bounds_to_native(*bounds))
        .map(|()| GTRUE)
        .unwrap_or(GFALSE)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_definition_get_geometry(
    value: *const OfflineRegionDefinition,
) -> *mut Geometry {
    match offline_region_definition_ref(value).map(|value| &value.value) {
        Some(core::OfflineRegionDefinition::GeometryRegion { geometry, .. }) => {
            Box::into_raw(Box::new(Geometry {
                value: geometry.clone(),
            }))
        }
        _ => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_definition_get_min_zoom(
    value: *const OfflineRegionDefinition,
) -> f64 {
    offline_region_definition_ref(value).map_or(0.0, |value| match &value.value {
        core::OfflineRegionDefinition::TilePyramid { min_zoom, .. }
        | core::OfflineRegionDefinition::GeometryRegion { min_zoom, .. } => *min_zoom,
        _ => 0.0,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_definition_get_max_zoom(
    value: *const OfflineRegionDefinition,
) -> f64 {
    offline_region_definition_ref(value).map_or(0.0, |value| match &value.value {
        core::OfflineRegionDefinition::TilePyramid { max_zoom, .. }
        | core::OfflineRegionDefinition::GeometryRegion { max_zoom, .. } => *max_zoom,
        _ => 0.0,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_definition_get_pixel_ratio(
    value: *const OfflineRegionDefinition,
) -> f32 {
    offline_region_definition_ref(value).map_or(0.0, |value| match &value.value {
        core::OfflineRegionDefinition::TilePyramid { pixel_ratio, .. }
        | core::OfflineRegionDefinition::GeometryRegion { pixel_ratio, .. } => *pixel_ratio,
        _ => 0.0,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_definition_get_include_ideographs(
    value: *const OfflineRegionDefinition,
) -> bool {
    offline_region_definition_ref(value).is_some_and(|value| match &value.value {
        core::OfflineRegionDefinition::TilePyramid {
            include_ideographs, ..
        }
        | core::OfflineRegionDefinition::GeometryRegion {
            include_ideographs, ..
        } => *include_ideographs,
        _ => false,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_info_get_id(value: *const OfflineRegionInfo) -> i64 {
    offline_region_info_ref(value).map_or(0, |value| value.value.id)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_info_get_definition(
    value: *const OfflineRegionInfo,
) -> *mut OfflineRegionDefinition {
    offline_region_info_ref(value).map_or(ptr::null_mut(), |value| {
        Box::into_raw(Box::new(OfflineRegionDefinition {
            value: value.value.definition.clone(),
        }))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_info_dup_metadata(
    value: *const OfflineRegionInfo,
) -> *mut glib::GBytes {
    offline_region_info_ref(value)
        .map(|value| glib::bytes_new(&value.value.metadata))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_info_list_count(
    value: *const OfflineRegionInfoList,
) -> usize {
    offline_region_info_list_ref(value).map_or(0, |value| value.regions.len())
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_info_list_get(
    value: *const OfflineRegionInfoList,
    index: usize,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionInfo {
    match offline_region_info_list_get(value, index) {
        Ok(info) => Box::into_raw(Box::new(info)),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

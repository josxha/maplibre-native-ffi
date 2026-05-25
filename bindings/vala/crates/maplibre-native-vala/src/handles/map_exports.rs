use super::*;

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_get_type() -> GType {
    glib::register_object_type::<RuntimeHandle>(RUNTIME_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_type() -> GType {
    glib::register_object_type::<MapHandle>(MAP_TYPE_NAME)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_options_default(
    out_options: *mut sys::mln_runtime_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_runtime_options(out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_options_default(
    out_options: *mut sys::mln_map_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_map_options(out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_new(
    runtime: *mut RuntimeHandle,
    width: u32,
    height: u32,
    scale_factor: f64,
    error_out: *mut *mut GError,
) -> *mut MapHandle {
    gerror_pointer(
        create_map_handle(runtime, width, height, scale_factor),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_new_with_options(
    runtime: *mut RuntimeHandle,
    options: *const sys::mln_map_options,
    error_out: *mut *mut GError,
) -> *mut MapHandle {
    gerror_pointer(create_map_handle_with_options(runtime, options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_close(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(close_map_handle(handle), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_request_repaint(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(request_repaint(handle), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_request_still_image(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(request_still_image(handle), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_debug_options(
    handle: *mut MapHandle,
    options: u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_debug_options(handle, options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_debug_options(
    handle: *mut MapHandle,
    out_options: *mut u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(get_debug_options(handle, out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_rendering_stats_view_enabled(
    handle: *mut MapHandle,
    enabled: GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        set_rendering_stats_view_enabled(handle, enabled != GFALSE),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_rendering_stats_view_enabled(
    handle: *mut MapHandle,
    out_enabled: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        get_rendering_stats_view_enabled(handle, out_enabled),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_is_fully_loaded(
    handle: *mut MapHandle,
    out_loaded: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(is_fully_loaded(handle, out_loaded), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_dump_debug_logs(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(dump_debug_logs(handle), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_camera(
    handle: *mut MapHandle,
    out_camera: *mut sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(get_camera(handle, out_camera), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_viewport_options(
    handle: *mut MapHandle,
    out_options: *mut sys::mln_map_viewport_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(get_viewport_options(handle, out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_viewport_options(
    handle: *mut MapHandle,
    options: *const sys::mln_map_viewport_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_viewport_options(handle, options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_tile_options(
    handle: *mut MapHandle,
    out_options: *mut sys::mln_map_tile_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(get_tile_options(handle, out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_tile_options(
    handle: *mut MapHandle,
    options: *const sys::mln_map_tile_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_tile_options(handle, options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_bounds(
    handle: *mut MapHandle,
    out_options: *mut sys::mln_bound_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(get_bounds(handle, out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_bounds(
    handle: *mut MapHandle,
    options: *const sys::mln_bound_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_bounds(handle, options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_free_camera_options(
    handle: *mut MapHandle,
    out_options: *mut sys::mln_free_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(get_free_camera_options(handle, out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_free_camera_options(
    handle: *mut MapHandle,
    options: *const sys::mln_free_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_free_camera_options(handle, options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_projection_mode(
    handle: *mut MapHandle,
    out_mode: *mut sys::mln_projection_mode,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(get_projection_mode(handle, out_mode), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_projection_mode(
    handle: *mut MapHandle,
    mode: *const sys::mln_projection_mode,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_projection_mode(handle, mode), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_camera_for_lat_lng_bounds(
    handle: *mut MapHandle,
    bounds: *const sys::mln_lat_lng_bounds,
    fit_options: *const sys::mln_camera_fit_options,
    out_camera: *mut sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        camera_for_lat_lng_bounds(handle, bounds, fit_options, out_camera),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_camera_for_lat_lngs(
    handle: *mut MapHandle,
    coordinates: *const LatLng,
    coordinate_count: usize,
    fit_options: *const sys::mln_camera_fit_options,
    out_camera: *mut sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        camera_for_lat_lngs(
            handle,
            coordinates,
            coordinate_count,
            fit_options,
            out_camera,
        ),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_camera_for_geometry(
    handle: *mut MapHandle,
    geometry: *const Geometry,
    fit_options: *const sys::mln_camera_fit_options,
    out_camera: *mut sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        camera_for_geometry(handle, geometry, fit_options, out_camera),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_lat_lng_bounds_for_camera(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    out_bounds: *mut sys::mln_lat_lng_bounds,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        lat_lng_bounds_for_camera(handle, camera, out_bounds, false),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_lat_lng_bounds_for_camera_unwrapped(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    out_bounds: *mut sys::mln_lat_lng_bounds,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        lat_lng_bounds_for_camera(handle, camera, out_bounds, true),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_jump_to(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(jump_to(handle, camera), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_ease_to(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    animation: *const sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(ease_to(handle, camera, animation), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_fly_to(
    handle: *mut MapHandle,
    camera: *const sys::mln_camera_options,
    animation: *const sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(fly_to(handle, camera, animation), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_move_by(
    handle: *mut MapHandle,
    delta_x: f64,
    delta_y: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(move_by(handle, delta_x, delta_y), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_move_by_animated(
    handle: *mut MapHandle,
    delta_x: f64,
    delta_y: f64,
    animation: *const sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        move_by_animated(handle, delta_x, delta_y, animation),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_scale_by(
    handle: *mut MapHandle,
    scale: f64,
    anchor: *const sys::mln_screen_point,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(scale_by(handle, scale, anchor), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_scale_by_animated(
    handle: *mut MapHandle,
    scale: f64,
    anchor: *const sys::mln_screen_point,
    animation: *const sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        scale_by_animated(handle, scale, anchor, animation),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_rotate_by(
    handle: *mut MapHandle,
    first: *const sys::mln_screen_point,
    second: *const sys::mln_screen_point,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(rotate_by(handle, first, second), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_rotate_by_animated(
    handle: *mut MapHandle,
    first: *const sys::mln_screen_point,
    second: *const sys::mln_screen_point,
    animation: *const sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        rotate_by_animated(handle, first, second, animation),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_pitch_by(
    handle: *mut MapHandle,
    pitch: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(pitch_by(handle, pitch), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_pitch_by_animated(
    handle: *mut MapHandle,
    pitch: f64,
    animation: *const sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(pitch_by_animated(handle, pitch, animation), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_cancel_transitions(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(cancel_transitions(handle), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_pixel_for_lat_lng(
    handle: *mut MapHandle,
    coordinate: *const LatLng,
    out_point: *mut ScreenPoint,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        map_pixel_for_lat_lng(handle, coordinate, out_point),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_lat_lng_for_pixel(
    handle: *mut MapHandle,
    point: *const ScreenPoint,
    out_coordinate: *mut LatLng,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        map_lat_lng_for_pixel(handle, point, out_coordinate),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_pixels_for_lat_lngs(
    handle: *mut MapHandle,
    coordinates: *const LatLng,
    coordinate_count: usize,
    out_points: *mut ScreenPoint,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        map_pixels_for_lat_lngs(handle, coordinates, coordinate_count, out_points),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_lat_lngs_for_pixels(
    handle: *mut MapHandle,
    points: *const ScreenPoint,
    point_count: usize,
    out_coordinates: *mut LatLng,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        map_lat_lngs_for_pixels(handle, points, point_count, out_coordinates),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_style_url(
    handle: *mut MapHandle,
    url: *const std::ffi::c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_style_url(handle, url), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_style_json(
    handle: *mut MapHandle,
    json: *const std::ffi::c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_style_json(handle, json), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_geojson_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(add_geojson_source_url(handle, source_id, url), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_geojson_source_data(
    handle: *mut MapHandle,
    source_id: *const c_char,
    data: *const GeoJson,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(add_geojson_source_data(handle, source_id, data), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_geojson_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_geojson_source_url(handle, source_id, url), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_geojson_source_data(
    handle: *mut MapHandle,
    source_id: *const c_char,
    data: *const GeoJson,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_geojson_source_data(handle, source_id, data), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_vector_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    options: *const sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_tile_source_url(handle, source_id, url, options, TileSourceKind::Vector),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_raster_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    options: *const sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_tile_source_url(handle, source_id, url, options, TileSourceKind::Raster),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_raster_dem_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    options: *const sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_tile_source_url(handle, source_id, url, options, TileSourceKind::RasterDem),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_vector_source_tiles(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tiles: *const StringList,
    options: *const sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_tile_source_tiles(handle, source_id, tiles, options, TileSourceKind::Vector),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_raster_source_tiles(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tiles: *const StringList,
    options: *const sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_tile_source_tiles(handle, source_id, tiles, options, TileSourceKind::Raster),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_raster_dem_source_tiles(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tiles: *const StringList,
    options: *const sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_tile_source_tiles(handle, source_id, tiles, options, TileSourceKind::RasterDem),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_custom_geometry_source(
    handle: *mut MapHandle,
    source_id: *const c_char,
    options: *const sys::mln_custom_geometry_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_custom_geometry_source(handle, source_id, options),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_custom_geometry_source_with_callbacks(
    handle: *mut MapHandle,
    source_id: *const c_char,
    fetch_tile: Option<CustomGeometrySourceTileDelegate>,
    fetch_user_data: *mut c_void,
    fetch_destroy_notify: GDestroyNotify,
    cancel_tile: Option<CustomGeometrySourceTileDelegate>,
    cancel_user_data: *mut c_void,
    cancel_destroy_notify: GDestroyNotify,
    options: *const sys::mln_custom_geometry_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_custom_geometry_source_with_callbacks(
            handle,
            source_id,
            fetch_tile,
            fetch_user_data,
            fetch_destroy_notify,
            cancel_tile,
            cancel_user_data,
            cancel_destroy_notify,
            options,
        ),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_custom_geometry_source_tile_data(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tile_id: *const sys::mln_canonical_tile_id,
    data: *const GeoJson,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        set_custom_geometry_source_tile_data(handle, source_id, tile_id, data),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_invalidate_custom_geometry_source_tile(
    handle: *mut MapHandle,
    source_id: *const c_char,
    tile_id: *const sys::mln_canonical_tile_id,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        invalidate_custom_geometry_source_tile(handle, source_id, tile_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_invalidate_custom_geometry_source_region(
    handle: *mut MapHandle,
    source_id: *const c_char,
    bounds: *const sys::mln_lat_lng_bounds,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        invalidate_custom_geometry_source_region(handle, source_id, bounds),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_style_source_exists(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_exists: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        style_source_exists(handle, source_id, out_exists),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_style_source_type(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_source_type: *mut u32,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        get_style_source_type(handle, source_id, out_source_type, out_found),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_style_source_info(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_info: *mut sys::mln_style_source_info,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        get_style_source_info(handle, source_id, out_info, out_found),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_copy_style_source_attribution(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_attribution: *mut c_char,
    attribution_capacity: usize,
    out_attribution_size: *mut usize,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        copy_style_source_attribution(
            handle,
            source_id,
            out_attribution,
            attribution_capacity,
            out_attribution_size,
            out_found,
        ),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_remove_style_source(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_removed: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        remove_style_source(handle, source_id, out_removed),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_style_image(
    handle: *mut MapHandle,
    image_id: *const c_char,
    image: *const sys::mln_premultiplied_rgba8_image,
    options: *const sys::mln_style_image_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_style_image(handle, image_id, image, options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_remove_style_image(
    handle: *mut MapHandle,
    image_id: *const c_char,
    out_removed: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(remove_style_image(handle, image_id, out_removed), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_style_image_exists(
    handle: *mut MapHandle,
    image_id: *const c_char,
    out_exists: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(style_image_exists(handle, image_id, out_exists), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_style_image_info(
    handle: *mut MapHandle,
    image_id: *const c_char,
    out_info: *mut sys::mln_style_image_info,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        get_style_image_info(handle, image_id, out_info, out_found),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_copy_style_image_premultiplied_rgba8(
    handle: *mut MapHandle,
    image_id: *const c_char,
    out_pixels: *mut u8,
    pixel_capacity: usize,
    out_byte_length: *mut usize,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        copy_style_image_premultiplied_rgba8(
            handle,
            image_id,
            out_pixels,
            pixel_capacity,
            out_byte_length,
            out_found,
        ),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_image_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    coordinates: *const sys::mln_lat_lng,
    coordinate_count: usize,
    url: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_image_source_url(handle, source_id, coordinates, coordinate_count, url),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_image_source_image(
    handle: *mut MapHandle,
    source_id: *const c_char,
    coordinates: *const sys::mln_lat_lng,
    coordinate_count: usize,
    image: *const sys::mln_premultiplied_rgba8_image,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_image_source_image(handle, source_id, coordinates, coordinate_count, image),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_image_source_url(
    handle: *mut MapHandle,
    source_id: *const c_char,
    url: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_image_source_url(handle, source_id, url), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_image_source_image(
    handle: *mut MapHandle,
    source_id: *const c_char,
    image: *const sys::mln_premultiplied_rgba8_image,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_image_source_image(handle, source_id, image), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_image_source_coordinates(
    handle: *mut MapHandle,
    source_id: *const c_char,
    coordinates: *const sys::mln_lat_lng,
    coordinate_count: usize,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        set_image_source_coordinates(handle, source_id, coordinates, coordinate_count),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_image_source_coordinates(
    handle: *mut MapHandle,
    source_id: *const c_char,
    out_coordinates: *mut sys::mln_lat_lng,
    coordinate_capacity: usize,
    out_coordinate_count: *mut usize,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        get_image_source_coordinates(
            handle,
            source_id,
            out_coordinates,
            coordinate_capacity,
            out_coordinate_count,
            out_found,
        ),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_hillshade_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    source_id: *const c_char,
    before_layer_id: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_dem_layer(
            handle,
            layer_id,
            source_id,
            before_layer_id,
            DemLayerKind::Hillshade,
        ),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_color_relief_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    source_id: *const c_char,
    before_layer_id: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_dem_layer(
            handle,
            layer_id,
            source_id,
            before_layer_id,
            DemLayerKind::ColorRelief,
        ),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_location_indicator_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    before_layer_id: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_location_indicator_layer(handle, layer_id, before_layer_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_location_indicator_location(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    coordinate: *const sys::mln_lat_lng,
    altitude: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        set_location_indicator_location(handle, layer_id, coordinate, altitude),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_location_indicator_bearing(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    bearing: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        set_location_indicator_bearing(handle, layer_id, bearing),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_location_indicator_accuracy_radius(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    radius: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        set_location_indicator_accuracy_radius(handle, layer_id, radius),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_location_indicator_image_name(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    image_kind: u32,
    image_id: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        set_location_indicator_image_name(handle, layer_id, image_kind, image_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_style_source_json(
    handle: *mut MapHandle,
    source_id: *const c_char,
    source_json: *const JsonValue,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_style_source_json(handle, source_id, source_json),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_add_style_layer_json(
    handle: *mut MapHandle,
    layer_json: *const JsonValue,
    before_layer_id: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        add_style_layer_json(handle, layer_json, before_layer_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_style_layer_exists(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    out_exists: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(style_layer_exists(handle, layer_id, out_exists), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_move_style_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    before_layer_id: *const c_char,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        move_style_layer(handle, layer_id, before_layer_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_remove_style_layer(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    out_removed: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(remove_style_layer(handle, layer_id, out_removed), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_style_layer_type(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    out_layer_type: *mut *mut c_char,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        get_style_layer_type(handle, layer_id, out_layer_type, out_found),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_list_style_source_ids(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> *mut StringList {
    match list_style_source_ids(handle) {
        Ok(list) => Box::into_raw(Box::new(list)),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_style_layer_json(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> *mut JsonValue {
    match get_style_layer_json(handle, layer_id, out_found) {
        Ok(value) => value.map_or(ptr::null_mut(), |value| {
            Box::into_raw(Box::new(JsonValue { value }))
        }),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_style_light_json(
    handle: *mut MapHandle,
    light_json: *const JsonValue,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_style_light_json(handle, light_json), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_style_light_property(
    handle: *mut MapHandle,
    property_name: *const c_char,
    value: *const JsonValue,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        set_style_light_property(handle, property_name, value),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_style_light_property(
    handle: *mut MapHandle,
    property_name: *const c_char,
    error_out: *mut *mut GError,
) -> *mut JsonValue {
    match get_style_light_property(handle, property_name) {
        Ok(value) => value.map_or(ptr::null_mut(), |value| {
            Box::into_raw(Box::new(JsonValue { value }))
        }),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_layer_property(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    property_name: *const c_char,
    value: *const JsonValue,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        set_layer_property(handle, layer_id, property_name, value),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_layer_property(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    property_name: *const c_char,
    error_out: *mut *mut GError,
) -> *mut JsonValue {
    match get_layer_property(handle, layer_id, property_name) {
        Ok(value) => value.map_or(ptr::null_mut(), |value| {
            Box::into_raw(Box::new(JsonValue { value }))
        }),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_set_layer_filter(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    filter: *const JsonValue,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_layer_filter(handle, layer_id, filter), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_get_layer_filter(
    handle: *mut MapHandle,
    layer_id: *const c_char,
    error_out: *mut *mut GError,
) -> *mut JsonValue {
    match get_layer_filter(handle, layer_id) {
        Ok(value) => value.map_or(ptr::null_mut(), |value| {
            Box::into_raw(Box::new(JsonValue { value }))
        }),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_handle_list_style_layer_ids(
    handle: *mut MapHandle,
    error_out: *mut *mut GError,
) -> *mut StringList {
    match list_style_layer_ids(handle) {
        Ok(list) => Box::into_raw(Box::new(list)),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_id_list_handle_count(
    handle: *mut StyleIdListHandle,
    out_count: *mut usize,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(style_id_list_count(handle, out_count), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_id_list_handle_get(
    handle: *mut StyleIdListHandle,
    index: usize,
    error_out: *mut *mut GError,
) -> *mut c_char {
    gerror_pointer(style_id_list_get(handle, index), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_id_list_handle_close(handle: *mut StyleIdListHandle) {
    close_style_id_list(handle);
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_snapshot_handle_get(
    handle: *mut JsonSnapshotHandle,
    out_value: *mut *mut JsonValue,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(json_snapshot_get(handle, out_value), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_json_snapshot_handle_close(handle: *mut JsonSnapshotHandle) {
    close_json_snapshot(handle);
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_snapshot_handle_get(
    handle: *mut OfflineRegionSnapshotHandle,
    out_info: *mut sys::mln_offline_region_info,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(offline_region_snapshot_get(handle, out_info), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_snapshot_handle_close(
    handle: *mut OfflineRegionSnapshotHandle,
) {
    close_offline_region_snapshot(handle);
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_list_handle_count(
    handle: *mut OfflineRegionListHandle,
    out_count: *mut usize,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(offline_region_list_count(handle, out_count), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_list_handle_get(
    handle: *mut OfflineRegionListHandle,
    index: usize,
    out_info: *mut sys::mln_offline_region_info,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(offline_region_list_get(handle, index, out_info), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_offline_region_list_handle_close(handle: *mut OfflineRegionListHandle) {
    close_offline_region_list(handle);
}

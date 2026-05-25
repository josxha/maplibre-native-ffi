use super::*;

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_camera_options_default(
    out_options: *mut sys::mln_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_camera_options(out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_animation_options_default(
    out_options: *mut sys::mln_animation_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_animation_options(out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_camera_fit_options_default(
    out_options: *mut sys::mln_camera_fit_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_camera_fit_options(out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_bound_options_default(
    out_options: *mut sys::mln_bound_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_bound_options(out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_free_camera_options_default(
    out_options: *mut sys::mln_free_camera_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_free_camera_options(out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_projection_mode_default(
    out_mode: *mut sys::mln_projection_mode,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_projection_mode(out_mode), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_viewport_options_default(
    out_options: *mut sys::mln_map_viewport_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_map_viewport_options(out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_map_tile_options_default(
    out_options: *mut sys::mln_map_tile_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_map_tile_options(out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_tile_source_options_default(
    out_options: *mut sys::mln_style_tile_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_style_tile_source_options(out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_tile_source_options_set_min_zoom(
    options: *mut sys::mln_style_tile_source_options,
    min_zoom: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_style_tile_min_zoom(options, min_zoom), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_tile_source_options_set_max_zoom(
    options: *mut sys::mln_style_tile_source_options,
    max_zoom: f64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_style_tile_max_zoom(options, max_zoom), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_tile_source_options_set_tile_size(
    options: *mut sys::mln_style_tile_source_options,
    tile_size: u32,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(set_style_tile_tile_size(options, tile_size), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_custom_geometry_source_options_default(
    out_options: *mut sys::mln_custom_geometry_source_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        default_custom_geometry_source_options(out_options),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_premultiplied_rgba8_image_default(
    out_image: *mut sys::mln_premultiplied_rgba8_image,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_premultiplied_rgba8_image(out_image), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_premultiplied_rgba8_image_init(
    out_image: *mut sys::mln_premultiplied_rgba8_image,
    width: u32,
    height: u32,
    stride: u32,
    pixels: *const u8,
    byte_length: usize,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        init_premultiplied_rgba8_image(out_image, width, height, stride, pixels, byte_length),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_image_options_default(
    out_options: *mut sys::mln_style_image_options,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_style_image_options(out_options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_style_image_info_default(
    out_info: *mut sys::mln_style_image_info,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(default_style_image_info(out_info), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_options_set_maximum_cache_size(
    options: *mut sys::mln_runtime_options,
    maximum_cache_size: u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        set_runtime_maximum_cache_size(options, maximum_cache_size),
        error_out,
    )
}

macro_rules! export_bool_setter {
    ($name:ident, $helper:ident, $options:ty, $value:ty, $arg:ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(
            options: *mut $options,
            $arg: $value,
            error_out: *mut *mut GError,
        ) -> GBoolean {
            gerror_bool($helper(options, $arg), error_out)
        }
    };
}

export_bool_setter!(
    mln_vala_camera_options_set_center,
    set_camera_center,
    sys::mln_camera_options,
    *const sys::mln_lat_lng,
    center
);
export_bool_setter!(
    mln_vala_camera_options_set_zoom,
    set_camera_zoom,
    sys::mln_camera_options,
    f64,
    zoom
);
export_bool_setter!(
    mln_vala_camera_options_set_bearing,
    set_camera_bearing,
    sys::mln_camera_options,
    f64,
    bearing
);
export_bool_setter!(
    mln_vala_camera_options_set_pitch,
    set_camera_pitch,
    sys::mln_camera_options,
    f64,
    pitch
);
export_bool_setter!(
    mln_vala_camera_options_set_center_altitude,
    set_camera_center_altitude,
    sys::mln_camera_options,
    f64,
    center_altitude
);
export_bool_setter!(
    mln_vala_camera_options_set_padding,
    set_camera_padding,
    sys::mln_camera_options,
    *const sys::mln_edge_insets,
    padding
);
export_bool_setter!(
    mln_vala_camera_options_set_anchor,
    set_camera_anchor,
    sys::mln_camera_options,
    *const sys::mln_screen_point,
    anchor
);
export_bool_setter!(
    mln_vala_camera_options_set_roll,
    set_camera_roll,
    sys::mln_camera_options,
    f64,
    roll
);
export_bool_setter!(
    mln_vala_camera_options_set_field_of_view,
    set_camera_field_of_view,
    sys::mln_camera_options,
    f64,
    field_of_view
);
export_bool_setter!(
    mln_vala_animation_options_set_duration_ms,
    set_animation_duration_ms,
    sys::mln_animation_options,
    f64,
    duration_ms
);
export_bool_setter!(
    mln_vala_animation_options_set_velocity,
    set_animation_velocity,
    sys::mln_animation_options,
    f64,
    velocity
);
export_bool_setter!(
    mln_vala_animation_options_set_min_zoom,
    set_animation_min_zoom,
    sys::mln_animation_options,
    f64,
    min_zoom
);
export_bool_setter!(
    mln_vala_animation_options_set_easing,
    set_animation_easing,
    sys::mln_animation_options,
    *const sys::mln_unit_bezier,
    easing
);
export_bool_setter!(
    mln_vala_camera_fit_options_set_padding,
    set_camera_fit_padding,
    sys::mln_camera_fit_options,
    *const sys::mln_edge_insets,
    padding
);
export_bool_setter!(
    mln_vala_camera_fit_options_set_bearing,
    set_camera_fit_bearing,
    sys::mln_camera_fit_options,
    f64,
    bearing
);
export_bool_setter!(
    mln_vala_camera_fit_options_set_pitch,
    set_camera_fit_pitch,
    sys::mln_camera_fit_options,
    f64,
    pitch
);
export_bool_setter!(
    mln_vala_bound_options_set_bounds,
    set_bound_bounds,
    sys::mln_bound_options,
    *const sys::mln_lat_lng_bounds,
    bounds
);
export_bool_setter!(
    mln_vala_bound_options_set_min_zoom,
    set_bound_min_zoom,
    sys::mln_bound_options,
    f64,
    min_zoom
);
export_bool_setter!(
    mln_vala_bound_options_set_max_zoom,
    set_bound_max_zoom,
    sys::mln_bound_options,
    f64,
    max_zoom
);
export_bool_setter!(
    mln_vala_bound_options_set_min_pitch,
    set_bound_min_pitch,
    sys::mln_bound_options,
    f64,
    min_pitch
);
export_bool_setter!(
    mln_vala_bound_options_set_max_pitch,
    set_bound_max_pitch,
    sys::mln_bound_options,
    f64,
    max_pitch
);
export_bool_setter!(
    mln_vala_free_camera_options_set_position,
    set_free_camera_position,
    sys::mln_free_camera_options,
    *const sys::mln_vec3,
    position
);
export_bool_setter!(
    mln_vala_free_camera_options_set_orientation,
    set_free_camera_orientation,
    sys::mln_free_camera_options,
    *const sys::mln_quaternion,
    orientation
);
export_bool_setter!(
    mln_vala_projection_mode_set_axonometric,
    set_projection_axonometric,
    sys::mln_projection_mode,
    bool,
    axonometric
);
export_bool_setter!(
    mln_vala_projection_mode_set_x_skew,
    set_projection_x_skew,
    sys::mln_projection_mode,
    f64,
    x_skew
);
export_bool_setter!(
    mln_vala_projection_mode_set_y_skew,
    set_projection_y_skew,
    sys::mln_projection_mode,
    f64,
    y_skew
);
export_bool_setter!(
    mln_vala_map_viewport_options_set_north_orientation,
    set_viewport_north_orientation,
    sys::mln_map_viewport_options,
    u32,
    north_orientation
);
export_bool_setter!(
    mln_vala_map_viewport_options_set_constrain_mode,
    set_viewport_constrain_mode,
    sys::mln_map_viewport_options,
    u32,
    constrain_mode
);
export_bool_setter!(
    mln_vala_map_viewport_options_set_viewport_mode,
    set_viewport_viewport_mode,
    sys::mln_map_viewport_options,
    u32,
    viewport_mode
);
export_bool_setter!(
    mln_vala_map_viewport_options_set_frustum_offset,
    set_viewport_frustum_offset,
    sys::mln_map_viewport_options,
    *const sys::mln_edge_insets,
    frustum_offset
);
export_bool_setter!(
    mln_vala_map_tile_options_set_prefetch_zoom_delta,
    set_tile_prefetch_zoom_delta,
    sys::mln_map_tile_options,
    u32,
    prefetch_zoom_delta
);
export_bool_setter!(
    mln_vala_map_tile_options_set_lod_min_radius,
    set_tile_lod_min_radius,
    sys::mln_map_tile_options,
    f64,
    lod_min_radius
);
export_bool_setter!(
    mln_vala_map_tile_options_set_lod_scale,
    set_tile_lod_scale,
    sys::mln_map_tile_options,
    f64,
    lod_scale
);
export_bool_setter!(
    mln_vala_map_tile_options_set_lod_pitch_threshold,
    set_tile_lod_pitch_threshold,
    sys::mln_map_tile_options,
    f64,
    lod_pitch_threshold
);
export_bool_setter!(
    mln_vala_map_tile_options_set_lod_zoom_shift,
    set_tile_lod_zoom_shift,
    sys::mln_map_tile_options,
    f64,
    lod_zoom_shift
);
export_bool_setter!(
    mln_vala_map_tile_options_set_lod_mode,
    set_tile_lod_mode,
    sys::mln_map_tile_options,
    u32,
    lod_mode
);
export_bool_setter!(
    mln_vala_style_tile_source_options_set_scheme,
    set_style_tile_scheme,
    sys::mln_style_tile_source_options,
    u32,
    scheme
);
export_bool_setter!(
    mln_vala_style_tile_source_options_set_bounds,
    set_style_tile_bounds,
    sys::mln_style_tile_source_options,
    *const sys::mln_lat_lng_bounds,
    bounds
);
export_bool_setter!(
    mln_vala_style_tile_source_options_set_vector_encoding,
    set_style_tile_vector_encoding,
    sys::mln_style_tile_source_options,
    u32,
    vector_encoding
);
export_bool_setter!(
    mln_vala_style_tile_source_options_set_raster_encoding,
    set_style_tile_raster_encoding,
    sys::mln_style_tile_source_options,
    u32,
    raster_encoding
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_min_zoom,
    set_custom_geometry_min_zoom,
    sys::mln_custom_geometry_source_options,
    f64,
    min_zoom
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_max_zoom,
    set_custom_geometry_max_zoom,
    sys::mln_custom_geometry_source_options,
    f64,
    max_zoom
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_tolerance,
    set_custom_geometry_tolerance,
    sys::mln_custom_geometry_source_options,
    f64,
    tolerance
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_tile_size,
    set_custom_geometry_tile_size,
    sys::mln_custom_geometry_source_options,
    u32,
    tile_size
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_buffer,
    set_custom_geometry_buffer,
    sys::mln_custom_geometry_source_options,
    u32,
    buffer
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_clip,
    set_custom_geometry_clip,
    sys::mln_custom_geometry_source_options,
    bool,
    clip
);
export_bool_setter!(
    mln_vala_custom_geometry_source_options_set_wrap,
    set_custom_geometry_wrap,
    sys::mln_custom_geometry_source_options,
    bool,
    wrap
);
export_bool_setter!(
    mln_vala_style_image_options_set_pixel_ratio,
    set_style_image_pixel_ratio,
    sys::mln_style_image_options,
    f32,
    pixel_ratio
);
export_bool_setter!(
    mln_vala_style_image_options_set_sdf,
    set_style_image_sdf,
    sys::mln_style_image_options,
    bool,
    sdf
);

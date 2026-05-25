use super::*;

pub(super) fn default_runtime_options(
    out_options: *mut sys::mln_runtime_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_runtime_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

pub(super) fn default_map_options(out_options: *mut sys::mln_map_options) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_map_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

pub(super) fn request_repaint(handle: *mut MapHandle) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map_native` returns a live native map pointer.
    error::check(unsafe { sys::mln_map_request_repaint(map) })
}

pub(super) fn request_still_image(handle: *mut MapHandle) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map_native` returns a live native map pointer.
    error::check(unsafe { sys::mln_map_request_still_image(map) })
}

pub(super) fn set_debug_options(handle: *mut MapHandle, options: u32) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map_native` returns a live native map pointer. The C API
    // validates debug-option bits.
    error::check(unsafe { sys::mln_map_set_debug_options(map, options) })
}

pub(super) fn get_debug_options(
    handle: *mut MapHandle,
    out_options: *mut u32,
) -> error::Result<()> {
    let map = map_native(handle)?;
    if out_options.is_null() {
        return Err(Error::invalid_argument(
            "debug options output pointer is null",
        ));
    }
    // SAFETY: `map` is live and `out_options` points to writable storage.
    error::check(unsafe { sys::mln_map_get_debug_options(map, out_options) })
}

pub(super) fn set_rendering_stats_view_enabled(
    handle: *mut MapHandle,
    enabled: bool,
) -> error::Result<()> {
    let map = map_native(handle)?;
    // SAFETY: `map_native` returns a live native map pointer.
    error::check(unsafe { sys::mln_map_set_rendering_stats_view_enabled(map, enabled) })
}

pub(super) fn get_rendering_stats_view_enabled(
    handle: *mut MapHandle,
    out_enabled: *mut GBoolean,
) -> error::Result<()> {
    let map = map_native(handle)?;
    let mut enabled = false;
    // SAFETY: `map` is live and `enabled` is valid output storage.
    error::check(unsafe { sys::mln_map_get_rendering_stats_view_enabled(map, &mut enabled) })?;
    glib::clear_optional_out_pointer(out_enabled, if enabled { GTRUE } else { GFALSE })
}

pub(super) fn default_camera_options(
    out_options: *mut sys::mln_camera_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_camera_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

pub(super) fn default_animation_options(
    out_options: *mut sys::mln_animation_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_animation_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

pub(super) fn default_camera_fit_options(
    out_options: *mut sys::mln_camera_fit_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_camera_fit_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

pub(super) fn default_bound_options(out_options: *mut sys::mln_bound_options) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_bound_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

pub(super) fn default_free_camera_options(
    out_options: *mut sys::mln_free_camera_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_free_camera_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

pub(super) fn default_projection_mode(
    out_mode: *mut sys::mln_projection_mode,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let mode = unsafe { sys::mln_projection_mode_default() };
    glib::clear_optional_out_pointer(out_mode, mode)
}

pub(super) fn default_map_viewport_options(
    out_options: *mut sys::mln_map_viewport_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_map_viewport_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

pub(super) fn default_map_tile_options(
    out_options: *mut sys::mln_map_tile_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_map_tile_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

pub(super) fn default_style_tile_source_options(
    out_options: *mut sys::mln_style_tile_source_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_style_tile_source_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

pub(super) fn style_tile_options_mut(
    options: *mut sys::mln_style_tile_source_options,
) -> error::Result<&'static mut sys::mln_style_tile_source_options> {
    if options.is_null() {
        return Err(Error::invalid_argument(
            "style tile source options are null",
        ));
    }
    Ok(unsafe { &mut *options })
}

pub(super) fn set_style_tile_min_zoom(
    options: *mut sys::mln_style_tile_source_options,
    min_zoom: f64,
) -> error::Result<()> {
    let options = style_tile_options_mut(options)?;
    options.min_zoom = min_zoom;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_MIN_ZOOM;
    Ok(())
}

pub(super) fn set_style_tile_max_zoom(
    options: *mut sys::mln_style_tile_source_options,
    max_zoom: f64,
) -> error::Result<()> {
    let options = style_tile_options_mut(options)?;
    options.max_zoom = max_zoom;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_MAX_ZOOM;
    Ok(())
}

pub(super) fn set_style_tile_tile_size(
    options: *mut sys::mln_style_tile_source_options,
    tile_size: u32,
) -> error::Result<()> {
    let options = style_tile_options_mut(options)?;
    options.tile_size = tile_size;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_TILE_SIZE;
    Ok(())
}

pub(super) fn mut_struct<T>(ptr: *mut T, name: &str) -> error::Result<&'static mut T> {
    if ptr.is_null() {
        return Err(Error::invalid_argument(format!("{name} is null")));
    }
    // SAFETY: The caller supplies writable struct storage for this call.
    Ok(unsafe { &mut *ptr })
}

pub(super) fn copy_struct<T: Copy>(ptr: *const T, name: &str) -> error::Result<T> {
    if ptr.is_null() {
        return Err(Error::invalid_argument(format!("{name} is null")));
    }
    // SAFETY: The caller supplies readable struct storage for this call.
    Ok(unsafe { *ptr })
}

pub(super) fn set_runtime_maximum_cache_size(
    options: *mut sys::mln_runtime_options,
    maximum_cache_size: u64,
) -> error::Result<()> {
    let options = mut_struct(options, "runtime options")?;
    options.maximum_cache_size = maximum_cache_size;
    options.flags |= sys::MLN_RUNTIME_OPTION_MAXIMUM_CACHE_SIZE;
    Ok(())
}

pub(super) fn set_camera_center(
    options: *mut sys::mln_camera_options,
    center: *const sys::mln_lat_lng,
) -> error::Result<()> {
    let center = copy_struct(center, "camera center")?;
    let options = mut_struct(options, "camera options")?;
    options.latitude = center.latitude;
    options.longitude = center.longitude;
    options.fields |= sys::MLN_CAMERA_OPTION_CENTER;
    Ok(())
}

pub(super) fn set_camera_zoom(
    options: *mut sys::mln_camera_options,
    zoom: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "camera options")?;
    options.zoom = zoom;
    options.fields |= sys::MLN_CAMERA_OPTION_ZOOM;
    Ok(())
}

pub(super) fn set_camera_bearing(
    options: *mut sys::mln_camera_options,
    bearing: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "camera options")?;
    options.bearing = bearing;
    options.fields |= sys::MLN_CAMERA_OPTION_BEARING;
    Ok(())
}

pub(super) fn set_camera_pitch(
    options: *mut sys::mln_camera_options,
    pitch: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "camera options")?;
    options.pitch = pitch;
    options.fields |= sys::MLN_CAMERA_OPTION_PITCH;
    Ok(())
}

pub(super) fn set_camera_center_altitude(
    options: *mut sys::mln_camera_options,
    center_altitude: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "camera options")?;
    options.center_altitude = center_altitude;
    options.fields |= sys::MLN_CAMERA_OPTION_CENTER_ALTITUDE;
    Ok(())
}

pub(super) fn set_camera_padding(
    options: *mut sys::mln_camera_options,
    padding: *const sys::mln_edge_insets,
) -> error::Result<()> {
    let padding = copy_struct(padding, "camera padding")?;
    let options = mut_struct(options, "camera options")?;
    options.padding = padding;
    options.fields |= sys::MLN_CAMERA_OPTION_PADDING;
    Ok(())
}

pub(super) fn set_camera_anchor(
    options: *mut sys::mln_camera_options,
    anchor: *const sys::mln_screen_point,
) -> error::Result<()> {
    let anchor = copy_struct(anchor, "camera anchor")?;
    let options = mut_struct(options, "camera options")?;
    options.anchor = anchor;
    options.fields |= sys::MLN_CAMERA_OPTION_ANCHOR;
    Ok(())
}

pub(super) fn set_camera_roll(
    options: *mut sys::mln_camera_options,
    roll: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "camera options")?;
    options.roll = roll;
    options.fields |= sys::MLN_CAMERA_OPTION_ROLL;
    Ok(())
}

pub(super) fn set_camera_field_of_view(
    options: *mut sys::mln_camera_options,
    field_of_view: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "camera options")?;
    options.field_of_view = field_of_view;
    options.fields |= sys::MLN_CAMERA_OPTION_FOV;
    Ok(())
}

pub(super) fn set_animation_duration_ms(
    options: *mut sys::mln_animation_options,
    duration_ms: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "animation options")?;
    options.duration_ms = duration_ms;
    options.fields |= sys::MLN_ANIMATION_OPTION_DURATION;
    Ok(())
}

pub(super) fn set_animation_velocity(
    options: *mut sys::mln_animation_options,
    velocity: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "animation options")?;
    options.velocity = velocity;
    options.fields |= sys::MLN_ANIMATION_OPTION_VELOCITY;
    Ok(())
}

pub(super) fn set_animation_min_zoom(
    options: *mut sys::mln_animation_options,
    min_zoom: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "animation options")?;
    options.min_zoom = min_zoom;
    options.fields |= sys::MLN_ANIMATION_OPTION_MIN_ZOOM;
    Ok(())
}

pub(super) fn set_animation_easing(
    options: *mut sys::mln_animation_options,
    easing: *const sys::mln_unit_bezier,
) -> error::Result<()> {
    let easing = copy_struct(easing, "animation easing")?;
    let options = mut_struct(options, "animation options")?;
    options.easing = easing;
    options.fields |= sys::MLN_ANIMATION_OPTION_EASING;
    Ok(())
}

pub(super) fn set_camera_fit_padding(
    options: *mut sys::mln_camera_fit_options,
    padding: *const sys::mln_edge_insets,
) -> error::Result<()> {
    let padding = copy_struct(padding, "camera fit padding")?;
    let options = mut_struct(options, "camera fit options")?;
    options.padding = padding;
    options.fields |= sys::MLN_CAMERA_FIT_OPTION_PADDING;
    Ok(())
}

pub(super) fn set_camera_fit_bearing(
    options: *mut sys::mln_camera_fit_options,
    bearing: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "camera fit options")?;
    options.bearing = bearing;
    options.fields |= sys::MLN_CAMERA_FIT_OPTION_BEARING;
    Ok(())
}

pub(super) fn set_camera_fit_pitch(
    options: *mut sys::mln_camera_fit_options,
    pitch: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "camera fit options")?;
    options.pitch = pitch;
    options.fields |= sys::MLN_CAMERA_FIT_OPTION_PITCH;
    Ok(())
}

pub(super) fn set_bound_bounds(
    options: *mut sys::mln_bound_options,
    bounds: *const sys::mln_lat_lng_bounds,
) -> error::Result<()> {
    let bounds = copy_struct(bounds, "bound options bounds")?;
    let options = mut_struct(options, "bound options")?;
    options.bounds = bounds;
    options.fields |= sys::MLN_BOUND_OPTION_BOUNDS;
    Ok(())
}

pub(super) fn set_bound_min_zoom(
    options: *mut sys::mln_bound_options,
    min_zoom: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "bound options")?;
    options.min_zoom = min_zoom;
    options.fields |= sys::MLN_BOUND_OPTION_MIN_ZOOM;
    Ok(())
}

pub(super) fn set_bound_max_zoom(
    options: *mut sys::mln_bound_options,
    max_zoom: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "bound options")?;
    options.max_zoom = max_zoom;
    options.fields |= sys::MLN_BOUND_OPTION_MAX_ZOOM;
    Ok(())
}

pub(super) fn set_bound_min_pitch(
    options: *mut sys::mln_bound_options,
    min_pitch: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "bound options")?;
    options.min_pitch = min_pitch;
    options.fields |= sys::MLN_BOUND_OPTION_MIN_PITCH;
    Ok(())
}

pub(super) fn set_bound_max_pitch(
    options: *mut sys::mln_bound_options,
    max_pitch: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "bound options")?;
    options.max_pitch = max_pitch;
    options.fields |= sys::MLN_BOUND_OPTION_MAX_PITCH;
    Ok(())
}

pub(super) fn set_free_camera_position(
    options: *mut sys::mln_free_camera_options,
    position: *const sys::mln_vec3,
) -> error::Result<()> {
    let position = copy_struct(position, "free camera position")?;
    let options = mut_struct(options, "free camera options")?;
    options.position = position;
    options.fields |= sys::MLN_FREE_CAMERA_OPTION_POSITION;
    Ok(())
}

pub(super) fn set_free_camera_orientation(
    options: *mut sys::mln_free_camera_options,
    orientation: *const sys::mln_quaternion,
) -> error::Result<()> {
    let orientation = copy_struct(orientation, "free camera orientation")?;
    let options = mut_struct(options, "free camera options")?;
    options.orientation = orientation;
    options.fields |= sys::MLN_FREE_CAMERA_OPTION_ORIENTATION;
    Ok(())
}

pub(super) fn set_projection_axonometric(
    mode: *mut sys::mln_projection_mode,
    axonometric: bool,
) -> error::Result<()> {
    let mode = mut_struct(mode, "projection mode")?;
    mode.axonometric = axonometric;
    mode.fields |= sys::MLN_PROJECTION_MODE_AXONOMETRIC;
    Ok(())
}

pub(super) fn set_projection_x_skew(
    mode: *mut sys::mln_projection_mode,
    x_skew: f64,
) -> error::Result<()> {
    let mode = mut_struct(mode, "projection mode")?;
    mode.x_skew = x_skew;
    mode.fields |= sys::MLN_PROJECTION_MODE_X_SKEW;
    Ok(())
}

pub(super) fn set_projection_y_skew(
    mode: *mut sys::mln_projection_mode,
    y_skew: f64,
) -> error::Result<()> {
    let mode = mut_struct(mode, "projection mode")?;
    mode.y_skew = y_skew;
    mode.fields |= sys::MLN_PROJECTION_MODE_Y_SKEW;
    Ok(())
}

pub(super) fn set_viewport_north_orientation(
    options: *mut sys::mln_map_viewport_options,
    north_orientation: u32,
) -> error::Result<()> {
    let options = mut_struct(options, "map viewport options")?;
    options.north_orientation = north_orientation;
    options.fields |= sys::MLN_MAP_VIEWPORT_OPTION_NORTH_ORIENTATION;
    Ok(())
}

pub(super) fn set_viewport_constrain_mode(
    options: *mut sys::mln_map_viewport_options,
    constrain_mode: u32,
) -> error::Result<()> {
    let options = mut_struct(options, "map viewport options")?;
    options.constrain_mode = constrain_mode;
    options.fields |= sys::MLN_MAP_VIEWPORT_OPTION_CONSTRAIN_MODE;
    Ok(())
}

pub(super) fn set_viewport_viewport_mode(
    options: *mut sys::mln_map_viewport_options,
    viewport_mode: u32,
) -> error::Result<()> {
    let options = mut_struct(options, "map viewport options")?;
    options.viewport_mode = viewport_mode;
    options.fields |= sys::MLN_MAP_VIEWPORT_OPTION_VIEWPORT_MODE;
    Ok(())
}

pub(super) fn set_viewport_frustum_offset(
    options: *mut sys::mln_map_viewport_options,
    frustum_offset: *const sys::mln_edge_insets,
) -> error::Result<()> {
    let frustum_offset = copy_struct(frustum_offset, "viewport frustum offset")?;
    let options = mut_struct(options, "map viewport options")?;
    options.frustum_offset = frustum_offset;
    options.fields |= sys::MLN_MAP_VIEWPORT_OPTION_FRUSTUM_OFFSET;
    Ok(())
}

pub(super) fn set_tile_prefetch_zoom_delta(
    options: *mut sys::mln_map_tile_options,
    prefetch_zoom_delta: u32,
) -> error::Result<()> {
    let options = mut_struct(options, "map tile options")?;
    options.prefetch_zoom_delta = prefetch_zoom_delta;
    options.fields |= sys::MLN_MAP_TILE_OPTION_PREFETCH_ZOOM_DELTA;
    Ok(())
}

pub(super) fn set_tile_lod_min_radius(
    options: *mut sys::mln_map_tile_options,
    lod_min_radius: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "map tile options")?;
    options.lod_min_radius = lod_min_radius;
    options.fields |= sys::MLN_MAP_TILE_OPTION_LOD_MIN_RADIUS;
    Ok(())
}

pub(super) fn set_tile_lod_scale(
    options: *mut sys::mln_map_tile_options,
    lod_scale: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "map tile options")?;
    options.lod_scale = lod_scale;
    options.fields |= sys::MLN_MAP_TILE_OPTION_LOD_SCALE;
    Ok(())
}

pub(super) fn set_tile_lod_pitch_threshold(
    options: *mut sys::mln_map_tile_options,
    lod_pitch_threshold: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "map tile options")?;
    options.lod_pitch_threshold = lod_pitch_threshold;
    options.fields |= sys::MLN_MAP_TILE_OPTION_LOD_PITCH_THRESHOLD;
    Ok(())
}

pub(super) fn set_tile_lod_zoom_shift(
    options: *mut sys::mln_map_tile_options,
    lod_zoom_shift: f64,
) -> error::Result<()> {
    let options = mut_struct(options, "map tile options")?;
    options.lod_zoom_shift = lod_zoom_shift;
    options.fields |= sys::MLN_MAP_TILE_OPTION_LOD_ZOOM_SHIFT;
    Ok(())
}

pub(super) fn set_tile_lod_mode(
    options: *mut sys::mln_map_tile_options,
    lod_mode: u32,
) -> error::Result<()> {
    let options = mut_struct(options, "map tile options")?;
    options.lod_mode = lod_mode;
    options.fields |= sys::MLN_MAP_TILE_OPTION_LOD_MODE;
    Ok(())
}

pub(super) fn set_style_tile_scheme(
    options: *mut sys::mln_style_tile_source_options,
    scheme: u32,
) -> error::Result<()> {
    let options = style_tile_options_mut(options)?;
    options.scheme = scheme;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_SCHEME;
    Ok(())
}

pub(super) fn set_style_tile_bounds(
    options: *mut sys::mln_style_tile_source_options,
    bounds: *const sys::mln_lat_lng_bounds,
) -> error::Result<()> {
    let bounds = copy_struct(bounds, "style tile bounds")?;
    let options = style_tile_options_mut(options)?;
    options.bounds = bounds;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_BOUNDS;
    Ok(())
}

pub(super) fn set_style_tile_vector_encoding(
    options: *mut sys::mln_style_tile_source_options,
    vector_encoding: u32,
) -> error::Result<()> {
    let options = style_tile_options_mut(options)?;
    options.vector_encoding = vector_encoding;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_VECTOR_ENCODING;
    Ok(())
}

pub(super) fn set_style_tile_raster_encoding(
    options: *mut sys::mln_style_tile_source_options,
    raster_encoding: u32,
) -> error::Result<()> {
    let options = style_tile_options_mut(options)?;
    options.raster_encoding = raster_encoding;
    options.fields |= sys::MLN_STYLE_TILE_SOURCE_OPTION_RASTER_ENCODING;
    Ok(())
}

pub(super) fn custom_geometry_options_mut(
    options: *mut sys::mln_custom_geometry_source_options,
) -> error::Result<&'static mut sys::mln_custom_geometry_source_options> {
    mut_struct(options, "custom geometry source options")
}

pub(super) fn set_custom_geometry_min_zoom(
    options: *mut sys::mln_custom_geometry_source_options,
    min_zoom: f64,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.min_zoom = min_zoom;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_MIN_ZOOM;
    Ok(())
}

pub(super) fn set_custom_geometry_max_zoom(
    options: *mut sys::mln_custom_geometry_source_options,
    max_zoom: f64,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.max_zoom = max_zoom;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_MAX_ZOOM;
    Ok(())
}

pub(super) fn set_custom_geometry_tolerance(
    options: *mut sys::mln_custom_geometry_source_options,
    tolerance: f64,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.tolerance = tolerance;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_TOLERANCE;
    Ok(())
}

pub(super) fn set_custom_geometry_tile_size(
    options: *mut sys::mln_custom_geometry_source_options,
    tile_size: u32,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.tile_size = tile_size;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_TILE_SIZE;
    Ok(())
}

pub(super) fn set_custom_geometry_buffer(
    options: *mut sys::mln_custom_geometry_source_options,
    buffer: u32,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.buffer = buffer;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_BUFFER;
    Ok(())
}

pub(super) fn set_custom_geometry_clip(
    options: *mut sys::mln_custom_geometry_source_options,
    clip: bool,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.clip = clip;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_CLIP;
    Ok(())
}

pub(super) fn set_custom_geometry_wrap(
    options: *mut sys::mln_custom_geometry_source_options,
    wrap: bool,
) -> error::Result<()> {
    let options = custom_geometry_options_mut(options)?;
    options.wrap = wrap;
    options.fields |= sys::MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_WRAP;
    Ok(())
}

pub(super) fn set_style_image_pixel_ratio(
    options: *mut sys::mln_style_image_options,
    pixel_ratio: f32,
) -> error::Result<()> {
    let options = mut_struct(options, "style image options")?;
    options.pixel_ratio = pixel_ratio;
    options.fields |= sys::MLN_STYLE_IMAGE_OPTION_PIXEL_RATIO;
    Ok(())
}

pub(super) fn set_style_image_sdf(
    options: *mut sys::mln_style_image_options,
    sdf: bool,
) -> error::Result<()> {
    let options = mut_struct(options, "style image options")?;
    options.sdf = sdf;
    options.fields |= sys::MLN_STYLE_IMAGE_OPTION_SDF;
    Ok(())
}

pub(super) fn default_custom_geometry_source_options(
    out_options: *mut sys::mln_custom_geometry_source_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_custom_geometry_source_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

pub(super) fn default_premultiplied_rgba8_image(
    out_image: *mut sys::mln_premultiplied_rgba8_image,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let image = unsafe { sys::mln_premultiplied_rgba8_image_default() };
    glib::clear_optional_out_pointer(out_image, image)
}

pub(super) fn init_premultiplied_rgba8_image(
    out_image: *mut sys::mln_premultiplied_rgba8_image,
    width: u32,
    height: u32,
    stride: u32,
    pixels: *const u8,
    byte_length: usize,
) -> error::Result<()> {
    if pixels.is_null() && byte_length != 0 {
        return Err(Error::invalid_argument("image pixel buffer is null"));
    }
    let image = sys::mln_premultiplied_rgba8_image {
        size: std::mem::size_of::<sys::mln_premultiplied_rgba8_image>() as u32,
        width,
        height,
        stride,
        pixels,
        byte_length,
    };
    glib::clear_optional_out_pointer(out_image, image)
}

pub(super) fn default_style_image_options(
    out_options: *mut sys::mln_style_image_options,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let options = unsafe { sys::mln_style_image_options_default() };
    glib::clear_optional_out_pointer(out_options, options)
}

pub(super) fn default_style_image_info(
    out_info: *mut sys::mln_style_image_info,
) -> error::Result<()> {
    // SAFETY: Default constructor returns a value initialized for this C ABI.
    let info = unsafe { sys::mln_style_image_info_default() };
    glib::clear_optional_out_pointer(out_info, info)
}

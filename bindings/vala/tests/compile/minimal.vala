bool handle_log(MaplibreNative.LogSeverity severity, MaplibreNative.LogEvent event, int64 code, string? message) {
  return false;
}

string? transform_resource(MaplibreNative.ResourceKind kind, string url) {
  return null;
}

int main(string[] args) {
  uint32 version = MaplibreNative.c_version();
  MaplibreNative.NetworkStatus status;

  try {
    MaplibreNative.RenderBackendFlags backends = MaplibreNative.supported_render_backends();
    MaplibreNative.NetworkStatus.get(out status);
    status.set();
    MaplibreNative.log_set_async_severity_mask(MaplibreNative.LogSeverityFlags.DEFAULT);
    MaplibreNative.log_set_callback(handle_log);
    MaplibreNative.log_clear_callback();

    MaplibreNative.NativePointer native_pointer;
    MaplibreNative.NativePointer.@new(0x1234, out native_pointer);
    size_t pointer_bits = native_pointer.get_bits();
    MaplibreNative.ResourceResponse response = {};
    response.default();

    MaplibreNative.LatLng coordinate = { 37.7749, -122.4194 };
    MaplibreNative.ProjectedMeters meters;
    MaplibreNative.ProjectedMeters.for_lat_lng(coordinate, out meters);
    MaplibreNative.LatLng round_trip;
    MaplibreNative.LatLng.for_projected_meters(meters, out round_trip);

    MaplibreNative.RuntimeOptions runtime_options = {};
    runtime_options.default();
    var runtime = new MaplibreNative.RuntimeHandle.with_options(runtime_options);
    runtime.set_resource_transform(transform_resource);
    runtime.run_once();
    MaplibreNative.RuntimeEvent event;
    runtime.poll_event(out event);

    MaplibreNative.MapOptions map_options = {};
    map_options.default();
    map_options.width = 512;
    map_options.height = 512;
    map_options.scale_factor = 1.0;
    map_options.map_mode = MaplibreNative.MapMode.CONTINUOUS;
    var map = new MaplibreNative.MapHandle.with_options(runtime, map_options);
    map.set_style_url("asset://missing-style.json");
    map.set_style_json("{\"version\":8,\"sources\":{},\"layers\":[]}");
    map.add_geojson_source_url("fixture-source", "asset://fixture.geojson");
    bool source_exists = false;
    map.style_source_exists("fixture-source", out source_exists);
    MaplibreNative.StyleSourceType source_type;
    bool source_found = false;
    map.get_style_source_type("fixture-source", out source_type, out source_found);
    MaplibreNative.StyleSourceInfo source_info;
    map.get_style_source_info("fixture-source", out source_info, out source_found);
    map.set_geojson_source_url("fixture-source", "asset://fixture-updated.geojson");
    bool source_removed = false;
    map.remove_style_source("fixture-source", out source_removed);
    MaplibreNative.StyleTileSourceOptions tile_source_options = {};
    tile_source_options.default();
    map.add_vector_source_url("vector-source", "asset://vector-source.json", tile_source_options);
    map.remove_style_source("vector-source", out source_removed);
    map.add_raster_source_url("raster-source", "asset://raster-source.json", tile_source_options);
    map.remove_style_source("raster-source", out source_removed);
    map.add_raster_dem_source_url("dem-source", "asset://dem-source.json", tile_source_options);
    map.remove_style_source("dem-source", out source_removed);
    MaplibreNative.MetalSurfaceDescriptor metal_surface = {};
    metal_surface.default();
    MaplibreNative.VulkanSurfaceDescriptor vulkan_surface = {};
    vulkan_surface.default();
    MaplibreNative.MetalOwnedTextureDescriptor metal_owned_texture = {};
    metal_owned_texture.default();
    MaplibreNative.TextureImageInfo texture_info = {};
    texture_info.default();
    map.set_debug_options(MaplibreNative.MapDebugOptions.TILE_BORDERS);
    MaplibreNative.MapDebugOptions debug_options;
    map.get_debug_options(out debug_options);
    map.set_rendering_stats_view_enabled(true);
    bool rendering_stats_enabled = false;
    map.get_rendering_stats_view_enabled(out rendering_stats_enabled);
    MaplibreNative.CameraOptions camera = {};
    camera.default();
    map.get_camera(out camera);
    MaplibreNative.AnimationOptions animation = {};
    animation.default();
    map.jump_to(camera);
    map.ease_to(camera, animation);
    map.fly_to(camera, null);
    map.move_by(0.0, 0.0);
    map.move_by_animated(0.0, 0.0, animation);
    map.scale_by(1.0, null);
    map.scale_by_animated(1.0, null, animation);
    MaplibreNative.ScreenPoint origin = { 0.0, 0.0 };
    map.rotate_by(origin, origin);
    map.rotate_by_animated(origin, origin, animation);
    map.pitch_by(0.0);
    map.pitch_by_animated(0.0, animation);
    map.cancel_transitions();

    MaplibreNative.BoundOptions bounds = {};
    bounds.default();
    map.get_bounds(out bounds);
    map.set_bounds(bounds);
    MaplibreNative.FreeCameraOptions free_camera = {};
    free_camera.default();
    map.get_free_camera_options(out free_camera);
    map.set_free_camera_options(free_camera);
    MaplibreNative.ProjectionMode projection_mode = {};
    projection_mode.default();
    map.get_projection_mode(out projection_mode);
    map.set_projection_mode(projection_mode);
    MaplibreNative.MapViewportOptions viewport_options = {};
    viewport_options.default();
    map.get_viewport_options(out viewport_options);
    map.set_viewport_options(viewport_options);
    MaplibreNative.MapTileOptions tile_options = {};
    tile_options.default();
    map.get_tile_options(out tile_options);
    map.set_tile_options(tile_options);

    bool loaded = false;
    map.is_fully_loaded(out loaded);
    map.dump_debug_logs();

    var projection = new MaplibreNative.MapProjectionHandle(map);
    MaplibreNative.ScreenPoint point;
    projection.pixel_for_lat_lng(coordinate, out point);
    projection.lat_lng_for_pixel(point, out round_trip);
    projection.close();

    map.close();
    runtime.clear_resource_transform();
    runtime.close();

    if (backends == 0 || pointer_bits != 0x1234) {
      return 1;
    }
  } catch (GLib.Error error) {
    return 1;
  }

  return version == 0 ? 0 : 0;
}

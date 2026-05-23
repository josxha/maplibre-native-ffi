int main(string[] args) {
  uint32 version = MaplibreNative.c_version();
  MaplibreNative.NetworkStatus status;

  try {
    MaplibreNative.RenderBackendFlags backends = MaplibreNative.supported_render_backends();
    MaplibreNative.NetworkStatus.get(out status);
    status.set();
    MaplibreNative.log_set_async_severity_mask(MaplibreNative.LogSeverityFlags.DEFAULT);
    MaplibreNative.log_clear_callback();

    MaplibreNative.NativePointer native_pointer;
    MaplibreNative.NativePointer.@new(0x1234, out native_pointer);
    size_t pointer_bits = native_pointer.get_bits();

    MaplibreNative.LatLng coordinate = { 37.7749, -122.4194 };
    MaplibreNative.ProjectedMeters meters;
    MaplibreNative.ProjectedMeters.for_lat_lng(coordinate, out meters);
    MaplibreNative.LatLng round_trip;
    MaplibreNative.LatLng.for_projected_meters(meters, out round_trip);

    var runtime = new MaplibreNative.RuntimeHandle();
    runtime.run_once();
    MaplibreNative.RuntimeEvent event;
    runtime.poll_event(out event);

    var map = new MaplibreNative.MapHandle(runtime, 512, 512, 1.0);
    map.set_style_url("asset://missing-style.json");
    map.set_style_json("{\"version\":8,\"sources\":{},\"layers\":[]}");
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
    runtime.close();

    if (backends == 0 || pointer_bits != 0x1234) {
      return 1;
    }
  } catch (GLib.Error error) {
    return 1;
  }

  return version == 0 ? 0 : 0;
}

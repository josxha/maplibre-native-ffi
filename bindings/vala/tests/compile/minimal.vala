[CCode(has_target = false)]
delegate void* MetalCreateSystemDefaultDeviceFunc();

int log_count = 0;

void* create_system_default_metal_device() {
  var module = GLib.Module.open("/System/Library/Frameworks/Metal.framework/Metal", GLib.ModuleFlags.LAZY);
  if (module == null) {
    return null;
  }
  void* symbol = null;
  if (!module.symbol("MTLCreateSystemDefaultDevice", out symbol) || symbol == null) {
    return null;
  }
  var create_device = (MetalCreateSystemDefaultDeviceFunc) symbol;
  return create_device();
}

bool wait_for_runtime_event(MaplibreNative.RuntimeHandle runtime, MaplibreNative.RuntimeEventType event_type, uint attempts) throws MaplibreNative.Error {
  for (uint attempt = 0; attempt < attempts; attempt++) {
    runtime.run_once();
    var event = runtime.poll_event();
    if (event != null && event.event_type == event_type) {
      return true;
    }
  }
  return false;
}

bool handle_log(MaplibreNative.LogSeverity severity, MaplibreNative.LogEvent event, int64 code, string? message) {
  log_count++;
  return false;
}

string? transform_resource(MaplibreNative.ResourceKind kind, string url) {
  return null;
}

int main() {
  try {
    assert(MaplibreNative.c_version() == 0);
    var backends = MaplibreNative.supported_render_backends();
    assert(backends != 0);

    var original_status = MaplibreNative.network_status();
    MaplibreNative.set_network_status(MaplibreNative.NetworkStatus.OFFLINE);
    assert(MaplibreNative.network_status() == MaplibreNative.NetworkStatus.OFFLINE);
    MaplibreNative.set_network_status(original_status);

    var runtime_options = new MaplibreNative.RuntimeOptions();
    runtime_options.maximum_cache_size = 1024 * 1024;
    var runtime = new MaplibreNative.RuntimeHandle(runtime_options);
    runtime.set_resource_transform(transform_resource);
    runtime.clear_resource_transform();
    runtime.run_once();
    assert(runtime.poll_event() == null);

    var map_options = new MaplibreNative.MapOptions();
    map_options.width = 128;
    map_options.height = 64;
    map_options.scale_factor = 1.0;
    var map = new MaplibreNative.MapHandle(runtime, map_options);

    bool close_runtime_with_live_map_failed = false;
    try {
      runtime.close();
    } catch (MaplibreNative.Error error) {
      close_runtime_with_live_map_failed = true;
    }
    assert(close_runtime_with_live_map_failed);

    map.set_style_json("{\"version\":8,\"sources\":{},\"layers\":[]}");
    map.set_debug_options(0);
    assert(map.get_debug_options() == 0);
    map.set_rendering_stats_view_enabled(false);
    assert(!map.get_rendering_stats_view_enabled());
    map.is_fully_loaded();
    MaplibreNative.set_log_callback(handle_log);
    bool parse_error_mapped = false;
    try {
      map.set_style_json("{");
    } catch (MaplibreNative.Error error) {
      parse_error_mapped = error.message.length > 0;
    }
    assert(parse_error_mapped);
    assert(log_count > 0);
    MaplibreNative.clear_log_callback();
    map.set_style_json("{\"version\":8,\"sources\":{},\"layers\":[]}");

    bool invalid_argument_mapped = false;
    try {
      map.add_geojson_source_url("", "https://example.invalid/points.geojson");
    } catch (MaplibreNative.Error.INVALID_ARGUMENT error) {
      invalid_argument_mapped = error.message.length > 0;
    }
    assert(invalid_argument_mapped);

    uint8[] image_pixels = { 255, 0, 0, 255 };
    var image = new MaplibreNative.PremultipliedRgba8Image(1, 1, 4, image_pixels);
    var image_options = new MaplibreNative.StyleImageOptions();
    image_options.pixel_ratio = 1.0f;
    image_options.sdf = false;
    map.set_style_image("marker", image, image_options);
    assert(map.style_image_exists("marker"));
    var image_info = map.get_style_image_info("marker");
    assert(image_info != null && image_info.width == 1 && image_info.height == 1);
    var copied_image = map.copy_style_image_premultiplied_rgba8("marker");
    assert(copied_image != null && copied_image.length == 4 && copied_image[0] == 255);
    assert(map.remove_style_image("marker"));
    assert(!map.remove_style_image("marker"));

    var camera = new MaplibreNative.CameraOptions();
    camera.set_center(MaplibreNative.LatLng(0.0, 0.0));
    camera.set_zoom(1.0);
    map.jump_to(camera);
    var copied_camera = map.get_camera();
    double zoom;
    copied_camera.get_zoom(out zoom);

    map.add_geojson_source_url("points", "https://example.invalid/points.geojson");
    assert(map.style_source_exists("points"));
    assert(map.get_style_source_type("points") == MaplibreNative.StyleSourceType.GEOJSON);
    var source_info = map.get_style_source_info("points");
    assert(source_info != null && source_info.source_type == MaplibreNative.StyleSourceType.GEOJSON && source_info.id_size == "points".length);
    assert(map.list_style_source_ids().contains("points"));
    map.set_geojson_source_url("points", "https://example.invalid/updated.geojson");
    assert(map.remove_style_source("points"));
    assert(!map.remove_style_source("points"));

    map.add_location_indicator_layer("location");
    assert(map.style_layer_exists("location"));
    assert(map.list_style_layer_ids().contains("location"));
    assert(map.remove_style_layer("location"));
    assert(!map.remove_style_layer("location"));

    uint64 operation_id = runtime.run_ambient_cache_operation_start(MaplibreNative.AmbientCacheOperation.INVALIDATE);
    runtime.discard_offline_operation(operation_id);

    var projection = map.create_projection();
    var pixel = projection.pixel_for_lat_lng(MaplibreNative.LatLng(0.0, 0.0));
    var round_trip = projection.lat_lng_for_pixel(pixel);
    assert(round_trip.latitude > -90.0 && round_trip.latitude < 90.0);
    projection.set_camera(camera);
    projection.set_visible_coordinates({ MaplibreNative.LatLng(-1.0, -1.0), MaplibreNative.LatLng(1.0, 1.0) }, MaplibreNative.EdgeInsets(0.0, 0.0, 0.0, 0.0));
    projection.close();
    bool closed_projection_failed = false;
    try {
      projection.get_camera();
    } catch (MaplibreNative.Error error) {
      closed_projection_failed = true;
    }
    assert(closed_projection_failed);

    var meters = MaplibreNative.projected_meters_for_lat_lng(MaplibreNative.LatLng(0.0, 0.0));
    var meters_coordinate = MaplibreNative.lat_lng_for_projected_meters(meters);
    assert(meters_coordinate.latitude > -1.0 && meters_coordinate.latitude < 1.0);

    if ((backends & MaplibreNative.RenderBackendFlags.METAL) != 0) {
      void* device = create_system_default_metal_device();
      if (device != null) {
        var texture = new MaplibreNative.MetalOwnedTextureDescriptor(MaplibreNative.NativePointer((size_t) device));
        texture.width = 32;
        texture.height = 16;
        texture.scale_factor = 1.0;
        var session = map.attach_metal_owned_texture(texture);
        assert(wait_for_runtime_event(runtime, MaplibreNative.RuntimeEventType.MAP_RENDER_UPDATE_AVAILABLE, 64));
        session.render_update();
        var query_result = session.query_rendered_features(MaplibreNative.RenderedQueryGeometry.point(MaplibreNative.ScreenPoint(0.0, 0.0)));
        query_result.count();
        query_result.close();
        uint8[] pixels = new uint8[32 * 16 * 4];
        var info = session.read_premultiplied_rgba8(pixels);
        assert(info.width == 32);
        assert(info.height == 16);
        assert(info.stride * info.height <= pixels.length);

        var frame = session.acquire_metal_owned_texture_frame();
        assert(frame.get_width() == 32);
        bool render_while_acquired_failed = false;
        try {
          session.render_update();
        } catch (MaplibreNative.Error error) {
          render_while_acquired_failed = true;
        }
        assert(render_while_acquired_failed);
        frame.close();
        bool closed_frame_failed = false;
        try {
          frame.get_width();
        } catch (MaplibreNative.Error error) {
          closed_frame_failed = true;
        }
        assert(closed_frame_failed);
        session.close();
      }
    }

    map.close();
    map.close();
    runtime.close();
    runtime.close();
    assert(map.closed);
    assert(runtime.closed);
    return 0;
  } catch (MaplibreNative.Error error) {
    stderr.printf("Vala binding smoke test failed: %s\n", error.message);
    return 1;
  }
}

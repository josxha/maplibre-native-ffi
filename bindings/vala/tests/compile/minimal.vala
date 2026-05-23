bool handle_log(MaplibreNative.LogSeverity severity, MaplibreNative.LogEvent event, int64 code, string? message) {
  return false;
}

string? transform_resource(MaplibreNative.ResourceKind kind, string url) {
  return null;
}

MaplibreNative.ResourceProviderDecision provide_resource(MaplibreNative.ResourceRequest request, MaplibreNative.ResourceRequestHandle handle) {
  if (request.url == null) {
    return MaplibreNative.ResourceProviderDecision.PASS_THROUGH;
  }
  return MaplibreNative.ResourceProviderDecision.PASS_THROUGH;
}

void inspect_metal_owned_texture_frame(MaplibreNative.MetalOwnedTextureFrameHandle frame) throws GLib.Error {
  uint64 generation;
  uint32 width;
  uint32 height;
  double scale_factor;
  uint64 frame_id;
  uint64 pixel_format;
  frame.get_generation(out generation);
  frame.get_width(out width);
  frame.get_height(out height);
  frame.get_scale_factor(out scale_factor);
  frame.get_frame_id(out frame_id);
  frame.get_pixel_format(out pixel_format);
  MaplibreNative.NativePointer texture = frame.get_texture();
  MaplibreNative.NativePointer device = frame.get_device();
  bool frame_fields_seen = texture.get_bits() != 0 && device.get_bits() != 0 && generation != frame_id && width != height && scale_factor != 0.0 && pixel_format != 0;
  if (frame_fields_seen) {
    GLib.stderr.printf("");
  }
  frame.close();
}

void inspect_vulkan_owned_texture_frame(MaplibreNative.VulkanOwnedTextureFrameHandle frame) throws GLib.Error {
  uint64 generation;
  uint32 width;
  uint32 height;
  double scale_factor;
  uint64 frame_id;
  uint32 format;
  uint32 layout;
  frame.get_generation(out generation);
  frame.get_width(out width);
  frame.get_height(out height);
  frame.get_scale_factor(out scale_factor);
  frame.get_frame_id(out frame_id);
  frame.get_format(out format);
  frame.get_layout(out layout);
  MaplibreNative.NativePointer image = frame.get_image();
  MaplibreNative.NativePointer image_view = frame.get_image_view();
  MaplibreNative.NativePointer device = frame.get_device();
  bool frame_fields_seen = image.get_bits() != 0 && image_view.get_bits() != 0 && device.get_bits() != 0 && generation != frame_id && width != height && scale_factor != 0.0 && format != layout;
  if (frame_fields_seen) {
    GLib.stderr.printf("");
  }
  frame.close();
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
    MaplibreNative.RenderedFeatureQueryOptions rendered_query_options = {};
    rendered_query_options.default();
    MaplibreNative.SourceFeatureQueryOptions source_query_options = {};
    source_query_options.default();

    MaplibreNative.LatLng coordinate = { 37.7749, -122.4194 };
    MaplibreNative.ScreenPoint query_point = { 0.0, 0.0 };
    MaplibreNative.RenderedQueryGeometry query_geometry;
    MaplibreNative.RenderedQueryGeometry.point(query_point, out query_geometry);
    MaplibreNative.ScreenBox query_box = { { 0.0, 0.0 }, { 1.0, 1.0 } };
    MaplibreNative.RenderedQueryGeometry.box(query_box, out query_geometry);
    MaplibreNative.ScreenPoint[] query_points = { { 0.0, 0.0 }, { 1.0, 1.0 } };
    MaplibreNative.RenderedQueryGeometry.line_string(query_points, out query_geometry);

    MaplibreNative.ProjectedMeters meters;
    MaplibreNative.ProjectedMeters.for_lat_lng(coordinate, out meters);
    MaplibreNative.LatLng round_trip;
    MaplibreNative.LatLng.for_projected_meters(meters, out round_trip);

    MaplibreNative.RuntimeOptions runtime_options = {};
    runtime_options.default();
    var runtime = new MaplibreNative.RuntimeHandle.with_options(runtime_options);
    runtime.set_resource_provider(provide_resource);
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
    size_t attribution_size = 0;
    bool attribution_found = false;
    map.copy_style_source_attribution("fixture-source", null, out attribution_size, out attribution_found);
    map.set_geojson_source_url("fixture-source", "asset://fixture-updated.geojson");
    var source_ids = map.list_style_source_ids();
    size_t source_id_count;
    source_ids.count(out source_id_count);
    if (source_id_count > 0) {
      string first_source_id = source_ids.get(0);
      if (first_source_id.length == 0) {
        return 1;
      }
    }
    source_ids.close();
    bool source_removed = false;
    map.remove_style_source("fixture-source", out source_removed);
    MaplibreNative.StyleTileSourceOptions tile_source_options = {};
    tile_source_options.default();
    map.add_vector_source_url("vector-source", "asset://vector-source.json", tile_source_options);
    map.remove_style_source("vector-source", out source_removed);
    map.add_raster_source_url("raster-source", "asset://raster-source.json", tile_source_options);
    map.remove_style_source("raster-source", out source_removed);
    map.add_raster_dem_source_url("dem-source", "asset://dem-source.json", tile_source_options);
    map.add_hillshade_layer("hillshade-layer", "dem-source", "");
    map.add_color_relief_layer("color-relief-layer", "dem-source", "");
    map.remove_style_layer("hillshade-layer", out source_removed);
    map.remove_style_layer("color-relief-layer", out source_removed);
    map.remove_style_source("dem-source", out source_removed);
    uint8[] image_pixels = { 255, 0, 0, 255 };
    MaplibreNative.PremultipliedRgba8Image style_image = {};
    style_image.init(1, 1, 4, image_pixels);
    MaplibreNative.StyleImageOptions style_image_options = {};
    style_image_options.default();
    MaplibreNative.StyleImageInfo style_image_info = {};
    style_image_info.default();
    map.set_style_image("fixture-image", style_image, style_image_options);
    bool image_exists = false;
    map.style_image_exists("fixture-image", out image_exists);
    bool image_found = false;
    map.get_style_image_info("fixture-image", out style_image_info, out image_found);
    bool image_removed = false;
    map.remove_style_image("fixture-image", out image_removed);
    MaplibreNative.LatLng[] image_coordinates = {
      { 0.0, 0.0 },
      { 0.0, 1.0 },
      { 1.0, 1.0 },
      { 1.0, 0.0 },
    };
    map.add_image_source_url("image-source", image_coordinates, "asset://image.png");
    map.set_image_source_url("image-source", "asset://image-updated.png");
    map.set_image_source_coordinates("image-source", image_coordinates);
    map.remove_style_source("image-source", out source_removed);
    map.add_image_source_image("inline-image-source", image_coordinates, style_image);
    map.set_image_source_image("inline-image-source", style_image);
    map.remove_style_source("inline-image-source", out source_removed);
    map.add_location_indicator_layer("location-layer", "");
    map.set_location_indicator_location("location-layer", coordinate, 0.0);
    map.set_location_indicator_bearing("location-layer", 0.0);
    map.set_location_indicator_accuracy_radius("location-layer", 0.0);
    map.set_location_indicator_image_name("location-layer", MaplibreNative.LocationIndicatorImageKind.TOP, "fixture-image");
    bool layer_exists = false;
    map.style_layer_exists("location-layer", out layer_exists);
    string? layer_type;
    bool layer_found = false;
    map.get_style_layer_type("location-layer", out layer_type, out layer_found);
    var layer_ids = map.list_style_layer_ids();
    size_t layer_id_count;
    layer_ids.count(out layer_id_count);
    if (layer_id_count > 0) {
      string first_layer_id = layer_ids.get(0);
      if (first_layer_id.length == 0) {
        return 1;
      }
    }
    layer_ids.close();
    map.move_style_layer("location-layer", "");
    bool layer_removed = false;
    map.remove_style_layer("location-layer", out layer_removed);
    MaplibreNative.MetalSurfaceDescriptor metal_surface = {};
    metal_surface.default();
    MaplibreNative.VulkanSurfaceDescriptor vulkan_surface = {};
    vulkan_surface.default();
    MaplibreNative.MetalOwnedTextureDescriptor metal_owned_texture = {};
    metal_owned_texture.default();
    MaplibreNative.MetalBorrowedTextureDescriptor metal_borrowed_texture = {};
    metal_borrowed_texture.default();
    MaplibreNative.VulkanOwnedTextureDescriptor vulkan_owned_texture = {};
    vulkan_owned_texture.default();
    MaplibreNative.VulkanBorrowedTextureDescriptor vulkan_borrowed_texture = {};
    vulkan_borrowed_texture.default();
    MaplibreNative.TextureImageInfo texture_info = {};
    texture_info.default();
    MaplibreNative.MetalOwnedTextureFrameHandle? metal_frame = null;
    if (metal_frame != null) {
      inspect_metal_owned_texture_frame(metal_frame);
    }
    MaplibreNative.VulkanOwnedTextureFrameHandle? vulkan_frame = null;
    if (vulkan_frame != null) {
      inspect_vulkan_owned_texture_frame(vulkan_frame);
    }
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
    MaplibreNative.ScreenPoint map_point;
    map.pixel_for_lat_lng(coordinate, out map_point);
    map.lat_lng_for_pixel(map_point, out round_trip);

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
    projection.get_camera(out camera);
    projection.set_camera(camera);
    MaplibreNative.EdgeInsets projection_padding = { 0.0, 0.0, 0.0, 0.0 };
    MaplibreNative.LatLng[] visible_coordinates = { coordinate, round_trip };
    projection.set_visible_coordinates(visible_coordinates, projection_padding);
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

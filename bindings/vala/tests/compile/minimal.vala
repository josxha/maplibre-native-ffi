[CCode (has_target = false)]
delegate void* MetalCreateSystemDefaultDeviceFunc();

GLib.Mutex log_lock;
int log_first_count = 0;
int log_second_count = 0;
GLib.Mutex provider_lock;
int provider_call_count = 0;
bool provider_cancel_checked = false;
bool provider_second_complete_failed = false;

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

int read_provider_call_count() {
  provider_lock.lock();
  int count = provider_call_count;
  provider_lock.unlock();
  return count;
}

void note_provider_call() {
  provider_lock.lock();
  provider_call_count++;
  provider_lock.unlock();
}

void note_provider_cancel_checked() {
  provider_lock.lock();
  provider_cancel_checked = true;
  provider_lock.unlock();
}

void note_provider_second_complete_failed() {
  provider_lock.lock();
  provider_second_complete_failed = true;
  provider_lock.unlock();
}

void read_provider_state(out int call_count, out bool cancel_checked, out bool second_complete_failed) {
  provider_lock.lock();
  call_count = provider_call_count;
  cancel_checked = provider_cancel_checked;
  second_complete_failed = provider_second_complete_failed;
  provider_lock.unlock();
}

void note_first_log() {
  log_lock.lock();
  log_first_count++;
  log_lock.unlock();
}

void note_replacement_log() {
  log_lock.lock();
  log_second_count++;
  log_lock.unlock();
}

void read_log_state(out int first_count, out int second_count) {
  log_lock.lock();
  first_count = log_first_count;
  second_count = log_second_count;
  log_lock.unlock();
}

bool handle_log(MaplibreNative.LogSeverity severity, MaplibreNative.LogEvent event, int64 code, string? message) {
  note_first_log();
  return false;
}

bool handle_replacement_log(MaplibreNative.LogSeverity severity, MaplibreNative.LogEvent event, int64 code, string? message) {
  note_replacement_log();
  return false;
}

string? transform_resource(MaplibreNative.ResourceKind kind, string url) {
  return null;
}

string? replacement_transform_resource(MaplibreNative.ResourceKind kind, string url) {
  return null;
}

MaplibreNative.ResourceProviderDecision provide_resource(MaplibreNative.ResourceRequest request, MaplibreNative.ResourceRequestHandle handle) {
  note_provider_call();
  try {
    bool cancelled = false;
    handle.is_cancelled(out cancelled);
    note_provider_cancel_checked();

    MaplibreNative.ResourceResponse response = {};
    response.default();
    response.status = MaplibreNative.ResourceResponseStatus.ERROR;
    response.error_reason = MaplibreNative.ResourceErrorReason.OTHER;
    response.error_message = "vala provider handled request";
    var retained = handle.retain_for_async();
    retained.complete(response);
    try {
      retained.complete(response);
    } catch (GLib.Error second_error) {
      note_provider_second_complete_failed();
    }
    retained.release();
    handle.release();
    handle.release();
    return MaplibreNative.ResourceProviderDecision.HANDLE;
  } catch (GLib.Error error) {
    return MaplibreNative.ResourceProviderDecision.PASS_THROUGH;
  }
}

bool wait_for_runtime_event(MaplibreNative.RuntimeHandle runtime, MaplibreNative.RuntimeEventType event_type, uint attempts) throws GLib.Error {
  for (uint attempt = 0; attempt < attempts; attempt++) {
    runtime.run_once();
    MaplibreNative.RuntimeEvent? event = null;
    runtime.poll_event(out event);
    if (event != null && event.get_event_type() == event_type) {
      return true;
    }
  }
  return false;
}

bool exercise_metal_owned_texture_runtime(MaplibreNative.RuntimeHandle runtime, MaplibreNative.MapHandle map, MaplibreNative.RenderBackendFlags backends) throws GLib.Error {
  if ((backends & MaplibreNative.RenderBackendFlags.METAL) == 0) {
    return true;
  }
  void* device = create_system_default_metal_device();
  if (device == null) {
    return true;
  }

  MaplibreNative.NativePointer device_pointer;
  MaplibreNative.NativePointer.@new((size_t) device, out device_pointer);

  MaplibreNative.MetalOwnedTextureDescriptor descriptor = {};
  descriptor.default();
  descriptor.extent.@set(32, 16, 1.0);
  descriptor.context.set_device(device_pointer);

  var session = map.attach_metal_owned_texture(descriptor);
  map.set_style_json("{\"version\":8,\"sources\":{},\"layers\":[{\"id\":\"background\",\"type\":\"background\",\"paint\":{\"background-color\":\"#d8f1ff\"}}]}");
  if (!wait_for_runtime_event(runtime, MaplibreNative.RuntimeEventType.MAP_RENDER_UPDATE_AVAILABLE, 64)) {
    session.close();
    return false;
  }
  session.render_update();

  uint8[] readback_pixels = new uint8[32 * 16 * 4];
  MaplibreNative.TextureImageInfo readback_info;
  session.read_premultiplied_rgba8(readback_pixels, out readback_info);
  bool readback_matches_extent = readback_info.width == 32 && readback_info.height == 16 && readback_info.stride * readback_info.height <= readback_pixels.length;

  var frame = session.acquire_metal_owned_texture_frame();
  uint32 width;
  frame.get_width(out width);
  if (width != 32 || !readback_matches_extent) {
    frame.close();
    session.close();
    return false;
  }

  bool reentrant_render_rejected = false;
  try {
    session.render_update();
  } catch (GLib.Error error) {
    reentrant_render_rejected = true;
  }
  frame.close();

  bool frame_closed_error_seen = false;
  try {
    frame.get_width(out width);
  } catch (GLib.Error error) {
    frame_closed_error_seen = true;
  }
  session.render_update();
  session.close();
  session.close();
  return reentrant_render_rejected && frame_closed_error_seen;
}

bool probe_wrong_thread(MaplibreNative.MapHandle map) {
  try {
    bool loaded = false;
    map.is_fully_loaded(out loaded);
    return false;
  } catch (GLib.Error error) {
    return true;
  }
}

void exercise_offline_operations(MaplibreNative.RuntimeHandle runtime) throws GLib.Error {
  uint64 operation_id;
  MaplibreNative.OfflineRegionDefinition definition = {};
  uint8[] metadata = { 1, 2, 3 };
  runtime.offline_region_create_start(definition, metadata, out operation_id);
  runtime.offline_region_get_start(1, out operation_id);
  runtime.offline_regions_list_start(out operation_id);
  runtime.offline_regions_merge_database_start("offline.db", out operation_id);
  runtime.offline_region_update_metadata_start(1, metadata, out operation_id);
  runtime.offline_region_get_status_start(1, out operation_id);
  runtime.offline_region_set_observed_start(1, true, out operation_id);
  runtime.offline_region_set_download_state_start(1, MaplibreNative.OfflineRegionDownloadState.INACTIVE, out operation_id);
  MaplibreNative.OfflineRegionStatus status;
  runtime.offline_region_get_status_take_result(operation_id, out status);
  var created_region = runtime.offline_region_create_take_result(operation_id);
  MaplibreNative.OfflineRegionInfo region_info;
  created_region.get(out region_info);
  created_region.close();
  bool found = false;
  var optional_region = runtime.offline_region_get_take_result(operation_id, out found);
  if (optional_region != null) {
    optional_region.close();
  }
  var updated_region = runtime.offline_region_update_metadata_take_result(operation_id);
  updated_region.close();
  var region_list = runtime.offline_regions_list_take_result(operation_id);
  size_t region_count;
  region_list.count(out region_count);
  region_list.get(0, out region_info);
  region_list.close();
  var merged_regions = runtime.offline_regions_merge_database_take_result(operation_id);
  merged_regions.close();
  runtime.offline_region_invalidate_start(1, out operation_id);
  runtime.offline_region_delete_start(1, out operation_id);
  runtime.offline_operation_discard(operation_id);
}

void exercise_json_style(MaplibreNative.MapHandle map, MaplibreNative.JsonValue json) throws GLib.Error {
  map.add_style_source_json("json-source", json);
  map.add_style_layer_json(json, "");
  bool found = false;
  var layer_json = map.get_style_layer_json("json-layer", out found);
  if (layer_json != null) {
    MaplibreNative.JsonValue root;
    layer_json.get(out root);
    root.free();
    layer_json.close();
  }
  map.set_style_light_json(json);
  map.set_style_light_property("anchor", json);
  var light_property = map.get_style_light_property("anchor");
  if (light_property != null) {
    light_property.close();
  }
  map.set_layer_property("json-layer", "visibility", json);
  var layer_property = map.get_layer_property("json-layer", "visibility");
  if (layer_property != null) {
    layer_property.close();
  }
  map.set_layer_filter("json-layer", json);
  var layer_filter = map.get_layer_filter("json-layer");
  if (layer_filter != null) {
    layer_filter.close();
  }
}

void exercise_inline_source_data(MaplibreNative.MapHandle map, MaplibreNative.Geometry geometry) throws GLib.Error {
  string source_id = "fixture-source";
  var data = new MaplibreNative.GeoJson.geometry(geometry);
  var feature_data = new MaplibreNative.GeoJson.feature(geometry);
  MaplibreNative.Geometry[] collection_geometries = { geometry };
  var collection_data = new MaplibreNative.GeoJson.feature_collection(collection_geometries);
  map.add_geojson_source_data(source_id, data);
  map.set_geojson_source_data(source_id, feature_data);
  map.set_geojson_source_data(source_id, collection_data);

  MaplibreNative.CustomGeometrySourceOptions options = {};
  options.default();
  int captured_fetch_count = 0;
  map.add_custom_geometry_source_with_callbacks("custom-geometry-source", (tile_id) => {
    captured_fetch_count++;
  }, null, options);
  MaplibreNative.CanonicalTileId tile_id = { 0, 0, 0 };
  map.set_custom_geometry_source_tile_data("custom-geometry-source", tile_id, data);
  map.invalidate_custom_geometry_source_tile("custom-geometry-source", tile_id);
  MaplibreNative.LatLngBounds bounds = { { -1.0, -1.0 }, { 1.0, 1.0 } };
  map.invalidate_custom_geometry_source_region("custom-geometry-source", bounds);
}

void exercise_feature_state(MaplibreNative.RenderSessionHandle session, MaplibreNative.JsonValue state) throws GLib.Error {
  string source_id = "fixture-source";
  string feature_id = "feature-1";
  MaplibreNative.FeatureStateSelector selector = {};
  selector.source_id = { source_id, source_id.length };
  selector.feature_id = { feature_id, feature_id.length };
  session.set_feature_state(selector, state);
  var snapshot = session.get_feature_state(selector);
  snapshot.close();
  session.remove_feature_state(selector);
}

void exercise_feature_queries(MaplibreNative.RenderSessionHandle session) throws GLib.Error {
  MaplibreNative.RenderedFeatureQueryOptions rendered_options = {};
  rendered_options.default();
  string rendered_layer_id = "background";
  MaplibreNative.StringView[] rendered_layers = { { rendered_layer_id, rendered_layer_id.length } };
  rendered_options.set_layer_ids(rendered_layers);
  MaplibreNative.SourceFeatureQueryOptions source_options = {};
  source_options.default();
  string source_layer_id = "fixture-layer";
  MaplibreNative.StringView[] source_layers = { { source_layer_id, source_layer_id.length } };
  source_options.set_source_layer_ids(source_layers);
  MaplibreNative.ScreenPoint point = { 0.0, 0.0 };
  MaplibreNative.RenderedQueryGeometry geometry;
  MaplibreNative.RenderedQueryGeometry.point(point, out geometry);
  var rendered_result = session.query_rendered_features(geometry, rendered_options);
  size_t rendered_count;
  rendered_result.count(out rendered_count);
  MaplibreNative.QueriedFeature queried_feature;
  rendered_result.get(0, out queried_feature);
  rendered_result.close();
  var source_result = session.query_source_features("fixture-source", source_options);
  size_t source_count;
  source_result.count(out source_count);
  source_result.close();
}

void exercise_geometry_camera(MaplibreNative.MapHandle map, MaplibreNative.MapProjectionHandle projection, MaplibreNative.Geometry geometry) throws GLib.Error {
  MaplibreNative.CameraFitOptions fit_options = {};
  fit_options.default();
  MaplibreNative.CameraOptions camera;
  map.camera_for_geometry(geometry, fit_options, out camera);
  MaplibreNative.EdgeInsets padding = { 0.0, 0.0, 0.0, 0.0 };
  projection.set_visible_geometry(geometry, padding);
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
    MaplibreNative.log_set_callback(handle_replacement_log);

    MaplibreNative.NativePointer native_pointer;
    MaplibreNative.NativePointer.@new(0x1234, out native_pointer);
    size_t pointer_bits = native_pointer.get_bits();
    MaplibreNative.ResourceResponse response = {};
    response.default();
    MaplibreNative.RenderedFeatureQueryOptions rendered_query_options = {};
    rendered_query_options.default();
    string rendered_option_layer_id = "background";
    MaplibreNative.StringView[] rendered_option_layers = { { rendered_option_layer_id, rendered_option_layer_id.length } };
    rendered_query_options.set_layer_ids(rendered_option_layers);
    MaplibreNative.SourceFeatureQueryOptions source_query_options = {};
    source_query_options.default();
    string source_option_layer_id = "fixture-layer";
    MaplibreNative.StringView[] source_option_layers = { { source_option_layer_id, source_option_layer_id.length } };
    source_query_options.set_source_layer_ids(source_option_layers);

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
    runtime_options.maximum_cache_size = 1024 * 1024;
    var runtime = new MaplibreNative.RuntimeHandle.with_options(runtime_options);
    runtime.set_resource_provider(provide_resource);
    runtime.set_resource_transform(transform_resource);
    runtime.set_resource_transform(replacement_transform_resource);
    runtime.run_once();
    MaplibreNative.RuntimeEvent? event = null;
    runtime.poll_event(out event);
    bool event_copy_matches = true;
    if (event != null) {
      var event_copy = event.copy();
      event_copy_matches = event_copy.get_event_type() == event.get_event_type() && event_copy.get_source_type() == event.get_source_type() && event_copy.get_payload_type() == event.get_payload_type() && event_copy.get_code() == event.get_code();
    }

    MaplibreNative.MapOptions map_options = {};
    map_options.default();
    map_options.width = 512;
    map_options.height = 512;
    map_options.scale_factor = 1.0;
    map_options.map_mode = MaplibreNative.MapMode.CONTINUOUS;
    var map = new MaplibreNative.MapHandle.with_options(runtime, map_options);
    map.set_style_url("custom://style.json");
    for (int provider_spin = 0; provider_spin < 200 && read_provider_call_count() == 0; provider_spin++) {
      runtime.run_once();
      GLib.Thread.usleep(10000);
    }
    map.set_style_json("{\"version\":8,\"sources\":{},\"layers\":[]}");
    map.add_geojson_source_url("fixture-source", "asset://fixture.geojson");
    bool source_exists = false;
    map.style_source_exists("fixture-source", out source_exists);
    MaplibreNative.StyleSourceType source_type;
    bool source_found = false;
    map.get_style_source_type("fixture-source", out source_type, out source_found);
    MaplibreNative.StyleSourceInfo source_info;
    map.get_style_source_info("fixture-source", out source_info, out source_found);
    char[] attribution_buffer = new char[256];
    size_t attribution_size = 0;
    bool attribution_found = false;
    map.copy_style_source_attribution("fixture-source", attribution_buffer, out attribution_size, out attribution_found);
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
    string vector_tile_url = "asset://vector/{z}/{x}/{y}.pbf";
    string raster_tile_url = "asset://raster/{z}/{x}/{y}.png";
    string dem_tile_url = "asset://dem/{z}/{x}/{y}.png";
    MaplibreNative.StringView[] vector_tile_urls = { { vector_tile_url, vector_tile_url.length } };
    MaplibreNative.StringView[] raster_tile_urls = { { raster_tile_url, raster_tile_url.length } };
    MaplibreNative.StringView[] dem_tile_urls = { { dem_tile_url, dem_tile_url.length } };
    map.add_vector_source_url("vector-source", "asset://vector-source.json", tile_source_options);
    map.remove_style_source("vector-source", out source_removed);
    map.add_vector_source_tiles("vector-tiles-source", vector_tile_urls, tile_source_options);
    map.remove_style_source("vector-tiles-source", out source_removed);
    map.add_raster_source_url("raster-source", "asset://raster-source.json", tile_source_options);
    map.remove_style_source("raster-source", out source_removed);
    map.add_raster_source_tiles("raster-tiles-source", raster_tile_urls, tile_source_options);
    map.remove_style_source("raster-tiles-source", out source_removed);
    map.add_raster_dem_source_url("dem-source", "asset://dem-source.json", tile_source_options);
    map.add_raster_dem_source_tiles("dem-tiles-source", dem_tile_urls, tile_source_options);
    map.remove_style_source("dem-tiles-source", out source_removed);
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
    uint8[] style_image_copy = new uint8[4];
    size_t style_image_copy_size = 0;
    bool style_image_copy_found = false;
    map.copy_style_image_premultiplied_rgba8("fixture-image", style_image_copy, out style_image_copy_size, out style_image_copy_found);
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
    MaplibreNative.LatLng[] copied_image_coordinates = new MaplibreNative.LatLng[4];
    size_t copied_image_coordinate_count = 0;
    map.get_image_source_coordinates("image-source", copied_image_coordinates, out copied_image_coordinate_count, out source_found);
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
    bool metal_frame_runtime_seen = exercise_metal_owned_texture_runtime(runtime, map, backends);
    var vulkan_frame_object = GLib.Object.new(typeof(MaplibreNative.VulkanOwnedTextureFrameHandle));
    var vulkan_frame = (MaplibreNative.VulkanOwnedTextureFrameHandle) vulkan_frame_object;
    vulkan_frame.close();
    vulkan_frame.close();
    bool vulkan_frame_closed_error_seen = false;
    try {
      uint32 closed_frame_width;
      vulkan_frame.get_width(out closed_frame_width);
    } catch (GLib.Error error) {
      vulkan_frame_closed_error_seen = true;
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
    MaplibreNative.CameraFitOptions fit_options = {};
    fit_options.default();
    MaplibreNative.LatLngBounds camera_bounds = {
      { -1.0, -1.0 },
      { 1.0, 1.0 },
    };
    MaplibreNative.CameraOptions fitted_camera;
    map.camera_for_lat_lng_bounds(camera_bounds, fit_options, out fitted_camera);
    MaplibreNative.LatLng[] fit_coordinates = {
      { -1.0, -1.0 },
      { 1.0, 1.0 },
    };
    map.camera_for_lat_lngs(fit_coordinates, fit_options, out fitted_camera);
    MaplibreNative.LatLngBounds fitted_bounds;
    map.lat_lng_bounds_for_camera(fitted_camera, out fitted_bounds);
    map.lat_lng_bounds_for_camera_unwrapped(fitted_camera, out fitted_bounds);
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
    MaplibreNative.ScreenPoint[] projected_points = new MaplibreNative.ScreenPoint[fit_coordinates.length];
    map.pixels_for_lat_lngs(fit_coordinates, projected_points);
    MaplibreNative.LatLng[] unprojected_coordinates = new MaplibreNative.LatLng[projected_points.length];
    map.lat_lngs_for_pixels(projected_points, unprojected_coordinates);

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
    var wrong_thread = new GLib.Thread<bool>("wrong-thread", () => {
      return probe_wrong_thread(map);
    });
    bool wrong_thread_error_seen = wrong_thread.join();
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

    unowned MaplibreNative.Geometry? bindability_geometry = null;
    unowned MaplibreNative.JsonValue? bindability_json = null;
    MaplibreNative.RenderSessionHandle? bindability_session = null;
    MaplibreNative.MetalOwnedTextureFrameHandle? bindability_metal_frame = null;
    MaplibreNative.VulkanOwnedTextureFrameHandle? bindability_vulkan_frame = null;
    if (args.length < 0 && bindability_geometry != null && bindability_json != null && bindability_session != null) {
      exercise_offline_operations(runtime);
      exercise_json_style(map, bindability_json);
      exercise_inline_source_data(map, bindability_geometry);
      exercise_feature_state(bindability_session, bindability_json);
      exercise_feature_queries(bindability_session);
      exercise_geometry_camera(map, projection, bindability_geometry);
      if (bindability_metal_frame != null) {
        inspect_metal_owned_texture_frame(bindability_metal_frame);
      }
      if (bindability_vulkan_frame != null) {
        inspect_vulkan_owned_texture_frame(bindability_vulkan_frame);
      }
    }

    projection.close();

    map.close();
    map.close();
    runtime.clear_resource_transform();
    runtime.close();
    runtime.close();
    MaplibreNative.log_clear_callback();

    int final_provider_call_count;
    bool final_provider_cancel_checked;
    bool final_provider_second_complete_failed;
    read_provider_state(out final_provider_call_count, out final_provider_cancel_checked, out final_provider_second_complete_failed);
    int final_log_first_count;
    int final_log_second_count;
    read_log_state(out final_log_first_count, out final_log_second_count);
    if (backends == 0 || pointer_bits != 0x1234 || !event_copy_matches || !wrong_thread_error_seen || !final_provider_cancel_checked || !final_provider_second_complete_failed || final_provider_call_count == 0 || final_log_first_count != 0 || final_log_second_count == 0 || !metal_frame_runtime_seen || !vulkan_frame_closed_error_seen) {
      GLib.stderr.printf("vala runtime check failed: backends=%u pointer=%" + size_t.FORMAT + " event=%s wrong_thread=%s provider_cancel=%s provider_second=%s provider_calls=%d log_first=%d log_second=%d metal_frame=%s vulkan_frame=%s\n", (uint) backends, pointer_bits, event_copy_matches.to_string(), wrong_thread_error_seen.to_string(), final_provider_cancel_checked.to_string(), final_provider_second_complete_failed.to_string(), final_provider_call_count, final_log_first_count, final_log_second_count, metal_frame_runtime_seen.to_string(), vulkan_frame_closed_error_seen.to_string());
      return 1;
    }
  } catch (GLib.Error error) {
    return 1;
  }

  return version == 0 ? 0 : 0;
}

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

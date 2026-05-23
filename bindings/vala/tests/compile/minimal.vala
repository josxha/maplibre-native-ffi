int main(string[] args) {
  uint32 version = MaplibreNative.c_version();
  MaplibreNative.NetworkStatus status;

  try {
    MaplibreNative.RenderBackendFlags backends = MaplibreNative.supported_render_backends();
    MaplibreNative.NetworkStatus.get(out status);
    status.set();

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
    map.set_style_json("{\"version\":8,\"sources\":{},\"layers\":[]}");

    var projection = new MaplibreNative.MapProjectionHandle(map);
    MaplibreNative.ScreenPoint point;
    projection.pixel_for_lat_lng(coordinate, out point);
    projection.lat_lng_for_pixel(point, out round_trip);
    projection.close();

    map.close();
    runtime.close();

    if (backends == 0) {
      return 1;
    }
  } catch (GLib.Error error) {
    return 1;
  }

  return version == 0 ? 0 : 0;
}

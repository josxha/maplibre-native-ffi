int main(string[] args) {
  uint32 version = MaplibreNative.c_version();
  uint32 status = 0;

  try {
    MaplibreNative.network_status_get(out status);
    MaplibreNative.network_status_set(status);

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
    map.close();
    runtime.close();
  } catch (GLib.Error error) {
    return 1;
  }

  return version == 0 ? 0 : 0;
}

int main(string[] args) {
  uint32 version = MaplibreNative.c_version();
  uint32 status = 0;

  try {
    MaplibreNative.network_status_get(out status);
    MaplibreNative.network_status_set(status);

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

int main(string[] args) {
  uint32 version = MaplibreNative.c_version();
  uint32 status = 0;

  try {
    MaplibreNative.network_status_get(out status);
    MaplibreNative.network_status_set(status);
  } catch (GLib.Error error) {
    return 1;
  }

  return version == 0 ? 0 : 0;
}

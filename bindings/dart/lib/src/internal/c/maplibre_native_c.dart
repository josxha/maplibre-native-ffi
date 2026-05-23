import 'dart:ffi';

import 'package:ffi/ffi.dart';

import '../loader/native_library.dart';
import 'maplibre_native_c.g.dart' as raw;

/// Curated internal facade over generated MapLibre Native C declarations.
final class MaplibreNativeCApi {
  MaplibreNativeCApi._(this.library) : _raw = raw.MaplibreNativeC(library);

  /// Opens the native library and resolves generated C symbols lazily.
  factory MaplibreNativeCApi.open({String? path}) =>
      MaplibreNativeCApi._(openMaplibreNativeCLibrary(path: path));

  /// Native library that owns the resolved C symbols.
  final DynamicLibrary library;
  final raw.MaplibreNativeC _raw;

  /// Returns the native C ABI contract version.
  int cVersion() => _raw.mln_c_version();

  /// Returns the render backend support mask reported by the native library.
  int supportedRenderBackendMask() => _raw.mln_supported_render_backend_mask();

  /// Copies the current thread-local native diagnostic message.
  String threadLastErrorMessage() {
    final pointer = _raw.mln_thread_last_error_message();
    if (pointer == nullptr) {
      return '';
    }
    return pointer.cast<Utf8>().toDartString();
  }

  /// Reads MapLibre Native's process-global network status.
  int networkStatusGet(Pointer<Uint32> outStatus) =>
      _raw.mln_network_status_get(outStatus).value;

  /// Sets MapLibre Native's process-global network status.
  int networkStatusSet(int status) => _raw.mln_network_status_set(status).value;

  /// Clears the process-global native log callback.
  int logClearCallback() => _raw.mln_log_clear_callback().value;

  /// Sets the asynchronous native log severity mask.
  int logSetAsyncSeverityMask(int mask) =>
      _raw.mln_log_set_async_severity_mask(mask).value;

  /// Creates a runtime handle.
  int runtimeCreate(
    Pointer<raw.mln_runtime_options> options,
    Pointer<Pointer<raw.mln_runtime>> outRuntime,
  ) => _raw.mln_runtime_create(options, outRuntime).value;

  /// Destroys a runtime handle.
  int runtimeDestroy(Pointer<raw.mln_runtime> runtime) =>
      _raw.mln_runtime_destroy(runtime).value;

  /// Runs one pending owner-thread task for a runtime.
  int runtimeRunOnce(Pointer<raw.mln_runtime> runtime) =>
      _raw.mln_runtime_run_once(runtime).value;

  /// Polls one queued runtime event.
  int runtimePollEvent(
    Pointer<raw.mln_runtime> runtime,
    Pointer<raw.mln_runtime_event> outEvent,
    Pointer<Bool> outHasEvent,
  ) => _raw.mln_runtime_poll_event(runtime, outEvent, outHasEvent).value;

  /// Returns native default map options.
  raw.mln_map_options mapOptionsDefault() => _raw.mln_map_options_default();

  /// Creates a map handle.
  int mapCreate(
    Pointer<raw.mln_runtime> runtime,
    Pointer<raw.mln_map_options> options,
    Pointer<Pointer<raw.mln_map>> outMap,
  ) => _raw.mln_map_create(runtime, options, outMap).value;

  /// Destroys a map handle.
  int mapDestroy(Pointer<raw.mln_map> map) => _raw.mln_map_destroy(map).value;

  /// Sets the map style URL.
  int mapSetStyleUrl(Pointer<raw.mln_map> map, Pointer<Char> url) =>
      _raw.mln_map_set_style_url(map, url).value;

  /// Sets the map style JSON.
  int mapSetStyleJson(Pointer<raw.mln_map> map, Pointer<Char> json) =>
      _raw.mln_map_set_style_json(map, json).value;

  /// Counts IDs in a style ID list.
  int styleIdListCount(
    Pointer<raw.mln_style_id_list> list,
    Pointer<Size> outCount,
  ) => _raw.mln_style_id_list_count(list, outCount).value;

  /// Borrows one ID from a style ID list.
  int styleIdListGet(
    Pointer<raw.mln_style_id_list> list,
    int index,
    Pointer<raw.mln_string_view> outId,
  ) => _raw.mln_style_id_list_get(list, index, outId).value;

  /// Destroys a style ID list handle.
  void styleIdListDestroy(Pointer<raw.mln_style_id_list> list) {
    _raw.mln_style_id_list_destroy(list);
  }

  /// Adds one style source from a style-spec source JSON object.
  int mapAddStyleSourceJson(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_json_value> sourceJson,
  ) => _raw.mln_map_add_style_source_json(map, sourceId, sourceJson).value;

  /// Adds a GeoJSON source with inline data.
  int mapAddGeoJsonSourceData(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_geojson> data,
  ) => _raw.mln_map_add_geojson_source_data(map, sourceId, data).value;

  /// Updates a GeoJSON source with inline data.
  int mapSetGeoJsonSourceData(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_geojson> data,
  ) => _raw.mln_map_set_geojson_source_data(map, sourceId, data).value;

  /// Reports whether a style source exists.
  int mapStyleSourceExists(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<Bool> outExists,
  ) => _raw.mln_map_style_source_exists(map, sourceId, outExists).value;

  /// Removes one style source by ID.
  int mapRemoveStyleSource(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<Bool> outRemoved,
  ) => _raw.mln_map_remove_style_source(map, sourceId, outRemoved).value;

  /// Gets fixed style source metadata.
  int mapGetStyleSourceInfo(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_style_source_info> outInfo,
    Pointer<Bool> outFound,
  ) => _raw
      .mln_map_get_style_source_info(map, sourceId, outInfo, outFound)
      .value;

  /// Copies one style source attribution string.
  int mapCopyStyleSourceAttribution(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<Char> outAttribution,
    int attributionCapacity,
    Pointer<Size> outAttributionSize,
    Pointer<Bool> outFound,
  ) => _raw
      .mln_map_copy_style_source_attribution(
        map,
        sourceId,
        outAttribution,
        attributionCapacity,
        outAttributionSize,
        outFound,
      )
      .value;

  /// Lists style source IDs.
  int mapListStyleSourceIds(
    Pointer<raw.mln_map> map,
    Pointer<Pointer<raw.mln_style_id_list>> outSourceIds,
  ) => _raw.mln_map_list_style_source_ids(map, outSourceIds).value;

  /// Adds one style layer from a style-spec layer JSON object.
  int mapAddStyleLayerJson(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_json_value> layerJson,
    raw.mln_string_view beforeLayerId,
  ) => _raw.mln_map_add_style_layer_json(map, layerJson, beforeLayerId).value;

  /// Reports whether a style layer exists.
  int mapStyleLayerExists(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    Pointer<Bool> outExists,
  ) => _raw.mln_map_style_layer_exists(map, layerId, outExists).value;

  /// Gets one style layer type string.
  int mapGetStyleLayerType(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    Pointer<raw.mln_string_view> outLayerType,
    Pointer<Bool> outFound,
  ) => _raw
      .mln_map_get_style_layer_type(map, layerId, outLayerType, outFound)
      .value;

  /// Removes one style layer by ID.
  int mapRemoveStyleLayer(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    Pointer<Bool> outRemoved,
  ) => _raw.mln_map_remove_style_layer(map, layerId, outRemoved).value;

  /// Lists style layer IDs.
  int mapListStyleLayerIds(
    Pointer<raw.mln_map> map,
    Pointer<Pointer<raw.mln_style_id_list>> outLayerIds,
  ) => _raw.mln_map_list_style_layer_ids(map, outLayerIds).value;
}

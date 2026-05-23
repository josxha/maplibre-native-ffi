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

  /// Copies the current camera snapshot.
  int mapGetCamera(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_camera_options> outCamera,
  ) => _raw.mln_map_get_camera(map, outCamera).value;

  /// Applies a camera jump command.
  int mapJumpTo(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_camera_options> camera,
  ) => _raw.mln_map_jump_to(map, camera).value;

  /// Applies a camera ease transition command.
  int mapEaseTo(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_camera_options> camera,
    Pointer<raw.mln_animation_options> animation,
  ) => _raw.mln_map_ease_to(map, camera, animation).value;

  /// Applies a camera fly transition command.
  int mapFlyTo(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_camera_options> camera,
    Pointer<raw.mln_animation_options> animation,
  ) => _raw.mln_map_fly_to(map, camera, animation).value;

  /// Applies a screen-space pan command.
  int mapMoveBy(Pointer<raw.mln_map> map, double deltaX, double deltaY) =>
      _raw.mln_map_move_by(map, deltaX, deltaY).value;

  /// Applies an animated screen-space pan command.
  int mapMoveByAnimated(
    Pointer<raw.mln_map> map,
    double deltaX,
    double deltaY,
    Pointer<raw.mln_animation_options> animation,
  ) => _raw.mln_map_move_by_animated(map, deltaX, deltaY, animation).value;

  /// Applies a screen-space zoom command.
  int mapScaleBy(
    Pointer<raw.mln_map> map,
    double scale,
    Pointer<raw.mln_screen_point> anchor,
  ) => _raw.mln_map_scale_by(map, scale, anchor).value;

  /// Applies an animated screen-space zoom command.
  int mapScaleByAnimated(
    Pointer<raw.mln_map> map,
    double scale,
    Pointer<raw.mln_screen_point> anchor,
    Pointer<raw.mln_animation_options> animation,
  ) => _raw.mln_map_scale_by_animated(map, scale, anchor, animation).value;

  /// Applies a screen-space rotate command.
  int mapRotateBy(
    Pointer<raw.mln_map> map,
    raw.mln_screen_point first,
    raw.mln_screen_point second,
  ) => _raw.mln_map_rotate_by(map, first, second).value;

  /// Applies an animated screen-space rotate command.
  int mapRotateByAnimated(
    Pointer<raw.mln_map> map,
    raw.mln_screen_point first,
    raw.mln_screen_point second,
    Pointer<raw.mln_animation_options> animation,
  ) => _raw.mln_map_rotate_by_animated(map, first, second, animation).value;

  /// Applies a pitch delta command.
  int mapPitchBy(Pointer<raw.mln_map> map, double pitch) =>
      _raw.mln_map_pitch_by(map, pitch).value;

  /// Applies an animated pitch delta command.
  int mapPitchByAnimated(
    Pointer<raw.mln_map> map,
    double pitch,
    Pointer<raw.mln_animation_options> animation,
  ) => _raw.mln_map_pitch_by_animated(map, pitch, animation).value;

  /// Cancels active camera transitions.
  int mapCancelTransitions(Pointer<raw.mln_map> map) =>
      _raw.mln_map_cancel_transitions(map).value;

  /// Converts a geographic coordinate to a screen point.
  int mapPixelForLatLng(
    Pointer<raw.mln_map> map,
    raw.mln_lat_lng coordinate,
    Pointer<raw.mln_screen_point> outPoint,
  ) => _raw.mln_map_pixel_for_lat_lng(map, coordinate, outPoint).value;

  /// Converts a screen point to a geographic coordinate.
  int mapLatLngForPixel(
    Pointer<raw.mln_map> map,
    raw.mln_screen_point point,
    Pointer<raw.mln_lat_lng> outCoordinate,
  ) => _raw.mln_map_lat_lng_for_pixel(map, point, outCoordinate).value;

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

  /// Borrows the root value from a JSON snapshot.
  int jsonSnapshotGet(
    Pointer<raw.mln_json_snapshot> snapshot,
    Pointer<Pointer<raw.mln_json_value>> outValue,
  ) => _raw.mln_json_snapshot_get(snapshot, outValue).value;

  /// Destroys a JSON snapshot handle.
  void jsonSnapshotDestroy(Pointer<raw.mln_json_snapshot> snapshot) {
    _raw.mln_json_snapshot_destroy(snapshot);
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

  /// Copies one style layer as a style-spec layer JSON snapshot.
  int mapGetStyleLayerJson(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    Pointer<Pointer<raw.mln_json_snapshot>> outLayer,
    Pointer<Bool> outFound,
  ) =>
      _raw.mln_map_get_style_layer_json(map, layerId, outLayer, outFound).value;

  /// Gets one style layer type string.
  int mapGetStyleLayerType(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    Pointer<raw.mln_string_view> outLayerType,
    Pointer<Bool> outFound,
  ) => _raw
      .mln_map_get_style_layer_type(map, layerId, outLayerType, outFound)
      .value;

  /// Sets the style light from a style-spec light JSON object.
  int mapSetStyleLightJson(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_json_value> lightJson,
  ) => _raw.mln_map_set_style_light_json(map, lightJson).value;

  /// Sets one style light property.
  int mapSetStyleLightProperty(
    Pointer<raw.mln_map> map,
    raw.mln_string_view propertyName,
    Pointer<raw.mln_json_value> value,
  ) => _raw.mln_map_set_style_light_property(map, propertyName, value).value;

  /// Copies one style light property as a JSON snapshot.
  int mapGetStyleLightProperty(
    Pointer<raw.mln_map> map,
    raw.mln_string_view propertyName,
    Pointer<Pointer<raw.mln_json_snapshot>> outValue,
  ) => _raw.mln_map_get_style_light_property(map, propertyName, outValue).value;

  /// Sets one layer property.
  int mapSetLayerProperty(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    raw.mln_string_view propertyName,
    Pointer<raw.mln_json_value> value,
  ) => _raw.mln_map_set_layer_property(map, layerId, propertyName, value).value;

  /// Copies one layer property as a JSON snapshot.
  int mapGetLayerProperty(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    raw.mln_string_view propertyName,
    Pointer<Pointer<raw.mln_json_snapshot>> outValue,
  ) => _raw
      .mln_map_get_layer_property(map, layerId, propertyName, outValue)
      .value;

  /// Sets or clears one layer filter.
  int mapSetLayerFilter(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    Pointer<raw.mln_json_value> filter,
  ) => _raw.mln_map_set_layer_filter(map, layerId, filter).value;

  /// Copies one layer filter as a JSON snapshot.
  int mapGetLayerFilter(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    Pointer<Pointer<raw.mln_json_snapshot>> outFilter,
  ) => _raw.mln_map_get_layer_filter(map, layerId, outFilter).value;

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

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

  /// Starts an ambient cache maintenance operation.
  int runtimeRunAmbientCacheOperationStart(
    Pointer<raw.mln_runtime> runtime,
    int operation,
    Pointer<Uint64> outOperationId,
  ) => _raw
      .mln_runtime_run_ambient_cache_operation_start(
        runtime,
        operation,
        outOperationId,
      )
      .value;

  /// Discards runtime-owned state for an offline database operation.
  int runtimeOfflineOperationDiscard(
    Pointer<raw.mln_runtime> runtime,
    int operationId,
  ) => _raw.mln_runtime_offline_operation_discard(runtime, operationId).value;

  /// Starts creating an offline region.
  int runtimeOfflineRegionCreateStart(
    Pointer<raw.mln_runtime> runtime,
    Pointer<raw.mln_offline_region_definition> definition,
    Pointer<Uint8> metadata,
    int metadataSize,
    Pointer<Uint64> outOperationId,
  ) => _raw
      .mln_runtime_offline_region_create_start(
        runtime,
        definition,
        metadata,
        metadataSize,
        outOperationId,
      )
      .value;

  /// Starts getting an offline region by ID.
  int runtimeOfflineRegionGetStart(
    Pointer<raw.mln_runtime> runtime,
    int regionId,
    Pointer<Uint64> outOperationId,
  ) => _raw
      .mln_runtime_offline_region_get_start(runtime, regionId, outOperationId)
      .value;

  /// Starts listing offline regions.
  int runtimeOfflineRegionsListStart(
    Pointer<raw.mln_runtime> runtime,
    Pointer<Uint64> outOperationId,
  ) => _raw
      .mln_runtime_offline_regions_list_start(runtime, outOperationId)
      .value;

  /// Starts merging offline regions from another database path.
  int runtimeOfflineRegionsMergeDatabaseStart(
    Pointer<raw.mln_runtime> runtime,
    Pointer<Char> sideDatabasePath,
    Pointer<Uint64> outOperationId,
  ) => _raw
      .mln_runtime_offline_regions_merge_database_start(
        runtime,
        sideDatabasePath,
        outOperationId,
      )
      .value;

  /// Starts updating offline region metadata.
  int runtimeOfflineRegionUpdateMetadataStart(
    Pointer<raw.mln_runtime> runtime,
    int regionId,
    Pointer<Uint8> metadata,
    int metadataSize,
    Pointer<Uint64> outOperationId,
  ) => _raw
      .mln_runtime_offline_region_update_metadata_start(
        runtime,
        regionId,
        metadata,
        metadataSize,
        outOperationId,
      )
      .value;

  /// Starts getting offline region status.
  int runtimeOfflineRegionGetStatusStart(
    Pointer<raw.mln_runtime> runtime,
    int regionId,
    Pointer<Uint64> outOperationId,
  ) => _raw
      .mln_runtime_offline_region_get_status_start(
        runtime,
        regionId,
        outOperationId,
      )
      .value;

  /// Starts enabling or disabling offline region observation.
  int runtimeOfflineRegionSetObservedStart(
    Pointer<raw.mln_runtime> runtime,
    int regionId,
    bool observed,
    Pointer<Uint64> outOperationId,
  ) => _raw
      .mln_runtime_offline_region_set_observed_start(
        runtime,
        regionId,
        observed,
        outOperationId,
      )
      .value;

  /// Starts setting offline region download state.
  int runtimeOfflineRegionSetDownloadStateStart(
    Pointer<raw.mln_runtime> runtime,
    int regionId,
    int state,
    Pointer<Uint64> outOperationId,
  ) => _raw
      .mln_runtime_offline_region_set_download_state_start(
        runtime,
        regionId,
        state,
        outOperationId,
      )
      .value;

  /// Starts invalidating an offline region.
  int runtimeOfflineRegionInvalidateStart(
    Pointer<raw.mln_runtime> runtime,
    int regionId,
    Pointer<Uint64> outOperationId,
  ) => _raw
      .mln_runtime_offline_region_invalidate_start(
        runtime,
        regionId,
        outOperationId,
      )
      .value;

  /// Starts deleting an offline region.
  int runtimeOfflineRegionDeleteStart(
    Pointer<raw.mln_runtime> runtime,
    int regionId,
    Pointer<Uint64> outOperationId,
  ) => _raw
      .mln_runtime_offline_region_delete_start(
        runtime,
        regionId,
        outOperationId,
      )
      .value;

  /// Takes an offline region create result.
  int runtimeOfflineRegionCreateTakeResult(
    Pointer<raw.mln_runtime> runtime,
    int operationId,
    Pointer<Pointer<raw.mln_offline_region_snapshot>> outRegion,
  ) => _raw
      .mln_runtime_offline_region_create_take_result(
        runtime,
        operationId,
        outRegion,
      )
      .value;

  /// Takes an offline region get result.
  int runtimeOfflineRegionGetTakeResult(
    Pointer<raw.mln_runtime> runtime,
    int operationId,
    Pointer<Pointer<raw.mln_offline_region_snapshot>> outRegion,
    Pointer<Bool> outFound,
  ) => _raw
      .mln_runtime_offline_region_get_take_result(
        runtime,
        operationId,
        outRegion,
        outFound,
      )
      .value;

  /// Takes an offline regions list result.
  int runtimeOfflineRegionsListTakeResult(
    Pointer<raw.mln_runtime> runtime,
    int operationId,
    Pointer<Pointer<raw.mln_offline_region_list>> outRegions,
  ) => _raw
      .mln_runtime_offline_regions_list_take_result(
        runtime,
        operationId,
        outRegions,
      )
      .value;

  /// Takes an offline database merge result.
  int runtimeOfflineRegionsMergeDatabaseTakeResult(
    Pointer<raw.mln_runtime> runtime,
    int operationId,
    Pointer<Pointer<raw.mln_offline_region_list>> outRegions,
  ) => _raw
      .mln_runtime_offline_regions_merge_database_take_result(
        runtime,
        operationId,
        outRegions,
      )
      .value;

  /// Takes an offline region metadata update result.
  int runtimeOfflineRegionUpdateMetadataTakeResult(
    Pointer<raw.mln_runtime> runtime,
    int operationId,
    Pointer<Pointer<raw.mln_offline_region_snapshot>> outRegion,
  ) => _raw
      .mln_runtime_offline_region_update_metadata_take_result(
        runtime,
        operationId,
        outRegion,
      )
      .value;

  /// Takes an offline region status result.
  int runtimeOfflineRegionGetStatusTakeResult(
    Pointer<raw.mln_runtime> runtime,
    int operationId,
    Pointer<raw.mln_offline_region_status> outStatus,
  ) => _raw
      .mln_runtime_offline_region_get_status_take_result(
        runtime,
        operationId,
        outStatus,
      )
      .value;

  /// Copies one offline region snapshot.
  int offlineRegionSnapshotGet(
    Pointer<raw.mln_offline_region_snapshot> snapshot,
    Pointer<raw.mln_offline_region_info> outInfo,
  ) => _raw.mln_offline_region_snapshot_get(snapshot, outInfo).value;

  /// Destroys an offline region snapshot.
  void offlineRegionSnapshotDestroy(
    Pointer<raw.mln_offline_region_snapshot> snapshot,
  ) {
    _raw.mln_offline_region_snapshot_destroy(snapshot);
  }

  /// Counts entries in an offline region list.
  int offlineRegionListCount(
    Pointer<raw.mln_offline_region_list> list,
    Pointer<Size> outCount,
  ) => _raw.mln_offline_region_list_count(list, outCount).value;

  /// Copies one offline region list entry.
  int offlineRegionListGet(
    Pointer<raw.mln_offline_region_list> list,
    int index,
    Pointer<raw.mln_offline_region_info> outInfo,
  ) => _raw.mln_offline_region_list_get(list, index, outInfo).value;

  /// Destroys an offline region list.
  void offlineRegionListDestroy(Pointer<raw.mln_offline_region_list> list) {
    _raw.mln_offline_region_list_destroy(list);
  }

  /// Runs one pending owner-thread task for a runtime.
  int runtimeRunOnce(Pointer<raw.mln_runtime> runtime) =>
      _raw.mln_runtime_run_once(runtime).value;

  /// Polls one queued runtime event.
  int runtimePollEvent(
    Pointer<raw.mln_runtime> runtime,
    Pointer<raw.mln_runtime_event> outEvent,
    Pointer<Bool> outHasEvent,
  ) => _raw.mln_runtime_poll_event(runtime, outEvent, outHasEvent).value;

  /// Sets a runtime-scoped network resource provider.
  int runtimeSetResourceProvider(
    Pointer<raw.mln_runtime> runtime,
    Pointer<raw.mln_resource_provider> provider,
  ) => _raw.mln_runtime_set_resource_provider(runtime, provider).value;

  /// Native Dart-shim callback for exact URL resource providers.
  Pointer<NativeFunction<raw.mln_resource_provider_callbackFunction>>
  dartResourceProviderRulesCallback() =>
      library.lookup('mln_dart_resource_provider_rules_callback');

  /// Native Dart-shim callback for queued resource providers.
  Pointer<NativeFunction<raw.mln_resource_provider_callbackFunction>>
  dartQueuedResourceProviderCallback() =>
      library.lookup('mln_dart_queued_resource_provider_callback');

  /// Native Dart-shim callback for exact URL resource transforms.
  Pointer<NativeFunction<raw.mln_resource_transform_callbackFunction>>
  dartResourceTransformRewriteCallback() =>
      library.lookup('mln_dart_resource_transform_rewrite_callback');

  /// Completes a resource provider request.
  int resourceRequestComplete(
    Pointer<raw.mln_resource_request_handle> handle,
    Pointer<raw.mln_resource_response> response,
  ) => _raw.mln_resource_request_complete(handle, response).value;

  /// Reports whether a resource provider request was cancelled.
  int resourceRequestCancelled(
    Pointer<raw.mln_resource_request_handle> handle,
    Pointer<Bool> outCancelled,
  ) => _raw.mln_resource_request_cancelled(handle, outCancelled).value;

  /// Releases the provider's reference to a handled resource request.
  void resourceRequestRelease(Pointer<raw.mln_resource_request_handle> handle) {
    _raw.mln_resource_request_release(handle);
  }

  /// Destroys a copied Dart-shim resource request record.
  void dartResourceProviderRequestDestroy(Pointer<Void> request) {
    library.lookupFunction<
      Void Function(Pointer<Void>),
      void Function(Pointer<Void>)
    >('mln_dart_resource_provider_request_destroy')(request);
  }

  /// Registers or updates a runtime-scoped URL transform.
  int runtimeSetResourceTransform(
    Pointer<raw.mln_runtime> runtime,
    Pointer<raw.mln_resource_transform> transform,
  ) => _raw.mln_runtime_set_resource_transform(runtime, transform).value;

  /// Clears the runtime-scoped URL transform.
  int runtimeClearResourceTransform(Pointer<raw.mln_runtime> runtime) =>
      _raw.mln_runtime_clear_resource_transform(runtime).value;

  /// Returns native default map options.
  raw.mln_map_options mapOptionsDefault() => _raw.mln_map_options_default();

  /// Converts geographic coordinates to projected meters.
  int projectedMetersForLatLng(
    raw.mln_lat_lng coordinate,
    Pointer<raw.mln_projected_meters> outMeters,
  ) => _raw.mln_projected_meters_for_lat_lng(coordinate, outMeters).value;

  /// Converts projected meters to geographic coordinates.
  int latLngForProjectedMeters(
    raw.mln_projected_meters meters,
    Pointer<raw.mln_lat_lng> outCoordinate,
  ) => _raw.mln_lat_lng_for_projected_meters(meters, outCoordinate).value;

  /// Creates a map projection helper.
  int mapProjectionCreate(
    Pointer<raw.mln_map> map,
    Pointer<Pointer<raw.mln_map_projection>> outProjection,
  ) => _raw.mln_map_projection_create(map, outProjection).value;

  /// Destroys a map projection helper.
  int mapProjectionDestroy(Pointer<raw.mln_map_projection> projection) =>
      _raw.mln_map_projection_destroy(projection).value;

  /// Copies a projection helper camera.
  int mapProjectionGetCamera(
    Pointer<raw.mln_map_projection> projection,
    Pointer<raw.mln_camera_options> outCamera,
  ) => _raw.mln_map_projection_get_camera(projection, outCamera).value;

  /// Sets a projection helper camera.
  int mapProjectionSetCamera(
    Pointer<raw.mln_map_projection> projection,
    Pointer<raw.mln_camera_options> camera,
  ) => _raw.mln_map_projection_set_camera(projection, camera).value;

  /// Fits visible coordinates on a projection helper.
  int mapProjectionSetVisibleCoordinates(
    Pointer<raw.mln_map_projection> projection,
    Pointer<raw.mln_lat_lng> coordinates,
    int coordinateCount,
    raw.mln_edge_insets padding,
  ) => _raw
      .mln_map_projection_set_visible_coordinates(
        projection,
        coordinates,
        coordinateCount,
        padding,
      )
      .value;

  /// Fits visible geometry on a projection helper.
  int mapProjectionSetVisibleGeometry(
    Pointer<raw.mln_map_projection> projection,
    Pointer<raw.mln_geometry> geometry,
    raw.mln_edge_insets padding,
  ) => _raw
      .mln_map_projection_set_visible_geometry(projection, geometry, padding)
      .value;

  /// Converts a geographic coordinate to a projection-helper screen point.
  int mapProjectionPixelForLatLng(
    Pointer<raw.mln_map_projection> projection,
    raw.mln_lat_lng coordinate,
    Pointer<raw.mln_screen_point> outPoint,
  ) => _raw
      .mln_map_projection_pixel_for_lat_lng(projection, coordinate, outPoint)
      .value;

  /// Converts a projection-helper screen point to a geographic coordinate.
  int mapProjectionLatLngForPixel(
    Pointer<raw.mln_map_projection> projection,
    raw.mln_screen_point point,
    Pointer<raw.mln_lat_lng> outCoordinate,
  ) => _raw
      .mln_map_projection_lat_lng_for_pixel(projection, point, outCoordinate)
      .value;

  /// Creates a map handle.
  int mapCreate(
    Pointer<raw.mln_runtime> runtime,
    Pointer<raw.mln_map_options> options,
    Pointer<Pointer<raw.mln_map>> outMap,
  ) => _raw.mln_map_create(runtime, options, outMap).value;

  /// Destroys a map handle.
  int mapDestroy(Pointer<raw.mln_map> map) => _raw.mln_map_destroy(map).value;

  /// Requests a continuous-map repaint.
  int mapRequestRepaint(Pointer<raw.mln_map> map) =>
      _raw.mln_map_request_repaint(map).value;

  /// Requests one still image for static or tile maps.
  int mapRequestStillImage(Pointer<raw.mln_map> map) =>
      _raw.mln_map_request_still_image(map).value;

  /// Sets debug overlay options.
  int mapSetDebugOptions(Pointer<raw.mln_map> map, int options) =>
      _raw.mln_map_set_debug_options(map, options).value;

  /// Copies debug overlay options.
  int mapGetDebugOptions(
    Pointer<raw.mln_map> map,
    Pointer<Uint32> outOptions,
  ) => _raw.mln_map_get_debug_options(map, outOptions).value;

  /// Enables or disables the rendering stats overlay.
  int mapSetRenderingStatsViewEnabled(Pointer<raw.mln_map> map, bool enabled) =>
      _raw.mln_map_set_rendering_stats_view_enabled(map, enabled).value;

  /// Copies whether the rendering stats overlay is enabled.
  int mapGetRenderingStatsViewEnabled(
    Pointer<raw.mln_map> map,
    Pointer<Bool> outEnabled,
  ) => _raw.mln_map_get_rendering_stats_view_enabled(map, outEnabled).value;

  /// Copies whether the map is fully loaded.
  int mapIsFullyLoaded(Pointer<raw.mln_map> map, Pointer<Bool> outLoaded) =>
      _raw.mln_map_is_fully_loaded(map, outLoaded).value;

  /// Dumps map debug logs through native logging.
  int mapDumpDebugLogs(Pointer<raw.mln_map> map) =>
      _raw.mln_map_dump_debug_logs(map).value;

  /// Returns default camera fitting options.
  raw.mln_camera_fit_options cameraFitOptionsDefault() =>
      _raw.mln_camera_fit_options_default();

  /// Returns default map bound options.
  raw.mln_bound_options boundOptionsDefault() =>
      _raw.mln_bound_options_default();

  /// Returns default free camera options.
  raw.mln_free_camera_options freeCameraOptionsDefault() =>
      _raw.mln_free_camera_options_default();

  /// Returns default projection mode options.
  raw.mln_projection_mode projectionModeDefault() =>
      _raw.mln_projection_mode_default();

  /// Returns default viewport options.
  raw.mln_map_viewport_options mapViewportOptionsDefault() =>
      _raw.mln_map_viewport_options_default();

  /// Returns default tile options.
  raw.mln_map_tile_options mapTileOptionsDefault() =>
      _raw.mln_map_tile_options_default();

  /// Sets the map style URL.
  int mapSetStyleUrl(Pointer<raw.mln_map> map, Pointer<Char> url) =>
      _raw.mln_map_set_style_url(map, url).value;

  /// Sets the map style JSON.
  int mapSetStyleJson(Pointer<raw.mln_map> map, Pointer<Char> json) =>
      _raw.mln_map_set_style_json(map, json).value;

  /// Returns native default Metal surface descriptor.
  raw.mln_metal_surface_descriptor metalSurfaceDescriptorDefault() =>
      _raw.mln_metal_surface_descriptor_default();

  /// Returns native default Vulkan surface descriptor.
  raw.mln_vulkan_surface_descriptor vulkanSurfaceDescriptorDefault() =>
      _raw.mln_vulkan_surface_descriptor_default();

  /// Attaches a Metal native surface render target.
  int metalSurfaceAttach(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_metal_surface_descriptor> descriptor,
    Pointer<Pointer<raw.mln_render_session>> outSession,
  ) => _raw.mln_metal_surface_attach(map, descriptor, outSession).value;

  /// Attaches a Vulkan native surface render target.
  int vulkanSurfaceAttach(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_vulkan_surface_descriptor> descriptor,
    Pointer<Pointer<raw.mln_render_session>> outSession,
  ) => _raw.mln_vulkan_surface_attach(map, descriptor, outSession).value;

  /// Attaches a Metal texture render target owned by the session.
  int metalOwnedTextureAttach(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_metal_owned_texture_descriptor> descriptor,
    Pointer<Pointer<raw.mln_render_session>> outSession,
  ) => _raw.mln_metal_owned_texture_attach(map, descriptor, outSession).value;

  /// Attaches a Metal caller-owned texture render target.
  int metalBorrowedTextureAttach(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_metal_borrowed_texture_descriptor> descriptor,
    Pointer<Pointer<raw.mln_render_session>> outSession,
  ) =>
      _raw.mln_metal_borrowed_texture_attach(map, descriptor, outSession).value;

  /// Attaches a Vulkan texture render target owned by the session.
  int vulkanOwnedTextureAttach(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_vulkan_owned_texture_descriptor> descriptor,
    Pointer<Pointer<raw.mln_render_session>> outSession,
  ) => _raw.mln_vulkan_owned_texture_attach(map, descriptor, outSession).value;

  /// Attaches a Vulkan caller-owned texture render target.
  int vulkanBorrowedTextureAttach(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_vulkan_borrowed_texture_descriptor> descriptor,
    Pointer<Pointer<raw.mln_render_session>> outSession,
  ) => _raw
      .mln_vulkan_borrowed_texture_attach(map, descriptor, outSession)
      .value;

  /// Resizes an attached render session.
  int renderSessionResize(
    Pointer<raw.mln_render_session> session,
    int width,
    int height,
    double scaleFactor,
  ) =>
      _raw.mln_render_session_resize(session, width, height, scaleFactor).value;

  /// Processes the latest map render update for a render session.
  int renderSessionRenderUpdate(Pointer<raw.mln_render_session> session) =>
      _raw.mln_render_session_render_update(session).value;

  /// Detaches backend-bound render resources.
  int renderSessionDetach(Pointer<raw.mln_render_session> session) =>
      _raw.mln_render_session_detach(session).value;

  /// Destroys a render session handle.
  int renderSessionDestroy(Pointer<raw.mln_render_session> session) =>
      _raw.mln_render_session_destroy(session).value;

  /// Asks the session renderer to release cached resources.
  int renderSessionReduceMemoryUse(Pointer<raw.mln_render_session> session) =>
      _raw.mln_render_session_reduce_memory_use(session).value;

  /// Clears renderer data for the session.
  int renderSessionClearData(Pointer<raw.mln_render_session> session) =>
      _raw.mln_render_session_clear_data(session).value;

  /// Dumps renderer debug logs for the session.
  int renderSessionDumpDebugLogs(Pointer<raw.mln_render_session> session) =>
      _raw.mln_render_session_dump_debug_logs(session).value;

  /// Sets per-feature state.
  int renderSessionSetFeatureState(
    Pointer<raw.mln_render_session> session,
    Pointer<raw.mln_feature_state_selector> selector,
    Pointer<raw.mln_json_value> state,
  ) =>
      _raw.mln_render_session_set_feature_state(session, selector, state).value;

  /// Copies per-feature state.
  int renderSessionGetFeatureState(
    Pointer<raw.mln_render_session> session,
    Pointer<raw.mln_feature_state_selector> selector,
    Pointer<Pointer<raw.mln_json_snapshot>> outState,
  ) => _raw
      .mln_render_session_get_feature_state(session, selector, outState)
      .value;

  /// Removes per-feature state.
  int renderSessionRemoveFeatureState(
    Pointer<raw.mln_render_session> session,
    Pointer<raw.mln_feature_state_selector> selector,
  ) => _raw.mln_render_session_remove_feature_state(session, selector).value;

  /// Queries rendered features from the latest render session state.
  int renderSessionQueryRenderedFeatures(
    Pointer<raw.mln_render_session> session,
    Pointer<raw.mln_rendered_query_geometry> geometry,
    Pointer<raw.mln_rendered_feature_query_options> options,
    Pointer<Pointer<raw.mln_feature_query_result>> outResult,
  ) => _raw
      .mln_render_session_query_rendered_features(
        session,
        geometry,
        options,
        outResult,
      )
      .value;

  /// Queries source features from the latest render session state.
  int renderSessionQuerySourceFeatures(
    Pointer<raw.mln_render_session> session,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_source_feature_query_options> options,
    Pointer<Pointer<raw.mln_feature_query_result>> outResult,
  ) => _raw
      .mln_render_session_query_source_features(
        session,
        sourceId,
        options,
        outResult,
      )
      .value;

  /// Queries a feature extension from the latest render session state.
  int renderSessionQueryFeatureExtensions(
    Pointer<raw.mln_render_session> session,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_feature> feature,
    raw.mln_string_view extension,
    raw.mln_string_view extensionField,
    Pointer<raw.mln_json_value> arguments,
    Pointer<Pointer<raw.mln_feature_extension_result>> outResult,
  ) => _raw
      .mln_render_session_query_feature_extensions(
        session,
        sourceId,
        feature,
        extension,
        extensionField,
        arguments,
        outResult,
      )
      .value;

  /// Counts features in a query result handle.
  int featureQueryResultCount(
    Pointer<raw.mln_feature_query_result> result,
    Pointer<Size> outCount,
  ) => _raw.mln_feature_query_result_count(result, outCount).value;

  /// Borrows one feature from a query result handle.
  int featureQueryResultGet(
    Pointer<raw.mln_feature_query_result> result,
    int index,
    Pointer<raw.mln_queried_feature> outFeature,
  ) => _raw.mln_feature_query_result_get(result, index, outFeature).value;

  /// Destroys a feature query result handle.
  void featureQueryResultDestroy(Pointer<raw.mln_feature_query_result> result) {
    _raw.mln_feature_query_result_destroy(result);
  }

  /// Borrows a feature extension query result view.
  int featureExtensionResultGet(
    Pointer<raw.mln_feature_extension_result> result,
    Pointer<raw.mln_feature_extension_result_info> outInfo,
  ) => _raw.mln_feature_extension_result_get(result, outInfo).value;

  /// Destroys a feature extension result handle.
  void featureExtensionResultDestroy(
    Pointer<raw.mln_feature_extension_result> result,
  ) {
    _raw.mln_feature_extension_result_destroy(result);
  }

  /// Reads the most recently rendered texture frame into caller-owned storage.
  int textureReadPremultipliedRgba8(
    Pointer<raw.mln_render_session> session,
    Pointer<Uint8> outData,
    int outDataCapacity,
    Pointer<raw.mln_texture_image_info> outInfo,
  ) => _raw
      .mln_texture_read_premultiplied_rgba8(
        session,
        outData,
        outDataCapacity,
        outInfo,
      )
      .value;

  /// Acquires the most recently rendered Metal texture frame.
  int metalOwnedTextureAcquireFrame(
    Pointer<raw.mln_render_session> session,
    Pointer<raw.mln_metal_owned_texture_frame> outFrame,
  ) => _raw.mln_metal_owned_texture_acquire_frame(session, outFrame).value;

  /// Releases a Metal texture frame.
  int metalOwnedTextureReleaseFrame(
    Pointer<raw.mln_render_session> session,
    Pointer<raw.mln_metal_owned_texture_frame> frame,
  ) => _raw.mln_metal_owned_texture_release_frame(session, frame).value;

  /// Acquires the most recently rendered Vulkan texture frame.
  int vulkanOwnedTextureAcquireFrame(
    Pointer<raw.mln_render_session> session,
    Pointer<raw.mln_vulkan_owned_texture_frame> outFrame,
  ) => _raw.mln_vulkan_owned_texture_acquire_frame(session, outFrame).value;

  /// Releases a Vulkan texture frame.
  int vulkanOwnedTextureReleaseFrame(
    Pointer<raw.mln_render_session> session,
    Pointer<raw.mln_vulkan_owned_texture_frame> frame,
  ) => _raw.mln_vulkan_owned_texture_release_frame(session, frame).value;

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

  /// Copies live map viewport options.
  int mapGetViewportOptions(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_map_viewport_options> outOptions,
  ) => _raw.mln_map_get_viewport_options(map, outOptions).value;

  /// Applies live map viewport options.
  int mapSetViewportOptions(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_map_viewport_options> options,
  ) => _raw.mln_map_set_viewport_options(map, options).value;

  /// Copies map tile options.
  int mapGetTileOptions(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_map_tile_options> outOptions,
  ) => _raw.mln_map_get_tile_options(map, outOptions).value;

  /// Applies map tile options.
  int mapSetTileOptions(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_map_tile_options> options,
  ) => _raw.mln_map_set_tile_options(map, options).value;

  /// Copies map camera bounds constraints.
  int mapGetBounds(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_bound_options> outOptions,
  ) => _raw.mln_map_get_bounds(map, outOptions).value;

  /// Applies map camera bounds constraints.
  int mapSetBounds(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_bound_options> options,
  ) => _raw.mln_map_set_bounds(map, options).value;

  /// Copies free camera options.
  int mapGetFreeCameraOptions(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_free_camera_options> outOptions,
  ) => _raw.mln_map_get_free_camera_options(map, outOptions).value;

  /// Applies free camera options.
  int mapSetFreeCameraOptions(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_free_camera_options> options,
  ) => _raw.mln_map_set_free_camera_options(map, options).value;

  /// Copies projection mode options.
  int mapGetProjectionMode(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_projection_mode> outMode,
  ) => _raw.mln_map_get_projection_mode(map, outMode).value;

  /// Applies projection mode options.
  int mapSetProjectionMode(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_projection_mode> mode,
  ) => _raw.mln_map_set_projection_mode(map, mode).value;

  /// Computes camera options fitting geographic bounds.
  int mapCameraForLatLngBounds(
    Pointer<raw.mln_map> map,
    raw.mln_lat_lng_bounds bounds,
    Pointer<raw.mln_camera_fit_options> fitOptions,
    Pointer<raw.mln_camera_options> outCamera,
  ) => _raw
      .mln_map_camera_for_lat_lng_bounds(map, bounds, fitOptions, outCamera)
      .value;

  /// Computes camera options fitting geographic coordinates.
  int mapCameraForLatLngs(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_lat_lng> coordinates,
    int coordinateCount,
    Pointer<raw.mln_camera_fit_options> fitOptions,
    Pointer<raw.mln_camera_options> outCamera,
  ) => _raw
      .mln_map_camera_for_lat_lngs(
        map,
        coordinates,
        coordinateCount,
        fitOptions,
        outCamera,
      )
      .value;

  /// Computes camera options fitting geometry.
  int mapCameraForGeometry(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_geometry> geometry,
    Pointer<raw.mln_camera_fit_options> fitOptions,
    Pointer<raw.mln_camera_options> outCamera,
  ) => _raw
      .mln_map_camera_for_geometry(map, geometry, fitOptions, outCamera)
      .value;

  /// Computes wrapped geographic bounds for camera options.
  int mapLatLngBoundsForCamera(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_camera_options> camera,
    Pointer<raw.mln_lat_lng_bounds> outBounds,
  ) => _raw.mln_map_lat_lng_bounds_for_camera(map, camera, outBounds).value;

  /// Computes unwrapped geographic bounds for camera options.
  int mapLatLngBoundsForCameraUnwrapped(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_camera_options> camera,
    Pointer<raw.mln_lat_lng_bounds> outBounds,
  ) => _raw
      .mln_map_lat_lng_bounds_for_camera_unwrapped(map, camera, outBounds)
      .value;

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

  /// Converts geographic coordinates to screen points.
  int mapPixelsForLatLngs(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_lat_lng> coordinates,
    int coordinateCount,
    Pointer<raw.mln_screen_point> outPoints,
  ) => _raw
      .mln_map_pixels_for_lat_lngs(map, coordinates, coordinateCount, outPoints)
      .value;

  /// Converts screen points to geographic coordinates.
  int mapLatLngsForPixels(
    Pointer<raw.mln_map> map,
    Pointer<raw.mln_screen_point> points,
    int pointCount,
    Pointer<raw.mln_lat_lng> outCoordinates,
  ) => _raw
      .mln_map_lat_lngs_for_pixels(map, points, pointCount, outCoordinates)
      .value;

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

  /// Returns default rendered feature query options.
  raw.mln_rendered_feature_query_options renderedFeatureQueryOptionsDefault() =>
      _raw.mln_rendered_feature_query_options_default();

  /// Returns default source feature query options.
  raw.mln_source_feature_query_options sourceFeatureQueryOptionsDefault() =>
      _raw.mln_source_feature_query_options_default();

  /// Returns a rendered point query geometry descriptor.
  raw.mln_rendered_query_geometry renderedQueryGeometryPoint(
    raw.mln_screen_point point,
  ) => _raw.mln_rendered_query_geometry_point(point);

  /// Returns a rendered box query geometry descriptor.
  raw.mln_rendered_query_geometry renderedQueryGeometryBox(
    raw.mln_screen_box box,
  ) => _raw.mln_rendered_query_geometry_box(box);

  /// Returns a rendered line-string query geometry descriptor.
  raw.mln_rendered_query_geometry renderedQueryGeometryLineString(
    Pointer<raw.mln_screen_point> points,
    int pointCount,
  ) => _raw.mln_rendered_query_geometry_line_string(points, pointCount);

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

  /// Adds a GeoJSON source with URL data.
  int mapAddGeoJsonSourceUrl(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    raw.mln_string_view url,
  ) => _raw.mln_map_add_geojson_source_url(map, sourceId, url).value;

  /// Adds a GeoJSON source with inline data.
  int mapAddGeoJsonSourceData(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_geojson> data,
  ) => _raw.mln_map_add_geojson_source_data(map, sourceId, data).value;

  /// Updates a GeoJSON source to load data from a URL.
  int mapSetGeoJsonSourceUrl(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    raw.mln_string_view url,
  ) => _raw.mln_map_set_geojson_source_url(map, sourceId, url).value;

  /// Updates a GeoJSON source with inline data.
  int mapSetGeoJsonSourceData(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_geojson> data,
  ) => _raw.mln_map_set_geojson_source_data(map, sourceId, data).value;

  /// Adds a vector source with a TileJSON URL.
  int mapAddVectorSourceUrl(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    raw.mln_string_view url,
    Pointer<raw.mln_style_tile_source_options> options,
  ) => _raw.mln_map_add_vector_source_url(map, sourceId, url, options).value;

  /// Adds a vector source with inline tile URLs.
  int mapAddVectorSourceTiles(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_string_view> tiles,
    int tileCount,
    Pointer<raw.mln_style_tile_source_options> options,
  ) => _raw
      .mln_map_add_vector_source_tiles(map, sourceId, tiles, tileCount, options)
      .value;

  /// Adds a raster source with a TileJSON URL.
  int mapAddRasterSourceUrl(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    raw.mln_string_view url,
    Pointer<raw.mln_style_tile_source_options> options,
  ) => _raw.mln_map_add_raster_source_url(map, sourceId, url, options).value;

  /// Adds a raster source with inline tile URLs.
  int mapAddRasterSourceTiles(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_string_view> tiles,
    int tileCount,
    Pointer<raw.mln_style_tile_source_options> options,
  ) => _raw
      .mln_map_add_raster_source_tiles(map, sourceId, tiles, tileCount, options)
      .value;

  /// Adds a raster DEM source with a TileJSON URL.
  int mapAddRasterDemSourceUrl(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    raw.mln_string_view url,
    Pointer<raw.mln_style_tile_source_options> options,
  ) =>
      _raw.mln_map_add_raster_dem_source_url(map, sourceId, url, options).value;

  /// Adds a raster DEM source with inline tile URLs.
  int mapAddRasterDemSourceTiles(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_string_view> tiles,
    int tileCount,
    Pointer<raw.mln_style_tile_source_options> options,
  ) => _raw
      .mln_map_add_raster_dem_source_tiles(
        map,
        sourceId,
        tiles,
        tileCount,
        options,
      )
      .value;

  /// Adds an image source that loads from a URL.
  int mapAddImageSourceUrl(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_lat_lng> coordinates,
    int coordinateCount,
    raw.mln_string_view url,
  ) => _raw
      .mln_map_add_image_source_url(
        map,
        sourceId,
        coordinates,
        coordinateCount,
        url,
      )
      .value;

  /// Adds an image source with inline image pixels.
  int mapAddImageSourceImage(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_lat_lng> coordinates,
    int coordinateCount,
    Pointer<raw.mln_premultiplied_rgba8_image> image,
  ) => _raw
      .mln_map_add_image_source_image(
        map,
        sourceId,
        coordinates,
        coordinateCount,
        image,
      )
      .value;

  /// Updates an image source to load from a URL.
  int mapSetImageSourceUrl(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    raw.mln_string_view url,
  ) => _raw.mln_map_set_image_source_url(map, sourceId, url).value;

  /// Updates an image source with inline image pixels.
  int mapSetImageSourceImage(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_premultiplied_rgba8_image> image,
  ) => _raw.mln_map_set_image_source_image(map, sourceId, image).value;

  /// Updates image source coordinates.
  int mapSetImageSourceCoordinates(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_lat_lng> coordinates,
    int coordinateCount,
  ) => _raw
      .mln_map_set_image_source_coordinates(
        map,
        sourceId,
        coordinates,
        coordinateCount,
      )
      .value;

  /// Copies image source coordinates.
  int mapGetImageSourceCoordinates(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_lat_lng> outCoordinates,
    int coordinateCapacity,
    Pointer<Size> outCoordinateCount,
    Pointer<Bool> outFound,
  ) => _raw
      .mln_map_get_image_source_coordinates(
        map,
        sourceId,
        outCoordinates,
        coordinateCapacity,
        outCoordinateCount,
        outFound,
      )
      .value;

  /// Returns default tile source options.
  raw.mln_style_tile_source_options styleTileSourceOptionsDefault() =>
      _raw.mln_style_tile_source_options_default();

  /// Returns default custom geometry source options.
  raw.mln_custom_geometry_source_options customGeometrySourceOptionsDefault() =>
      _raw.mln_custom_geometry_source_options_default();

  /// Returns default premultiplied RGBA8 image descriptor.
  raw.mln_premultiplied_rgba8_image premultipliedRgba8ImageDefault() =>
      _raw.mln_premultiplied_rgba8_image_default();

  /// Returns default style image options.
  raw.mln_style_image_options styleImageOptionsDefault() =>
      _raw.mln_style_image_options_default();

  /// Sets or replaces one runtime style image.
  int mapSetStyleImage(
    Pointer<raw.mln_map> map,
    raw.mln_string_view imageId,
    Pointer<raw.mln_premultiplied_rgba8_image> image,
    Pointer<raw.mln_style_image_options> options,
  ) => _raw.mln_map_set_style_image(map, imageId, image, options).value;

  /// Removes one runtime style image.
  int mapRemoveStyleImage(
    Pointer<raw.mln_map> map,
    raw.mln_string_view imageId,
    Pointer<Bool> outRemoved,
  ) => _raw.mln_map_remove_style_image(map, imageId, outRemoved).value;

  /// Reports whether one runtime style image exists.
  int mapStyleImageExists(
    Pointer<raw.mln_map> map,
    raw.mln_string_view imageId,
    Pointer<Bool> outExists,
  ) => _raw.mln_map_style_image_exists(map, imageId, outExists).value;

  /// Copies style image metadata.
  int mapGetStyleImageInfo(
    Pointer<raw.mln_map> map,
    raw.mln_string_view imageId,
    Pointer<raw.mln_style_image_info> outInfo,
    Pointer<Bool> outFound,
  ) => _raw.mln_map_get_style_image_info(map, imageId, outInfo, outFound).value;

  /// Copies style image pixels as premultiplied RGBA8.
  int mapCopyStyleImagePremultipliedRgba8(
    Pointer<raw.mln_map> map,
    raw.mln_string_view imageId,
    Pointer<Uint8> outPixels,
    int pixelCapacity,
    Pointer<Size> outByteLength,
    Pointer<Bool> outFound,
  ) => _raw
      .mln_map_copy_style_image_premultiplied_rgba8(
        map,
        imageId,
        outPixels,
        pixelCapacity,
        outByteLength,
        outFound,
      )
      .value;

  /// Adds a custom geometry source.
  int mapAddCustomGeometrySource(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    Pointer<raw.mln_custom_geometry_source_options> options,
  ) => _raw.mln_map_add_custom_geometry_source(map, sourceId, options).value;

  /// Sets custom geometry source data for one canonical tile.
  int mapSetCustomGeometrySourceTileData(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    raw.mln_canonical_tile_id tileId,
    Pointer<raw.mln_geojson> data,
  ) => _raw
      .mln_map_set_custom_geometry_source_tile_data(map, sourceId, tileId, data)
      .value;

  /// Invalidates custom geometry source data for one canonical tile.
  int mapInvalidateCustomGeometrySourceTile(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    raw.mln_canonical_tile_id tileId,
  ) => _raw
      .mln_map_invalidate_custom_geometry_source_tile(map, sourceId, tileId)
      .value;

  /// Invalidates custom geometry source data inside one geographic region.
  int mapInvalidateCustomGeometrySourceRegion(
    Pointer<raw.mln_map> map,
    raw.mln_string_view sourceId,
    raw.mln_lat_lng_bounds bounds,
  ) => _raw
      .mln_map_invalidate_custom_geometry_source_region(map, sourceId, bounds)
      .value;

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

  /// Adds a hillshade layer.
  int mapAddHillshadeLayer(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    raw.mln_string_view sourceId,
    raw.mln_string_view beforeLayerId,
  ) => _raw
      .mln_map_add_hillshade_layer(map, layerId, sourceId, beforeLayerId)
      .value;

  /// Adds a color-relief layer.
  int mapAddColorReliefLayer(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    raw.mln_string_view sourceId,
    raw.mln_string_view beforeLayerId,
  ) => _raw
      .mln_map_add_color_relief_layer(map, layerId, sourceId, beforeLayerId)
      .value;

  /// Adds a location indicator layer.
  int mapAddLocationIndicatorLayer(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    raw.mln_string_view beforeLayerId,
  ) => _raw
      .mln_map_add_location_indicator_layer(map, layerId, beforeLayerId)
      .value;

  /// Sets a location indicator layer location.
  int mapSetLocationIndicatorLocation(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    raw.mln_lat_lng coordinate,
    double altitude,
  ) => _raw
      .mln_map_set_location_indicator_location(
        map,
        layerId,
        coordinate,
        altitude,
      )
      .value;

  /// Sets a location indicator layer bearing.
  int mapSetLocationIndicatorBearing(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    double bearing,
  ) => _raw.mln_map_set_location_indicator_bearing(map, layerId, bearing).value;

  /// Sets a location indicator layer accuracy radius.
  int mapSetLocationIndicatorAccuracyRadius(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    double radius,
  ) => _raw
      .mln_map_set_location_indicator_accuracy_radius(map, layerId, radius)
      .value;

  /// Sets one location indicator image-name property.
  int mapSetLocationIndicatorImageName(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    int imageKind,
    raw.mln_string_view imageId,
  ) => _raw
      .mln_map_set_location_indicator_image_name(
        map,
        layerId,
        imageKind,
        imageId,
      )
      .value;

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

  /// Moves one style layer before another layer or to the top.
  int mapMoveStyleLayer(
    Pointer<raw.mln_map> map,
    raw.mln_string_view layerId,
    raw.mln_string_view beforeLayerId,
  ) => _raw.mln_map_move_style_layer(map, layerId, beforeLayerId).value;

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

import 'dart:ffi';
import 'dart:isolate';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';

import '../camera/camera.dart';
import '../geo/geo.dart';
import '../internal/callback/callback_state.dart';
import '../internal/c/maplibre_native_c.dart';
import '../internal/c/maplibre_native_c.g.dart' as raw;
import '../internal/lifecycle/lifecycle.dart';
import '../internal/memory/memory.dart';
import '../internal/status/status.dart';
import '../internal/struct/geometry.dart' as native_geometry;
import '../internal/struct/json.dart' as native_json;
import '../internal/struct/struct.dart' as native_struct;
import '../json/json.dart';
import '../offline/offline.dart';
import '../query/query.dart';
import '../render/native_pointer.dart';
import '../render/targets.dart';
import '../resource/resource.dart';
import '../style/style.dart';

part 'runtime_resource_callbacks.dart';
part 'runtime_offline.dart';
part 'runtime_render_handles.dart';
part 'runtime_native_conversions.dart';

final MaplibreNativeCApi _c = MaplibreNativeCApi.open();

const int _resourceKindWildcard = 0xffffffff;

/// Dart resource provider callback run on the owner isolate.
typedef ResourceProviderCallback =
    void Function(ResourceRequest request, ResourceRequestHandle handle);

/// Owner-isolate resource provider definition.
final class ResourceProvider {
  /// Creates a resource provider with native-owned routing rules.
  const ResourceProvider({required this.routes, required this.callback});

  /// Exact routes handled by this provider.
  final List<ResourceProviderRoute> routes;

  /// Callback invoked on the owner isolate for matching requests.
  final ResourceProviderCallback callback;
}

/// Runtime creation options.
final class RuntimeOptions {
  /// Creates runtime options.
  const RuntimeOptions({this.assetPath, this.cachePath, this.maximumCacheSize});

  /// Filesystem root for `asset://` URLs.
  final String? assetPath;

  /// Cache database path.
  final String? cachePath;

  /// Maximum ambient cache size in bytes.
  final int? maximumCacheSize;
}

/// Owner-thread runtime handle for MapLibre Native work and event polling.
final class RuntimeHandle {
  RuntimeHandle._(Pointer<raw.mln_runtime> pointer)
    : _state = NativeHandleState(pointer, 'RuntimeHandle');

  final NativeHandleState<raw.mln_runtime> _state;
  final _maps = <int, WeakReference<MapHandle>>{};
  _ResourceTransformState? _resourceTransformState;
  _ResourceProviderRulesState? _resourceProviderRulesState;
  _ResourceProviderCallbackState? _resourceProviderCallbackState;

  /// Creates a runtime on the current native thread.
  factory RuntimeHandle.create({
    RuntimeOptions options = const RuntimeOptions(),
  }) {
    return withNativeArena((arena) {
      final nativeOptions = arena<raw.mln_runtime_options>();
      nativeOptions.ref = _runtimeOptionsToNative(options, arena);
      final outRuntime = arena<Pointer<raw.mln_runtime>>();
      outRuntime.value = nullptr;
      _check(_c.raw.mln_runtime_create(nativeOptions, outRuntime));
      return RuntimeHandle._(outRuntime.value);
    });
  }

  Pointer<raw.mln_runtime> get _pointer => _state.pointer;

  /// Whether this runtime has been closed by the Dart binding.
  bool get isClosed => _state.isClosed;

  /// Runs one pending owner-thread task for this runtime.
  void runOnce() {
    _check(_c.raw.mln_runtime_run_once(_pointer));
  }

  /// Polls one queued runtime event and copies borrowed fields into Dart values.
  RuntimeEvent? pollEvent() {
    return withNativeArena((arena) {
      final event = arena<raw.mln_runtime_event>();
      event.ref.size = sizeOf<raw.mln_runtime_event>();
      final hasEvent = arena<Bool>();
      hasEvent.value = false;

      _check(_c.raw.mln_runtime_poll_event(_pointer, event, hasEvent));
      if (!hasEvent.value) {
        return null;
      }

      final copiedEvent = RuntimeEvent._fromNative(event.ref);
      _handleRuntimeEvent(copiedEvent);
      return copiedEvent;
    });
  }

  /// Polls and discards queued runtime events until the queue is empty.
  int drainEvents() {
    var count = 0;
    while (pollEvent() != null) {
      count += 1;
    }
    return count;
  }

  /// Registers exact native-owned URL rewrite rules for network resources.
  void setResourceUrlRewriteRules(List<ResourceUrlRewriteRule> rules) {
    final state = _ResourceTransformState(rules);
    try {
      withNativeArena((arena) {
        final transform = arena<raw.mln_resource_transform>();
        transform.ref.size = sizeOf<raw.mln_resource_transform>();
        transform.ref.callback = _c.dartResourceTransformRewriteCallback();
        transform.ref.user_data = state.pointer.cast<Void>();
        _check(_c.raw.mln_runtime_set_resource_transform(_pointer, transform));
      });
      _resourceTransformState?.close();
      _resourceTransformState = state;
    } catch (_) {
      state.close();
      rethrow;
    }
  }

  /// Clears runtime-scoped URL rewrite rules.
  void clearResourceTransform() {
    _check(_c.raw.mln_runtime_clear_resource_transform(_pointer));
    _resourceTransformState?.close();
    _resourceTransformState = null;
  }

  /// Registers exact native-owned response rules for network resources.
  ///
  /// The provider must be set before this runtime creates maps.
  void setResourceProviderRules(List<ResourceProviderRule> rules) {
    final state = _ResourceProviderRulesState(rules);
    try {
      withNativeArena((arena) {
        final provider = arena<raw.mln_resource_provider>();
        provider.ref.size = sizeOf<raw.mln_resource_provider>();
        provider.ref.callback = _c.dartResourceProviderRulesCallback();
        provider.ref.user_data = state.pointer.cast<Void>();
        _check(_c.raw.mln_runtime_set_resource_provider(_pointer, provider));
      });
      _resourceProviderRulesState?.close();
      _resourceProviderRulesState = state;
      _resourceProviderCallbackState?.close();
      _resourceProviderCallbackState = null;
    } catch (_) {
      state.close();
      rethrow;
    }
  }

  /// Registers a queued Dart resource provider callback.
  ///
  /// The provider must be set before this runtime creates maps.
  void setResourceProvider(ResourceProvider provider) {
    final state = _ResourceProviderCallbackState(provider);
    try {
      withNativeArena((arena) {
        final nativeProvider = arena<raw.mln_resource_provider>();
        nativeProvider.ref.size = sizeOf<raw.mln_resource_provider>();
        nativeProvider.ref.callback = _c.dartQueuedResourceProviderCallback();
        nativeProvider.ref.user_data = state.pointer.cast<Void>();
        _check(
          _c.raw.mln_runtime_set_resource_provider(_pointer, nativeProvider),
        );
      });
      _resourceProviderCallbackState?.close();
      _resourceProviderCallbackState = state;
      _resourceProviderRulesState?.close();
      _resourceProviderRulesState = null;
    } catch (_) {
      state.close();
      rethrow;
    }
  }

  /// Starts an ambient cache maintenance operation.
  OfflineOperationHandle runAmbientCacheOperation(
    AmbientCacheOperation operation,
  ) {
    return withNativeArena((arena) {
      final outOperationId = arena<Uint64>();
      _check(
        _c.raw.mln_runtime_run_ambient_cache_operation_start(
          _pointer,
          operation.rawValue,
          outOperationId,
        ),
      );
      return OfflineOperationHandle._(this, outOperationId.value);
    });
  }

  /// Starts creating an offline region.
  OfflineOperationHandle createOfflineRegion(
    OfflineRegionDefinition definition, {
    Uint8List? metadata,
  }) {
    return withNativeArena((arena) {
      final nativeDefinition = arena<raw.mln_offline_region_definition>();
      nativeDefinition.ref = _offlineRegionDefinitionToNative(
        definition,
        arena,
      );
      final nativeMetadata = _nativeBytes(metadata, arena);
      final outOperationId = arena<Uint64>();
      _check(
        _c.raw.mln_runtime_offline_region_create_start(
          _pointer,
          nativeDefinition,
          nativeMetadata,
          metadata?.length ?? 0,
          outOperationId,
        ),
      );
      return OfflineOperationHandle._(this, outOperationId.value);
    });
  }

  /// Starts getting an offline region snapshot by ID.
  OfflineOperationHandle getOfflineRegion(int regionId) =>
      _startOfflineOperation((outOperationId) {
        _check(
          _c.raw.mln_runtime_offline_region_get_start(
            _pointer,
            regionId,
            outOperationId,
          ),
        );
      });

  /// Starts listing offline region snapshots.
  OfflineOperationHandle listOfflineRegions() => _startOfflineOperation((
    outOperationId,
  ) {
    _check(
      _c.raw.mln_runtime_offline_regions_list_start(_pointer, outOperationId),
    );
  });

  /// Starts merging offline regions from another database path.
  OfflineOperationHandle mergeOfflineRegionDatabase(String sideDatabasePath) {
    return withNativeArena((arena) {
      final nativePath = nativeUtf8CString(sideDatabasePath, arena);
      final outOperationId = arena<Uint64>();
      _check(
        _c.raw.mln_runtime_offline_regions_merge_database_start(
          _pointer,
          nativePath.pointer.cast<Char>(),
          outOperationId,
        ),
      );
      return OfflineOperationHandle._(this, outOperationId.value);
    });
  }

  /// Starts updating opaque offline region metadata.
  OfflineOperationHandle updateOfflineRegionMetadata(
    int regionId,
    Uint8List metadata,
  ) {
    return withNativeArena((arena) {
      final nativeMetadata = _nativeBytes(metadata, arena);
      final outOperationId = arena<Uint64>();
      _check(
        _c.raw.mln_runtime_offline_region_update_metadata_start(
          _pointer,
          regionId,
          nativeMetadata,
          metadata.length,
          outOperationId,
        ),
      );
      return OfflineOperationHandle._(this, outOperationId.value);
    });
  }

  /// Starts getting the current offline region status.
  OfflineOperationHandle getOfflineRegionStatus(int regionId) =>
      _startOfflineOperation((outOperationId) {
        _check(
          _c.raw.mln_runtime_offline_region_get_status_start(
            _pointer,
            regionId,
            outOperationId,
          ),
        );
      });

  /// Starts enabling or disabling offline region observation.
  OfflineOperationHandle setOfflineRegionObserved(
    int regionId,
    bool observed,
  ) => _startOfflineOperation((outOperationId) {
    _check(
      _c.raw.mln_runtime_offline_region_set_observed_start(
        _pointer,
        regionId,
        observed,
        outOperationId,
      ),
    );
  });

  /// Starts changing an offline region's download state.
  OfflineOperationHandle setOfflineRegionDownloadState(
    int regionId,
    OfflineRegionDownloadState state,
  ) => _startOfflineOperation((outOperationId) {
    _check(
      _c.raw.mln_runtime_offline_region_set_download_state_start(
        _pointer,
        regionId,
        state.rawValue,
        outOperationId,
      ),
    );
  });

  /// Starts invalidating cached resources for an offline region.
  OfflineOperationHandle invalidateOfflineRegion(int regionId) =>
      _startOfflineOperation((outOperationId) {
        _check(
          _c.raw.mln_runtime_offline_region_invalidate_start(
            _pointer,
            regionId,
            outOperationId,
          ),
        );
      });

  /// Starts deleting an offline region.
  OfflineOperationHandle deleteOfflineRegion(int regionId) =>
      _startOfflineOperation((outOperationId) {
        _check(
          _c.raw.mln_runtime_offline_region_delete_start(
            _pointer,
            regionId,
            outOperationId,
          ),
        );
      });

  OfflineOperationHandle _startOfflineOperation(
    void Function(Pointer<Uint64> outOperationId) start,
  ) {
    return withNativeArena((arena) {
      final outOperationId = arena<Uint64>();
      start(outOperationId);
      return OfflineOperationHandle._(this, outOperationId.value);
    });
  }

  /// Creates a map owned by this runtime.
  MapHandle createMap({MapOptions options = const MapOptions()}) =>
      MapHandle.create(this, options: options);

  void _registerMap(MapHandle map) {
    _maps[map._pointer.address] = WeakReference(map);
  }

  void _unregisterMapAddress(int? address) {
    if (address != null) {
      _maps.remove(address);
    }
  }

  void _handleRuntimeEvent(RuntimeEvent event) {
    if (event.sourceType !=
        raw.mln_runtime_event_source_type.MLN_RUNTIME_EVENT_SOURCE_MAP.value) {
      return;
    }
    if (event.type !=
        raw.mln_runtime_event_type.MLN_RUNTIME_EVENT_MAP_STYLE_LOADED.value) {
      return;
    }
    final reference = _maps[event.sourceAddress];
    final map = reference?.target;
    if (map == null) {
      _maps.remove(event.sourceAddress);
      return;
    }
    map._clearCustomGeometryCallbacksAfterUrlStyleLoad();
  }

  /// Explicitly destroys this runtime.
  void close() {
    _state.close(
      (pointer) => _c.raw.mln_runtime_destroy(pointer).value,
      _c.threadLastErrorMessage,
    );
    _resourceTransformState?.close();
    _resourceTransformState = null;
    _resourceProviderRulesState?.close();
    _resourceProviderRulesState = null;
    _resourceProviderCallbackState?.close();
    _resourceProviderCallbackState = null;
  }
}

final class RuntimeEvent {
  RuntimeEvent._({
    required this.type,
    required this.sourceType,
    required this.sourceAddress,
    required this.code,
    required this.payloadType,
    required this.payloadSize,
    required this.message,
  });

  factory RuntimeEvent._fromNative(raw.mln_runtime_event event) {
    return RuntimeEvent._(
      type: event.type,
      sourceType: event.source_type,
      sourceAddress: event.source.address,
      code: event.code,
      payloadType: event.payload_type,
      payloadSize: event.payload_size,
      message: _copyNativeString(event.message, event.message_size),
    );
  }

  /// Raw native event type.
  final int type;

  /// Raw native event source type.
  final int sourceType;

  /// Borrowed native source handle address copied as an opaque value.
  final int sourceAddress;

  /// Native event code.
  final int code;

  /// Raw native payload type.
  final int payloadType;

  /// Native payload byte size.
  final int payloadSize;

  /// Copied event message, when one was provided.
  final String? message;
}

/// Map rendering mode used when creating a map.
final class MapMode {
  const MapMode._(this.rawValue, this.name);

  /// Continuously updates as data arrives and map state changes.
  static const continuous = MapMode._(0, 'continuous');

  /// Produces one-off still images of an arbitrary viewport.
  static const staticMap = MapMode._(1, 'static');

  /// Produces one-off still images for a single tile.
  static const tile = MapMode._(2, 'tile');

  /// Raw native value.
  final int rawValue;

  /// Human-readable name.
  final String name;
}

/// Map debug overlay option mask.
final class MapDebugOptions {
  /// Creates a debug option mask from raw bits.
  const MapDebugOptions(this.bits);

  /// No debug overlays.
  static const none = MapDebugOptions(0);

  /// Tile border overlay.
  static const tileBorders = MapDebugOptions(1 << 1);

  /// Parse status overlay.
  static const parseStatus = MapDebugOptions(1 << 2);

  /// Timestamp overlay.
  static const timestamps = MapDebugOptions(1 << 3);

  /// Collision overlay.
  static const collision = MapDebugOptions(1 << 4);

  /// Overdraw overlay.
  static const overdraw = MapDebugOptions(1 << 5);

  /// Stencil clip overlay.
  static const stencilClip = MapDebugOptions(1 << 6);

  /// Depth buffer overlay.
  static const depthBuffer = MapDebugOptions(1 << 7);

  /// Raw debug overlay bits.
  final int bits;

  /// Returns a mask containing bits from this mask and [other].
  MapDebugOptions union(MapDebugOptions other) =>
      MapDebugOptions(bits | other.bits);

  /// Returns true when all [option] bits are present.
  bool contains(MapDebugOptions option) => (bits & option.bits) == option.bits;
}

/// Map creation options.
final class MapOptions {
  /// Creates map options.
  const MapOptions({
    this.width = 256,
    this.height = 256,
    this.scaleFactor = 1,
    this.mapMode = MapMode.continuous,
  });

  /// Initial map width in logical pixels.
  final int width;

  /// Initial map height in logical pixels.
  final int height;

  /// Initial map scale factor.
  final double scaleFactor;

  /// Map rendering mode.
  final MapMode mapMode;
}

/// Owner-thread map handle bound to a retained runtime.
final class MapHandle {
  MapHandle._(this._runtime, Pointer<raw.mln_map> pointer)
    : _state = NativeHandleState(pointer, 'MapHandle');

  /// Creates a map owned by [runtime].
  factory MapHandle.create(
    RuntimeHandle runtime, {
    MapOptions options = const MapOptions(),
  }) {
    return withNativeArena((arena) {
      final nativeOptions = arena<raw.mln_map_options>();
      nativeOptions.ref = _c.raw.mln_map_options_default();
      nativeOptions.ref.width = _positiveUint32(options.width, 'map width');
      nativeOptions.ref.height = _positiveUint32(options.height, 'map height');
      nativeOptions.ref.scale_factor = options.scaleFactor;
      nativeOptions.ref.map_mode = options.mapMode.rawValue;
      final outMap = arena<Pointer<raw.mln_map>>();
      outMap.value = nullptr;

      _check(_c.raw.mln_map_create(runtime._pointer, nativeOptions, outMap));
      final map = MapHandle._(runtime, outMap.value);
      runtime._registerMap(map);
      return map;
    });
  }

  final RuntimeHandle _runtime;
  final NativeHandleState<raw.mln_map> _state;
  final _customGeometryCallbacks = <String, _CustomGeometryCallbackState>{};
  var _clearCustomGeometryCallbacksWhenUrlStyleLoads = false;

  /// Whether this map has been closed by the Dart binding.
  bool get isClosed => _state.isClosed;

  Pointer<raw.mln_map> get _pointer {
    final _ = _runtime._pointer;
    return _state.pointer;
  }

  /// Loads a style URL through MapLibre Native style APIs.
  void setStyleUrl(String url) {
    withNativeArena((arena) {
      final nativeUrl = nativeUtf8CString(url, arena);
      _check(
        _c.raw.mln_map_set_style_url(_pointer, nativeUrl.pointer.cast<Char>()),
      );
    });
    _clearCustomGeometryCallbacksWhenUrlStyleLoads =
        _customGeometryCallbacks.isNotEmpty;
  }

  /// Loads inline style JSON through MapLibre Native style APIs.
  void setStyleJson(String json) {
    withNativeArena((arena) {
      final nativeJson = nativeUtf8CString(json, arena);
      _check(
        _c.raw.mln_map_set_style_json(
          _pointer,
          nativeJson.pointer.cast<Char>(),
        ),
      );
    });
    _clearCustomGeometryCallbacks();
  }

  /// Requests a repaint for a continuous map.
  void requestRepaint() {
    _check(_c.raw.mln_map_request_repaint(_pointer));
  }

  /// Requests one still image for a static or tile map.
  void requestStillImage() {
    _check(_c.raw.mln_map_request_still_image(_pointer));
  }

  /// Applies MapLibre debug overlay options.
  void setDebugOptions(MapDebugOptions options) {
    _check(_c.raw.mln_map_set_debug_options(_pointer, options.bits));
  }

  /// Copies current MapLibre debug overlay options.
  MapDebugOptions debugOptions() {
    return withNativeArena((arena) {
      final outOptions = arena<Uint32>();
      _check(_c.raw.mln_map_get_debug_options(_pointer, outOptions));
      return MapDebugOptions(outOptions.value);
    });
  }

  /// Dumps map debug logs through MapLibre Native logging.
  void dumpDebugLogs() {
    _check(_c.raw.mln_map_dump_debug_logs(_pointer));
  }

  /// Attaches a Metal native surface render target.
  RenderSessionHandle attachMetalSurface(MetalSurfaceDescriptor descriptor) {
    return withNativeArena((arena) {
      final nativeDescriptor = arena<raw.mln_metal_surface_descriptor>();
      nativeDescriptor.ref = _metalSurfaceDescriptorToNative(descriptor);
      final outSession = arena<Pointer<raw.mln_render_session>>();
      outSession.value = nullptr;
      _check(
        _c.raw.mln_metal_surface_attach(_pointer, nativeDescriptor, outSession),
      );
      return RenderSessionHandle._(this, outSession.value);
    });
  }

  /// Attaches a Vulkan native surface render target.
  RenderSessionHandle attachVulkanSurface(VulkanSurfaceDescriptor descriptor) {
    return withNativeArena((arena) {
      final nativeDescriptor = arena<raw.mln_vulkan_surface_descriptor>();
      nativeDescriptor.ref = _vulkanSurfaceDescriptorToNative(descriptor);
      final outSession = arena<Pointer<raw.mln_render_session>>();
      outSession.value = nullptr;
      _check(
        _c.raw.mln_vulkan_surface_attach(
          _pointer,
          nativeDescriptor,
          outSession,
        ),
      );
      return RenderSessionHandle._(this, outSession.value);
    });
  }

  /// Attaches an OpenGL native surface render target.
  RenderSessionHandle attachOpenGLSurface(OpenGLSurfaceDescriptor descriptor) {
    return withNativeArena((arena) {
      final nativeDescriptor = arena<raw.mln_opengl_surface_descriptor>();
      nativeDescriptor.ref = _openglSurfaceDescriptorToNative(descriptor);
      final outSession = arena<Pointer<raw.mln_render_session>>();
      outSession.value = nullptr;
      _check(
        _c.raw.mln_opengl_surface_attach(
          _pointer,
          nativeDescriptor,
          outSession,
        ),
      );
      return RenderSessionHandle._(this, outSession.value);
    });
  }

  /// Attaches a Metal texture render target owned by the render session.
  RenderSessionHandle attachMetalOwnedTexture(
    MetalOwnedTextureDescriptor descriptor,
  ) {
    return withNativeArena((arena) {
      final nativeDescriptor = arena<raw.mln_metal_owned_texture_descriptor>();
      nativeDescriptor.ref = _metalOwnedTextureDescriptorToNative(descriptor);
      final outSession = arena<Pointer<raw.mln_render_session>>();
      outSession.value = nullptr;
      _check(
        _c.raw.mln_metal_owned_texture_attach(
          _pointer,
          nativeDescriptor,
          outSession,
        ),
      );
      return RenderSessionHandle._(this, outSession.value);
    });
  }

  /// Attaches a Metal caller-owned texture render target.
  RenderSessionHandle attachMetalBorrowedTexture(
    MetalBorrowedTextureDescriptor descriptor,
  ) {
    return withNativeArena((arena) {
      final nativeDescriptor =
          arena<raw.mln_metal_borrowed_texture_descriptor>();
      nativeDescriptor.ref = _metalBorrowedTextureDescriptorToNative(
        descriptor,
      );
      final outSession = arena<Pointer<raw.mln_render_session>>();
      outSession.value = nullptr;
      _check(
        _c.raw.mln_metal_borrowed_texture_attach(
          _pointer,
          nativeDescriptor,
          outSession,
        ),
      );
      return RenderSessionHandle._(this, outSession.value);
    });
  }

  /// Attaches a Vulkan texture render target owned by the render session.
  RenderSessionHandle attachVulkanOwnedTexture(
    VulkanOwnedTextureDescriptor descriptor,
  ) {
    return withNativeArena((arena) {
      final nativeDescriptor = arena<raw.mln_vulkan_owned_texture_descriptor>();
      nativeDescriptor.ref = _vulkanOwnedTextureDescriptorToNative(descriptor);
      final outSession = arena<Pointer<raw.mln_render_session>>();
      outSession.value = nullptr;
      _check(
        _c.raw.mln_vulkan_owned_texture_attach(
          _pointer,
          nativeDescriptor,
          outSession,
        ),
      );
      return RenderSessionHandle._(this, outSession.value);
    });
  }

  /// Attaches a Vulkan caller-owned texture render target.
  RenderSessionHandle attachVulkanBorrowedTexture(
    VulkanBorrowedTextureDescriptor descriptor,
  ) {
    return withNativeArena((arena) {
      final nativeDescriptor =
          arena<raw.mln_vulkan_borrowed_texture_descriptor>();
      nativeDescriptor.ref = _vulkanBorrowedTextureDescriptorToNative(
        descriptor,
      );
      final outSession = arena<Pointer<raw.mln_render_session>>();
      outSession.value = nullptr;
      _check(
        _c.raw.mln_vulkan_borrowed_texture_attach(
          _pointer,
          nativeDescriptor,
          outSession,
        ),
      );
      return RenderSessionHandle._(this, outSession.value);
    });
  }

  /// Attaches an OpenGL texture render target owned by the render session.
  RenderSessionHandle attachOpenGLOwnedTexture(
    OpenGLOwnedTextureDescriptor descriptor,
  ) {
    return withNativeArena((arena) {
      final nativeDescriptor = arena<raw.mln_opengl_owned_texture_descriptor>();
      nativeDescriptor.ref = _openglOwnedTextureDescriptorToNative(descriptor);
      final outSession = arena<Pointer<raw.mln_render_session>>();
      outSession.value = nullptr;
      _check(
        _c.raw.mln_opengl_owned_texture_attach(
          _pointer,
          nativeDescriptor,
          outSession,
        ),
      );
      return RenderSessionHandle._(this, outSession.value);
    });
  }

  /// Attaches an OpenGL caller-owned texture render target.
  RenderSessionHandle attachOpenGLBorrowedTexture(
    OpenGLBorrowedTextureDescriptor descriptor,
  ) {
    return withNativeArena((arena) {
      final nativeDescriptor =
          arena<raw.mln_opengl_borrowed_texture_descriptor>();
      nativeDescriptor.ref = _openglBorrowedTextureDescriptorToNative(
        descriptor,
      );
      final outSession = arena<Pointer<raw.mln_render_session>>();
      outSession.value = nullptr;
      _check(
        _c.raw.mln_opengl_borrowed_texture_attach(
          _pointer,
          nativeDescriptor,
          outSession,
        ),
      );
      return RenderSessionHandle._(this, outSession.value);
    });
  }

  /// Copies the current camera snapshot.
  CameraOptions camera() {
    return withNativeArena((arena) {
      final outCamera = arena<raw.mln_camera_options>();
      outCamera.ref.size = sizeOf<raw.mln_camera_options>();
      _check(_c.raw.mln_map_get_camera(_pointer, outCamera));
      return native_struct.cameraOptionsFromNative(outCamera.ref);
    });
  }

  /// Applies a camera jump command.
  void jumpTo(CameraOptions camera) {
    withNativeArena((arena) {
      final nativeCamera = _nativeCamera(camera, arena);
      _check(_c.raw.mln_map_jump_to(_pointer, nativeCamera));
    });
  }

  /// Applies a camera ease transition command.
  void easeTo(CameraOptions camera, {AnimationOptions? animation}) {
    withNativeArena((arena) {
      final nativeCamera = _nativeCamera(camera, arena);
      final nativeAnimation = _nativeAnimation(animation, arena);
      _check(_c.raw.mln_map_ease_to(_pointer, nativeCamera, nativeAnimation));
    });
  }

  /// Applies a camera fly transition command.
  void flyTo(CameraOptions camera, {AnimationOptions? animation}) {
    withNativeArena((arena) {
      final nativeCamera = _nativeCamera(camera, arena);
      final nativeAnimation = _nativeAnimation(animation, arena);
      _check(_c.raw.mln_map_fly_to(_pointer, nativeCamera, nativeAnimation));
    });
  }

  /// Applies a screen-space pan command.
  void moveBy(double deltaX, double deltaY, {AnimationOptions? animation}) {
    withNativeArena((arena) {
      final nativeAnimation = _nativeAnimation(animation, arena);
      _check(
        animation == null
            ? _c.raw.mln_map_move_by(_pointer, deltaX, deltaY)
            : _c.raw.mln_map_move_by_animated(
                _pointer,
                deltaX,
                deltaY,
                nativeAnimation,
              ),
      );
    });
  }

  /// Applies a screen-space zoom command.
  void scaleBy(
    double scale, {
    ScreenPoint? anchor,
    AnimationOptions? animation,
  }) {
    withNativeArena((arena) {
      final nativeAnchor = _nativeScreenPoint(anchor, arena);
      final nativeAnimation = _nativeAnimation(animation, arena);
      _check(
        animation == null
            ? _c.raw.mln_map_scale_by(_pointer, scale, nativeAnchor)
            : _c.raw.mln_map_scale_by_animated(
                _pointer,
                scale,
                nativeAnchor,
                nativeAnimation,
              ),
      );
    });
  }

  /// Applies a screen-space rotate command.
  void rotateBy(
    ScreenPoint first,
    ScreenPoint second, {
    AnimationOptions? animation,
  }) {
    withNativeArena((arena) {
      final nativeFirst = native_struct.screenPointToNative(first);
      final nativeSecond = native_struct.screenPointToNative(second);
      final nativeAnimation = _nativeAnimation(animation, arena);
      _check(
        animation == null
            ? _c.raw.mln_map_rotate_by(_pointer, nativeFirst, nativeSecond)
            : _c.raw.mln_map_rotate_by_animated(
                _pointer,
                nativeFirst,
                nativeSecond,
                nativeAnimation,
              ),
      );
    });
  }

  /// Applies a pitch delta command.
  void pitchBy(double pitch, {AnimationOptions? animation}) {
    withNativeArena((arena) {
      final nativeAnimation = _nativeAnimation(animation, arena);
      _check(
        animation == null
            ? _c.raw.mln_map_pitch_by(_pointer, pitch)
            : _c.raw.mln_map_pitch_by_animated(
                _pointer,
                pitch,
                nativeAnimation,
              ),
      );
    });
  }

  /// Cancels active camera transitions.
  void cancelTransitions() {
    _check(_c.raw.mln_map_cancel_transitions(_pointer));
  }

  /// Enables or disables the rendering stats overlay.
  void setRenderingStatsViewEnabled(bool enabled) {
    _check(_c.raw.mln_map_set_rendering_stats_view_enabled(_pointer, enabled));
  }

  /// Copies whether the rendering stats overlay is enabled.
  bool renderingStatsViewEnabled() {
    return withNativeArena((arena) {
      final outEnabled = arena<Bool>();
      _check(
        _c.raw.mln_map_get_rendering_stats_view_enabled(_pointer, outEnabled),
      );
      return outEnabled.value;
    });
  }

  /// Copies whether MapLibre currently considers the map fully loaded.
  bool isFullyLoaded() {
    return withNativeArena((arena) {
      final outLoaded = arena<Bool>();
      _check(_c.raw.mln_map_is_fully_loaded(_pointer, outLoaded));
      return outLoaded.value;
    });
  }

  /// Copies live map viewport and render-transform controls.
  MapViewportOptions viewportOptions() {
    return withNativeArena((arena) {
      final outOptions = arena<raw.mln_map_viewport_options>();
      outOptions.ref.size = sizeOf<raw.mln_map_viewport_options>();
      _check(_c.raw.mln_map_get_viewport_options(_pointer, outOptions));
      return native_struct.mapViewportOptionsFromNative(outOptions.ref);
    });
  }

  /// Applies selected live map viewport and render-transform controls.
  void setViewportOptions(MapViewportOptions options) {
    withNativeArena((arena) {
      final nativeOptions = arena<raw.mln_map_viewport_options>();
      nativeOptions.ref = native_struct.mapViewportOptionsToNative(options);
      _check(_c.raw.mln_map_set_viewport_options(_pointer, nativeOptions));
    });
  }

  /// Copies tile prefetch and LOD tuning controls.
  MapTileOptions tileOptions() {
    return withNativeArena((arena) {
      final outOptions = arena<raw.mln_map_tile_options>();
      outOptions.ref.size = sizeOf<raw.mln_map_tile_options>();
      _check(_c.raw.mln_map_get_tile_options(_pointer, outOptions));
      return native_struct.mapTileOptionsFromNative(outOptions.ref);
    });
  }

  /// Applies selected tile prefetch and LOD tuning controls.
  void setTileOptions(MapTileOptions options) {
    withNativeArena((arena) {
      final nativeOptions = arena<raw.mln_map_tile_options>();
      nativeOptions.ref = native_struct.mapTileOptionsToNative(options);
      _check(_c.raw.mln_map_set_tile_options(_pointer, nativeOptions));
    });
  }

  /// Copies map camera constraint options.
  BoundOptions bounds() {
    return withNativeArena((arena) {
      final outOptions = arena<raw.mln_bound_options>();
      outOptions.ref.size = sizeOf<raw.mln_bound_options>();
      _check(_c.raw.mln_map_get_bounds(_pointer, outOptions));
      return native_struct.boundOptionsFromNative(outOptions.ref);
    });
  }

  /// Applies selected map camera constraint options.
  void setBounds(BoundOptions options) {
    withNativeArena((arena) {
      final nativeOptions = arena<raw.mln_bound_options>();
      nativeOptions.ref = native_struct.boundOptionsToNative(options);
      _check(_c.raw.mln_map_set_bounds(_pointer, nativeOptions));
    });
  }

  /// Copies the current free camera position and orientation.
  FreeCameraOptions freeCameraOptions() {
    return withNativeArena((arena) {
      final outOptions = arena<raw.mln_free_camera_options>();
      outOptions.ref.size = sizeOf<raw.mln_free_camera_options>();
      _check(_c.raw.mln_map_get_free_camera_options(_pointer, outOptions));
      return native_struct.freeCameraOptionsFromNative(outOptions.ref);
    });
  }

  /// Applies selected free camera position and orientation fields.
  void setFreeCameraOptions(FreeCameraOptions options) {
    withNativeArena((arena) {
      final nativeOptions = arena<raw.mln_free_camera_options>();
      nativeOptions.ref = native_struct.freeCameraOptionsToNative(options);
      _check(_c.raw.mln_map_set_free_camera_options(_pointer, nativeOptions));
    });
  }

  /// Copies the current axonometric rendering options.
  ProjectionModeOptions projectionMode() {
    return withNativeArena((arena) {
      final outMode = arena<raw.mln_projection_mode>();
      outMode.ref.size = sizeOf<raw.mln_projection_mode>();
      _check(_c.raw.mln_map_get_projection_mode(_pointer, outMode));
      return native_struct.projectionModeOptionsFromNative(outMode.ref);
    });
  }

  /// Applies selected axonometric rendering option fields.
  void setProjectionMode(ProjectionModeOptions mode) {
    withNativeArena((arena) {
      final nativeMode = arena<raw.mln_projection_mode>();
      nativeMode.ref = native_struct.projectionModeOptionsToNative(mode);
      _check(_c.raw.mln_map_set_projection_mode(_pointer, nativeMode));
    });
  }

  /// Computes a camera that fits geographic bounds in the current viewport.
  CameraOptions cameraForLatLngBounds(
    LatLngBounds bounds, {
    CameraFitOptions fitOptions = const CameraFitOptions(),
  }) {
    return withNativeArena((arena) {
      final outCamera = arena<raw.mln_camera_options>();
      outCamera.ref.size = sizeOf<raw.mln_camera_options>();
      final nativeFitOptions = arena<raw.mln_camera_fit_options>();
      nativeFitOptions.ref = native_struct.cameraFitOptionsToNative(fitOptions);
      _check(
        _c.raw.mln_map_camera_for_lat_lng_bounds(
          _pointer,
          native_struct.latLngBoundsToNative(bounds),
          nativeFitOptions,
          outCamera,
        ),
      );
      return native_struct.cameraOptionsFromNative(outCamera.ref);
    });
  }

  /// Computes a camera that fits geographic coordinates in the current viewport.
  CameraOptions cameraForLatLngs(
    List<LatLng> coordinates, {
    CameraFitOptions fitOptions = const CameraFitOptions(),
  }) {
    return withNativeArena((arena) {
      final outCamera = arena<raw.mln_camera_options>();
      outCamera.ref.size = sizeOf<raw.mln_camera_options>();
      final nativeFitOptions = arena<raw.mln_camera_fit_options>();
      nativeFitOptions.ref = native_struct.cameraFitOptionsToNative(fitOptions);
      _check(
        _c.raw.mln_map_camera_for_lat_lngs(
          _pointer,
          _latLngArray(coordinates, arena),
          coordinates.length,
          nativeFitOptions,
          outCamera,
        ),
      );
      return native_struct.cameraOptionsFromNative(outCamera.ref);
    });
  }

  /// Computes a camera that fits a geometry in the current viewport.
  CameraOptions cameraForGeometry(
    Geometry geometry, {
    CameraFitOptions fitOptions = const CameraFitOptions(),
  }) {
    return withNativeArena((arena) {
      final outCamera = arena<raw.mln_camera_options>();
      outCamera.ref.size = sizeOf<raw.mln_camera_options>();
      final nativeFitOptions = arena<raw.mln_camera_fit_options>();
      nativeFitOptions.ref = native_struct.cameraFitOptionsToNative(fitOptions);
      final nativeGeometry = native_geometry.nativeGeometry(geometry, arena);
      _check(
        _c.raw.mln_map_camera_for_geometry(
          _pointer,
          nativeGeometry.pointer,
          nativeFitOptions,
          outCamera,
        ),
      );
      return native_struct.cameraOptionsFromNative(outCamera.ref);
    });
  }

  /// Computes wrapped geographic bounds for a camera.
  LatLngBounds latLngBoundsForCamera(CameraOptions camera) =>
      _latLngBoundsForCamera(camera, unwrapped: false);

  /// Computes unwrapped geographic bounds for a camera.
  LatLngBounds latLngBoundsForCameraUnwrapped(CameraOptions camera) =>
      _latLngBoundsForCamera(camera, unwrapped: true);

  LatLngBounds _latLngBoundsForCamera(
    CameraOptions camera, {
    required bool unwrapped,
  }) {
    return withNativeArena((arena) {
      final nativeCamera = _nativeCamera(camera, arena);
      final outBounds = arena<raw.mln_lat_lng_bounds>();
      _check(
        unwrapped
            ? _c.raw.mln_map_lat_lng_bounds_for_camera_unwrapped(
                _pointer,
                nativeCamera,
                outBounds,
              )
            : _c.raw.mln_map_lat_lng_bounds_for_camera(
                _pointer,
                nativeCamera,
                outBounds,
              ),
      );
      return native_struct.latLngBoundsFromNative(outBounds.ref);
    });
  }

  /// Converts a geographic world coordinate to a screen point.
  ScreenPoint pixelForLatLng(LatLng coordinate) {
    return withNativeArena((arena) {
      final outPoint = arena<raw.mln_screen_point>();
      _check(
        _c.raw.mln_map_pixel_for_lat_lng(
          _pointer,
          native_struct.latLngToNative(coordinate),
          outPoint,
        ),
      );
      return native_struct.screenPointFromNative(outPoint.ref);
    });
  }

  /// Converts a screen point to a geographic world coordinate.
  LatLng latLngForPixel(ScreenPoint point) {
    return withNativeArena((arena) {
      final outCoordinate = arena<raw.mln_lat_lng>();
      _check(
        _c.raw.mln_map_lat_lng_for_pixel(
          _pointer,
          native_struct.screenPointToNative(point),
          outCoordinate,
        ),
      );
      return native_struct.latLngFromNative(outCoordinate.ref);
    });
  }

  /// Converts geographic coordinates to screen points.
  List<ScreenPoint> pixelsForLatLngs(List<LatLng> coordinates) {
    return withNativeArena((arena) {
      final outPoints = coordinates.isEmpty
          ? nullptr.cast<raw.mln_screen_point>()
          : arena<raw.mln_screen_point>(coordinates.length);
      _check(
        _c.raw.mln_map_pixels_for_lat_lngs(
          _pointer,
          _latLngArray(coordinates, arena),
          coordinates.length,
          outPoints,
        ),
      );
      return [
        for (var index = 0; index < coordinates.length; index += 1)
          native_struct.screenPointFromNative(outPoints[index]),
      ];
    });
  }

  /// Converts screen points to geographic coordinates.
  List<LatLng> latLngsForPixels(List<ScreenPoint> points) {
    return withNativeArena((arena) {
      final nativePoints = points.isEmpty
          ? nullptr.cast<raw.mln_screen_point>()
          : arena<raw.mln_screen_point>(points.length);
      for (var index = 0; index < points.length; index += 1) {
        nativePoints[index] = native_struct.screenPointToNative(points[index]);
      }
      final outCoordinates = points.isEmpty
          ? nullptr.cast<raw.mln_lat_lng>()
          : arena<raw.mln_lat_lng>(points.length);
      _check(
        _c.raw.mln_map_lat_lngs_for_pixels(
          _pointer,
          nativePoints,
          points.length,
          outCoordinates,
        ),
      );
      return [
        for (var index = 0; index < points.length; index += 1)
          native_struct.latLngFromNative(outCoordinates[index]),
      ];
    });
  }

  /// Creates a standalone projection helper from the current map transform.
  MapProjectionHandle createProjection() {
    return withNativeArena((arena) {
      final outProjection = arena<Pointer<raw.mln_map_projection>>();
      outProjection.value = nullptr;
      _check(_c.raw.mln_map_projection_create(_pointer, outProjection));
      return MapProjectionHandle._(outProjection.value);
    });
  }

  /// Sets or replaces one runtime style image.
  void setStyleImage(
    String imageId,
    PremultipliedRgba8Image image, {
    StyleImageOptions options = const StyleImageOptions(),
  }) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(imageId, arena);
      final nativeImage = arena<raw.mln_premultiplied_rgba8_image>();
      nativeImage.ref = _premultipliedRgba8ImageToNative(image, arena);
      final nativeOptions = arena<raw.mln_style_image_options>();
      nativeOptions.ref = _styleImageOptionsToNative(options);
      _check(
        _c.raw.mln_map_set_style_image(
          _pointer,
          nativeId.value,
          nativeImage,
          nativeOptions,
        ),
      );
    });
  }

  /// Removes one runtime style image and returns whether one was removed.
  bool removeStyleImage(String imageId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(imageId, arena);
      final outRemoved = arena<Bool>();
      _check(
        _c.raw.mln_map_remove_style_image(_pointer, nativeId.value, outRemoved),
      );
      return outRemoved.value;
    });
  }

  /// Reports whether one runtime style image exists.
  bool styleImageExists(String imageId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(imageId, arena);
      final outExists = arena<Bool>();
      _check(
        _c.raw.mln_map_style_image_exists(_pointer, nativeId.value, outExists),
      );
      return outExists.value;
    });
  }

  /// Copies fixed metadata for one runtime style image.
  StyleImageInfo? getStyleImageInfo(String imageId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(imageId, arena);
      final outInfo = arena<raw.mln_style_image_info>();
      outInfo.ref = _c.raw.mln_style_image_info_default();
      final outFound = arena<Bool>();
      _check(
        _c.raw.mln_map_get_style_image_info(
          _pointer,
          nativeId.value,
          outInfo,
          outFound,
        ),
      );
      return outFound.value ? _styleImageInfoFromNative(outInfo.ref) : null;
    });
  }

  /// Copies one runtime style image as premultiplied RGBA8 pixels.
  StyleImage? copyStyleImagePremultipliedRgba8(String imageId) {
    final info = getStyleImageInfo(imageId);
    if (info == null) {
      return null;
    }
    return withNativeArena((arena) {
      final nativeId = nativeStringView(imageId, arena);
      final pixels = info.byteLength == 0
          ? nullptr.cast<Uint8>()
          : arena<Uint8>(info.byteLength);
      final outByteLength = arena<Size>();
      final outFound = arena<Bool>();
      _check(
        _c.raw.mln_map_copy_style_image_premultiplied_rgba8(
          _pointer,
          nativeId.value,
          pixels,
          info.byteLength,
          outByteLength,
          outFound,
        ),
      );
      if (!outFound.value) {
        return null;
      }
      return StyleImage(
        info: info,
        bytes: Uint8List.fromList(pixels.asTypedList(outByteLength.value)),
      );
    });
  }

  /// Adds one style source from a style-spec source JSON object.
  void addStyleSourceJson(String sourceId, JsonValue sourceJson) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeSourceJson = native_json.nativeJsonValue(sourceJson, arena);
      _check(
        _c.raw.mln_map_add_style_source_json(
          _pointer,
          nativeId.value,
          nativeSourceJson.pointer,
        ),
      );
    });
  }

  /// Adds a GeoJSON source that loads from [url].
  void addGeoJsonSourceUrl(String sourceId, String url) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeUrl = nativeStringView(url, arena);
      _check(
        _c.raw.mln_map_add_geojson_source_url(
          _pointer,
          nativeId.value,
          nativeUrl.value,
        ),
      );
    });
  }

  /// Adds a GeoJSON source with inline data.
  void addGeoJsonSourceData(String sourceId, GeoJson data) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeData = native_geometry.nativeGeoJson(data, arena);
      _check(
        _c.raw.mln_map_add_geojson_source_data(
          _pointer,
          nativeId.value,
          nativeData.pointer,
        ),
      );
    });
  }

  /// Updates one GeoJSON source to load from [url].
  void setGeoJsonSourceUrl(String sourceId, String url) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeUrl = nativeStringView(url, arena);
      _check(
        _c.raw.mln_map_set_geojson_source_url(
          _pointer,
          nativeId.value,
          nativeUrl.value,
        ),
      );
    });
  }

  /// Updates one GeoJSON source with inline data.
  void setGeoJsonSourceData(String sourceId, GeoJson data) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeData = native_geometry.nativeGeoJson(data, arena);
      _check(
        _c.raw.mln_map_set_geojson_source_data(
          _pointer,
          nativeId.value,
          nativeData.pointer,
        ),
      );
    });
  }

  /// Adds a vector source with a TileJSON URL.
  void addVectorSourceUrl(
    String sourceId,
    String url, {
    TileSourceOptions options = const TileSourceOptions(),
  }) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeUrl = nativeStringView(url, arena);
      _check(
        _c.raw.mln_map_add_vector_source_url(
          _pointer,
          nativeId.value,
          nativeUrl.value,
          _nativeTileSourceOptions(options, arena),
        ),
      );
    });
  }

  /// Adds a vector source with inline tile URL templates.
  void addVectorSourceTiles(
    String sourceId,
    List<String> tiles, {
    TileSourceOptions options = const TileSourceOptions(),
  }) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      _check(
        _c.raw.mln_map_add_vector_source_tiles(
          _pointer,
          nativeId.value,
          _stringViewArray(tiles, arena),
          tiles.length,
          _nativeTileSourceOptions(options, arena),
        ),
      );
    });
  }

  /// Adds a raster source with a TileJSON URL.
  void addRasterSourceUrl(
    String sourceId,
    String url, {
    TileSourceOptions options = const TileSourceOptions(),
  }) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeUrl = nativeStringView(url, arena);
      _check(
        _c.raw.mln_map_add_raster_source_url(
          _pointer,
          nativeId.value,
          nativeUrl.value,
          _nativeTileSourceOptions(options, arena),
        ),
      );
    });
  }

  /// Adds a raster source with inline tile URL templates.
  void addRasterSourceTiles(
    String sourceId,
    List<String> tiles, {
    TileSourceOptions options = const TileSourceOptions(),
  }) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      _check(
        _c.raw.mln_map_add_raster_source_tiles(
          _pointer,
          nativeId.value,
          _stringViewArray(tiles, arena),
          tiles.length,
          _nativeTileSourceOptions(options, arena),
        ),
      );
    });
  }

  /// Adds a raster DEM source with a TileJSON URL.
  void addRasterDemSourceUrl(
    String sourceId,
    String url, {
    TileSourceOptions options = const TileSourceOptions(),
  }) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeUrl = nativeStringView(url, arena);
      _check(
        _c.raw.mln_map_add_raster_dem_source_url(
          _pointer,
          nativeId.value,
          nativeUrl.value,
          _nativeTileSourceOptions(options, arena),
        ),
      );
    });
  }

  /// Adds a raster DEM source with inline tile URL templates.
  void addRasterDemSourceTiles(
    String sourceId,
    List<String> tiles, {
    TileSourceOptions options = const TileSourceOptions(),
  }) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      _check(
        _c.raw.mln_map_add_raster_dem_source_tiles(
          _pointer,
          nativeId.value,
          _stringViewArray(tiles, arena),
          tiles.length,
          _nativeTileSourceOptions(options, arena),
        ),
      );
    });
  }

  /// Adds an image source that loads its image from [url].
  void addImageSourceUrl(
    String sourceId,
    List<LatLng> coordinates,
    String url,
  ) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeUrl = nativeStringView(url, arena);
      _check(
        _c.raw.mln_map_add_image_source_url(
          _pointer,
          nativeId.value,
          _latLngArray(coordinates, arena),
          coordinates.length,
          nativeUrl.value,
        ),
      );
    });
  }

  /// Adds an image source with inline image pixels.
  void addImageSourceImage(
    String sourceId,
    List<LatLng> coordinates,
    PremultipliedRgba8Image image,
  ) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeImage = arena<raw.mln_premultiplied_rgba8_image>();
      nativeImage.ref = _premultipliedRgba8ImageToNative(image, arena);
      _check(
        _c.raw.mln_map_add_image_source_image(
          _pointer,
          nativeId.value,
          _latLngArray(coordinates, arena),
          coordinates.length,
          nativeImage,
        ),
      );
    });
  }

  /// Updates an image source to load from [url].
  void setImageSourceUrl(String sourceId, String url) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeUrl = nativeStringView(url, arena);
      _check(
        _c.raw.mln_map_set_image_source_url(
          _pointer,
          nativeId.value,
          nativeUrl.value,
        ),
      );
    });
  }

  /// Updates an image source with inline image pixels.
  void setImageSourceImage(String sourceId, PremultipliedRgba8Image image) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeImage = arena<raw.mln_premultiplied_rgba8_image>();
      nativeImage.ref = _premultipliedRgba8ImageToNative(image, arena);
      _check(
        _c.raw.mln_map_set_image_source_image(
          _pointer,
          nativeId.value,
          nativeImage,
        ),
      );
    });
  }

  /// Updates image source coordinates.
  void setImageSourceCoordinates(String sourceId, List<LatLng> coordinates) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      _check(
        _c.raw.mln_map_set_image_source_coordinates(
          _pointer,
          nativeId.value,
          _latLngArray(coordinates, arena),
          coordinates.length,
        ),
      );
    });
  }

  /// Copies image source coordinates, or null when the source is missing.
  List<LatLng>? getImageSourceCoordinates(String sourceId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final outCoordinates = arena<raw.mln_lat_lng>(4);
      final outCount = arena<Size>();
      final outFound = arena<Bool>();
      _check(
        _c.raw.mln_map_get_image_source_coordinates(
          _pointer,
          nativeId.value,
          outCoordinates,
          4,
          outCount,
          outFound,
        ),
      );
      return outFound.value
          ? [
              for (var index = 0; index < outCount.value; index += 1)
                native_struct.latLngFromNative(outCoordinates[index]),
            ]
          : null;
    });
  }

  /// Adds a custom geometry source with queued fetch/cancel notifications.
  void addCustomGeometrySource(
    String sourceId,
    CustomGeometrySourceOptions options,
  ) {
    final callbackState = _CustomGeometryCallbackState(options);
    try {
      withNativeArena((arena) {
        final nativeId = nativeStringView(sourceId, arena);
        final nativeOptions = arena<raw.mln_custom_geometry_source_options>();
        nativeOptions.ref = _customGeometrySourceOptionsToNative(
          options,
          callbackState,
        );
        _check(
          _c.raw.mln_map_add_custom_geometry_source(
            _pointer,
            nativeId.value,
            nativeOptions,
          ),
        );
      });
      _customGeometryCallbacks.remove(sourceId)?.close();
      _customGeometryCallbacks[sourceId] = callbackState;
    } catch (_) {
      callbackState.close();
      rethrow;
    }
  }

  /// Sets custom geometry source data for one canonical tile.
  void setCustomGeometrySourceTileData(
    String sourceId,
    CanonicalTileId tileId,
    GeoJson data,
  ) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeData = native_geometry.nativeGeoJson(data, arena);
      _check(
        _c.raw.mln_map_set_custom_geometry_source_tile_data(
          _pointer,
          nativeId.value,
          _canonicalTileIdToNative(tileId),
          nativeData.pointer,
        ),
      );
    });
  }

  /// Invalidates custom geometry source data for one canonical tile.
  void invalidateCustomGeometrySourceTile(
    String sourceId,
    CanonicalTileId tileId,
  ) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      _check(
        _c.raw.mln_map_invalidate_custom_geometry_source_tile(
          _pointer,
          nativeId.value,
          _canonicalTileIdToNative(tileId),
        ),
      );
    });
  }

  /// Invalidates custom geometry source data inside one geographic region.
  void invalidateCustomGeometrySourceRegion(
    String sourceId,
    LatLngBounds bounds,
  ) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      _check(
        _c.raw.mln_map_invalidate_custom_geometry_source_region(
          _pointer,
          nativeId.value,
          native_struct.latLngBoundsToNative(bounds),
        ),
      );
    });
  }

  /// Reports whether a style source ID exists.
  bool styleSourceExists(String sourceId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final outExists = arena<Bool>();
      _check(
        _c.raw.mln_map_style_source_exists(_pointer, nativeId.value, outExists),
      );
      return outExists.value;
    });
  }

  /// Removes one style source by ID and returns whether one was removed.
  bool removeStyleSource(String sourceId) {
    final removed = withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final outRemoved = arena<Bool>();
      _check(
        _c.raw.mln_map_remove_style_source(
          _pointer,
          nativeId.value,
          outRemoved,
        ),
      );
      return outRemoved.value;
    });
    if (removed) {
      _customGeometryCallbacks.remove(sourceId)?.close();
    }
    return removed;
  }

  /// Copies fixed style source metadata, or returns null when the source is absent.
  SourceInfo? getStyleSourceInfo(String sourceId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final outInfo = arena<raw.mln_style_source_info>();
      outInfo.ref.size = sizeOf<raw.mln_style_source_info>();
      final outFound = arena<Bool>();
      _check(
        _c.raw.mln_map_get_style_source_info(
          _pointer,
          nativeId.value,
          outInfo,
          outFound,
        ),
      );
      if (!outFound.value) {
        return null;
      }
      final info = outInfo.ref;
      return SourceInfo(
        type: SourceType.fromRaw(info.type),
        id: sourceId,
        isVolatile: info.is_volatile,
        attribution: _copyStyleSourceAttribution(
          _pointer,
          nativeId.value,
          info.has_attribution,
          info.attribution_size,
          arena,
        ),
      );
    });
  }

  /// Copies style source IDs in style order.
  List<String> listStyleSourceIds() {
    return withNativeArena((arena) {
      final outList = arena<Pointer<raw.mln_style_id_list>>();
      outList.value = nullptr;
      _check(_c.raw.mln_map_list_style_source_ids(_pointer, outList));
      return _copyStyleIdList(outList.value);
    });
  }

  /// Adds a hillshade layer for a raster DEM source.
  void addHillshadeLayer(
    String layerId,
    String sourceId, {
    String? beforeLayerId,
  }) {
    withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      final nativeSourceId = nativeStringView(sourceId, arena);
      final nativeBeforeLayerId = nativeStringView(beforeLayerId ?? '', arena);
      _check(
        _c.raw.mln_map_add_hillshade_layer(
          _pointer,
          nativeLayerId.value,
          nativeSourceId.value,
          nativeBeforeLayerId.value,
        ),
      );
    });
  }

  /// Adds a color-relief layer for a raster DEM source.
  void addColorReliefLayer(
    String layerId,
    String sourceId, {
    String? beforeLayerId,
  }) {
    withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      final nativeSourceId = nativeStringView(sourceId, arena);
      final nativeBeforeLayerId = nativeStringView(beforeLayerId ?? '', arena);
      _check(
        _c.raw.mln_map_add_color_relief_layer(
          _pointer,
          nativeLayerId.value,
          nativeSourceId.value,
          nativeBeforeLayerId.value,
        ),
      );
    });
  }

  /// Adds a source-free location indicator layer.
  void addLocationIndicatorLayer(String layerId, {String? beforeLayerId}) {
    withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      final nativeBeforeLayerId = nativeStringView(beforeLayerId ?? '', arena);
      _check(
        _c.raw.mln_map_add_location_indicator_layer(
          _pointer,
          nativeLayerId.value,
          nativeBeforeLayerId.value,
        ),
      );
    });
  }

  /// Sets a location indicator layer location.
  void setLocationIndicatorLocation(
    String layerId,
    LatLng coordinate, {
    double altitude = 0,
  }) {
    withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      _check(
        _c.raw.mln_map_set_location_indicator_location(
          _pointer,
          nativeLayerId.value,
          native_struct.latLngToNative(coordinate),
          altitude,
        ),
      );
    });
  }

  /// Sets a location indicator layer bearing in degrees.
  void setLocationIndicatorBearing(String layerId, double bearing) {
    withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      _check(
        _c.raw.mln_map_set_location_indicator_bearing(
          _pointer,
          nativeLayerId.value,
          bearing,
        ),
      );
    });
  }

  /// Sets a location indicator layer accuracy radius in logical pixels.
  void setLocationIndicatorAccuracyRadius(String layerId, double radius) {
    withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      _check(
        _c.raw.mln_map_set_location_indicator_accuracy_radius(
          _pointer,
          nativeLayerId.value,
          radius,
        ),
      );
    });
  }

  /// Sets one location indicator image-name property.
  void setLocationIndicatorImageName(
    String layerId,
    LocationIndicatorImageKind imageKind,
    String imageId,
  ) {
    withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      final nativeImageId = nativeStringView(imageId, arena);
      _check(
        _c.raw.mln_map_set_location_indicator_image_name(
          _pointer,
          nativeLayerId.value,
          imageKind.rawValue,
          nativeImageId.value,
        ),
      );
    });
  }

  /// Adds one style layer from a full style-spec layer JSON object.
  void addStyleLayerJson(JsonValue layerJson, {String? beforeLayerId}) {
    withNativeArena((arena) {
      final nativeLayerJson = native_json.nativeJsonValue(layerJson, arena);
      final nativeBeforeLayerId = nativeStringView(beforeLayerId ?? '', arena);
      _check(
        _c.raw.mln_map_add_style_layer_json(
          _pointer,
          nativeLayerJson.pointer,
          nativeBeforeLayerId.value,
        ),
      );
    });
  }

  /// Copies one style layer as a full style-spec layer JSON snapshot.
  JsonValue? getStyleLayerJson(String layerId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(layerId, arena);
      final outLayer = arena<Pointer<raw.mln_json_snapshot>>();
      outLayer.value = nullptr;
      final outFound = arena<Bool>();
      _check(
        _c.raw.mln_map_get_style_layer_json(
          _pointer,
          nativeId.value,
          outLayer,
          outFound,
        ),
      );
      if (!outFound.value) {
        return null;
      }
      return _copyJsonSnapshot(outLayer.value);
    });
  }

  /// Sets the style light from a style-spec light JSON object.
  void setStyleLightJson(JsonValue lightJson) {
    withNativeArena((arena) {
      final nativeLightJson = native_json.nativeJsonValue(lightJson, arena);
      _check(
        _c.raw.mln_map_set_style_light_json(_pointer, nativeLightJson.pointer),
      );
    });
  }

  /// Sets one style light property by style-spec property name.
  void setStyleLightProperty(String propertyName, JsonValue value) {
    withNativeArena((arena) {
      final nativePropertyName = nativeStringView(propertyName, arena);
      final nativeValue = native_json.nativeJsonValue(value, arena);
      _check(
        _c.raw.mln_map_set_style_light_property(
          _pointer,
          nativePropertyName.value,
          nativeValue.pointer,
        ),
      );
    });
  }

  /// Copies one style light property, or null when the property is undefined.
  JsonValue? getStyleLightProperty(String propertyName) {
    return withNativeArena((arena) {
      final nativePropertyName = nativeStringView(propertyName, arena);
      final outValue = arena<Pointer<raw.mln_json_snapshot>>();
      outValue.value = nullptr;
      _check(
        _c.raw.mln_map_get_style_light_property(
          _pointer,
          nativePropertyName.value,
          outValue,
        ),
      );
      return _copyJsonSnapshot(outValue.value);
    });
  }

  /// Sets one layer property by style-spec property name.
  void setLayerProperty(String layerId, String propertyName, JsonValue value) {
    withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      final nativePropertyName = nativeStringView(propertyName, arena);
      final nativeValue = native_json.nativeJsonValue(value, arena);
      _check(
        _c.raw.mln_map_set_layer_property(
          _pointer,
          nativeLayerId.value,
          nativePropertyName.value,
          nativeValue.pointer,
        ),
      );
    });
  }

  /// Copies one layer property, or null when the property is undefined.
  JsonValue? getLayerProperty(String layerId, String propertyName) {
    return withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      final nativePropertyName = nativeStringView(propertyName, arena);
      final outValue = arena<Pointer<raw.mln_json_snapshot>>();
      outValue.value = nullptr;
      _check(
        _c.raw.mln_map_get_layer_property(
          _pointer,
          nativeLayerId.value,
          nativePropertyName.value,
          outValue,
        ),
      );
      return _copyJsonSnapshot(outValue.value);
    });
  }

  /// Sets or clears one layer filter.
  void setLayerFilter(String layerId, JsonValue? filter) {
    withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      final nativeFilter = filter == null
          ? nullptr.cast<raw.mln_json_value>()
          : native_json.nativeJsonValue(filter, arena).pointer;
      _check(
        _c.raw.mln_map_set_layer_filter(
          _pointer,
          nativeLayerId.value,
          nativeFilter,
        ),
      );
    });
  }

  /// Copies one layer filter, or null when the layer has no filter.
  JsonValue? getLayerFilter(String layerId) {
    return withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      final outFilter = arena<Pointer<raw.mln_json_snapshot>>();
      outFilter.value = nullptr;
      _check(
        _c.raw.mln_map_get_layer_filter(
          _pointer,
          nativeLayerId.value,
          outFilter,
        ),
      );
      return _copyJsonSnapshot(outFilter.value);
    });
  }

  /// Reports whether a style layer ID exists.
  bool styleLayerExists(String layerId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(layerId, arena);
      final outExists = arena<Bool>();
      _check(
        _c.raw.mln_map_style_layer_exists(_pointer, nativeId.value, outExists),
      );
      return outExists.value;
    });
  }

  /// Borrows one style layer type string, or returns null when absent.
  String? getStyleLayerType(String layerId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(layerId, arena);
      final outLayerType = arena<raw.mln_string_view>();
      final outFound = arena<Bool>();
      _check(
        _c.raw.mln_map_get_style_layer_type(
          _pointer,
          nativeId.value,
          outLayerType,
          outFound,
        ),
      );
      if (!outFound.value) {
        return null;
      }
      return _copyStringView(outLayerType.ref);
    });
  }

  /// Moves one style layer before another layer or to the top.
  void moveStyleLayer(String layerId, {String? beforeLayerId}) {
    withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      final nativeBeforeLayerId = nativeStringView(beforeLayerId ?? '', arena);
      _check(
        _c.raw.mln_map_move_style_layer(
          _pointer,
          nativeLayerId.value,
          nativeBeforeLayerId.value,
        ),
      );
    });
  }

  /// Removes one style layer by ID and returns whether one was removed.
  bool removeStyleLayer(String layerId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(layerId, arena);
      final outRemoved = arena<Bool>();
      _check(
        _c.raw.mln_map_remove_style_layer(_pointer, nativeId.value, outRemoved),
      );
      return outRemoved.value;
    });
  }

  /// Copies style layer IDs in style order.
  List<String> listStyleLayerIds() {
    return withNativeArena((arena) {
      final outList = arena<Pointer<raw.mln_style_id_list>>();
      outList.value = nullptr;
      _check(_c.raw.mln_map_list_style_layer_ids(_pointer, outList));
      return _copyStyleIdList(outList.value);
    });
  }

  /// Explicitly destroys this map.
  void close() {
    final address = _state.pointerAddress;
    _state.close(
      (pointer) => _c.raw.mln_map_destroy(pointer).value,
      _c.threadLastErrorMessage,
    );
    _runtime._unregisterMapAddress(address);
    _clearCustomGeometryCallbacks();
  }

  void _clearCustomGeometryCallbacksAfterUrlStyleLoad() {
    if (!_clearCustomGeometryCallbacksWhenUrlStyleLoads) {
      return;
    }
    _clearCustomGeometryCallbacksWhenUrlStyleLoads = false;
    _clearCustomGeometryCallbacks();
  }

  void _clearCustomGeometryCallbacks() {
    for (final state in _customGeometryCallbacks.values) {
      state.close();
    }
    _customGeometryCallbacks.clear();
  }
}

/// Standalone projection helper snapshot from a map transform.

import 'dart:ffi';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';

import '../camera/camera.dart';
import '../geo/geo.dart';
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

final MaplibreNativeCApi _c = MaplibreNativeCApi.open();

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

/// Owner-thread runtime handle for MapLibre Native work and event polling.
final class RuntimeHandle {
  RuntimeHandle._(Pointer<raw.mln_runtime> pointer)
    : _state = NativeHandleState(pointer, 'RuntimeHandle');

  final NativeHandleState<raw.mln_runtime> _state;
  _ResourceTransformState? _resourceTransformState;
  _ResourceProviderRulesState? _resourceProviderRulesState;
  _ResourceProviderCallbackState? _resourceProviderCallbackState;

  /// Creates a runtime on the current native thread using native defaults.
  factory RuntimeHandle.create() {
    return withNativeArena((arena) {
      final outRuntime = arena<Pointer<raw.mln_runtime>>();
      outRuntime.value = nullptr;
      _check(_c.runtimeCreate(nullptr, outRuntime));
      return RuntimeHandle._(outRuntime.value);
    });
  }

  Pointer<raw.mln_runtime> get _pointer => _state.pointer;

  /// Whether this runtime has been closed by the Dart binding.
  bool get isClosed => _state.isClosed;

  /// Runs one pending owner-thread task for this runtime.
  void runOnce() {
    _check(_c.runtimeRunOnce(_pointer));
  }

  /// Polls one queued runtime event and copies borrowed fields into Dart values.
  RuntimeEvent? pollEvent() {
    return withNativeArena((arena) {
      final event = arena<raw.mln_runtime_event>();
      event.ref.size = sizeOf<raw.mln_runtime_event>();
      final hasEvent = arena<Bool>();
      hasEvent.value = false;

      _check(_c.runtimePollEvent(_pointer, event, hasEvent));
      if (!hasEvent.value) {
        return null;
      }

      return RuntimeEvent._fromNative(event.ref);
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
        _check(_c.runtimeSetResourceTransform(_pointer, transform));
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
    _check(_c.runtimeClearResourceTransform(_pointer));
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
        _check(_c.runtimeSetResourceProvider(_pointer, provider));
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
        _check(_c.runtimeSetResourceProvider(_pointer, nativeProvider));
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
        _c.runtimeRunAmbientCacheOperationStart(
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
        _c.runtimeOfflineRegionCreateStart(
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
          _c.runtimeOfflineRegionGetStart(_pointer, regionId, outOperationId),
        );
      });

  /// Starts listing offline region snapshots.
  OfflineOperationHandle listOfflineRegions() =>
      _startOfflineOperation((outOperationId) {
        _check(_c.runtimeOfflineRegionsListStart(_pointer, outOperationId));
      });

  /// Starts merging offline regions from another database path.
  OfflineOperationHandle mergeOfflineRegionDatabase(String sideDatabasePath) {
    return withNativeArena((arena) {
      final nativePath = nativeUtf8CString(sideDatabasePath, arena);
      final outOperationId = arena<Uint64>();
      _check(
        _c.runtimeOfflineRegionsMergeDatabaseStart(
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
        _c.runtimeOfflineRegionUpdateMetadataStart(
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
          _c.runtimeOfflineRegionGetStatusStart(
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
      _c.runtimeOfflineRegionSetObservedStart(
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
      _c.runtimeOfflineRegionSetDownloadStateStart(
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
          _c.runtimeOfflineRegionInvalidateStart(
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
          _c.runtimeOfflineRegionDeleteStart(
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

  /// Explicitly destroys this runtime.
  void close() {
    _state.close(_c.runtimeDestroy, _c.threadLastErrorMessage);
    _resourceTransformState?.close();
    _resourceTransformState = null;
    _resourceProviderRulesState?.close();
    _resourceProviderRulesState = null;
    _resourceProviderCallbackState?.close();
    _resourceProviderCallbackState = null;
  }
}

final class _NativeResourceRewriteRules extends Struct {
  external Pointer<_NativeResourceRewriteRule> rules;

  @Size()
  external int count;
}

final class _NativeResourceRewriteRule extends Struct {
  @Uint32()
  external int kind;

  external Pointer<Char> url;

  external Pointer<Char> replacementUrl;
}

final class _ResourceTransformState {
  _ResourceTransformState(List<ResourceUrlRewriteRule> rules) {
    pointer = calloc<_NativeResourceRewriteRules>();
    pointer.ref.count = rules.length;
    pointer.ref.rules = rules.isEmpty
        ? nullptr.cast<_NativeResourceRewriteRule>()
        : calloc<_NativeResourceRewriteRule>(rules.length);
    for (var index = 0; index < rules.length; index += 1) {
      final rule = rules[index];
      pointer.ref.rules[index].kind = rule.kind?.rawValue ?? 0;
      pointer.ref.rules[index].url = rule.url.toNativeUtf8().cast<Char>();
      pointer.ref.rules[index].replacementUrl = rule.replacementUrl
          .toNativeUtf8()
          .cast<Char>();
    }
  }

  late final Pointer<_NativeResourceRewriteRules> pointer;

  void close() {
    final rules = pointer.ref.rules;
    for (var index = 0; index < pointer.ref.count; index += 1) {
      calloc.free(rules[index].url);
      calloc.free(rules[index].replacementUrl);
    }
    if (rules != nullptr) {
      calloc.free(rules);
    }
    calloc.free(pointer);
  }
}

final class _NativeResourceProviderRules extends Struct {
  external Pointer<_NativeResourceProviderRule> rules;

  @Size()
  external int count;
}

final class _NativeResourceProviderRule extends Struct {
  @Uint32()
  external int kind;

  external Pointer<Char> url;

  external raw.mln_resource_response response;
}

typedef _QueuedResourceRequestListenerFunction = Void Function(Pointer<Void>);

final class _NativeQueuedResourceProviderRoute extends Struct {
  @Uint32()
  external int kind;

  external Pointer<Char> url;
}

final class _NativeQueuedResourceProvider extends Struct {
  external Pointer<_NativeQueuedResourceProviderRoute> routes;

  @Size()
  external int routeCount;

  external Pointer<NativeFunction<_QueuedResourceRequestListenerFunction>>
  listener;
}

final class _NativeQueuedResourceRequest extends Struct {
  external Pointer<Void> owner;

  external Pointer<raw.mln_resource_request_handle> handle;

  external Pointer<Char> url;

  @Uint32()
  external int kind;

  @Uint32()
  external int loadingMethod;

  @Uint32()
  external int priority;

  @Uint32()
  external int usage;

  @Uint32()
  external int storagePolicy;

  @Bool()
  external bool hasRange;

  @Uint64()
  external int rangeStart;

  @Uint64()
  external int rangeEnd;

  @Bool()
  external bool hasPriorModified;

  @Int64()
  external int priorModifiedUnixMs;

  @Bool()
  external bool hasPriorExpires;

  @Int64()
  external int priorExpiresUnixMs;

  external Pointer<Char> priorEtag;

  external Pointer<Uint8> priorData;

  @Size()
  external int priorDataSize;
}

final class _ResourceProviderRulesState {
  _ResourceProviderRulesState(List<ResourceProviderRule> rules) {
    pointer = calloc<_NativeResourceProviderRules>();
    pointer.ref.count = rules.length;
    pointer.ref.rules = rules.isEmpty
        ? nullptr.cast<_NativeResourceProviderRule>()
        : calloc<_NativeResourceProviderRule>(rules.length);
    for (var index = 0; index < rules.length; index += 1) {
      final rule = rules[index];
      pointer.ref.rules[index].kind = rule.kind?.rawValue ?? 0;
      pointer.ref.rules[index].url = rule.url.toNativeUtf8().cast<Char>();
      pointer.ref.rules[index].response = _resourceResponseToNative(
        rule.response,
        calloc,
      );
    }
  }

  late final Pointer<_NativeResourceProviderRules> pointer;

  void close() {
    final rules = pointer.ref.rules;
    for (var index = 0; index < pointer.ref.count; index += 1) {
      calloc.free(rules[index].url);
      _freeNativeResourceResponse(rules[index].response, calloc);
    }
    if (rules != nullptr) {
      calloc.free(rules);
    }
    calloc.free(pointer);
  }
}

final class _ResourceProviderCallbackState {
  _ResourceProviderCallbackState(ResourceProvider provider) {
    callback = NativeCallable<_QueuedResourceRequestListenerFunction>.listener((
      Pointer<Void> request,
    ) {
      _invokeQueuedResourceProvider(provider.callback, request);
    });
    pointer = calloc<_NativeQueuedResourceProvider>();
    pointer.ref.routeCount = provider.routes.length;
    pointer.ref.routes = provider.routes.isEmpty
        ? nullptr.cast<_NativeQueuedResourceProviderRoute>()
        : calloc<_NativeQueuedResourceProviderRoute>(provider.routes.length);
    for (var index = 0; index < provider.routes.length; index += 1) {
      final route = provider.routes[index];
      pointer.ref.routes[index].kind = route.kind?.rawValue ?? 0;
      pointer.ref.routes[index].url = route.url.toNativeUtf8().cast<Char>();
    }
    pointer.ref.listener = callback.nativeFunction;
  }

  late final Pointer<_NativeQueuedResourceProvider> pointer;
  late final NativeCallable<_QueuedResourceRequestListenerFunction> callback;

  void close() {
    final routes = pointer.ref.routes;
    for (var index = 0; index < pointer.ref.routeCount; index += 1) {
      calloc.free(routes[index].url);
    }
    if (routes != nullptr) {
      calloc.free(routes);
    }
    calloc.free(pointer);
    callback.close();
  }
}

void _invokeQueuedResourceProvider(
  ResourceProviderCallback callback,
  Pointer<Void> rawRequest,
) {
  try {
    final request = rawRequest.cast<_NativeQueuedResourceRequest>().ref;
    final handle = ResourceRequestHandle._(request.handle);
    try {
      callback(_copyResourceRequest(request), handle);
    } catch (_) {
      if (!handle.isReleased) {
        try {
          handle.complete(
            const ResourceResponse(
              status: ResourceResponseStatus.error,
              errorReason: ResourceErrorReason.other,
              errorMessage: 'Dart resource provider callback threw',
            ),
          );
        } catch (_) {
          handle.close();
        }
      }
    }
  } finally {
    _c.dartResourceProviderRequestDestroy(rawRequest);
  }
}

ResourceRequest _copyResourceRequest(_NativeQueuedResourceRequest request) {
  final priorData = request.priorData == nullptr || request.priorDataSize == 0
      ? null
      : Uint8List.fromList(
          request.priorData.asTypedList(request.priorDataSize),
        );
  return ResourceRequest(
    url: request.url.cast<Utf8>().toDartString(),
    kind: ResourceKind.fromRawValue(request.kind),
    loadingMethod: ResourceLoadingMethod.fromRawValue(request.loadingMethod),
    priority: ResourcePriority.fromRawValue(request.priority),
    usage: ResourceUsage.fromRawValue(request.usage),
    storagePolicy: ResourceStoragePolicy.fromRawValue(request.storagePolicy),
    range: request.hasRange
        ? (start: request.rangeStart, end: request.rangeEnd)
        : null,
    priorModifiedUnixMs: request.hasPriorModified
        ? request.priorModifiedUnixMs
        : null,
    priorExpiresUnixMs: request.hasPriorExpires
        ? request.priorExpiresUnixMs
        : null,
    priorEtag: request.priorEtag == nullptr
        ? null
        : request.priorEtag.cast<Utf8>().toDartString(),
    priorData: priorData,
  );
}

/// Runtime-owned offline operation handle.
final class OfflineOperationHandle {
  OfflineOperationHandle._(this._runtime, this.id);

  final RuntimeHandle _runtime;

  /// Native operation identifier copied into Dart.
  final int id;

  var _discarded = false;

  /// Whether this operation has been discarded by Dart.
  bool get isDiscarded => _discarded;

  /// Takes a completed offline region create result.
  OfflineRegionInfo takeCreatedRegion() {
    return withNativeArena((arena) {
      final outRegion = arena<Pointer<raw.mln_offline_region_snapshot>>();
      outRegion.value = nullptr;
      _check(
        _c.runtimeOfflineRegionCreateTakeResult(
          _runtime._pointer,
          id,
          outRegion,
        ),
      );
      _discarded = true;
      return _copyOfflineRegionSnapshot(outRegion.value);
    });
  }

  /// Takes a completed optional offline region get result.
  OfflineRegionInfo? takeOptionalRegion() {
    return withNativeArena((arena) {
      final outRegion = arena<Pointer<raw.mln_offline_region_snapshot>>();
      outRegion.value = nullptr;
      final outFound = arena<Bool>();
      _check(
        _c.runtimeOfflineRegionGetTakeResult(
          _runtime._pointer,
          id,
          outRegion,
          outFound,
        ),
      );
      _discarded = true;
      return outFound.value
          ? _copyOfflineRegionSnapshot(outRegion.value)
          : null;
    });
  }

  /// Takes a completed offline regions list result.
  List<OfflineRegionInfo> takeRegionList() {
    return withNativeArena((arena) {
      final outRegions = arena<Pointer<raw.mln_offline_region_list>>();
      outRegions.value = nullptr;
      _check(
        _c.runtimeOfflineRegionsListTakeResult(
          _runtime._pointer,
          id,
          outRegions,
        ),
      );
      _discarded = true;
      return _copyOfflineRegionList(outRegions.value);
    });
  }

  /// Takes a completed offline regions merge result.
  List<OfflineRegionInfo> takeMergedRegionList() {
    return withNativeArena((arena) {
      final outRegions = arena<Pointer<raw.mln_offline_region_list>>();
      outRegions.value = nullptr;
      _check(
        _c.runtimeOfflineRegionsMergeDatabaseTakeResult(
          _runtime._pointer,
          id,
          outRegions,
        ),
      );
      _discarded = true;
      return _copyOfflineRegionList(outRegions.value);
    });
  }

  /// Takes a completed offline region metadata update result.
  OfflineRegionInfo takeUpdatedRegionMetadata() {
    return withNativeArena((arena) {
      final outRegion = arena<Pointer<raw.mln_offline_region_snapshot>>();
      outRegion.value = nullptr;
      _check(
        _c.runtimeOfflineRegionUpdateMetadataTakeResult(
          _runtime._pointer,
          id,
          outRegion,
        ),
      );
      _discarded = true;
      return _copyOfflineRegionSnapshot(outRegion.value);
    });
  }

  /// Takes a completed offline region status result.
  OfflineRegionStatus takeRegionStatus() {
    return withNativeArena((arena) {
      final outStatus = arena<raw.mln_offline_region_status>();
      outStatus.ref.size = sizeOf<raw.mln_offline_region_status>();
      _check(
        _c.runtimeOfflineRegionGetStatusTakeResult(
          _runtime._pointer,
          id,
          outStatus,
        ),
      );
      _discarded = true;
      return _offlineRegionStatusFromNative(outStatus.ref);
    });
  }

  /// Discards runtime-owned state for this operation.
  void discard() {
    if (_discarded) {
      return;
    }
    _check(_c.runtimeOfflineOperationDiscard(_runtime._pointer, id));
    _discarded = true;
  }
}

raw.mln_offline_region_definition _offlineRegionDefinitionToNative(
  OfflineRegionDefinition definition,
  Allocator allocator,
) {
  final result = Struct.create<raw.mln_offline_region_definition>();
  result.size = sizeOf<raw.mln_offline_region_definition>();
  switch (definition) {
    case OfflineTilePyramidRegionDefinition():
      result.type = raw
          .mln_offline_region_definition_type
          .MLN_OFFLINE_REGION_DEFINITION_TILE_PYRAMID
          .value;
      result.data.tile_pyramid = _offlineTilePyramidDefinitionToNative(
        definition,
        allocator,
      );
    case OfflineGeometryRegionDefinition():
      result.type = raw
          .mln_offline_region_definition_type
          .MLN_OFFLINE_REGION_DEFINITION_GEOMETRY
          .value;
      result.data.geometry = _offlineGeometryDefinitionToNative(
        definition,
        allocator,
      );
  }
  return result;
}

raw.mln_offline_tile_pyramid_region_definition
_offlineTilePyramidDefinitionToNative(
  OfflineTilePyramidRegionDefinition definition,
  Allocator allocator,
) {
  final result =
      Struct.create<raw.mln_offline_tile_pyramid_region_definition>();
  result.size = sizeOf<raw.mln_offline_tile_pyramid_region_definition>();
  result.style_url = nativeUtf8CString(
    definition.styleUrl,
    allocator,
  ).pointer.cast<Char>();
  result.bounds = native_struct.latLngBoundsToNative(definition.bounds);
  result.min_zoom = definition.minZoom;
  result.max_zoom = definition.maxZoom;
  result.pixel_ratio = definition.pixelRatio;
  result.include_ideographs = definition.includeIdeographs;
  return result;
}

raw.mln_offline_geometry_region_definition _offlineGeometryDefinitionToNative(
  OfflineGeometryRegionDefinition definition,
  Allocator allocator,
) {
  final result = Struct.create<raw.mln_offline_geometry_region_definition>();
  result.size = sizeOf<raw.mln_offline_geometry_region_definition>();
  result.style_url = nativeUtf8CString(
    definition.styleUrl,
    allocator,
  ).pointer.cast<Char>();
  result.geometry = native_geometry
      .nativeGeometry(definition.geometry, allocator)
      .pointer;
  result.min_zoom = definition.minZoom;
  result.max_zoom = definition.maxZoom;
  result.pixel_ratio = definition.pixelRatio;
  result.include_ideographs = definition.includeIdeographs;
  return result;
}

Pointer<Uint8> _nativeBytes(Uint8List? bytes, Allocator allocator) {
  if (bytes == null || bytes.isEmpty) {
    return nullptr.cast<Uint8>();
  }
  final nativeBytes = allocator<Uint8>(bytes.length);
  for (var index = 0; index < bytes.length; index += 1) {
    nativeBytes[index] = bytes[index];
  }
  return nativeBytes;
}

OfflineRegionInfo _copyOfflineRegionSnapshot(
  Pointer<raw.mln_offline_region_snapshot> snapshot,
) {
  try {
    return withNativeArena((arena) {
      final outInfo = arena<raw.mln_offline_region_info>();
      outInfo.ref.size = sizeOf<raw.mln_offline_region_info>();
      _check(_c.offlineRegionSnapshotGet(snapshot, outInfo));
      return _offlineRegionInfoFromNative(outInfo.ref);
    });
  } finally {
    _c.offlineRegionSnapshotDestroy(snapshot);
  }
}

List<OfflineRegionInfo> _copyOfflineRegionList(
  Pointer<raw.mln_offline_region_list> list,
) {
  try {
    return withNativeArena((arena) {
      final outCount = arena<Size>();
      _check(_c.offlineRegionListCount(list, outCount));
      return [
        for (var index = 0; index < outCount.value; index += 1)
          _copyOfflineRegionListEntry(list, index, arena),
      ];
    });
  } finally {
    _c.offlineRegionListDestroy(list);
  }
}

OfflineRegionInfo _copyOfflineRegionListEntry(
  Pointer<raw.mln_offline_region_list> list,
  int index,
  Allocator allocator,
) {
  final outInfo = allocator<raw.mln_offline_region_info>();
  outInfo.ref.size = sizeOf<raw.mln_offline_region_info>();
  _check(_c.offlineRegionListGet(list, index, outInfo));
  return _offlineRegionInfoFromNative(outInfo.ref);
}

OfflineRegionInfo _offlineRegionInfoFromNative(
  raw.mln_offline_region_info info,
) {
  return OfflineRegionInfo(
    id: info.id,
    definition: _offlineRegionDefinitionFromNative(info.definition),
    metadata: info.metadata == nullptr || info.metadata_size == 0
        ? Uint8List(0)
        : Uint8List.fromList(info.metadata.asTypedList(info.metadata_size)),
  );
}

OfflineRegionDefinition _offlineRegionDefinitionFromNative(
  raw.mln_offline_region_definition definition,
) {
  if (definition.type ==
      raw
          .mln_offline_region_definition_type
          .MLN_OFFLINE_REGION_DEFINITION_TILE_PYRAMID
          .value) {
    final tilePyramid = definition.data.tile_pyramid;
    return OfflineTilePyramidRegionDefinition(
      styleUrl: tilePyramid.style_url.cast<Utf8>().toDartString(),
      bounds: native_struct.latLngBoundsFromNative(tilePyramid.bounds),
      minZoom: tilePyramid.min_zoom,
      maxZoom: tilePyramid.max_zoom,
      pixelRatio: tilePyramid.pixel_ratio,
      includeIdeographs: tilePyramid.include_ideographs,
    );
  }
  final geometry = definition.data.geometry;
  return OfflineGeometryRegionDefinition(
    styleUrl: geometry.style_url.cast<Utf8>().toDartString(),
    geometry: native_geometry.geometryFromNative(geometry.geometry.ref),
    minZoom: geometry.min_zoom,
    maxZoom: geometry.max_zoom,
    pixelRatio: geometry.pixel_ratio,
    includeIdeographs: geometry.include_ideographs,
  );
}

OfflineRegionStatus _offlineRegionStatusFromNative(
  raw.mln_offline_region_status status,
) {
  return OfflineRegionStatus(
    downloadState: OfflineRegionDownloadState.fromRawValue(
      status.download_state,
    ),
    completedResourceCount: status.completed_resource_count,
    completedResourceSize: status.completed_resource_size,
    completedTileCount: status.completed_tile_count,
    requiredTileCount: status.required_tile_count,
    completedTileSize: status.completed_tile_size,
    requiredResourceCount: status.required_resource_count,
    requiredResourceCountIsPrecise: status.required_resource_count_is_precise,
    complete: status.complete,
  );
}

/// Copied runtime event returned by [RuntimeHandle.pollEvent].
final class RuntimeEvent {
  RuntimeEvent._({
    required this.type,
    required this.sourceType,
    required this.code,
    required this.payloadType,
    required this.payloadSize,
    required this.message,
  });

  factory RuntimeEvent._fromNative(raw.mln_runtime_event event) {
    return RuntimeEvent._(
      type: event.type,
      sourceType: event.source_type,
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
      nativeOptions.ref = _c.mapOptionsDefault();
      nativeOptions.ref.width = options.width;
      nativeOptions.ref.height = options.height;
      nativeOptions.ref.scale_factor = options.scaleFactor;
      nativeOptions.ref.map_mode = options.mapMode.rawValue;
      final outMap = arena<Pointer<raw.mln_map>>();
      outMap.value = nullptr;

      _check(_c.mapCreate(runtime._pointer, nativeOptions, outMap));
      return MapHandle._(runtime, outMap.value);
    });
  }

  final RuntimeHandle _runtime;
  final NativeHandleState<raw.mln_map> _state;
  final _customGeometryCallbacks = <String, _CustomGeometryCallbackState>{};

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
      _check(_c.mapSetStyleUrl(_pointer, nativeUrl.pointer.cast<Char>()));
    });
    _clearCustomGeometryCallbacks();
  }

  /// Loads inline style JSON through MapLibre Native style APIs.
  void setStyleJson(String json) {
    withNativeArena((arena) {
      final nativeJson = nativeUtf8CString(json, arena);
      _check(_c.mapSetStyleJson(_pointer, nativeJson.pointer.cast<Char>()));
    });
    _clearCustomGeometryCallbacks();
  }

  /// Requests a repaint for a continuous map.
  void requestRepaint() {
    _check(_c.mapRequestRepaint(_pointer));
  }

  /// Requests one still image for a static or tile map.
  void requestStillImage() {
    _check(_c.mapRequestStillImage(_pointer));
  }

  /// Applies MapLibre debug overlay options.
  void setDebugOptions(MapDebugOptions options) {
    _check(_c.mapSetDebugOptions(_pointer, options.bits));
  }

  /// Copies current MapLibre debug overlay options.
  MapDebugOptions debugOptions() {
    return withNativeArena((arena) {
      final outOptions = arena<Uint32>();
      _check(_c.mapGetDebugOptions(_pointer, outOptions));
      return MapDebugOptions(outOptions.value);
    });
  }

  /// Dumps map debug logs through MapLibre Native logging.
  void dumpDebugLogs() {
    _check(_c.mapDumpDebugLogs(_pointer));
  }

  /// Attaches a Metal native surface render target.
  RenderSessionHandle attachMetalSurface(MetalSurfaceDescriptor descriptor) {
    return withNativeArena((arena) {
      final nativeDescriptor = arena<raw.mln_metal_surface_descriptor>();
      nativeDescriptor.ref = _metalSurfaceDescriptorToNative(descriptor);
      final outSession = arena<Pointer<raw.mln_render_session>>();
      outSession.value = nullptr;
      _check(_c.metalSurfaceAttach(_pointer, nativeDescriptor, outSession));
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
      _check(_c.vulkanSurfaceAttach(_pointer, nativeDescriptor, outSession));
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
        _c.metalOwnedTextureAttach(_pointer, nativeDescriptor, outSession),
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
        _c.metalBorrowedTextureAttach(_pointer, nativeDescriptor, outSession),
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
        _c.vulkanOwnedTextureAttach(_pointer, nativeDescriptor, outSession),
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
        _c.vulkanBorrowedTextureAttach(_pointer, nativeDescriptor, outSession),
      );
      return RenderSessionHandle._(this, outSession.value);
    });
  }

  /// Copies the current camera snapshot.
  CameraOptions camera() {
    return withNativeArena((arena) {
      final outCamera = arena<raw.mln_camera_options>();
      outCamera.ref.size = sizeOf<raw.mln_camera_options>();
      _check(_c.mapGetCamera(_pointer, outCamera));
      return native_struct.cameraOptionsFromNative(outCamera.ref);
    });
  }

  /// Applies a camera jump command.
  void jumpTo(CameraOptions camera) {
    withNativeArena((arena) {
      final nativeCamera = _nativeCamera(camera, arena);
      _check(_c.mapJumpTo(_pointer, nativeCamera));
    });
  }

  /// Applies a camera ease transition command.
  void easeTo(CameraOptions camera, {AnimationOptions? animation}) {
    withNativeArena((arena) {
      final nativeCamera = _nativeCamera(camera, arena);
      final nativeAnimation = _nativeAnimation(animation, arena);
      _check(_c.mapEaseTo(_pointer, nativeCamera, nativeAnimation));
    });
  }

  /// Applies a camera fly transition command.
  void flyTo(CameraOptions camera, {AnimationOptions? animation}) {
    withNativeArena((arena) {
      final nativeCamera = _nativeCamera(camera, arena);
      final nativeAnimation = _nativeAnimation(animation, arena);
      _check(_c.mapFlyTo(_pointer, nativeCamera, nativeAnimation));
    });
  }

  /// Applies a screen-space pan command.
  void moveBy(double deltaX, double deltaY, {AnimationOptions? animation}) {
    withNativeArena((arena) {
      final nativeAnimation = _nativeAnimation(animation, arena);
      _check(
        animation == null
            ? _c.mapMoveBy(_pointer, deltaX, deltaY)
            : _c.mapMoveByAnimated(_pointer, deltaX, deltaY, nativeAnimation),
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
            ? _c.mapScaleBy(_pointer, scale, nativeAnchor)
            : _c.mapScaleByAnimated(
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
            ? _c.mapRotateBy(_pointer, nativeFirst, nativeSecond)
            : _c.mapRotateByAnimated(
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
            ? _c.mapPitchBy(_pointer, pitch)
            : _c.mapPitchByAnimated(_pointer, pitch, nativeAnimation),
      );
    });
  }

  /// Cancels active camera transitions.
  void cancelTransitions() {
    _check(_c.mapCancelTransitions(_pointer));
  }

  /// Enables or disables the rendering stats overlay.
  void setRenderingStatsViewEnabled(bool enabled) {
    _check(_c.mapSetRenderingStatsViewEnabled(_pointer, enabled));
  }

  /// Copies whether the rendering stats overlay is enabled.
  bool renderingStatsViewEnabled() {
    return withNativeArena((arena) {
      final outEnabled = arena<Bool>();
      _check(_c.mapGetRenderingStatsViewEnabled(_pointer, outEnabled));
      return outEnabled.value;
    });
  }

  /// Copies whether MapLibre currently considers the map fully loaded.
  bool isFullyLoaded() {
    return withNativeArena((arena) {
      final outLoaded = arena<Bool>();
      _check(_c.mapIsFullyLoaded(_pointer, outLoaded));
      return outLoaded.value;
    });
  }

  /// Copies live map viewport and render-transform controls.
  MapViewportOptions viewportOptions() {
    return withNativeArena((arena) {
      final outOptions = arena<raw.mln_map_viewport_options>();
      outOptions.ref.size = sizeOf<raw.mln_map_viewport_options>();
      _check(_c.mapGetViewportOptions(_pointer, outOptions));
      return native_struct.mapViewportOptionsFromNative(outOptions.ref);
    });
  }

  /// Applies selected live map viewport and render-transform controls.
  void setViewportOptions(MapViewportOptions options) {
    withNativeArena((arena) {
      final nativeOptions = arena<raw.mln_map_viewport_options>();
      nativeOptions.ref = native_struct.mapViewportOptionsToNative(options);
      _check(_c.mapSetViewportOptions(_pointer, nativeOptions));
    });
  }

  /// Copies tile prefetch and LOD tuning controls.
  MapTileOptions tileOptions() {
    return withNativeArena((arena) {
      final outOptions = arena<raw.mln_map_tile_options>();
      outOptions.ref.size = sizeOf<raw.mln_map_tile_options>();
      _check(_c.mapGetTileOptions(_pointer, outOptions));
      return native_struct.mapTileOptionsFromNative(outOptions.ref);
    });
  }

  /// Applies selected tile prefetch and LOD tuning controls.
  void setTileOptions(MapTileOptions options) {
    withNativeArena((arena) {
      final nativeOptions = arena<raw.mln_map_tile_options>();
      nativeOptions.ref = native_struct.mapTileOptionsToNative(options);
      _check(_c.mapSetTileOptions(_pointer, nativeOptions));
    });
  }

  /// Copies map camera constraint options.
  BoundOptions bounds() {
    return withNativeArena((arena) {
      final outOptions = arena<raw.mln_bound_options>();
      outOptions.ref.size = sizeOf<raw.mln_bound_options>();
      _check(_c.mapGetBounds(_pointer, outOptions));
      return native_struct.boundOptionsFromNative(outOptions.ref);
    });
  }

  /// Applies selected map camera constraint options.
  void setBounds(BoundOptions options) {
    withNativeArena((arena) {
      final nativeOptions = arena<raw.mln_bound_options>();
      nativeOptions.ref = native_struct.boundOptionsToNative(options);
      _check(_c.mapSetBounds(_pointer, nativeOptions));
    });
  }

  /// Copies the current free camera position and orientation.
  FreeCameraOptions freeCameraOptions() {
    return withNativeArena((arena) {
      final outOptions = arena<raw.mln_free_camera_options>();
      outOptions.ref.size = sizeOf<raw.mln_free_camera_options>();
      _check(_c.mapGetFreeCameraOptions(_pointer, outOptions));
      return native_struct.freeCameraOptionsFromNative(outOptions.ref);
    });
  }

  /// Applies selected free camera position and orientation fields.
  void setFreeCameraOptions(FreeCameraOptions options) {
    withNativeArena((arena) {
      final nativeOptions = arena<raw.mln_free_camera_options>();
      nativeOptions.ref = native_struct.freeCameraOptionsToNative(options);
      _check(_c.mapSetFreeCameraOptions(_pointer, nativeOptions));
    });
  }

  /// Copies the current axonometric rendering options.
  ProjectionModeOptions projectionMode() {
    return withNativeArena((arena) {
      final outMode = arena<raw.mln_projection_mode>();
      outMode.ref.size = sizeOf<raw.mln_projection_mode>();
      _check(_c.mapGetProjectionMode(_pointer, outMode));
      return native_struct.projectionModeOptionsFromNative(outMode.ref);
    });
  }

  /// Applies selected axonometric rendering option fields.
  void setProjectionMode(ProjectionModeOptions mode) {
    withNativeArena((arena) {
      final nativeMode = arena<raw.mln_projection_mode>();
      nativeMode.ref = native_struct.projectionModeOptionsToNative(mode);
      _check(_c.mapSetProjectionMode(_pointer, nativeMode));
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
        _c.mapCameraForLatLngBounds(
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
        _c.mapCameraForLatLngs(
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
        _c.mapCameraForGeometry(
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
            ? _c.mapLatLngBoundsForCameraUnwrapped(
                _pointer,
                nativeCamera,
                outBounds,
              )
            : _c.mapLatLngBoundsForCamera(_pointer, nativeCamera, outBounds),
      );
      return native_struct.latLngBoundsFromNative(outBounds.ref);
    });
  }

  /// Converts a geographic world coordinate to a screen point.
  ScreenPoint pixelForLatLng(LatLng coordinate) {
    return withNativeArena((arena) {
      final outPoint = arena<raw.mln_screen_point>();
      _check(
        _c.mapPixelForLatLng(
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
        _c.mapLatLngForPixel(
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
        _c.mapPixelsForLatLngs(
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
        _c.mapLatLngsForPixels(
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
      _check(_c.mapProjectionCreate(_pointer, outProjection));
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
        _c.mapSetStyleImage(
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
      _check(_c.mapRemoveStyleImage(_pointer, nativeId.value, outRemoved));
      return outRemoved.value;
    });
  }

  /// Reports whether one runtime style image exists.
  bool styleImageExists(String imageId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(imageId, arena);
      final outExists = arena<Bool>();
      _check(_c.mapStyleImageExists(_pointer, nativeId.value, outExists));
      return outExists.value;
    });
  }

  /// Copies fixed metadata for one runtime style image.
  StyleImageInfo? getStyleImageInfo(String imageId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(imageId, arena);
      final outInfo = arena<raw.mln_style_image_info>();
      outInfo.ref.size = sizeOf<raw.mln_style_image_info>();
      final outFound = arena<Bool>();
      _check(
        _c.mapGetStyleImageInfo(_pointer, nativeId.value, outInfo, outFound),
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
        _c.mapCopyStyleImagePremultipliedRgba8(
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
        _c.mapAddStyleSourceJson(
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
        _c.mapAddGeoJsonSourceUrl(_pointer, nativeId.value, nativeUrl.value),
      );
    });
  }

  /// Adds a GeoJSON source with inline data.
  void addGeoJsonSourceData(String sourceId, GeoJson data) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeData = native_geometry.nativeGeoJson(data, arena);
      _check(
        _c.mapAddGeoJsonSourceData(
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
        _c.mapSetGeoJsonSourceUrl(_pointer, nativeId.value, nativeUrl.value),
      );
    });
  }

  /// Updates one GeoJSON source with inline data.
  void setGeoJsonSourceData(String sourceId, GeoJson data) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeData = native_geometry.nativeGeoJson(data, arena);
      _check(
        _c.mapSetGeoJsonSourceData(
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
        _c.mapAddVectorSourceUrl(
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
        _c.mapAddVectorSourceTiles(
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
        _c.mapAddRasterSourceUrl(
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
        _c.mapAddRasterSourceTiles(
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
        _c.mapAddRasterDemSourceUrl(
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
        _c.mapAddRasterDemSourceTiles(
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
        _c.mapAddImageSourceUrl(
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
        _c.mapAddImageSourceImage(
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
        _c.mapSetImageSourceUrl(_pointer, nativeId.value, nativeUrl.value),
      );
    });
  }

  /// Updates an image source with inline image pixels.
  void setImageSourceImage(String sourceId, PremultipliedRgba8Image image) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final nativeImage = arena<raw.mln_premultiplied_rgba8_image>();
      nativeImage.ref = _premultipliedRgba8ImageToNative(image, arena);
      _check(_c.mapSetImageSourceImage(_pointer, nativeId.value, nativeImage));
    });
  }

  /// Updates image source coordinates.
  void setImageSourceCoordinates(String sourceId, List<LatLng> coordinates) {
    withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      _check(
        _c.mapSetImageSourceCoordinates(
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
        _c.mapGetImageSourceCoordinates(
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
          _c.mapAddCustomGeometrySource(
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
        _c.mapSetCustomGeometrySourceTileData(
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
        _c.mapInvalidateCustomGeometrySourceTile(
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
        _c.mapInvalidateCustomGeometrySourceRegion(
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
      _check(_c.mapStyleSourceExists(_pointer, nativeId.value, outExists));
      return outExists.value;
    });
  }

  /// Removes one style source by ID and returns whether one was removed.
  bool removeStyleSource(String sourceId) {
    final removed = withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final outRemoved = arena<Bool>();
      _check(_c.mapRemoveStyleSource(_pointer, nativeId.value, outRemoved));
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
        _c.mapGetStyleSourceInfo(_pointer, nativeId.value, outInfo, outFound),
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
      _check(_c.mapListStyleSourceIds(_pointer, outList));
      return _copyStyleIdList(outList.value);
    });
  }

  /// Adds one style layer from a full style-spec layer JSON object.
  void addStyleLayerJson(JsonValue layerJson, {String? beforeLayerId}) {
    withNativeArena((arena) {
      final nativeLayerJson = native_json.nativeJsonValue(layerJson, arena);
      final nativeBeforeLayerId = nativeStringView(beforeLayerId ?? '', arena);
      _check(
        _c.mapAddStyleLayerJson(
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
        _c.mapGetStyleLayerJson(_pointer, nativeId.value, outLayer, outFound),
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
      _check(_c.mapSetStyleLightJson(_pointer, nativeLightJson.pointer));
    });
  }

  /// Sets one style light property by style-spec property name.
  void setStyleLightProperty(String propertyName, JsonValue value) {
    withNativeArena((arena) {
      final nativePropertyName = nativeStringView(propertyName, arena);
      final nativeValue = native_json.nativeJsonValue(value, arena);
      _check(
        _c.mapSetStyleLightProperty(
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
        _c.mapGetStyleLightProperty(
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
        _c.mapSetLayerProperty(
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
        _c.mapGetLayerProperty(
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
      _check(_c.mapSetLayerFilter(_pointer, nativeLayerId.value, nativeFilter));
    });
  }

  /// Copies one layer filter, or null when the layer has no filter.
  JsonValue? getLayerFilter(String layerId) {
    return withNativeArena((arena) {
      final nativeLayerId = nativeStringView(layerId, arena);
      final outFilter = arena<Pointer<raw.mln_json_snapshot>>();
      outFilter.value = nullptr;
      _check(_c.mapGetLayerFilter(_pointer, nativeLayerId.value, outFilter));
      return _copyJsonSnapshot(outFilter.value);
    });
  }

  /// Reports whether a style layer ID exists.
  bool styleLayerExists(String layerId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(layerId, arena);
      final outExists = arena<Bool>();
      _check(_c.mapStyleLayerExists(_pointer, nativeId.value, outExists));
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
        _c.mapGetStyleLayerType(
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

  /// Removes one style layer by ID and returns whether one was removed.
  bool removeStyleLayer(String layerId) {
    return withNativeArena((arena) {
      final nativeId = nativeStringView(layerId, arena);
      final outRemoved = arena<Bool>();
      _check(_c.mapRemoveStyleLayer(_pointer, nativeId.value, outRemoved));
      return outRemoved.value;
    });
  }

  /// Copies style layer IDs in style order.
  List<String> listStyleLayerIds() {
    return withNativeArena((arena) {
      final outList = arena<Pointer<raw.mln_style_id_list>>();
      outList.value = nullptr;
      _check(_c.mapListStyleLayerIds(_pointer, outList));
      return _copyStyleIdList(outList.value);
    });
  }

  /// Explicitly destroys this map.
  void close() {
    _state.close(_c.mapDestroy, _c.threadLastErrorMessage);
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
final class MapProjectionHandle {
  MapProjectionHandle._(Pointer<raw.mln_map_projection> pointer)
    : _state = NativeHandleState(pointer, 'MapProjectionHandle');

  final NativeHandleState<raw.mln_map_projection> _state;

  /// Whether this projection helper has been closed by the Dart binding.
  bool get isClosed => _state.isClosed;

  Pointer<raw.mln_map_projection> get _pointer => _state.pointer;

  /// Copies the current projection camera options.
  CameraOptions camera() {
    return withNativeArena((arena) {
      final outCamera = arena<raw.mln_camera_options>();
      outCamera.ref.size = sizeOf<raw.mln_camera_options>();
      _check(_c.mapProjectionGetCamera(_pointer, outCamera));
      return native_struct.cameraOptionsFromNative(outCamera.ref);
    });
  }

  /// Applies camera fields to the projection helper.
  void setCamera(CameraOptions camera) {
    withNativeArena((arena) {
      final nativeCamera = arena<raw.mln_camera_options>();
      nativeCamera.ref = native_struct.cameraOptionsToNative(camera);
      _check(_c.mapProjectionSetCamera(_pointer, nativeCamera));
    });
  }

  /// Updates the camera so coordinates are visible within [padding].
  void setVisibleCoordinates(
    List<LatLng> coordinates, {
    EdgeInsets padding = const EdgeInsets(),
  }) {
    withNativeArena((arena) {
      final nativeCoordinates = coordinates.isEmpty
          ? nullptr.cast<raw.mln_lat_lng>()
          : arena<raw.mln_lat_lng>(coordinates.length);
      for (var index = 0; index < coordinates.length; index += 1) {
        nativeCoordinates[index] = native_struct.latLngToNative(
          coordinates[index],
        );
      }
      _check(
        _c.mapProjectionSetVisibleCoordinates(
          _pointer,
          nativeCoordinates,
          coordinates.length,
          native_struct.edgeInsetsToNative(padding),
        ),
      );
    });
  }

  /// Updates the camera so geometry coordinates are visible within [padding].
  void setVisibleGeometry(
    Geometry geometry, {
    EdgeInsets padding = const EdgeInsets(),
  }) {
    withNativeArena((arena) {
      final nativeGeometry = native_geometry.nativeGeometry(geometry, arena);
      _check(
        _c.mapProjectionSetVisibleGeometry(
          _pointer,
          nativeGeometry.pointer,
          native_struct.edgeInsetsToNative(padding),
        ),
      );
    });
  }

  /// Converts a geographic world coordinate to a screen point.
  ScreenPoint pixelForLatLng(LatLng coordinate) {
    return withNativeArena((arena) {
      final outPoint = arena<raw.mln_screen_point>();
      _check(
        _c.mapProjectionPixelForLatLng(
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
        _c.mapProjectionLatLngForPixel(
          _pointer,
          native_struct.screenPointToNative(point),
          outCoordinate,
        ),
      );
      return native_struct.latLngFromNative(outCoordinate.ref);
    });
  }

  /// Explicitly destroys this projection helper.
  void close() {
    _state.close(_c.mapProjectionDestroy, _c.threadLastErrorMessage);
  }
}

/// Owner-thread render session handle attached to a retained map.
final class RenderSessionHandle {
  RenderSessionHandle._(this._map, Pointer<raw.mln_render_session> pointer)
    : _state = NativeHandleState(pointer, 'RenderSessionHandle');

  final MapHandle _map;
  final NativeHandleState<raw.mln_render_session> _state;

  /// Whether this render session has been closed by the Dart binding.
  bool get isClosed => _state.isClosed;

  Pointer<raw.mln_render_session> get _pointer {
    final _ = _map._pointer;
    return _state.pointer;
  }

  /// Resizes an attached render session.
  void resize(int width, int height, {double scaleFactor = 1}) {
    _check(_c.renderSessionResize(_pointer, width, height, scaleFactor));
  }

  /// Processes the latest map render update for this session.
  void renderUpdate() {
    _check(_c.renderSessionRenderUpdate(_pointer));
  }

  /// Detaches backend-bound render resources while keeping the handle live.
  void detach() {
    _check(_c.renderSessionDetach(_pointer));
  }

  /// Asks the session renderer to release cached resources where possible.
  void reduceMemoryUse() {
    _check(_c.renderSessionReduceMemoryUse(_pointer));
  }

  /// Clears renderer data for the session.
  void clearData() {
    _check(_c.renderSessionClearData(_pointer));
  }

  /// Dumps renderer debug logs through MapLibre Native logging.
  void dumpDebugLogs() {
    _check(_c.renderSessionDumpDebugLogs(_pointer));
  }

  /// Sets per-feature state on a render source.
  void setFeatureState(FeatureStateSelector selector, JsonObject state) {
    withNativeArena((arena) {
      final nativeSelector = _featureStateSelectorToNative(selector, arena);
      final nativeState = native_json.nativeJsonValue(state, arena);
      _check(
        _c.renderSessionSetFeatureState(
          _pointer,
          nativeSelector,
          nativeState.pointer,
        ),
      );
    });
  }

  /// Copies per-feature state from a render source.
  JsonValue? getFeatureState(FeatureStateSelector selector) {
    return withNativeArena((arena) {
      final nativeSelector = _featureStateSelectorToNative(selector, arena);
      final outState = arena<Pointer<raw.mln_json_snapshot>>();
      outState.value = nullptr;
      _check(
        _c.renderSessionGetFeatureState(_pointer, nativeSelector, outState),
      );
      return _copyJsonSnapshot(outState.value);
    });
  }

  /// Removes per-feature state from a render source.
  void removeFeatureState(FeatureStateSelector selector) {
    withNativeArena((arena) {
      final nativeSelector = _featureStateSelectorToNative(selector, arena);
      _check(_c.renderSessionRemoveFeatureState(_pointer, nativeSelector));
    });
  }

  /// Queries rendered features from the latest render session state.
  List<QueriedFeature> queryRenderedFeatures(
    RenderedQueryGeometry geometry, {
    RenderedFeatureQueryOptions options = const RenderedFeatureQueryOptions(),
  }) {
    return withNativeArena((arena) {
      final nativeGeometry = arena<raw.mln_rendered_query_geometry>();
      nativeGeometry.ref = _renderedQueryGeometryToNative(geometry, arena);
      final nativeOptions = _renderedFeatureQueryOptionsToNative(
        options,
        arena,
      );
      final outResult = arena<Pointer<raw.mln_feature_query_result>>();
      outResult.value = nullptr;
      _check(
        _c.renderSessionQueryRenderedFeatures(
          _pointer,
          nativeGeometry,
          nativeOptions,
          outResult,
        ),
      );
      return _copyFeatureQueryResult(outResult.value);
    });
  }

  /// Queries source features from the latest render session state.
  List<QueriedFeature> querySourceFeatures(
    String sourceId, {
    SourceFeatureQueryOptions options = const SourceFeatureQueryOptions(),
  }) {
    return withNativeArena((arena) {
      final nativeSourceId = nativeStringView(sourceId, arena);
      final nativeOptions = _sourceFeatureQueryOptionsToNative(options, arena);
      final outResult = arena<Pointer<raw.mln_feature_query_result>>();
      outResult.value = nullptr;
      _check(
        _c.renderSessionQuerySourceFeatures(
          _pointer,
          nativeSourceId.value,
          nativeOptions,
          outResult,
        ),
      );
      return _copyFeatureQueryResult(outResult.value);
    });
  }

  /// Queries a feature extension from the latest render session state.
  FeatureExtensionResult queryFeatureExtensions({
    required String sourceId,
    required FeatureGeoJson feature,
    required String extension,
    required String extensionField,
    JsonValue? arguments,
  }) {
    return withNativeArena((arena) {
      final nativeSourceId = nativeStringView(sourceId, arena);
      final nativeFeature = native_geometry
          .nativeGeoJson(feature, arena)
          .pointer
          .ref
          .data
          .feature;
      final nativeExtension = nativeStringView(extension, arena);
      final nativeExtensionField = nativeStringView(extensionField, arena);
      final nativeArguments = arguments == null
          ? nullptr.cast<raw.mln_json_value>()
          : native_json.nativeJsonValue(arguments, arena).pointer;
      final outResult = arena<Pointer<raw.mln_feature_extension_result>>();
      outResult.value = nullptr;
      _check(
        _c.renderSessionQueryFeatureExtensions(
          _pointer,
          nativeSourceId.value,
          nativeFeature,
          nativeExtension.value,
          nativeExtensionField.value,
          nativeArguments,
          outResult,
        ),
      );
      return _copyFeatureExtensionResult(outResult.value);
    });
  }

  /// Reads the latest rendered session-owned texture as premultiplied RGBA8.
  TextureImage readPremultipliedRgba8() {
    return withNativeArena((arena) {
      final info = arena<raw.mln_texture_image_info>();
      info.ref.size = sizeOf<raw.mln_texture_image_info>();
      final probeStatus = _c.textureReadPremultipliedRgba8(
        _pointer,
        nullptr.cast<Uint8>(),
        0,
        info,
      );
      if (probeStatus != nativeStatusInvalidArgument ||
          info.ref.byte_length == 0) {
        _check(probeStatus);
      }

      final data = arena<Uint8>(info.ref.byte_length);
      _check(
        _c.textureReadPremultipliedRgba8(
          _pointer,
          data,
          info.ref.byte_length,
          info,
        ),
      );
      return TextureImage(
        info: TextureImageInfo._fromNative(info.ref),
        bytes: Uint8List.fromList(data.asTypedList(info.ref.byte_length)),
      );
    });
  }

  /// Acquires the latest Metal texture frame until [MetalOwnedTextureFrame.close].
  MetalOwnedTextureFrame acquireMetalTextureFrame() {
    return withNativeArena((arena) {
      final outFrame = arena<raw.mln_metal_owned_texture_frame>();
      outFrame.ref.size = sizeOf<raw.mln_metal_owned_texture_frame>();
      _check(_c.metalOwnedTextureAcquireFrame(_pointer, outFrame));
      return MetalOwnedTextureFrame._(this, outFrame.ref);
    });
  }

  /// Acquires the latest Vulkan texture frame until [VulkanOwnedTextureFrame.close].
  VulkanOwnedTextureFrame acquireVulkanTextureFrame() {
    return withNativeArena((arena) {
      final outFrame = arena<raw.mln_vulkan_owned_texture_frame>();
      outFrame.ref.size = sizeOf<raw.mln_vulkan_owned_texture_frame>();
      _check(_c.vulkanOwnedTextureAcquireFrame(_pointer, outFrame));
      return VulkanOwnedTextureFrame._(this, outFrame.ref);
    });
  }

  /// Explicitly destroys this render session.
  void close() {
    _state.close(_c.renderSessionDestroy, _c.threadLastErrorMessage);
  }
}

/// Releasable handle for a resource request owned by a Dart provider.
final class ResourceRequestHandle {
  ResourceRequestHandle._(this._pointer);

  Pointer<raw.mln_resource_request_handle> _pointer;
  var _released = false;

  /// Whether this provider reference has been released by Dart.
  bool get isReleased => _released;

  /// Reports whether MapLibre has cancelled this provider request.
  bool get isCancelled => cancelled();

  /// Reports whether MapLibre has cancelled this provider request.
  bool cancelled() {
    return withNativeArena((arena) {
      final outCancelled = arena<Bool>();
      _check(_c.resourceRequestCancelled(_livePointer, outCancelled));
      return outCancelled.value;
    });
  }

  /// Completes this request with [response] and releases it. Completion is one-shot.
  void complete(ResourceResponse response) {
    final pointer = _takePointer();
    try {
      withNativeArena((arena) {
        final nativeResponse = arena<raw.mln_resource_response>();
        nativeResponse.ref = _resourceResponseToNative(response, arena);
        _check(_c.resourceRequestComplete(pointer, nativeResponse));
      });
    } finally {
      _c.resourceRequestRelease(pointer);
    }
  }

  /// Releases the provider reference. The handle must not be used afterwards.
  void close() {
    if (_released) {
      return;
    }
    _c.resourceRequestRelease(_pointer);
    _pointer = nullptr;
    _released = true;
  }

  Pointer<raw.mln_resource_request_handle> _takePointer() {
    final pointer = _livePointer;
    _pointer = nullptr;
    _released = true;
    return pointer;
  }

  Pointer<raw.mln_resource_request_handle> get _livePointer {
    if (_released || _pointer == nullptr) {
      throwInvalidArgument('resource request handle has been released');
    }
    return _pointer;
  }
}

/// CPU image readback metadata for a texture session frame.
final class TextureImageInfo {
  const TextureImageInfo._({
    required this.width,
    required this.height,
    required this.stride,
    required this.byteLength,
  });

  factory TextureImageInfo._fromNative(raw.mln_texture_image_info value) =>
      TextureImageInfo._(
        width: value.width,
        height: value.height,
        stride: value.stride,
        byteLength: value.byte_length,
      );

  /// Physical image width in device pixels.
  final int width;

  /// Physical image height in device pixels.
  final int height;

  /// Bytes per image row.
  final int stride;

  /// Required output buffer byte length.
  final int byteLength;
}

/// Dart-owned premultiplied RGBA8 texture readback bytes.
final class TextureImage {
  const TextureImage({required this.info, required this.bytes});

  /// Image metadata.
  final TextureImageInfo info;

  /// Copied premultiplied RGBA8 bytes.
  final Uint8List bytes;
}

/// Scoped Metal texture frame borrowed from a session-owned texture target.
final class MetalOwnedTextureFrame {
  MetalOwnedTextureFrame._(this._session, this._frame);

  final RenderSessionHandle _session;
  final raw.mln_metal_owned_texture_frame _frame;
  var _closed = false;

  /// Physical texture width in device pixels.
  int get width => _frame.width;

  /// Physical texture height in device pixels.
  int get height => _frame.height;

  /// UI-to-device pixel scale used for this frame.
  double get scaleFactor => _frame.scale_factor;

  /// Backend-native Metal pixel format value.
  int get pixelFormat => _frame.pixel_format;

  /// Unsafe borrowed `id<MTLTexture>` / `MTL::Texture*` pointer.
  ///
  /// The pointer is valid only until [close] releases this frame.
  NativePointer get unsafeTexture => _borrowedPointer(_frame.texture);

  /// Unsafe borrowed `id<MTLDevice>` / `MTL::Device*` pointer.
  ///
  /// The pointer is valid only until [close] releases this frame.
  NativePointer get unsafeDevice => _borrowedPointer(_frame.device);

  /// Releases this frame. The unsafe backend pointers become invalid.
  void close() {
    if (_closed) {
      return;
    }
    withNativeArena((arena) {
      final nativeFrame = arena<raw.mln_metal_owned_texture_frame>();
      nativeFrame.ref = _frame;
      _check(_c.metalOwnedTextureReleaseFrame(_session._pointer, nativeFrame));
    });
    _closed = true;
  }

  NativePointer _borrowedPointer(Pointer<Void> pointer) {
    if (_closed) {
      throwInvalidArgument('Metal texture frame has already been released');
    }
    final _ = _session._pointer;
    return NativePointer(pointer.address);
  }
}

/// Scoped Vulkan texture frame borrowed from a session-owned texture target.
final class VulkanOwnedTextureFrame {
  VulkanOwnedTextureFrame._(this._session, this._frame);

  final RenderSessionHandle _session;
  final raw.mln_vulkan_owned_texture_frame _frame;
  var _closed = false;

  /// Physical image width in device pixels.
  int get width => _frame.width;

  /// Physical image height in device pixels.
  int get height => _frame.height;

  /// UI-to-device pixel scale used for this frame.
  double get scaleFactor => _frame.scale_factor;

  /// Backend-native Vulkan format value.
  int get format => _frame.format;

  /// Backend-native Vulkan image layout value.
  int get layout => _frame.layout;

  /// Unsafe borrowed VkImage pointer.
  ///
  /// The pointer is valid only until [close] releases this frame.
  NativePointer get unsafeImage => _borrowedPointer(_frame.image);

  /// Unsafe borrowed VkImageView pointer.
  ///
  /// The pointer is valid only until [close] releases this frame.
  NativePointer get unsafeImageView => _borrowedPointer(_frame.image_view);

  /// Unsafe borrowed VkDevice pointer.
  ///
  /// The pointer is valid only until [close] releases this frame.
  NativePointer get unsafeDevice => _borrowedPointer(_frame.device);

  /// Releases this frame. The unsafe backend pointers become invalid.
  void close() {
    if (_closed) {
      return;
    }
    withNativeArena((arena) {
      final nativeFrame = arena<raw.mln_vulkan_owned_texture_frame>();
      nativeFrame.ref = _frame;
      _check(_c.vulkanOwnedTextureReleaseFrame(_session._pointer, nativeFrame));
    });
    _closed = true;
  }

  NativePointer _borrowedPointer(Pointer<Void> pointer) {
    if (_closed) {
      throwInvalidArgument('Vulkan texture frame has already been released');
    }
    final _ = _session._pointer;
    return NativePointer(pointer.address);
  }
}

raw.mln_premultiplied_rgba8_image _premultipliedRgba8ImageToNative(
  PremultipliedRgba8Image image,
  Allocator allocator,
) {
  final result = _c.premultipliedRgba8ImageDefault();
  result.width = image.width;
  result.height = image.height;
  result.stride = image.stride;
  final bytes = image.bytes;
  result.byte_length = bytes.length;
  if (bytes.isNotEmpty) {
    final nativeBytes = allocator<Uint8>(bytes.length);
    for (var index = 0; index < bytes.length; index += 1) {
      nativeBytes[index] = bytes[index];
    }
    result.pixels = nativeBytes;
  }
  return result;
}

raw.mln_style_image_options _styleImageOptionsToNative(
  StyleImageOptions options,
) {
  final result = _c.styleImageOptionsDefault();
  final pixelRatio = options.pixelRatio;
  if (pixelRatio != null) {
    result.fields |= raw
        .mln_style_image_option_field
        .MLN_STYLE_IMAGE_OPTION_PIXEL_RATIO
        .value;
    result.pixel_ratio = pixelRatio;
  }
  final sdf = options.sdf;
  if (sdf != null) {
    result.fields |=
        raw.mln_style_image_option_field.MLN_STYLE_IMAGE_OPTION_SDF.value;
    result.sdf = sdf;
  }
  return result;
}

StyleImageInfo _styleImageInfoFromNative(raw.mln_style_image_info info) {
  return StyleImageInfo(
    width: info.width,
    height: info.height,
    stride: info.stride,
    byteLength: info.byte_length,
    pixelRatio: info.pixel_ratio,
    sdf: info.sdf,
  );
}

final class _CustomGeometryCallbackState {
  _CustomGeometryCallbackState(CustomGeometrySourceOptions options)
    : fetchTile =
          NativeCallable<
            raw.mln_custom_geometry_source_tile_callbackFunction
          >.listener((Pointer<Void> _, raw.mln_canonical_tile_id tileId) {
            _invokeTileCallback(options.fetchTile, tileId);
          }),
      cancelTile = options.cancelTile == null
          ? null
          : NativeCallable<
              raw.mln_custom_geometry_source_tile_callbackFunction
            >.listener((Pointer<Void> _, raw.mln_canonical_tile_id tileId) {
              _invokeTileCallback(options.cancelTile!, tileId);
            });

  final NativeCallable<raw.mln_custom_geometry_source_tile_callbackFunction>
  fetchTile;
  final NativeCallable<raw.mln_custom_geometry_source_tile_callbackFunction>?
  cancelTile;

  void close() {
    fetchTile.close();
    cancelTile?.close();
  }
}

void _invokeTileCallback(
  CustomGeometryTileCallback callback,
  raw.mln_canonical_tile_id tileId,
) {
  try {
    callback(CanonicalTileId(z: tileId.z, x: tileId.x, y: tileId.y));
  } catch (_) {
    // Listener callbacks are asynchronous notifications; exceptions are
    // contained so they never escape through native callback machinery.
  }
}

raw.mln_custom_geometry_source_options _customGeometrySourceOptionsToNative(
  CustomGeometrySourceOptions options,
  _CustomGeometryCallbackState callbackState,
) {
  final result = _c.customGeometrySourceOptionsDefault();
  result.fetch_tile = callbackState.fetchTile.nativeFunction;
  result.cancel_tile =
      callbackState.cancelTile?.nativeFunction ??
      nullptr
          .cast<
            NativeFunction<raw.mln_custom_geometry_source_tile_callbackFunction>
          >();

  final minZoom = options.minZoom;
  if (minZoom != null) {
    result.fields |= raw
        .mln_custom_geometry_source_option_field
        .MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_MIN_ZOOM
        .value;
    result.min_zoom = minZoom;
  }
  final maxZoom = options.maxZoom;
  if (maxZoom != null) {
    result.fields |= raw
        .mln_custom_geometry_source_option_field
        .MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_MAX_ZOOM
        .value;
    result.max_zoom = maxZoom;
  }
  final tolerance = options.tolerance;
  if (tolerance != null) {
    result.fields |= raw
        .mln_custom_geometry_source_option_field
        .MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_TOLERANCE
        .value;
    result.tolerance = tolerance;
  }
  final tileSize = options.tileSize;
  if (tileSize != null) {
    result.fields |= raw
        .mln_custom_geometry_source_option_field
        .MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_TILE_SIZE
        .value;
    result.tile_size = tileSize;
  }
  final buffer = options.buffer;
  if (buffer != null) {
    result.fields |= raw
        .mln_custom_geometry_source_option_field
        .MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_BUFFER
        .value;
    result.buffer = buffer;
  }
  final clip = options.clip;
  if (clip != null) {
    result.fields |= raw
        .mln_custom_geometry_source_option_field
        .MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_CLIP
        .value;
    result.clip = clip;
  }
  final wrap = options.wrap;
  if (wrap != null) {
    result.fields |= raw
        .mln_custom_geometry_source_option_field
        .MLN_CUSTOM_GEOMETRY_SOURCE_OPTION_WRAP
        .value;
    result.wrap = wrap;
  }
  return result;
}

raw.mln_canonical_tile_id _canonicalTileIdToNative(CanonicalTileId tileId) {
  final result = Struct.create<raw.mln_canonical_tile_id>();
  result.z = tileId.z;
  result.x = tileId.x;
  result.y = tileId.y;
  return result;
}

raw.mln_resource_response _resourceResponseToNative(
  ResourceResponse response,
  Allocator allocator,
) {
  final result = Struct.create<raw.mln_resource_response>();
  result.size = sizeOf<raw.mln_resource_response>();
  result.status = response.status.rawValue;
  result.error_reason = response.errorReason.rawValue;
  final bytes = response.bytes;
  if (bytes != null && bytes.isNotEmpty) {
    final nativeBytes = allocator<Uint8>(bytes.length);
    for (var index = 0; index < bytes.length; index += 1) {
      nativeBytes[index] = bytes[index];
    }
    result.bytes = nativeBytes;
    result.byte_count = bytes.length;
  }
  final errorMessage = response.errorMessage;
  if (errorMessage != null) {
    result.error_message = nativeUtf8CString(
      errorMessage,
      allocator,
    ).pointer.cast<Char>();
  }
  result.must_revalidate = response.mustRevalidate;
  final modifiedUnixMs = response.modifiedUnixMs;
  if (modifiedUnixMs != null) {
    result.has_modified = true;
    result.modified_unix_ms = modifiedUnixMs;
  }
  final expiresUnixMs = response.expiresUnixMs;
  if (expiresUnixMs != null) {
    result.has_expires = true;
    result.expires_unix_ms = expiresUnixMs;
  }
  final etag = response.etag;
  if (etag != null) {
    result.etag = nativeUtf8CString(etag, allocator).pointer.cast<Char>();
  }
  final retryAfterUnixMs = response.retryAfterUnixMs;
  if (retryAfterUnixMs != null) {
    result.has_retry_after = true;
    result.retry_after_unix_ms = retryAfterUnixMs;
  }
  return result;
}

void _freeNativeResourceResponse(
  raw.mln_resource_response response,
  Allocator allocator,
) {
  if (response.bytes != nullptr) {
    allocator.free(response.bytes);
  }
  if (response.error_message != nullptr) {
    allocator.free(response.error_message);
  }
  if (response.etag != nullptr) {
    allocator.free(response.etag);
  }
}

raw.mln_rendered_query_geometry _renderedQueryGeometryToNative(
  RenderedQueryGeometry geometry,
  Allocator allocator,
) {
  switch (geometry) {
    case RenderedQueryPoint(:final point):
      return _c.renderedQueryGeometryPoint(
        native_struct.screenPointToNative(point),
      );
    case RenderedQueryBox(:final box):
      final nativeBox = Struct.create<raw.mln_screen_box>();
      nativeBox.min = native_struct.screenPointToNative(box.min);
      nativeBox.max = native_struct.screenPointToNative(box.max);
      return _c.renderedQueryGeometryBox(nativeBox);
    case RenderedQueryLineString(:final points):
      final nativePoints = points.isEmpty
          ? nullptr.cast<raw.mln_screen_point>()
          : allocator<raw.mln_screen_point>(points.length);
      for (var index = 0; index < points.length; index += 1) {
        nativePoints[index] = native_struct.screenPointToNative(points[index]);
      }
      return _c.renderedQueryGeometryLineString(nativePoints, points.length);
  }
}

Pointer<raw.mln_feature_state_selector> _featureStateSelectorToNative(
  FeatureStateSelector selector,
  Allocator allocator,
) {
  final nativeSelector = allocator<raw.mln_feature_state_selector>();
  nativeSelector.ref.size = sizeOf<raw.mln_feature_state_selector>();
  nativeSelector.ref.source_id = nativeStringView(
    selector.sourceId,
    allocator,
  ).value;
  final sourceLayerId = selector.sourceLayerId;
  if (sourceLayerId != null) {
    nativeSelector.ref.fields |= raw
        .mln_feature_state_selector_field
        .MLN_FEATURE_STATE_SELECTOR_SOURCE_LAYER_ID
        .value;
    nativeSelector.ref.source_layer_id = nativeStringView(
      sourceLayerId,
      allocator,
    ).value;
  }
  final featureId = selector.featureId;
  if (featureId != null) {
    nativeSelector.ref.fields |= raw
        .mln_feature_state_selector_field
        .MLN_FEATURE_STATE_SELECTOR_FEATURE_ID
        .value;
    nativeSelector.ref.feature_id = nativeStringView(
      featureId,
      allocator,
    ).value;
  }
  final stateKey = selector.stateKey;
  if (stateKey != null) {
    nativeSelector.ref.fields |= raw
        .mln_feature_state_selector_field
        .MLN_FEATURE_STATE_SELECTOR_STATE_KEY
        .value;
    nativeSelector.ref.state_key = nativeStringView(stateKey, allocator).value;
  }
  return nativeSelector;
}

Pointer<raw.mln_rendered_feature_query_options>
_renderedFeatureQueryOptionsToNative(
  RenderedFeatureQueryOptions options,
  Allocator allocator,
) {
  final nativeOptions = allocator<raw.mln_rendered_feature_query_options>();
  nativeOptions.ref = _c.renderedFeatureQueryOptionsDefault();
  final layerIds = options.layerIds;
  if (layerIds != null) {
    nativeOptions.ref.fields |= raw
        .mln_rendered_feature_query_option_field
        .MLN_RENDERED_FEATURE_QUERY_OPTION_LAYER_IDS
        .value;
    nativeOptions.ref.layer_ids = _stringViewArray(layerIds, allocator);
    nativeOptions.ref.layer_id_count = layerIds.length;
  }
  final filter = options.filter;
  if (filter != null) {
    nativeOptions.ref.filter = native_json
        .nativeJsonValue(filter, allocator)
        .pointer;
  }
  return nativeOptions;
}

Pointer<raw.mln_source_feature_query_options>
_sourceFeatureQueryOptionsToNative(
  SourceFeatureQueryOptions options,
  Allocator allocator,
) {
  final nativeOptions = allocator<raw.mln_source_feature_query_options>();
  nativeOptions.ref = _c.sourceFeatureQueryOptionsDefault();
  final sourceLayerIds = options.sourceLayerIds;
  if (sourceLayerIds != null) {
    nativeOptions.ref.fields |= raw
        .mln_source_feature_query_option_field
        .MLN_SOURCE_FEATURE_QUERY_OPTION_SOURCE_LAYER_IDS
        .value;
    nativeOptions.ref.source_layer_ids = _stringViewArray(
      sourceLayerIds,
      allocator,
    );
    nativeOptions.ref.source_layer_id_count = sourceLayerIds.length;
  }
  final filter = options.filter;
  if (filter != null) {
    nativeOptions.ref.filter = native_json
        .nativeJsonValue(filter, allocator)
        .pointer;
  }
  return nativeOptions;
}

Pointer<raw.mln_style_tile_source_options> _nativeTileSourceOptions(
  TileSourceOptions options,
  Allocator allocator,
) {
  final nativeOptions = allocator<raw.mln_style_tile_source_options>();
  nativeOptions.ref = _c.styleTileSourceOptionsDefault();
  final minZoom = options.minZoom;
  if (minZoom != null) {
    nativeOptions.ref.fields |= raw
        .mln_style_tile_source_option_field
        .MLN_STYLE_TILE_SOURCE_OPTION_MIN_ZOOM
        .value;
    nativeOptions.ref.min_zoom = minZoom;
  }
  final maxZoom = options.maxZoom;
  if (maxZoom != null) {
    nativeOptions.ref.fields |= raw
        .mln_style_tile_source_option_field
        .MLN_STYLE_TILE_SOURCE_OPTION_MAX_ZOOM
        .value;
    nativeOptions.ref.max_zoom = maxZoom;
  }
  final attribution = options.attribution;
  if (attribution != null) {
    nativeOptions.ref.fields |= raw
        .mln_style_tile_source_option_field
        .MLN_STYLE_TILE_SOURCE_OPTION_ATTRIBUTION
        .value;
    nativeOptions.ref.attribution = nativeStringView(
      attribution,
      allocator,
    ).value;
  }
  final scheme = options.scheme;
  if (scheme != null) {
    nativeOptions.ref.fields |= raw
        .mln_style_tile_source_option_field
        .MLN_STYLE_TILE_SOURCE_OPTION_SCHEME
        .value;
    nativeOptions.ref.scheme = scheme.rawValue;
  }
  final bounds = options.bounds;
  if (bounds != null) {
    nativeOptions.ref.fields |= raw
        .mln_style_tile_source_option_field
        .MLN_STYLE_TILE_SOURCE_OPTION_BOUNDS
        .value;
    nativeOptions.ref.bounds = native_struct.latLngBoundsToNative(bounds);
  }
  final tileSize = options.tileSize;
  if (tileSize != null) {
    nativeOptions.ref.fields |= raw
        .mln_style_tile_source_option_field
        .MLN_STYLE_TILE_SOURCE_OPTION_TILE_SIZE
        .value;
    nativeOptions.ref.tile_size = tileSize;
  }
  final vectorEncoding = options.vectorEncoding;
  if (vectorEncoding != null) {
    nativeOptions.ref.fields |= raw
        .mln_style_tile_source_option_field
        .MLN_STYLE_TILE_SOURCE_OPTION_VECTOR_ENCODING
        .value;
    nativeOptions.ref.vector_encoding = vectorEncoding.rawValue;
  }
  final rasterDemEncoding = options.rasterDemEncoding;
  if (rasterDemEncoding != null) {
    nativeOptions.ref.fields |= raw
        .mln_style_tile_source_option_field
        .MLN_STYLE_TILE_SOURCE_OPTION_RASTER_ENCODING
        .value;
    nativeOptions.ref.raster_encoding = rasterDemEncoding.rawValue;
  }
  return nativeOptions;
}

Pointer<raw.mln_lat_lng> _latLngArray(
  List<LatLng> coordinates,
  Allocator allocator,
) {
  if (coordinates.isEmpty) {
    return nullptr.cast<raw.mln_lat_lng>();
  }
  final nativeCoordinates = allocator<raw.mln_lat_lng>(coordinates.length);
  for (var index = 0; index < coordinates.length; index += 1) {
    nativeCoordinates[index] = native_struct.latLngToNative(coordinates[index]);
  }
  return nativeCoordinates;
}

Pointer<raw.mln_string_view> _stringViewArray(
  List<String> values,
  Allocator allocator,
) {
  if (values.isEmpty) {
    return nullptr.cast<raw.mln_string_view>();
  }
  final views = allocator<raw.mln_string_view>(values.length);
  for (var index = 0; index < values.length; index += 1) {
    views[index] = nativeStringView(values[index], allocator).value;
  }
  return views;
}

List<QueriedFeature> _copyFeatureQueryResult(
  Pointer<raw.mln_feature_query_result> result,
) {
  try {
    return withNativeArena((arena) {
      final outCount = arena<Size>();
      _check(_c.featureQueryResultCount(result, outCount));
      return [
        for (var index = 0; index < outCount.value; index += 1)
          _copyQueriedFeature(result, index, arena),
      ];
    });
  } finally {
    _c.featureQueryResultDestroy(result);
  }
}

QueriedFeature _copyQueriedFeature(
  Pointer<raw.mln_feature_query_result> result,
  int index,
  Allocator allocator,
) {
  final outFeature = allocator<raw.mln_queried_feature>();
  outFeature.ref.size = sizeOf<raw.mln_queried_feature>();
  _check(_c.featureQueryResultGet(result, index, outFeature));
  final feature = outFeature.ref;
  final state =
      (feature.fields &
                  raw
                      .mln_queried_feature_field
                      .MLN_QUERIED_FEATURE_STATE
                      .value) ==
              0 ||
          feature.state == nullptr
      ? null
      : native_json.jsonValueFromNative(feature.state.ref);
  return QueriedFeature(
    feature: native_geometry.featureGeoJsonFromNative(feature.feature),
    sourceId:
        (feature.fields &
                raw
                    .mln_queried_feature_field
                    .MLN_QUERIED_FEATURE_SOURCE_ID
                    .value) ==
            0
        ? null
        : _copyStringView(feature.source_id),
    sourceLayerId:
        (feature.fields &
                raw
                    .mln_queried_feature_field
                    .MLN_QUERIED_FEATURE_SOURCE_LAYER_ID
                    .value) ==
            0
        ? null
        : _copyStringView(feature.source_layer_id),
    state: state,
  );
}

FeatureExtensionResult _copyFeatureExtensionResult(
  Pointer<raw.mln_feature_extension_result> result,
) {
  try {
    return withNativeArena((arena) {
      final outInfo = arena<raw.mln_feature_extension_result_info>();
      outInfo.ref.size = sizeOf<raw.mln_feature_extension_result_info>();
      _check(_c.featureExtensionResultGet(result, outInfo));
      final info = outInfo.ref;
      return switch (info.type) {
        1 => FeatureExtensionValue(
          native_json.jsonValueFromNative(info.data.value.ref),
        ),
        2 => FeatureExtensionFeatureCollection(
          native_geometry.featureCollectionFromNative(
            info.data.feature_collection,
          ),
        ),
        _ => throwInvalidArgument(
          'unknown native feature extension result type: ${info.type}',
        ),
      };
    });
  } finally {
    _c.featureExtensionResultDestroy(result);
  }
}

raw.mln_render_target_extent _renderTargetExtentToNative(
  RenderTargetExtent value,
) {
  final result = Struct.create<raw.mln_render_target_extent>();
  result.size = sizeOf<raw.mln_render_target_extent>();
  result.width = value.width;
  result.height = value.height;
  result.scale_factor = value.scaleFactor;
  return result;
}

raw.mln_metal_context_descriptor _metalContextDescriptorToNative(
  MetalContextDescriptor value,
) {
  final result = Struct.create<raw.mln_metal_context_descriptor>();
  result.size = sizeOf<raw.mln_metal_context_descriptor>();
  result.device = Pointer<Void>.fromAddress(value.device.address);
  return result;
}

raw.mln_vulkan_context_descriptor _vulkanContextDescriptorToNative(
  VulkanContextDescriptor value,
) {
  final result = Struct.create<raw.mln_vulkan_context_descriptor>();
  result.size = sizeOf<raw.mln_vulkan_context_descriptor>();
  result.instance = Pointer<Void>.fromAddress(value.instance.address);
  result.physical_device = Pointer<Void>.fromAddress(
    value.physicalDevice.address,
  );
  result.device = Pointer<Void>.fromAddress(value.device.address);
  result.graphics_queue = Pointer<Void>.fromAddress(
    value.graphicsQueue.address,
  );
  result.graphics_queue_family_index = value.graphicsQueueFamilyIndex;
  return result;
}

raw.mln_metal_surface_descriptor _metalSurfaceDescriptorToNative(
  MetalSurfaceDescriptor value,
) {
  final result = _c.metalSurfaceDescriptorDefault();
  result.extent = _renderTargetExtentToNative(value.extent);
  result.context = _metalContextDescriptorToNative(value.context);
  result.layer = Pointer<Void>.fromAddress(value.layer.address);
  return result;
}

raw.mln_vulkan_surface_descriptor _vulkanSurfaceDescriptorToNative(
  VulkanSurfaceDescriptor value,
) {
  final result = _c.vulkanSurfaceDescriptorDefault();
  result.extent = _renderTargetExtentToNative(value.extent);
  result.context = _vulkanContextDescriptorToNative(value.context);
  result.surface = Pointer<Void>.fromAddress(value.surface.address);
  return result;
}

raw.mln_metal_owned_texture_descriptor _metalOwnedTextureDescriptorToNative(
  MetalOwnedTextureDescriptor value,
) {
  final result = Struct.create<raw.mln_metal_owned_texture_descriptor>();
  result.size = sizeOf<raw.mln_metal_owned_texture_descriptor>();
  result.extent = _renderTargetExtentToNative(value.extent);
  result.context = _metalContextDescriptorToNative(value.context);
  return result;
}

raw.mln_metal_borrowed_texture_descriptor
_metalBorrowedTextureDescriptorToNative(MetalBorrowedTextureDescriptor value) {
  final result = Struct.create<raw.mln_metal_borrowed_texture_descriptor>();
  result.size = sizeOf<raw.mln_metal_borrowed_texture_descriptor>();
  result.extent = _renderTargetExtentToNative(value.extent);
  result.texture = Pointer<Void>.fromAddress(value.texture.address);
  return result;
}

raw.mln_vulkan_owned_texture_descriptor _vulkanOwnedTextureDescriptorToNative(
  VulkanOwnedTextureDescriptor value,
) {
  final result = Struct.create<raw.mln_vulkan_owned_texture_descriptor>();
  result.size = sizeOf<raw.mln_vulkan_owned_texture_descriptor>();
  result.extent = _renderTargetExtentToNative(value.extent);
  result.context = _vulkanContextDescriptorToNative(value.context);
  return result;
}

raw.mln_vulkan_borrowed_texture_descriptor
_vulkanBorrowedTextureDescriptorToNative(
  VulkanBorrowedTextureDescriptor value,
) {
  final result = Struct.create<raw.mln_vulkan_borrowed_texture_descriptor>();
  result.size = sizeOf<raw.mln_vulkan_borrowed_texture_descriptor>();
  result.extent = _renderTargetExtentToNative(value.extent);
  result.context = _vulkanContextDescriptorToNative(value.context);
  result.image = Pointer<Void>.fromAddress(value.image.address);
  result.image_view = Pointer<Void>.fromAddress(value.imageView.address);
  result.format = value.format;
  result.initial_layout = value.initialLayout;
  result.final_layout = value.finalLayout;
  return result;
}

Pointer<raw.mln_camera_options> _nativeCamera(
  CameraOptions camera,
  Allocator allocator,
) {
  final nativeCamera = allocator<raw.mln_camera_options>();
  nativeCamera.ref = native_struct.cameraOptionsToNative(camera);
  return nativeCamera;
}

Pointer<raw.mln_animation_options> _nativeAnimation(
  AnimationOptions? animation,
  Allocator allocator,
) {
  if (animation == null) {
    return nullptr.cast<raw.mln_animation_options>();
  }
  final nativeAnimation = allocator<raw.mln_animation_options>();
  nativeAnimation.ref = native_struct.animationOptionsToNative(animation);
  return nativeAnimation;
}

Pointer<raw.mln_screen_point> _nativeScreenPoint(
  ScreenPoint? point,
  Allocator allocator,
) {
  if (point == null) {
    return nullptr.cast<raw.mln_screen_point>();
  }
  final nativePoint = allocator<raw.mln_screen_point>();
  nativePoint.ref = native_struct.screenPointToNative(point);
  return nativePoint;
}

String? _copyStyleSourceAttribution(
  Pointer<raw.mln_map> map,
  raw.mln_string_view sourceId,
  bool hasAttribution,
  int attributionSize,
  Allocator allocator,
) {
  if (!hasAttribution) {
    return null;
  }
  final buffer = attributionSize == 0
      ? nullptr.cast<Char>()
      : allocator<Char>(attributionSize);
  final outSize = allocator<Size>();
  final outFound = allocator<Bool>();
  _check(
    _c.mapCopyStyleSourceAttribution(
      map,
      sourceId,
      buffer,
      attributionSize,
      outSize,
      outFound,
    ),
  );
  if (!outFound.value) {
    return null;
  }
  if (outSize.value == 0) {
    return '';
  }
  return buffer.cast<Utf8>().toDartString(length: outSize.value);
}

JsonValue? _copyJsonSnapshot(Pointer<raw.mln_json_snapshot> snapshot) {
  if (snapshot == nullptr) {
    return null;
  }
  try {
    return withNativeArena((arena) {
      final outValue = arena<Pointer<raw.mln_json_value>>();
      outValue.value = nullptr;
      _check(_c.jsonSnapshotGet(snapshot, outValue));
      if (outValue.value == nullptr) {
        return null;
      }
      return native_json.jsonValueFromNative(outValue.value.ref);
    });
  } finally {
    _c.jsonSnapshotDestroy(snapshot);
  }
}

List<String> _copyStyleIdList(Pointer<raw.mln_style_id_list> list) {
  try {
    return withNativeArena((arena) {
      final outCount = arena<Size>();
      _check(_c.styleIdListCount(list, outCount));
      final ids = <String>[];
      for (var index = 0; index < outCount.value; index += 1) {
        final outId = arena<raw.mln_string_view>();
        _check(_c.styleIdListGet(list, index, outId));
        ids.add(_copyStringView(outId.ref) ?? '');
      }
      return ids;
    });
  } finally {
    _c.styleIdListDestroy(list);
  }
}

String? _copyStringView(raw.mln_string_view view) =>
    _copyNativeString(view.data, view.size);

String? _copyNativeString(Pointer<Char> pointer, int byteLength) {
  if (pointer == nullptr || byteLength == 0) {
    return null;
  }
  return pointer.cast<Utf8>().toDartString(length: byteLength);
}

void _check(int statusCode) {
  checkNativeStatus(statusCode, _c.threadLastErrorMessage);
}

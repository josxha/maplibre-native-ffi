part of 'runtime.dart';

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
        _c.raw.mln_runtime_offline_region_create_take_result(
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
        _c.raw.mln_runtime_offline_region_get_take_result(
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
        _c.raw.mln_runtime_offline_regions_list_take_result(
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
        _c.raw.mln_runtime_offline_regions_merge_database_take_result(
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
        _c.raw.mln_runtime_offline_region_update_metadata_take_result(
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
        _c.raw.mln_runtime_offline_region_get_status_take_result(
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
    _check(_c.raw.mln_runtime_offline_operation_discard(_runtime._pointer, id));
    _discarded = true;
  }
}

raw.mln_runtime_options _runtimeOptionsToNative(
  RuntimeOptions options,
  Allocator allocator,
) {
  final result = _c.raw.mln_runtime_options_default();
  final assetPath = options.assetPath;
  if (assetPath != null) {
    result.asset_path = nativeUtf8CString(
      assetPath,
      allocator,
    ).pointer.cast<Char>();
  }
  final cachePath = options.cachePath;
  if (cachePath != null) {
    result.cache_path = nativeUtf8CString(
      cachePath,
      allocator,
    ).pointer.cast<Char>();
  }
  final maximumCacheSize = options.maximumCacheSize;
  if (maximumCacheSize != null) {
    if (maximumCacheSize < 0) {
      throwInvalidArgument('maximum cache size must be non-negative');
    }
    result.flags |=
        raw.mln_runtime_option_flag.MLN_RUNTIME_OPTION_MAXIMUM_CACHE_SIZE.value;
    result.maximum_cache_size = maximumCacheSize;
  }
  return result;
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

Pointer<Char> _nativeOwnedCString(String value) =>
    nativeUtf8CString(value, calloc).pointer.cast<Char>();

void _checkNativeCString(String value) {
  if (value.contains('\u0000')) {
    throwInvalidArgument(
      'null-terminated strings must not contain embedded NUL',
    );
  }
}

void _checkOptionalNativeCString(String? value) {
  if (value != null) {
    _checkNativeCString(value);
  }
}

void _checkResourceResponseNativeStrings(ResourceResponse response) {
  _checkOptionalNativeCString(response.errorMessage);
  _checkOptionalNativeCString(response.etag);
}

int _uint32(int value, String name) {
  if (value < 0 || value > 0xffffffff) {
    throwInvalidArgument('$name must fit uint32');
  }
  return value;
}

int _positiveUint32(int value, String name) {
  if (value <= 0 || value > 0xffffffff) {
    throwInvalidArgument('$name must be between 1 and 4294967295');
  }
  return value;
}

int _uint16(int value, String name) {
  if (value < 0 || value > 0xffff) {
    throwInvalidArgument('$name must be between 0 and 65535');
  }
  return value;
}

int _uint16Positive(int value, String name) {
  if (value <= 0 || value > 0xffff) {
    throwInvalidArgument('$name must be between 1 and 65535');
  }
  return value;
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
      _check(_c.raw.mln_offline_region_snapshot_get(snapshot, outInfo));
      return _offlineRegionInfoFromNative(outInfo.ref);
    });
  } finally {
    _c.raw.mln_offline_region_snapshot_destroy(snapshot);
  }
}

List<OfflineRegionInfo> _copyOfflineRegionList(
  Pointer<raw.mln_offline_region_list> list,
) {
  try {
    return withNativeArena((arena) {
      final outCount = arena<Size>();
      _check(_c.raw.mln_offline_region_list_count(list, outCount));
      return [
        for (var index = 0; index < outCount.value; index += 1)
          _copyOfflineRegionListEntry(list, index, arena),
      ];
    });
  } finally {
    _c.raw.mln_offline_region_list_destroy(list);
  }
}

OfflineRegionInfo _copyOfflineRegionListEntry(
  Pointer<raw.mln_offline_region_list> list,
  int index,
  Allocator allocator,
) {
  final outInfo = allocator<raw.mln_offline_region_info>();
  outInfo.ref.size = sizeOf<raw.mln_offline_region_info>();
  _check(_c.raw.mln_offline_region_list_get(list, index, outInfo));
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

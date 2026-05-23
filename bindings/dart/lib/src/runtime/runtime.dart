import 'dart:ffi';

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
import '../style/style.dart';

final MaplibreNativeCApi _c = MaplibreNativeCApi.open();

/// Owner-thread runtime handle for MapLibre Native work and event polling.
final class RuntimeHandle {
  RuntimeHandle._(Pointer<raw.mln_runtime> pointer)
    : _state = NativeHandleState(pointer, 'RuntimeHandle');

  final NativeHandleState<raw.mln_runtime> _state;

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

  /// Creates a map owned by this runtime.
  MapHandle createMap({MapOptions options = const MapOptions()}) =>
      MapHandle.create(this, options: options);

  /// Explicitly destroys this runtime.
  void close() {
    _state.close(_c.runtimeDestroy, _c.threadLastErrorMessage);
  }
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
  }

  /// Loads inline style JSON through MapLibre Native style APIs.
  void setStyleJson(String json) {
    withNativeArena((arena) {
      final nativeJson = nativeUtf8CString(json, arena);
      _check(_c.mapSetStyleJson(_pointer, nativeJson.pointer.cast<Char>()));
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
    return withNativeArena((arena) {
      final nativeId = nativeStringView(sourceId, arena);
      final outRemoved = arena<Bool>();
      _check(_c.mapRemoveStyleSource(_pointer, nativeId.value, outRemoved));
      return outRemoved.value;
    });
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
  }
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

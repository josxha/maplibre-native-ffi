part of 'runtime.dart';

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
      _check(_c.raw.mln_map_projection_get_camera(_pointer, outCamera));
      return native_struct.cameraOptionsFromNative(outCamera.ref);
    });
  }

  /// Applies camera fields to the projection helper.
  void setCamera(CameraOptions camera) {
    withNativeArena((arena) {
      final nativeCamera = arena<raw.mln_camera_options>();
      nativeCamera.ref = native_struct.cameraOptionsToNative(camera);
      _check(_c.raw.mln_map_projection_set_camera(_pointer, nativeCamera));
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
        _c.raw.mln_map_projection_set_visible_coordinates(
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
        _c.raw.mln_map_projection_set_visible_geometry(
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
        _c.raw.mln_map_projection_pixel_for_lat_lng(
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
        _c.raw.mln_map_projection_lat_lng_for_pixel(
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
    _state.close(
      (pointer) => _c.raw.mln_map_projection_destroy(pointer).value,
      _c.threadLastErrorMessage,
    );
  }
}

/// Owner-thread render session handle attached to a retained map.
final class RenderSessionHandle {
  RenderSessionHandle._(this._map, Pointer<raw.mln_render_session> pointer)
    : _state = NativeHandleState(pointer, 'RenderSessionHandle');

  final MapHandle _map;
  final NativeHandleState<raw.mln_render_session> _state;
  Object? _activeTextureFrame;

  /// Whether this render session has been closed by the Dart binding.
  bool get isClosed => _state.isClosed;

  Pointer<raw.mln_render_session> get _pointer {
    final _ = _map._pointer;
    return _state.pointer;
  }

  /// Resizes an attached render session.
  void resize(int width, int height, {double scaleFactor = 1}) {
    _checkNoActiveTextureFrame('resize render session');
    _check(
      _c.raw.mln_render_session_resize(_pointer, width, height, scaleFactor),
    );
  }

  /// Processes the latest map render update for this session.
  void renderUpdate() {
    _checkNoActiveTextureFrame('render update');
    _check(_c.raw.mln_render_session_render_update(_pointer));
  }

  /// Detaches backend-bound render resources while keeping the handle live.
  void detach() {
    _checkNoActiveTextureFrame('detach render session');
    _check(_c.raw.mln_render_session_detach(_pointer));
  }

  /// Asks the session renderer to release cached resources where possible.
  void reduceMemoryUse() {
    _check(_c.raw.mln_render_session_reduce_memory_use(_pointer));
  }

  /// Clears renderer data for the session.
  void clearData() {
    _check(_c.raw.mln_render_session_clear_data(_pointer));
  }

  /// Dumps renderer debug logs through MapLibre Native logging.
  void dumpDebugLogs() {
    _check(_c.raw.mln_render_session_dump_debug_logs(_pointer));
  }

  /// Sets per-feature state on a render source.
  void setFeatureState(FeatureStateSelector selector, JsonObject state) {
    withNativeArena((arena) {
      final nativeSelector = _featureStateSelectorToNative(selector, arena);
      final nativeState = native_json.nativeJsonValue(state, arena);
      _check(
        _c.raw.mln_render_session_set_feature_state(
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
        _c.raw.mln_render_session_get_feature_state(
          _pointer,
          nativeSelector,
          outState,
        ),
      );
      return _copyJsonSnapshot(outState.value);
    });
  }

  /// Removes per-feature state from a render source.
  void removeFeatureState(FeatureStateSelector selector) {
    withNativeArena((arena) {
      final nativeSelector = _featureStateSelectorToNative(selector, arena);
      _check(
        _c.raw.mln_render_session_remove_feature_state(
          _pointer,
          nativeSelector,
        ),
      );
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
        _c.raw.mln_render_session_query_rendered_features(
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
        _c.raw.mln_render_session_query_source_features(
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
        _c.raw.mln_render_session_query_feature_extensions(
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
  TextureImageInfo textureImageInfo() {
    _checkNoActiveTextureFrame('read texture image info');
    return withNativeArena((arena) {
      final info = arena<raw.mln_texture_image_info>();
      info.ref = _c.raw.mln_texture_image_info_default();
      final probeStatus = _c.raw.mln_texture_read_premultiplied_rgba8(
        _pointer,
        nullptr.cast<Uint8>(),
        0,
        info,
      );
      if (_statusCode(probeStatus) != nativeStatusInvalidArgument ||
          info.ref.byte_length == 0) {
        _check(probeStatus);
      }
      return TextureImageInfo._fromNative(info.ref);
    });
  }

  /// Reads the latest texture into caller-owned native [buffer].
  TextureImageInfo readPremultipliedRgba8Into(NativeBuffer buffer) {
    _checkNoActiveTextureFrame('read texture image');
    final expectedInfo = textureImageInfo();
    if (buffer.byteLength < expectedInfo.byteLength) {
      throwInvalidArgument(
        'native buffer length ${buffer.byteLength} is smaller than required '
        '${expectedInfo.byteLength}',
      );
    }
    return withNativeArena((arena) {
      final info = arena<raw.mln_texture_image_info>();
      info.ref = _c.raw.mln_texture_image_info_default();
      _check(
        _c.raw.mln_texture_read_premultiplied_rgba8(
          _pointer,
          buffer.unsafePointer,
          buffer.byteLength,
          info,
        ),
      );
      return TextureImageInfo._fromNative(info.ref);
    });
  }

  /// Reads the latest rendered session-owned texture as premultiplied RGBA8.
  TextureImage readPremultipliedRgba8() {
    _checkNoActiveTextureFrame('read texture image');
    final info = textureImageInfo();
    return withNativeArena((arena) {
      final data = arena<Uint8>(info.byteLength);
      final readInfo = arena<raw.mln_texture_image_info>();
      readInfo.ref = _c.raw.mln_texture_image_info_default();
      _check(
        _c.raw.mln_texture_read_premultiplied_rgba8(
          _pointer,
          data,
          info.byteLength,
          readInfo,
        ),
      );
      final copiedInfo = TextureImageInfo._fromNative(readInfo.ref);
      return TextureImage(
        info: copiedInfo,
        bytes: Uint8List.fromList(data.asTypedList(copiedInfo.byteLength)),
      );
    });
  }

  /// Acquires the latest Metal texture frame until [MetalOwnedTextureFrame.close].
  MetalOwnedTextureFrame acquireMetalTextureFrame() {
    _checkNoActiveTextureFrame('acquire Metal texture frame');
    return withNativeArena((arena) {
      final outFrame = arena<raw.mln_metal_owned_texture_frame>();
      outFrame.ref.size = sizeOf<raw.mln_metal_owned_texture_frame>();
      _check(_c.raw.mln_metal_owned_texture_acquire_frame(_pointer, outFrame));
      return MetalOwnedTextureFrame._(this, outFrame.ref);
    });
  }

  /// Acquires the latest Vulkan texture frame until [VulkanOwnedTextureFrame.close].
  VulkanOwnedTextureFrame acquireVulkanTextureFrame() {
    _checkNoActiveTextureFrame('acquire Vulkan texture frame');
    return withNativeArena((arena) {
      final outFrame = arena<raw.mln_vulkan_owned_texture_frame>();
      outFrame.ref.size = sizeOf<raw.mln_vulkan_owned_texture_frame>();
      _check(_c.raw.mln_vulkan_owned_texture_acquire_frame(_pointer, outFrame));
      return VulkanOwnedTextureFrame._(this, outFrame.ref);
    });
  }

  /// Acquires the latest OpenGL texture frame until [OpenGLOwnedTextureFrame.close].
  OpenGLOwnedTextureFrame acquireOpenGLTextureFrame() {
    _checkNoActiveTextureFrame('acquire OpenGL texture frame');
    return withNativeArena((arena) {
      final outFrame = arena<raw.mln_opengl_owned_texture_frame>();
      outFrame.ref.size = sizeOf<raw.mln_opengl_owned_texture_frame>();
      _check(_c.raw.mln_opengl_owned_texture_acquire_frame(_pointer, outFrame));
      return OpenGLOwnedTextureFrame._(this, outFrame.ref);
    });
  }

  /// Explicitly destroys this render session.
  void close() {
    _checkNoActiveTextureFrame('close render session');
    _state.close(
      (pointer) => _c.raw.mln_render_session_destroy(pointer).value,
      _c.threadLastErrorMessage,
    );
  }

  void _registerTextureFrame(Object frame) {
    _checkNoActiveTextureFrame('acquire texture frame');
    _activeTextureFrame = frame;
  }

  void _releaseTextureFrame(Object frame) {
    if (identical(_activeTextureFrame, frame)) {
      _activeTextureFrame = null;
    }
  }

  void _checkNoActiveTextureFrame(String operation) {
    if (_activeTextureFrame != null) {
      throwInvalidState(
        '$operation requires releasing the active texture frame',
      );
    }
  }
}

/// Releasable handle for a resource request owned by a Dart provider.
final class ResourceRequestHandle {
  ResourceRequestHandle._(this._pointer)
    : _ownerIsolateHash = Isolate.current.hashCode;

  Pointer<raw.mln_resource_request_handle> _pointer;
  final int _ownerIsolateHash;
  var _released = false;

  /// Whether this provider reference has been released by Dart.
  bool get isReleased => _released;

  /// Reports whether MapLibre has cancelled this provider request.
  bool get isCancelled => cancelled();

  /// Reports whether MapLibre has cancelled this provider request.
  bool cancelled() {
    return withNativeArena((arena) {
      final outCancelled = arena<Bool>();
      _check(_c.raw.mln_resource_request_cancelled(_livePointer, outCancelled));
      return outCancelled.value;
    });
  }

  /// Completes this request with [response] and releases it. Completion is one-shot.
  void complete(ResourceResponse response) {
    _checkResourceResponseNativeStrings(response);
    final pointer = _livePointer;
    withNativeArena((arena) {
      final nativeResponse = arena<raw.mln_resource_response>();
      nativeResponse.ref = _resourceResponseToNative(response, arena);
      _check(_c.raw.mln_resource_request_complete(pointer, nativeResponse));
    });
    _pointer = nullptr;
    _released = true;
    _c.raw.mln_resource_request_release(pointer);
  }

  /// Releases the provider reference. The handle must not be used afterwards.
  void close() {
    _checkOwnerIsolate();
    if (_released) {
      return;
    }
    _c.raw.mln_resource_request_release(_pointer);
    _pointer = nullptr;
    _released = true;
  }

  Pointer<raw.mln_resource_request_handle> get _livePointer {
    _checkOwnerIsolate();
    if (_released || _pointer == nullptr) {
      throwInvalidArgument('resource request handle has been released');
    }
    return _pointer;
  }

  void _checkOwnerIsolate() {
    if (Isolate.current.hashCode != _ownerIsolateHash) {
      throwWrongThread(
        'ResourceRequestHandle belongs to a different Dart isolate',
      );
    }
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
  MetalOwnedTextureFrame._(this._session, this._frame) {
    _session._registerTextureFrame(this);
  }

  final RenderSessionHandle _session;
  final raw.mln_metal_owned_texture_frame _frame;
  var _closed = false;

  /// Physical texture width in device pixels.
  int get width {
    _checkOpen();
    return _frame.width;
  }

  /// Physical texture height in device pixels.
  int get height {
    _checkOpen();
    return _frame.height;
  }

  /// UI-to-device pixel scale used for this frame.
  double get scaleFactor {
    _checkOpen();
    return _frame.scale_factor;
  }

  /// Backend-native Metal pixel format value.
  int get pixelFormat {
    _checkOpen();
    return _frame.pixel_format;
  }

  /// Unsafe borrowed `id<MTLTexture>` / `MTL::Texture*` pointer.
  ///
  /// The pointer is valid only until [close] releases this frame.
  ScopedNativePointer get unsafeTexture =>
      _borrowedPointer(_frame.texture, 'Metal texture');

  /// Unsafe borrowed `id<MTLDevice>` / `MTL::Device*` pointer.
  ///
  /// The pointer is valid only until [close] releases this frame.
  ScopedNativePointer get unsafeDevice =>
      _borrowedPointer(_frame.device, 'Metal device');

  /// Releases this frame. The unsafe backend pointers become invalid.
  void close() {
    if (_closed) {
      return;
    }
    withNativeArena((arena) {
      final nativeFrame = arena<raw.mln_metal_owned_texture_frame>();
      nativeFrame.ref = _frame;
      _check(
        _c.raw.mln_metal_owned_texture_release_frame(
          _session._pointer,
          nativeFrame,
        ),
      );
    });
    _closed = true;
    _session._releaseTextureFrame(this);
  }

  ScopedNativePointer _borrowedPointer(Pointer<Void> pointer, String name) {
    _checkOpen();
    return ScopedNativePointer(
      pointer.address,
      checkValid: _checkOpen,
      debugName: name,
    );
  }

  void _checkOpen() {
    if (_closed) {
      throwInvalidArgument('Metal texture frame has already been released');
    }
    final _ = _session._pointer;
  }
}

/// Scoped Vulkan texture frame borrowed from a session-owned texture target.
final class VulkanOwnedTextureFrame {
  VulkanOwnedTextureFrame._(this._session, this._frame) {
    _session._registerTextureFrame(this);
  }

  final RenderSessionHandle _session;
  final raw.mln_vulkan_owned_texture_frame _frame;
  var _closed = false;

  /// Physical image width in device pixels.
  int get width {
    _checkOpen();
    return _frame.width;
  }

  /// Physical image height in device pixels.
  int get height {
    _checkOpen();
    return _frame.height;
  }

  /// UI-to-device pixel scale used for this frame.
  double get scaleFactor {
    _checkOpen();
    return _frame.scale_factor;
  }

  /// Backend-native Vulkan format value.
  int get format {
    _checkOpen();
    return _frame.format;
  }

  /// Backend-native Vulkan image layout value.
  int get layout {
    _checkOpen();
    return _frame.layout;
  }

  /// Unsafe borrowed VkImage pointer.
  ///
  /// The pointer is valid only until [close] releases this frame.
  ScopedNativePointer get unsafeImage =>
      _borrowedPointer(_frame.image, 'Vulkan image');

  /// Unsafe borrowed VkImageView pointer.
  ///
  /// The pointer is valid only until [close] releases this frame.
  ScopedNativePointer get unsafeImageView =>
      _borrowedPointer(_frame.image_view, 'Vulkan image view');

  /// Unsafe borrowed VkDevice pointer.
  ///
  /// The pointer is valid only until [close] releases this frame.
  ScopedNativePointer get unsafeDevice =>
      _borrowedPointer(_frame.device, 'Vulkan device');

  /// Releases this frame. The unsafe backend pointers become invalid.
  void close() {
    if (_closed) {
      return;
    }
    withNativeArena((arena) {
      final nativeFrame = arena<raw.mln_vulkan_owned_texture_frame>();
      nativeFrame.ref = _frame;
      _check(
        _c.raw.mln_vulkan_owned_texture_release_frame(
          _session._pointer,
          nativeFrame,
        ),
      );
    });
    _closed = true;
    _session._releaseTextureFrame(this);
  }

  ScopedNativePointer _borrowedPointer(Pointer<Void> pointer, String name) {
    _checkOpen();
    return ScopedNativePointer(
      pointer.address,
      checkValid: _checkOpen,
      debugName: name,
    );
  }

  void _checkOpen() {
    if (_closed) {
      throwInvalidArgument('Vulkan texture frame has already been released');
    }
    final _ = _session._pointer;
  }
}

/// Scoped OpenGL texture frame borrowed from a session-owned texture target.
final class OpenGLOwnedTextureFrame {
  OpenGLOwnedTextureFrame._(this._session, this._frame) {
    _session._registerTextureFrame(this);
  }

  final RenderSessionHandle _session;
  final raw.mln_opengl_owned_texture_frame _frame;
  var _closed = false;

  /// Physical texture width in device pixels.
  int get width {
    _checkOpen();
    return _frame.width;
  }

  /// Physical texture height in device pixels.
  int get height {
    _checkOpen();
    return _frame.height;
  }

  /// UI-to-device pixel scale used for this frame.
  double get scaleFactor {
    _checkOpen();
    return _frame.scale_factor;
  }

  /// Borrowed OpenGL texture name.
  ScopedNativeInt get texture {
    _checkOpen();
    return ScopedNativeInt(
      _frame.texture,
      checkValid: _checkOpen,
      debugName: 'OpenGL texture',
    );
  }

  /// Backend-native OpenGL texture target.
  int get target {
    _checkOpen();
    return _frame.target;
  }

  /// Backend-native OpenGL internal format.
  int get internalFormat {
    _checkOpen();
    return _frame.internal_format;
  }

  /// Backend-native OpenGL pixel format.
  int get format {
    _checkOpen();
    return _frame.format;
  }

  /// Backend-native OpenGL pixel type.
  int get type {
    _checkOpen();
    return _frame.type;
  }

  /// Releases this frame. The borrowed texture name becomes invalid.
  void close() {
    if (_closed) {
      return;
    }
    withNativeArena((arena) {
      final nativeFrame = arena<raw.mln_opengl_owned_texture_frame>();
      nativeFrame.ref = _frame;
      _check(
        _c.raw.mln_opengl_owned_texture_release_frame(
          _session._pointer,
          nativeFrame,
        ),
      );
    });
    _closed = true;
    _session._releaseTextureFrame(this);
  }

  void _checkOpen() {
    if (_closed) {
      throwInvalidArgument('OpenGL texture frame has already been released');
    }
    final _ = _session._pointer;
  }
}

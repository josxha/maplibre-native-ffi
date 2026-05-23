import 'dart:ffi';

import 'package:ffi/ffi.dart';

import '../internal/c/maplibre_native_c.dart';
import '../internal/c/maplibre_native_c.g.dart' as raw;
import '../internal/lifecycle/lifecycle.dart';
import '../internal/memory/memory.dart';
import '../internal/status/status.dart';

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

  /// Creates a map owned by this runtime using native default map options.
  MapHandle createMap() => MapHandle.create(this);

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

/// Owner-thread map handle bound to a retained runtime.
final class MapHandle {
  MapHandle._(this._runtime, Pointer<raw.mln_map> pointer)
    : _state = NativeHandleState(pointer, 'MapHandle');

  /// Creates a map owned by [runtime] using native default map options.
  factory MapHandle.create(RuntimeHandle runtime) {
    return withNativeArena((arena) {
      final options = arena<raw.mln_map_options>();
      options.ref = _c.mapOptionsDefault();
      final outMap = arena<Pointer<raw.mln_map>>();
      outMap.value = nullptr;

      _check(_c.mapCreate(runtime._pointer, options, outMap));
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

  /// Explicitly destroys this map.
  void close() {
    _state.close(_c.mapDestroy, _c.threadLastErrorMessage);
  }
}

String? _copyNativeString(Pointer<Char> pointer, int byteLength) {
  if (pointer == nullptr || byteLength == 0) {
    return null;
  }
  return pointer.cast<Utf8>().toDartString(length: byteLength);
}

void _check(int statusCode) {
  checkNativeStatus(statusCode, _c.threadLastErrorMessage);
}

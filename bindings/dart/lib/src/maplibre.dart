import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'geo/geo.dart';
import 'internal/callback/callback_state.dart';
import 'internal/c/maplibre_native_c.dart';
import 'internal/c/maplibre_native_c.g.dart' as raw;
import 'internal/memory/memory.dart';
import 'internal/status/status.dart';
import 'internal/struct/struct.dart' as native_struct;
import 'log/log.dart';

typedef _LogRecordListenerFunction = Void Function(Pointer<Void>);

final class _NativeLogCallbackState extends Struct {
  external Pointer<NativeFunction<_LogRecordListenerFunction>> listener;

  @Uint32()
  external int consume;
}

final class _NativeLogRecord extends Struct {
  external Pointer<Void> owner;

  @Uint32()
  external int severity;

  @Uint32()
  external int event;

  @Int64()
  external int code;

  external Pointer<Char> message;
}

final class _LogCallbackState extends RetainedCallbackState {
  _LogCallbackState(LogCallback callback, {required bool consume}) {
    listener = NativeCallable<_LogRecordListenerFunction>.listener((
      Pointer<Void> record,
    ) {
      final ran = runUpcall(() {
        try {
          try {
            callback(_copyLogRecord(record.cast<_NativeLogRecord>().ref));
          } catch (_) {
            // Log callbacks are notification boundaries; user exceptions are
            // contained so they never surface from native callback delivery.
          }
        } finally {
          Maplibre._c.dartLogRecordDestroy(record);
        }
      });
      if (!ran) {
        Maplibre._c.dartLogRecordDestroy(record);
      }
    });
    pointer = calloc<_NativeLogCallbackState>();
    pointer.ref.listener = listener.nativeFunction;
    pointer.ref.consume = consume ? 1 : 0;
  }

  late final Pointer<_NativeLogCallbackState> pointer;
  late final NativeCallable<_LogRecordListenerFunction> listener;

  @override
  void closeResources() {
    calloc.free(pointer);
    listener.close();
  }
}

LogRecord _copyLogRecord(_NativeLogRecord record) {
  return LogRecord(
    severity: LogSeverity.fromRawValue(record.severity),
    event: LogEvent.fromRawValue(record.event),
    code: record.code,
    message: record.message == nullptr
        ? ''
        : record.message.cast<Utf8>().toDartString(),
  );
}

/// Process-global entry points for the Dart binding.
final class Maplibre {
  Maplibre._();

  static final MaplibreNativeCApi _c = MaplibreNativeCApi.open();
  static _LogCallbackState? _logCallbackState;

  /// Returns the native C ABI contract version.
  static int cVersion() => _c.cVersion();

  /// Returns the render backends compiled into the linked native library.
  static RenderBackendMask supportedRenderBackends() =>
      RenderBackendMask(_c.supportedRenderBackendMask());

  /// Reads MapLibre Native's process-global network status.
  static NetworkStatus networkStatus() {
    return withNativeArena((arena) {
      final outStatus = arena<Uint32>();
      _checkStatus(_c.networkStatusGet(outStatus));
      return NetworkStatus.fromRawValue(outStatus.value);
    });
  }

  /// Sets MapLibre Native's process-global network status.
  static void setNetworkStatus(NetworkStatus status) {
    _checkStatus(_c.networkStatusSet(status.rawValueForSet()));
  }

  /// Sets the process-global native log callback.
  static void setLogCallback(LogCallback callback, {bool consume = false}) {
    final state = _LogCallbackState(callback, consume: consume);
    try {
      _checkStatus(
        _c.logSetCallback(_c.dartLogCallback(), state.pointer.cast<Void>()),
      );
      _logCallbackState?.close();
      _logCallbackState = state;
    } catch (_) {
      state.close();
      rethrow;
    }
  }

  /// Clears the process-global native log callback.
  static void clearLogCallback() {
    _checkStatus(_c.logClearCallback());
    _logCallbackState?.close();
    _logCallbackState = null;
  }

  /// Sets which log severities MapLibre Native may dispatch asynchronously.
  static void setAsyncLogSeverityMask(LogSeverityMask mask) {
    _checkStatus(_c.logSetAsyncSeverityMask(mask.bits));
  }

  /// Converts a geographic coordinate to spherical Mercator projected meters.
  static ProjectedMeters projectedMetersForLatLng(LatLng coordinate) {
    return withNativeArena((arena) {
      final outMeters = arena<raw.mln_projected_meters>();
      _checkStatus(
        _c.projectedMetersForLatLng(
          native_struct.latLngToNative(coordinate),
          outMeters,
        ),
      );
      return native_struct.projectedMetersFromNative(outMeters.ref);
    });
  }

  /// Converts spherical Mercator projected meters to a geographic coordinate.
  static LatLng latLngForProjectedMeters(ProjectedMeters meters) {
    return withNativeArena((arena) {
      final outCoordinate = arena<raw.mln_lat_lng>();
      _checkStatus(
        _c.latLngForProjectedMeters(
          native_struct.projectedMetersToNative(meters),
          outCoordinate,
        ),
      );
      return native_struct.latLngFromNative(outCoordinate.ref);
    });
  }

  /// Restores MapLibre Native's default async log severity mask.
  static void restoreDefaultAsyncLogSeverityMask() {
    setAsyncLogSeverityMask(LogSeverityMask.defaultMask);
  }

  static void _checkStatus(int statusCode) {
    checkNativeStatus(statusCode, _c.threadLastErrorMessage);
  }
}

/// Render backend support flags reported by this native library build.
final class RenderBackendMask {
  /// Creates a backend mask from raw C flag bits.
  const RenderBackendMask(this.bits);

  /// Metal backend support bit.
  static const metal = RenderBackendMask(1 << 0);

  /// Vulkan backend support bit.
  static const vulkan = RenderBackendMask(1 << 1);

  /// Raw backend mask bits.
  final int bits;

  /// Returns true when all [backend] bits are present in this mask.
  bool contains(RenderBackendMask backend) =>
      (bits & backend.bits) == backend.bits;

  @override
  String toString() => 'RenderBackendMask[bits=0x${bits.toRadixString(16)}]';
}

/// Process-global network status.
final class NetworkStatus {
  const NetworkStatus._(this.rawValue, this.name, this._canSet);

  /// Network requests are allowed.
  static const online = NetworkStatus._(1, 'online', true);

  /// Online source network requests are paused.
  static const offline = NetworkStatus._(2, 'offline', true);

  /// Creates the public network-status value for a raw native value.
  factory NetworkStatus.fromRawValue(int rawValue) => switch (rawValue) {
    1 => online,
    2 => offline,
    _ => NetworkStatus._(rawValue, 'unknown', false),
  };

  /// Raw native value.
  final int rawValue;

  /// Human-readable status name.
  final String name;

  final bool _canSet;

  /// Returns the raw value for native setter calls.
  int rawValueForSet() {
    if (!_canSet) {
      throwInvalidArgument('unknown network status $rawValue cannot be set');
    }
    return rawValue;
  }

  @override
  String toString() => name == 'unknown' ? 'unknown($rawValue)' : name;
}

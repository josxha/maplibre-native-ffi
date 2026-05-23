import 'dart:ffi';

import 'geo/geo.dart';
import 'internal/c/maplibre_native_c.dart';
import 'internal/c/maplibre_native_c.g.dart' as raw;
import 'internal/memory/memory.dart';
import 'internal/status/status.dart';
import 'internal/struct/struct.dart' as native_struct;
import 'log/log.dart';

/// Process-global entry points for the Dart binding.
final class Maplibre {
  Maplibre._();

  static final MaplibreNativeCApi _c = MaplibreNativeCApi.open();

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

  /// Clears the process-global native log callback.
  static void clearLogCallback() {
    _checkStatus(_c.logClearCallback());
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

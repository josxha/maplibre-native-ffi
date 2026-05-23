import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'error/maplibre_exception.dart';
import 'internal/c/maplibre_native_c.dart';

/// Process-global entry points for the Dart binding.
final class Maplibre {
  Maplibre._();

  static final MaplibreNativeC _c = MaplibreNativeC.open();

  /// Returns the native C ABI contract version.
  static int cVersion() => _c.cVersion();

  /// Returns the render backends compiled into the linked native library.
  static RenderBackendMask supportedRenderBackends() =>
      RenderBackendMask(_c.supportedRenderBackendMask());

  /// Reads MapLibre Native's process-global network status.
  static NetworkStatus networkStatus() {
    final outStatus = calloc<Uint32>();
    try {
      _checkStatus(_c.networkStatusGet(outStatus));
      return NetworkStatus.fromRawValue(outStatus.value);
    } finally {
      calloc.free(outStatus);
    }
  }

  /// Sets MapLibre Native's process-global network status.
  static void setNetworkStatus(NetworkStatus status) {
    _checkStatus(_c.networkStatusSet(status.rawValueForSet()));
  }

  static void _checkStatus(int statusCode) {
    if (statusCode == MaplibreStatus.ok.nativeStatusCode) {
      return;
    }
    throw MaplibreException.forNativeStatusCode(
      statusCode,
      _c.threadLastErrorMessage(),
    );
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
      throw MaplibreException.invalidArgument(
        'unknown network status $rawValue cannot be set',
      );
    }
    return rawValue;
  }

  @override
  String toString() => name == 'unknown' ? 'unknown($rawValue)' : name;
}

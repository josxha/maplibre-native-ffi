import 'dart:ffi';

import 'package:ffi/ffi.dart';

import '../loader/native_library.dart';

/// Minimal raw C facade used by the current Dart proof slice.
///
/// Future implementation work replaces this hand-written subset with a facade
/// over the private `ffigen` output generated from `include/maplibre_native_c.h`.
final class MaplibreNativeC {
  MaplibreNativeC._(this.library)
    : _mlnCVersion = library.lookupFunction<_MlnCVersionNative, _MlnCVersion>(
        'mln_c_version',
      ),
      _mlnSupportedRenderBackendMask = library
          .lookupFunction<
            _MlnSupportedRenderBackendMaskNative,
            _MlnSupportedRenderBackendMask
          >('mln_supported_render_backend_mask'),
      _mlnThreadLastErrorMessage = library
          .lookupFunction<
            _MlnThreadLastErrorMessageNative,
            _MlnThreadLastErrorMessage
          >('mln_thread_last_error_message'),
      _mlnNetworkStatusGet = library
          .lookupFunction<_MlnNetworkStatusGetNative, _MlnNetworkStatusGet>(
            'mln_network_status_get',
          ),
      _mlnNetworkStatusSet = library
          .lookupFunction<_MlnNetworkStatusSetNative, _MlnNetworkStatusSet>(
            'mln_network_status_set',
          );

  /// Opens the native library and resolves the proof-slice C symbols.
  factory MaplibreNativeC.open({String? path}) =>
      MaplibreNativeC._(openMaplibreNativeCLibrary(path: path));

  /// Native library that owns the resolved C symbols.
  final DynamicLibrary library;
  final _MlnCVersion _mlnCVersion;
  final _MlnSupportedRenderBackendMask _mlnSupportedRenderBackendMask;
  final _MlnThreadLastErrorMessage _mlnThreadLastErrorMessage;
  final _MlnNetworkStatusGet _mlnNetworkStatusGet;
  final _MlnNetworkStatusSet _mlnNetworkStatusSet;

  /// Returns the native C ABI contract version.
  int cVersion() => _mlnCVersion();

  /// Returns the render backend support mask reported by the native library.
  int supportedRenderBackendMask() => _mlnSupportedRenderBackendMask();

  /// Copies the current thread-local native diagnostic message.
  String threadLastErrorMessage() {
    final pointer = _mlnThreadLastErrorMessage();
    if (pointer == nullptr) {
      return '';
    }
    return pointer.toDartString();
  }

  /// Reads MapLibre Native's process-global network status.
  int networkStatusGet(Pointer<Uint32> outStatus) =>
      _mlnNetworkStatusGet(outStatus);

  /// Sets MapLibre Native's process-global network status.
  int networkStatusSet(int status) => _mlnNetworkStatusSet(status);
}

typedef _MlnCVersionNative = Uint32 Function();
typedef _MlnCVersion = int Function();

typedef _MlnSupportedRenderBackendMaskNative = Uint32 Function();
typedef _MlnSupportedRenderBackendMask = int Function();

typedef _MlnThreadLastErrorMessageNative = Pointer<Utf8> Function();
typedef _MlnThreadLastErrorMessage = Pointer<Utf8> Function();

typedef _MlnNetworkStatusGetNative = Int32 Function(Pointer<Uint32> outStatus);
typedef _MlnNetworkStatusGet = int Function(Pointer<Uint32> outStatus);

typedef _MlnNetworkStatusSetNative = Int32 Function(Uint32 status);
typedef _MlnNetworkStatusSet = int Function(int status);

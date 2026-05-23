import 'dart:io';

import 'package:maplibre_native_ffi/maplibre_native_ffi.dart';
import 'package:test/test.dart';

void main() {
  final nativeLibraryPath = _nativeLibraryPath();
  final hasNativeLibrary =
      nativeLibraryPath != null && File(nativeLibraryPath).existsSync();

  test(
    'process-global proof slice crosses the native C ABI',
    () {
      expect(Maplibre.cVersion(), greaterThanOrEqualTo(0));
      expect(Maplibre.supportedRenderBackends().bits, greaterThanOrEqualTo(0));

      final status = Maplibre.networkStatus();
      expect(
        status.rawValue,
        isIn([NetworkStatus.online.rawValue, NetworkStatus.offline.rawValue]),
      );
      Maplibre.setNetworkStatus(status);
    },
    skip: hasNativeLibrary ? false : 'Native C library is not built.',
  );

  test('native pointer preserves address value semantics', () {
    const pointer = NativePointer(0x1234);

    expect(pointer.address, 0x1234);
    expect(pointer.isNull, isFalse);
    expect(NativePointer.nullPointer.isNull, isTrue);
  });
}

String? _nativeLibraryPath() {
  final explicitPath = Platform.environment['MLN_FFI_NATIVE_LIBRARY'];
  if (explicitPath != null && explicitPath.isNotEmpty) {
    return explicitPath;
  }

  final buildDir = Platform.environment['MLN_FFI_BUILD_DIR'];
  if (buildDir == null || buildDir.isEmpty) {
    return null;
  }

  return '$buildDir${Platform.pathSeparator}${_platformLibraryFileName()}';
}

String _platformLibraryFileName() {
  if (Platform.isMacOS || Platform.isIOS) {
    return 'libmaplibre-native-c.dylib';
  }
  if (Platform.isWindows) {
    return 'maplibre-native-c.dll';
  }
  return 'libmaplibre-native-c.so';
}

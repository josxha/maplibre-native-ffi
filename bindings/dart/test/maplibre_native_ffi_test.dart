import 'dart:io';

import 'package:maplibre_native_ffi/maplibre_native_ffi.dart';
import 'package:test/test.dart';

const _emptyStyleJson = '{"version":8,"sources":{},"layers":[]}';

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
      Maplibre.setAsyncLogSeverityMask(LogSeverityMask.defaultMask);
      Maplibre.restoreDefaultAsyncLogSeverityMask();
      Maplibre.clearLogCallback();
    },
    skip: hasNativeLibrary ? false : 'Native C library is not built.',
  );

  test(
    'runtime and map handles use the native C ABI',
    () {
      final runtime = RuntimeHandle.create();
      expect(runtime.isClosed, isFalse);

      final map = runtime.createMap();
      expect(map.isClosed, isFalse);
      map.setStyleJson(_emptyStyleJson);
      runtime.runOnce();
      runtime.drainEvents();
      final sourceIds = map.listStyleSourceIds();
      expect(sourceIds, contains('org.maplibre.annotations'));
      expect(
        map.listStyleLayerIds(),
        contains('org.maplibre.annotations.points'),
      );
      expect(map.styleSourceExists('missing-source'), isFalse);
      expect(map.styleLayerExists('missing-layer'), isFalse);
      expect(map.removeStyleSource('missing-source'), isFalse);
      expect(map.removeStyleLayer('missing-layer'), isFalse);

      map.addGeoJsonSourceData(
        'dart-geojson-source',
        const FeatureGeoJson(
          geometry: PointGeometry(LatLng(0, 0)),
          properties: [JsonMember('kind', JsonString('dart'))],
        ),
      );
      expect(map.styleSourceExists('dart-geojson-source'), isTrue);
      final info = map.getStyleSourceInfo('dart-geojson-source');
      expect(info, isNotNull);
      expect(info!.type, SourceType.geoJson);
      expect(info.id, 'dart-geojson-source');
      expect(info.attribution, isNull);
      expect(map.listStyleSourceIds(), contains('dart-geojson-source'));

      map.setGeoJsonSourceData(
        'dart-geojson-source',
        const GeometryGeoJson(PointGeometry(LatLng(1, 2))),
      );
      map.addStyleLayerJson(
        const JsonObject([
          JsonMember('id', JsonString('dart-circle-layer')),
          JsonMember('type', JsonString('circle')),
          JsonMember('source', JsonString('dart-geojson-source')),
        ]),
      );
      expect(map.styleLayerExists('dart-circle-layer'), isTrue);
      expect(map.getStyleLayerType('dart-circle-layer'), 'circle');
      expect(map.listStyleLayerIds(), contains('dart-circle-layer'));
      final layerJson = map.getStyleLayerJson('dart-circle-layer');
      expect(layerJson, isA<JsonObject>());
      expect(
        (layerJson! as JsonObject).members.map((member) => member.key),
        contains('id'),
      );

      map.setLayerProperty(
        'dart-circle-layer',
        'circle-radius',
        const JsonDouble(6.5),
      );
      expect(
        map.getLayerProperty('dart-circle-layer', 'circle-radius'),
        isA<JsonDouble>(),
      );
      map.setLayerFilter(
        'dart-circle-layer',
        const JsonArray([
          JsonString('=='),
          JsonArray([JsonString('get'), JsonString('kind')]),
          JsonString('dart'),
        ]),
      );
      expect(map.getLayerFilter('dart-circle-layer'), isA<JsonArray>());
      map.setLayerFilter('dart-circle-layer', null);
      expect(
        map.getLayerFilter('dart-circle-layer'),
        anyOf(isNull, isA<JsonNull>()),
      );

      expect(map.removeStyleLayer('dart-circle-layer'), isTrue);
      expect(map.removeStyleSource('dart-geojson-source'), isTrue);

      map.close();
      expect(map.isClosed, isTrue);
      runtime.close();
      expect(runtime.isClosed, isTrue);
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

import 'dart:ffi';

import 'package:maplibre_native_ffi/maplibre_native_ffi.dart';
import 'package:maplibre_native_ffi/src/internal/c/maplibre_native_c.g.dart'
    as raw;
import 'package:maplibre_native_ffi/src/internal/memory/memory.dart';
import 'package:maplibre_native_ffi/src/internal/struct/json.dart';
import 'package:maplibre_native_ffi/src/internal/struct/struct.dart';
import 'package:test/test.dart';

void main() {
  test('geographic values preserve fields', () {
    const coordinate = LatLng(45, -122);
    final native = latLngToNative(coordinate);

    expect(native.latitude, 45);
    expect(native.longitude, -122);
    expect(latLngFromNative(native), coordinate);
    expect(
      latLngBoundsToNative(
        const LatLngBounds(LatLng(1, 2), LatLng(3, 4)),
      ).northeast.longitude,
      4,
    );
  });

  test('camera options materialize field masks and semantic fields', () {
    final native = cameraOptionsToNative(
      const CameraOptions(
        center: LatLng(1, 2),
        zoom: 3,
        padding: EdgeInsets(top: 4, left: 5, bottom: 6, right: 7),
        fieldOfView: 8,
      ),
    );

    expect(
      native.fields &
          raw.mln_camera_option_field.MLN_CAMERA_OPTION_CENTER.value,
      isNonZero,
    );
    expect(
      native.fields & raw.mln_camera_option_field.MLN_CAMERA_OPTION_ZOOM.value,
      isNonZero,
    );
    expect(
      native.fields &
          raw.mln_camera_option_field.MLN_CAMERA_OPTION_PADDING.value,
      isNonZero,
    );
    expect(native.latitude, 1);
    expect(native.longitude, 2);
    expect(native.zoom, 3);
    expect(native.padding.bottom, 6);
    expect(native.field_of_view, 8);
  });

  test('animation options materialize field masks', () {
    final native = animationOptionsToNative(
      const AnimationOptions(
        durationMs: 100,
        easing: UnitBezier(0, 0.25, 0.75, 1),
      ),
    );

    expect(
      native.fields &
          raw.mln_animation_option_field.MLN_ANIMATION_OPTION_DURATION.value,
      isNonZero,
    );
    expect(
      native.fields &
          raw.mln_animation_option_field.MLN_ANIMATION_OPTION_EASING.value,
      isNonZero,
    );
    expect(native.duration_ms, 100);
    expect(native.easing.y2, 1);
  });

  test('JSON values materialize and copy native descriptor trees', () {
    withNativeArena((arena) {
      final native = nativeJsonValue(
        const JsonObject([
          JsonMember('name', JsonString('maplibre')),
          JsonMember('enabled', JsonBool(true)),
          JsonMember('values', JsonArray([JsonInt(-1), JsonUInt(2)])),
        ]),
        arena,
      );

      expect(
        native.pointer.ref.type,
        raw.mln_json_value_type.MLN_JSON_VALUE_TYPE_OBJECT.value,
      );
      final object = native.pointer.ref.data.object_value;
      expect(object.member_count, 3);
      expect(object.members[0].key.size, 4);
      expect(object.members[1].value.ref.data.bool_value, isTrue);

      final copied = jsonValueFromNative(native.pointer.ref);
      expect(copied, isA<JsonObject>());
      final copiedObject = copied as JsonObject;
      expect(copiedObject.members[0].key, 'name');
      expect((copiedObject.members[2].value as JsonArray).values.length, 2);
    });
  });

  test('JSON double descriptors reject non-finite values before C calls', () {
    expect(
      () => withNativeArena(
        (arena) => nativeJsonValue(const JsonDouble(double.nan), arena),
      ),
      throwsA(isA<InvalidArgumentException>()),
    );
  });

  test('public enum-like values preserve native raw values', () {
    expect(ResourceKind.tile.rawValue, 3);
    expect(ResourceLoadingMethod.all.rawValue, 0);
    expect(ResourceStoragePolicy.permanent.rawValue, 0);
    expect(ResourceResponseStatus.error.rawValue, 1);
    expect(SourceType.customVector.rawValue, 8);
    expect(TileScheme.tms.rawValue, 1);
  });
}

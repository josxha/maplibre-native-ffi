import 'dart:ffi';

import '../../camera/camera.dart';
import '../c/maplibre_native_c.g.dart' as raw;

/// Converts a Dart coordinate to the native C value shape.
raw.mln_lat_lng latLngToNative(LatLng value) {
  final result = Struct.create<raw.mln_lat_lng>();
  result.latitude = value.latitude;
  result.longitude = value.longitude;
  return result;
}

/// Converts a native coordinate to a Dart value.
LatLng latLngFromNative(raw.mln_lat_lng value) =>
    LatLng(value.latitude, value.longitude);

/// Converts Dart bounds to the native C value shape.
raw.mln_lat_lng_bounds latLngBoundsToNative(LatLngBounds value) {
  final result = Struct.create<raw.mln_lat_lng_bounds>();
  result.southwest = latLngToNative(value.southwest);
  result.northeast = latLngToNative(value.northeast);
  return result;
}

/// Converts Dart screen point to the native C value shape.
raw.mln_screen_point screenPointToNative(ScreenPoint value) {
  final result = Struct.create<raw.mln_screen_point>();
  result.x = value.x;
  result.y = value.y;
  return result;
}

/// Converts Dart edge insets to the native C value shape.
raw.mln_edge_insets edgeInsetsToNative(EdgeInsets value) {
  final result = Struct.create<raw.mln_edge_insets>();
  result.top = value.top;
  result.left = value.left;
  result.bottom = value.bottom;
  result.right = value.right;
  return result;
}

/// Converts Dart vector to the native C value shape.
raw.mln_vec3 vec3ToNative(Vec3 value) {
  final result = Struct.create<raw.mln_vec3>();
  result.x = value.x;
  result.y = value.y;
  result.z = value.z;
  return result;
}

/// Converts Dart quaternion to the native C value shape.
raw.mln_quaternion quaternionToNative(Quaternion value) {
  final result = Struct.create<raw.mln_quaternion>();
  result.x = value.x;
  result.y = value.y;
  result.z = value.z;
  result.w = value.w;
  return result;
}

/// Converts Dart unit bezier to the native C value shape.
raw.mln_unit_bezier unitBezierToNative(UnitBezier value) {
  final result = Struct.create<raw.mln_unit_bezier>();
  result.x1 = value.x1;
  result.y1 = value.y1;
  result.x2 = value.x2;
  result.y2 = value.y2;
  return result;
}

/// Materializes Dart camera options into a native C struct.
raw.mln_camera_options cameraOptionsToNative(CameraOptions value) {
  final result = Struct.create<raw.mln_camera_options>();
  result.size = sizeOf<raw.mln_camera_options>();
  final center = value.center;
  if (center != null) {
    result.fields |= raw.mln_camera_option_field.MLN_CAMERA_OPTION_CENTER.value;
    result.latitude = center.latitude;
    result.longitude = center.longitude;
  }
  final zoom = value.zoom;
  if (zoom != null) {
    result.fields |= raw.mln_camera_option_field.MLN_CAMERA_OPTION_ZOOM.value;
    result.zoom = zoom;
  }
  final bearing = value.bearing;
  if (bearing != null) {
    result.fields |=
        raw.mln_camera_option_field.MLN_CAMERA_OPTION_BEARING.value;
    result.bearing = bearing;
  }
  final pitch = value.pitch;
  if (pitch != null) {
    result.fields |= raw.mln_camera_option_field.MLN_CAMERA_OPTION_PITCH.value;
    result.pitch = pitch;
  }
  final centerAltitude = value.centerAltitude;
  if (centerAltitude != null) {
    result.fields |=
        raw.mln_camera_option_field.MLN_CAMERA_OPTION_CENTER_ALTITUDE.value;
    result.center_altitude = centerAltitude;
  }
  final padding = value.padding;
  if (padding != null) {
    result.fields |=
        raw.mln_camera_option_field.MLN_CAMERA_OPTION_PADDING.value;
    result.padding = edgeInsetsToNative(padding);
  }
  final anchor = value.anchor;
  if (anchor != null) {
    result.fields |= raw.mln_camera_option_field.MLN_CAMERA_OPTION_ANCHOR.value;
    result.anchor = screenPointToNative(anchor);
  }
  final roll = value.roll;
  if (roll != null) {
    result.fields |= raw.mln_camera_option_field.MLN_CAMERA_OPTION_ROLL.value;
    result.roll = roll;
  }
  final fieldOfView = value.fieldOfView;
  if (fieldOfView != null) {
    result.fields |= raw.mln_camera_option_field.MLN_CAMERA_OPTION_FOV.value;
    result.field_of_view = fieldOfView;
  }
  return result;
}

/// Materializes Dart animation options into a native C struct.
raw.mln_animation_options animationOptionsToNative(AnimationOptions value) {
  final result = Struct.create<raw.mln_animation_options>();
  result.size = sizeOf<raw.mln_animation_options>();
  final durationMs = value.durationMs;
  if (durationMs != null) {
    result.fields |=
        raw.mln_animation_option_field.MLN_ANIMATION_OPTION_DURATION.value;
    result.duration_ms = durationMs;
  }
  final velocity = value.velocity;
  if (velocity != null) {
    result.fields |=
        raw.mln_animation_option_field.MLN_ANIMATION_OPTION_VELOCITY.value;
    result.velocity = velocity;
  }
  final minZoom = value.minZoom;
  if (minZoom != null) {
    result.fields |=
        raw.mln_animation_option_field.MLN_ANIMATION_OPTION_MIN_ZOOM.value;
    result.min_zoom = minZoom;
  }
  final easing = value.easing;
  if (easing != null) {
    result.fields |=
        raw.mln_animation_option_field.MLN_ANIMATION_OPTION_EASING.value;
    result.easing = unitBezierToNative(easing);
  }
  return result;
}

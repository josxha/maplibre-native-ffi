/// Camera descriptor and map camera operation types.
library;

import '../geo/geo.dart';

export '../geo/geo.dart'
    show EdgeInsets, LatLng, LatLngBounds, Quaternion, ScreenPoint, Vec3;

/// Cubic easing curve for animated camera transitions.
final class UnitBezier {
  /// Creates a cubic unit bezier.
  const UnitBezier(this.x1, this.y1, this.x2, this.y2);

  /// First control point x.
  final double x1;

  /// First control point y.
  final double y1;

  /// Second control point x.
  final double x2;

  /// Second control point y.
  final double y2;

  @override
  bool operator ==(Object other) =>
      other is UnitBezier &&
      other.x1 == x1 &&
      other.y1 == y1 &&
      other.x2 == x2 &&
      other.y2 == y2;

  @override
  int get hashCode => Object.hash(x1, y1, x2, y2);
}

/// Camera fields used for snapshots and camera commands.
final class CameraOptions {
  /// Creates camera options.
  const CameraOptions({
    this.center,
    this.zoom,
    this.bearing,
    this.pitch,
    this.centerAltitude,
    this.padding,
    this.anchor,
    this.roll,
    this.fieldOfView,
  });

  /// Optional center coordinate.
  final LatLng? center;

  /// Optional zoom level.
  final double? zoom;

  /// Optional bearing in degrees.
  final double? bearing;

  /// Optional pitch in degrees.
  final double? pitch;

  /// Optional center altitude.
  final double? centerAltitude;

  /// Optional viewport padding.
  final EdgeInsets? padding;

  /// Optional screen anchor.
  final ScreenPoint? anchor;

  /// Optional roll in degrees.
  final double? roll;

  /// Optional field of view.
  final double? fieldOfView;
}

/// Optional animation controls for camera transitions.
final class AnimationOptions {
  /// Creates animation options.
  const AnimationOptions({
    this.durationMs,
    this.velocity,
    this.minZoom,
    this.easing,
  });

  /// Optional duration in milliseconds.
  final double? durationMs;

  /// Optional average fly-to velocity in screenfuls per second.
  final double? velocity;

  /// Optional peak zoom for fly-to transitions.
  final double? minZoom;

  /// Optional easing curve.
  final UnitBezier? easing;
}

/// Optional fitting controls for camera-for-viewport queries.
final class CameraFitOptions {
  /// Creates camera fit options.
  const CameraFitOptions({this.padding, this.bearing, this.pitch});

  /// Optional viewport padding.
  final EdgeInsets? padding;

  /// Optional bearing in degrees.
  final double? bearing;

  /// Optional pitch in degrees.
  final double? pitch;
}

/// Optional map camera constraint fields.
final class BoundOptions {
  /// Creates bound options.
  const BoundOptions({
    this.bounds,
    this.minZoom,
    this.maxZoom,
    this.minPitch,
    this.maxPitch,
  });

  /// Optional coordinate bounds.
  final LatLngBounds? bounds;

  /// Optional minimum zoom.
  final double? minZoom;

  /// Optional maximum zoom.
  final double? maxZoom;

  /// Optional minimum pitch.
  final double? minPitch;

  /// Optional maximum pitch.
  final double? maxPitch;
}

/// Free camera position and orientation in MapLibre Native camera space.
final class FreeCameraOptions {
  /// Creates free camera options.
  const FreeCameraOptions({this.position, this.orientation});

  /// Optional position.
  final Vec3? position;

  /// Optional orientation.
  final Quaternion? orientation;
}

/// MapLibre axonometric rendering options used for snapshots and commands.
final class ProjectionModeOptions {
  /// Creates projection mode options.
  const ProjectionModeOptions({this.axonometric, this.xSkew, this.ySkew});

  /// Optional axonometric mode switch.
  final bool? axonometric;

  /// Optional native x-skew factor.
  final double? xSkew;

  /// Optional native y-skew factor.
  final double? ySkew;
}

package org.maplibre.nativeffi.map

import org.maplibre.nativeffi.camera.AnimationOptions
import org.maplibre.nativeffi.camera.BoundOptions
import org.maplibre.nativeffi.camera.CameraFitOptions
import org.maplibre.nativeffi.camera.CameraOptions
import org.maplibre.nativeffi.camera.FreeCameraOptions
import org.maplibre.nativeffi.geo.Geometry
import org.maplibre.nativeffi.geo.LatLng
import org.maplibre.nativeffi.geo.LatLngBounds
import org.maplibre.nativeffi.geo.ScreenPoint
import org.maplibre.nativeffi.runtime.RuntimeHandle

/** JVM actual placeholder until the FFM map bridge is migrated. */
public actual class MapHandle private constructor() : AutoCloseable {
  public actual val isClosed: Boolean
    get() = unsupportedMapHandle()

  public actual fun runtime(): RuntimeHandle = unsupportedMapHandle()

  public actual fun requestRepaint() {
    unsupportedMapHandle()
  }

  public actual fun requestStillImage() {
    unsupportedMapHandle()
  }

  public actual var debugOptions: Set<DebugOption>
    get() = unsupportedMapHandle()
    set(value) {
      unsupportedMapHandle()
    }

  public actual var isRenderingStatsViewEnabled: Boolean
    get() = unsupportedMapHandle()
    set(value) {
      unsupportedMapHandle()
    }

  public actual val isFullyLoaded: Boolean
    get() = unsupportedMapHandle()

  public actual fun dumpDebugLogs() {
    unsupportedMapHandle()
  }

  public actual var viewportOptions: ViewportOptions
    get() = unsupportedMapHandle()
    set(value) {
      unsupportedMapHandle()
    }

  public actual var tileOptions: TileOptions
    get() = unsupportedMapHandle()
    set(value) {
      unsupportedMapHandle()
    }

  public actual val camera: CameraOptions
    get() = unsupportedMapHandle()

  public actual fun jumpTo(camera: CameraOptions) {
    unsupportedMapHandle()
  }

  public actual fun easeTo(camera: CameraOptions, animation: AnimationOptions?) {
    unsupportedMapHandle()
  }

  public actual fun flyTo(camera: CameraOptions, animation: AnimationOptions?) {
    unsupportedMapHandle()
  }

  public actual fun moveBy(deltaX: Double, deltaY: Double) {
    unsupportedMapHandle()
  }

  public actual fun moveByAnimated(deltaX: Double, deltaY: Double, animation: AnimationOptions?) {
    unsupportedMapHandle()
  }

  public actual fun scaleBy(scale: Double, anchor: ScreenPoint?) {
    unsupportedMapHandle()
  }

  public actual fun scaleByAnimated(
    scale: Double,
    anchor: ScreenPoint?,
    animation: AnimationOptions?,
  ) {
    unsupportedMapHandle()
  }

  public actual fun rotateBy(first: ScreenPoint, second: ScreenPoint) {
    unsupportedMapHandle()
  }

  public actual fun rotateByAnimated(
    first: ScreenPoint,
    second: ScreenPoint,
    animation: AnimationOptions?,
  ) {
    unsupportedMapHandle()
  }

  public actual fun pitchBy(pitch: Double) {
    unsupportedMapHandle()
  }

  public actual fun pitchByAnimated(pitch: Double, animation: AnimationOptions?) {
    unsupportedMapHandle()
  }

  public actual fun cancelTransitions() {
    unsupportedMapHandle()
  }

  public actual fun cameraForLatLngBounds(
    bounds: LatLngBounds,
    fitOptions: CameraFitOptions?,
  ): CameraOptions = unsupportedMapHandle()

  public actual fun cameraForLatLngs(
    coordinates: List<LatLng>,
    fitOptions: CameraFitOptions?,
  ): CameraOptions = unsupportedMapHandle()

  public actual fun cameraForGeometry(
    geometry: Geometry,
    fitOptions: CameraFitOptions?,
  ): CameraOptions = unsupportedMapHandle()

  public actual fun latLngBoundsForCamera(camera: CameraOptions): LatLngBounds =
    unsupportedMapHandle()

  public actual fun latLngBoundsForCameraUnwrapped(camera: CameraOptions): LatLngBounds =
    unsupportedMapHandle()

  public actual var bounds: BoundOptions
    get() = unsupportedMapHandle()
    set(value) {
      unsupportedMapHandle()
    }

  public actual var freeCameraOptions: FreeCameraOptions
    get() = unsupportedMapHandle()
    set(value) {
      unsupportedMapHandle()
    }

  public actual var projectionMode: ProjectionModeOptions
    get() = unsupportedMapHandle()
    set(value) {
      unsupportedMapHandle()
    }

  public actual fun pixelForLatLng(coordinate: LatLng): ScreenPoint = unsupportedMapHandle()

  public actual fun latLngForPixel(point: ScreenPoint): LatLng = unsupportedMapHandle()

  public actual fun pixelsForLatLngs(coordinates: List<LatLng>): List<ScreenPoint> =
    unsupportedMapHandle()

  public actual fun latLngsForPixels(points: List<ScreenPoint>): List<LatLng> =
    unsupportedMapHandle()

  public actual fun createProjection(): MapProjectionHandle = unsupportedMapHandle()

  public actual override fun close() {
    unsupportedMapHandle()
  }
}

private fun unsupportedMapHandle(): Nothing =
  throw UnsupportedOperationException(
    "MapHandle is not available until the JVM map bridge is implemented"
  )

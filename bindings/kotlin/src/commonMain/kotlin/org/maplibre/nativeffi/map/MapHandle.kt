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

/** Owned map handle. Platform actuals own the native map carrier. */
public expect class MapHandle : AutoCloseable {
  public val isClosed: Boolean

  public fun runtime(): RuntimeHandle

  public fun requestRepaint()

  public fun requestStillImage()

  public var debugOptions: Set<DebugOption>

  public var isRenderingStatsViewEnabled: Boolean

  public val isFullyLoaded: Boolean

  public fun dumpDebugLogs()

  public var viewportOptions: ViewportOptions

  public var tileOptions: TileOptions

  public val camera: CameraOptions

  public fun jumpTo(camera: CameraOptions)

  public fun easeTo(camera: CameraOptions, animation: AnimationOptions?)

  public fun flyTo(camera: CameraOptions, animation: AnimationOptions?)

  public fun moveBy(deltaX: Double, deltaY: Double)

  public fun moveByAnimated(deltaX: Double, deltaY: Double, animation: AnimationOptions?)

  public fun scaleBy(scale: Double, anchor: ScreenPoint?)

  public fun scaleByAnimated(scale: Double, anchor: ScreenPoint?, animation: AnimationOptions?)

  public fun rotateBy(first: ScreenPoint, second: ScreenPoint)

  public fun rotateByAnimated(first: ScreenPoint, second: ScreenPoint, animation: AnimationOptions?)

  public fun pitchBy(pitch: Double)

  public fun pitchByAnimated(pitch: Double, animation: AnimationOptions?)

  public fun cancelTransitions()

  public fun cameraForLatLngBounds(
    bounds: LatLngBounds,
    fitOptions: CameraFitOptions?,
  ): CameraOptions

  public fun cameraForLatLngs(
    coordinates: List<LatLng>,
    fitOptions: CameraFitOptions?,
  ): CameraOptions

  public fun cameraForGeometry(geometry: Geometry, fitOptions: CameraFitOptions?): CameraOptions

  public fun latLngBoundsForCamera(camera: CameraOptions): LatLngBounds

  public fun latLngBoundsForCameraUnwrapped(camera: CameraOptions): LatLngBounds

  public var bounds: BoundOptions

  public var freeCameraOptions: FreeCameraOptions

  public var projectionMode: ProjectionModeOptions

  public fun pixelForLatLng(coordinate: LatLng): ScreenPoint

  public fun latLngForPixel(point: ScreenPoint): LatLng

  public fun pixelsForLatLngs(coordinates: List<LatLng>): List<ScreenPoint>

  public fun latLngsForPixels(points: List<ScreenPoint>): List<LatLng>

  public fun createProjection(): MapProjectionHandle

  override fun close()
}

package org.maplibre.nativeffi.map

import org.bytedeco.javacpp.Pointer
import org.maplibre.nativeffi.NativeAccess
import org.maplibre.nativeffi.camera.CameraOptions
import org.maplibre.nativeffi.camera.EdgeInsets
import org.maplibre.nativeffi.geo.Geometry
import org.maplibre.nativeffi.geo.LatLng
import org.maplibre.nativeffi.geo.ScreenPoint
import org.maplibre.nativeffi.internal.javacpp.MaplibreNativeC
import org.maplibre.nativeffi.internal.lifecycle.HandleStateCore
import org.maplibre.nativeffi.internal.status.Status

/** Owned Android JNI standalone projection snapshot. */
public actual class MapProjectionHandle internal constructor(private val handleAddress: Long) :
  AutoCloseable {
  private val core = HandleStateCore("MapProjectionHandle", handleAddress)

  public actual val camera: CameraOptions
    get() = unsupportedMapProjectionCamera()

  public actual fun setCamera(camera: CameraOptions) {
    unsupportedMapProjectionCamera()
  }

  public actual fun setVisibleCoordinates(coordinates: List<LatLng>, padding: EdgeInsets) {
    unsupportedMapProjectionCamera()
  }

  public actual fun setVisibleGeometry(geometry: Geometry, padding: EdgeInsets) {
    unsupportedMapProjectionCamera()
  }

  public actual fun pixelForLatLng(coordinate: LatLng): ScreenPoint {
    NativeAccess.ensureLoaded()
    MaplibreNativeC.mln_screen_point().use { outPoint ->
      Status.check(
        MaplibreNativeC.mln_map_projection_pixel_for_lat_lng(
          projection(requireLiveAddress()),
          MaplibreNativeC.mln_lat_lng()
            .latitude(coordinate.latitude)
            .longitude(coordinate.longitude),
          outPoint,
        )
      )
      return ScreenPoint(outPoint.x(), outPoint.y())
    }
  }

  public actual fun latLngForPixel(point: ScreenPoint): LatLng {
    NativeAccess.ensureLoaded()
    MaplibreNativeC.mln_lat_lng().use { outCoordinate ->
      Status.check(
        MaplibreNativeC.mln_map_projection_lat_lng_for_pixel(
          projection(requireLiveAddress()),
          MaplibreNativeC.mln_screen_point().x(point.x).y(point.y),
          outCoordinate,
        )
      )
      return LatLng(outCoordinate.latitude(), outCoordinate.longitude())
    }
  }

  public actual val isClosed: Boolean
    get() = core.isReleased()

  public actual override fun close() {
    core.closeOnce(
      destroy = { MaplibreNativeC.mln_map_projection_destroy(projection(handleAddress)) }
    )
  }

  private fun requireLiveAddress(): Long {
    core.requireLive()
    return handleAddress
  }
}

private fun projection(address: Long): MaplibreNativeC.mln_map_projection =
  MaplibreNativeC.mln_map_projection(ProjectionAddressPointer(address))

private class ProjectionAddressPointer(address: Long) : Pointer(null as Pointer?) {
  init {
    this.address = address
  }
}

private fun unsupportedMapProjectionCamera(): Nothing =
  throw UnsupportedOperationException(
    "MapProjectionHandle camera fitting is not available until the Android camera bridge is implemented"
  )

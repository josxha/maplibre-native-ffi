package org.maplibre.nativeffi.map

import org.maplibre.nativeffi.camera.CameraOptions
import org.maplibre.nativeffi.camera.EdgeInsets
import org.maplibre.nativeffi.geo.Geometry
import org.maplibre.nativeffi.geo.LatLng
import org.maplibre.nativeffi.geo.ScreenPoint

/** JVM actual placeholder until the FFM map projection bridge is migrated. */
public actual class MapProjectionHandle private constructor() : AutoCloseable {
  public actual val camera: CameraOptions
    get() = unsupportedMapProjectionHandle()

  public actual fun setCamera(camera: CameraOptions) {
    unsupportedMapProjectionHandle()
  }

  public actual fun setVisibleCoordinates(coordinates: List<LatLng>, padding: EdgeInsets) {
    unsupportedMapProjectionHandle()
  }

  public actual fun setVisibleGeometry(geometry: Geometry, padding: EdgeInsets) {
    unsupportedMapProjectionHandle()
  }

  public actual fun pixelForLatLng(coordinate: LatLng): ScreenPoint =
    unsupportedMapProjectionHandle()

  public actual fun latLngForPixel(point: ScreenPoint): LatLng = unsupportedMapProjectionHandle()

  public actual val isClosed: Boolean
    get() = unsupportedMapProjectionHandle()

  public actual override fun close() {
    unsupportedMapProjectionHandle()
  }
}

private fun unsupportedMapProjectionHandle(): Nothing =
  throw UnsupportedOperationException(
    "MapProjectionHandle is not available until the JVM map bridge is implemented"
  )

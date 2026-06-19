package org.maplibre.nativeffi.map

import org.maplibre.nativeffi.camera.AnimationOptions
import org.maplibre.nativeffi.camera.BoundOptions
import org.maplibre.nativeffi.camera.CameraFitOptions
import org.maplibre.nativeffi.camera.CameraOptions
import org.maplibre.nativeffi.camera.FreeCameraOptions
import org.maplibre.nativeffi.geo.CanonicalTileId
import org.maplibre.nativeffi.geo.GeoJson
import org.maplibre.nativeffi.geo.Geometry
import org.maplibre.nativeffi.geo.LatLng
import org.maplibre.nativeffi.geo.LatLngBounds
import org.maplibre.nativeffi.geo.ScreenPoint
import org.maplibre.nativeffi.internal.lifecycle.HandleStateCore
import org.maplibre.nativeffi.internal.loader.NativeAccess
import org.maplibre.nativeffi.json.JsonValue
import org.maplibre.nativeffi.render.MetalBorrowedTextureDescriptor
import org.maplibre.nativeffi.render.MetalOwnedTextureDescriptor
import org.maplibre.nativeffi.render.MetalSurfaceDescriptor
import org.maplibre.nativeffi.render.OpenGLBorrowedTextureDescriptor
import org.maplibre.nativeffi.render.OpenGLOwnedTextureDescriptor
import org.maplibre.nativeffi.render.OpenGLSurfaceDescriptor
import org.maplibre.nativeffi.render.PremultipliedRgba8Image
import org.maplibre.nativeffi.render.RenderSessionHandle
import org.maplibre.nativeffi.render.VulkanBorrowedTextureDescriptor
import org.maplibre.nativeffi.render.VulkanOwnedTextureDescriptor
import org.maplibre.nativeffi.render.VulkanSurfaceDescriptor
import org.maplibre.nativeffi.runtime.RuntimeHandle
import org.maplibre.nativeffi.style.CustomGeometrySourceOptions
import org.maplibre.nativeffi.style.LocationIndicatorImageKind
import org.maplibre.nativeffi.style.SourceInfo
import org.maplibre.nativeffi.style.SourceType
import org.maplibre.nativeffi.style.StyleImage
import org.maplibre.nativeffi.style.StyleImageInfo
import org.maplibre.nativeffi.style.StyleImageOptions
import org.maplibre.nativeffi.style.TileSourceOptions

/** Owned JVM FFM map handle. */
public actual class MapHandle
private constructor(
  private val runtime: RuntimeHandle,
  private val handle: java.lang.foreign.MemorySegment,
) : AutoCloseable {
  private val runtimeRetention = runtime.retainChild()
  private val core = HandleStateCore("MapHandle", handle.address())

  public actual val isClosed: Boolean
    get() = core.isReleased()

  public actual fun runtime(): RuntimeHandle = runtime

  public actual fun setStyleUrl(url: String) {
    NativeAccess.ensureLoaded()
    NativeAccess.setMapStyleUrl(requireLiveHandle(), url)
  }

  public actual fun setStyleJson(json: String) {
    NativeAccess.ensureLoaded()
    NativeAccess.setMapStyleJson(requireLiveHandle(), json)
  }

  public actual fun addStyleSourceJson(sourceId: String, sourceJson: JsonValue) {
    NativeAccess.ensureLoaded()
    NativeAccess.addStyleSourceJson(requireLiveHandle(), sourceId, sourceJson)
  }

  public actual fun removeStyleSource(sourceId: String): Boolean {
    NativeAccess.ensureLoaded()
    return NativeAccess.removeStyleSource(requireLiveHandle(), sourceId)
  }

  public actual fun styleSourceExists(sourceId: String): Boolean {
    NativeAccess.ensureLoaded()
    return NativeAccess.styleSourceExists(requireLiveHandle(), sourceId)
  }

  public actual fun styleSourceType(sourceId: String): SourceType? {
    NativeAccess.ensureLoaded()
    return NativeAccess.styleSourceType(requireLiveHandle(), sourceId)
  }

  public actual fun styleSourceInfo(sourceId: String): SourceInfo? {
    NativeAccess.ensureLoaded()
    return NativeAccess.styleSourceInfo(requireLiveHandle(), sourceId)
  }

  public actual fun styleSourceIds(): List<String> {
    NativeAccess.ensureLoaded()
    return NativeAccess.styleSourceIds(requireLiveHandle())
  }

  public actual fun addGeoJsonSourceUrl(sourceId: String, url: String) {
    unsupportedMapHandle()
  }

  public actual fun addGeoJsonSourceData(sourceId: String, data: GeoJson) {
    unsupportedMapHandle()
  }

  public actual fun setGeoJsonSourceUrl(sourceId: String, url: String) {
    unsupportedMapHandle()
  }

  public actual fun setGeoJsonSourceData(sourceId: String, data: GeoJson) {
    unsupportedMapHandle()
  }

  public actual fun addCustomGeometrySource(
    sourceId: String,
    options: CustomGeometrySourceOptions,
  ) {
    unsupportedMapHandle()
  }

  public actual fun setCustomGeometrySourceTileData(
    sourceId: String,
    tileId: CanonicalTileId,
    data: GeoJson,
  ) {
    unsupportedMapHandle()
  }

  public actual fun invalidateCustomGeometrySourceTile(sourceId: String, tileId: CanonicalTileId) {
    unsupportedMapHandle()
  }

  public actual fun invalidateCustomGeometrySourceRegion(sourceId: String, bounds: LatLngBounds) {
    unsupportedMapHandle()
  }

  public actual fun addVectorSourceUrl(sourceId: String, url: String, options: TileSourceOptions?) {
    unsupportedMapHandle()
  }

  public actual fun addVectorSourceTiles(
    sourceId: String,
    tiles: List<String>,
    options: TileSourceOptions?,
  ) {
    unsupportedMapHandle()
  }

  public actual fun addRasterSourceUrl(sourceId: String, url: String, options: TileSourceOptions?) {
    unsupportedMapHandle()
  }

  public actual fun addRasterSourceTiles(
    sourceId: String,
    tiles: List<String>,
    options: TileSourceOptions?,
  ) {
    unsupportedMapHandle()
  }

  public actual fun addRasterDemSourceUrl(
    sourceId: String,
    url: String,
    options: TileSourceOptions?,
  ) {
    unsupportedMapHandle()
  }

  public actual fun addRasterDemSourceTiles(
    sourceId: String,
    tiles: List<String>,
    options: TileSourceOptions?,
  ) {
    unsupportedMapHandle()
  }

  public actual fun setStyleImage(
    imageId: String,
    image: PremultipliedRgba8Image,
    options: StyleImageOptions,
  ) {
    unsupportedMapHandle()
  }

  public actual fun removeStyleImage(imageId: String): Boolean = unsupportedMapHandle()

  public actual fun styleImageExists(imageId: String): Boolean = unsupportedMapHandle()

  public actual fun styleImageInfo(imageId: String): StyleImageInfo? = unsupportedMapHandle()

  public actual fun copyStyleImagePremultipliedRgba8(imageId: String): StyleImage? =
    unsupportedMapHandle()

  public actual fun addImageSourceUrl(sourceId: String, coordinates: List<LatLng>, url: String) {
    unsupportedMapHandle()
  }

  public actual fun addImageSourceImage(
    sourceId: String,
    coordinates: List<LatLng>,
    image: PremultipliedRgba8Image,
  ) {
    unsupportedMapHandle()
  }

  public actual fun setImageSourceUrl(sourceId: String, url: String) {
    unsupportedMapHandle()
  }

  public actual fun setImageSourceImage(sourceId: String, image: PremultipliedRgba8Image) {
    unsupportedMapHandle()
  }

  public actual fun setImageSourceCoordinates(sourceId: String, coordinates: List<LatLng>) {
    unsupportedMapHandle()
  }

  public actual fun imageSourceCoordinates(sourceId: String): List<LatLng>? = unsupportedMapHandle()

  public actual fun addStyleLayerJson(layerJson: JsonValue, beforeLayerId: String) {
    unsupportedMapHandle()
  }

  public actual fun addHillshadeLayer(layerId: String, sourceId: String, beforeLayerId: String) {
    unsupportedMapHandle()
  }

  public actual fun addColorReliefLayer(layerId: String, sourceId: String, beforeLayerId: String) {
    unsupportedMapHandle()
  }

  public actual fun addLocationIndicatorLayer(layerId: String, beforeLayerId: String) {
    unsupportedMapHandle()
  }

  public actual fun setLocationIndicatorLocation(
    layerId: String,
    coordinate: LatLng,
    altitude: Double,
  ) {
    unsupportedMapHandle()
  }

  public actual fun setLocationIndicatorBearing(layerId: String, bearing: Double) {
    unsupportedMapHandle()
  }

  public actual fun setLocationIndicatorAccuracyRadius(layerId: String, radius: Double) {
    unsupportedMapHandle()
  }

  public actual fun setLocationIndicatorImageName(
    layerId: String,
    imageKind: LocationIndicatorImageKind,
    imageId: String,
  ) {
    unsupportedMapHandle()
  }

  public actual fun removeStyleLayer(layerId: String): Boolean = unsupportedMapHandle()

  public actual fun styleLayerExists(layerId: String): Boolean = unsupportedMapHandle()

  public actual fun styleLayerType(layerId: String): String? = unsupportedMapHandle()

  public actual fun styleLayerIds(): List<String> = unsupportedMapHandle()

  public actual fun moveStyleLayer(layerId: String, beforeLayerId: String) {
    unsupportedMapHandle()
  }

  public actual fun styleLayerJson(layerId: String): JsonValue? = unsupportedMapHandle()

  public actual fun setStyleLightJson(lightJson: JsonValue) {
    unsupportedMapHandle()
  }

  public actual fun setStyleLightProperty(propertyName: String, value: JsonValue) {
    unsupportedMapHandle()
  }

  public actual fun styleLightProperty(propertyName: String): JsonValue? = unsupportedMapHandle()

  public actual fun setLayerProperty(layerId: String, propertyName: String, value: JsonValue) {
    unsupportedMapHandle()
  }

  public actual fun layerProperty(layerId: String, propertyName: String): JsonValue? =
    unsupportedMapHandle()

  public actual fun setLayerFilter(layerId: String, filter: JsonValue) {
    unsupportedMapHandle()
  }

  public actual fun clearLayerFilter(layerId: String) {
    unsupportedMapHandle()
  }

  public actual fun layerFilter(layerId: String): JsonValue? = unsupportedMapHandle()

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

  public actual fun attachMetalOwnedTexture(
    descriptor: MetalOwnedTextureDescriptor
  ): RenderSessionHandle = unsupportedMapHandle()

  public actual fun attachMetalBorrowedTexture(
    descriptor: MetalBorrowedTextureDescriptor
  ): RenderSessionHandle = unsupportedMapHandle()

  public actual fun attachVulkanOwnedTexture(
    descriptor: VulkanOwnedTextureDescriptor
  ): RenderSessionHandle = unsupportedMapHandle()

  public actual fun attachVulkanBorrowedTexture(
    descriptor: VulkanBorrowedTextureDescriptor
  ): RenderSessionHandle = unsupportedMapHandle()

  public actual fun attachOpenGLOwnedTexture(
    descriptor: OpenGLOwnedTextureDescriptor
  ): RenderSessionHandle = unsupportedMapHandle()

  public actual fun attachOpenGLBorrowedTexture(
    descriptor: OpenGLBorrowedTextureDescriptor
  ): RenderSessionHandle = unsupportedMapHandle()

  public actual fun attachMetalSurface(descriptor: MetalSurfaceDescriptor): RenderSessionHandle =
    unsupportedMapHandle()

  public actual fun attachVulkanSurface(descriptor: VulkanSurfaceDescriptor): RenderSessionHandle =
    unsupportedMapHandle()

  public actual fun attachOpenGLSurface(descriptor: OpenGLSurfaceDescriptor): RenderSessionHandle =
    unsupportedMapHandle()

  public actual fun createProjection(): MapProjectionHandle = unsupportedMapHandle()

  public actual override fun close() {
    core.closeOnce(
      destroy = { NativeAccess.destroyMap(handle) },
      afterSuccess = { runtimeRetention.close() },
    )
  }

  public actual companion object {
    public actual fun create(runtime: RuntimeHandle, options: MapOptions): MapHandle {
      NativeAccess.ensureLoaded()
      return MapHandle(runtime, NativeAccess.createMap(runtime.nativeHandle(), options))
    }
  }

  private fun requireLiveHandle(): java.lang.foreign.MemorySegment {
    core.requireLive()
    return handle
  }
}

private fun unsupportedMapHandle(): Nothing =
  throw UnsupportedOperationException(
    "MapHandle is not available until the JVM map bridge is implemented"
  )

package org.maplibre.nativeffi.map

import org.bytedeco.javacpp.BytePointer
import org.bytedeco.javacpp.Pointer
import org.bytedeco.javacpp.PointerPointer
import org.bytedeco.javacpp.SizeTPointer
import org.maplibre.nativeffi.NativeAccess
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
import org.maplibre.nativeffi.internal.javacpp.MaplibreNativeC
import org.maplibre.nativeffi.internal.lifecycle.HandleStateCore
import org.maplibre.nativeffi.internal.status.Status
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

/** Owned Android JNI map handle. */
public actual class MapHandle
private constructor(private val runtime: RuntimeHandle, private val handleAddress: Long) :
  AutoCloseable {
  private val runtimeRetention = runtime.retainChild()
  private val core = HandleStateCore("MapHandle", handleAddress)

  public actual val isClosed: Boolean
    get() = core.isReleased()

  public actual fun runtime(): RuntimeHandle = runtime

  public actual fun setStyleUrl(url: String) {
    NativeAccess.ensureLoaded()
    optionalCString(url).use { nativeUrl ->
      Status.check(MaplibreNativeC.mln_map_set_style_url(map(requireLiveAddress()), nativeUrl))
    }
  }

  public actual fun setStyleJson(json: String) {
    NativeAccess.ensureLoaded()
    optionalCString(json).use { nativeJson ->
      Status.check(MaplibreNativeC.mln_map_set_style_json(map(requireLiveAddress()), nativeJson))
    }
  }

  public actual fun addStyleSourceJson(sourceId: String, sourceJson: JsonValue) {
    NativeAccess.ensureLoaded()
    StringViewScope(sourceId).use { nativeSourceId ->
      JsonScope(sourceJson).use { nativeSourceJson ->
        Status.check(
          MaplibreNativeC.mln_map_add_style_source_json(
            map(requireLiveAddress()),
            nativeSourceId.view,
            nativeSourceJson.value,
          )
        )
      }
    }
  }

  public actual fun removeStyleSource(sourceId: String): Boolean {
    NativeAccess.ensureLoaded()
    val outRemoved = booleanArrayOf(false)
    StringViewScope(sourceId).use { nativeSourceId ->
      Status.check(
        MaplibreNativeC.mln_map_remove_style_source(
          map(requireLiveAddress()),
          nativeSourceId.view,
          outRemoved,
        )
      )
    }
    return outRemoved[0]
  }

  public actual fun styleSourceExists(sourceId: String): Boolean {
    NativeAccess.ensureLoaded()
    val outExists = booleanArrayOf(false)
    StringViewScope(sourceId).use { nativeSourceId ->
      Status.check(
        MaplibreNativeC.mln_map_style_source_exists(
          map(requireLiveAddress()),
          nativeSourceId.view,
          outExists,
        )
      )
    }
    return outExists[0]
  }

  public actual fun styleSourceType(sourceId: String): SourceType? {
    NativeAccess.ensureLoaded()
    val outType = intArrayOf(0)
    val outFound = booleanArrayOf(false)
    StringViewScope(sourceId).use { nativeSourceId ->
      Status.check(
        MaplibreNativeC.mln_map_get_style_source_type(
          map(requireLiveAddress()),
          nativeSourceId.view,
          outType,
          outFound,
        )
      )
    }
    return if (outFound[0]) SourceType.fromNative(outType[0]) else null
  }

  public actual fun styleSourceInfo(sourceId: String): SourceInfo? {
    NativeAccess.ensureLoaded()
    StringViewScope(sourceId).use { nativeSourceId ->
      MaplibreNativeC.mln_style_source_info().use { outInfo ->
        outInfo.size(outInfo.sizeof())
        val outFound = booleanArrayOf(false)
        Status.check(
          MaplibreNativeC.mln_map_get_style_source_info(
            map(requireLiveAddress()),
            nativeSourceId.view,
            outInfo,
            outFound,
          )
        )
        if (!outFound[0]) return null
        val attribution =
          if (outInfo.has_attribution())
            copyStyleSourceAttribution(requireLiveAddress(), nativeSourceId, outInfo)
          else null
        return SourceInfo(SourceType.fromNative(outInfo.type()), outInfo.is_volatile(), attribution)
      }
    }
  }

  public actual fun styleSourceIds(): List<String> {
    NativeAccess.ensureLoaded()
    PointerPointer<MaplibreNativeC.mln_style_id_list>(1).use { outList ->
      outList.put(0, null as Pointer?)
      Status.check(
        MaplibreNativeC.mln_map_list_style_source_ids(map(requireLiveAddress()), outList)
      )
      val list = outList.get(MaplibreNativeC.mln_style_id_list::class.java, 0)
      return styleIdList(requireNotNull(list))
    }
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
    NativeAccess.ensureLoaded()
    JsonScope(layerJson).use { nativeLayerJson ->
      StringViewScope(beforeLayerId).use { nativeBeforeLayerId ->
        Status.check(
          MaplibreNativeC.mln_map_add_style_layer_json(
            map(requireLiveAddress()),
            nativeLayerJson.value,
            nativeBeforeLayerId.view,
          )
        )
      }
    }
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

  public actual fun removeStyleLayer(layerId: String): Boolean {
    NativeAccess.ensureLoaded()
    val outRemoved = booleanArrayOf(false)
    StringViewScope(layerId).use { nativeLayerId ->
      Status.check(
        MaplibreNativeC.mln_map_remove_style_layer(
          map(requireLiveAddress()),
          nativeLayerId.view,
          outRemoved,
        )
      )
    }
    return outRemoved[0]
  }

  public actual fun styleLayerExists(layerId: String): Boolean {
    NativeAccess.ensureLoaded()
    val outExists = booleanArrayOf(false)
    StringViewScope(layerId).use { nativeLayerId ->
      Status.check(
        MaplibreNativeC.mln_map_style_layer_exists(
          map(requireLiveAddress()),
          nativeLayerId.view,
          outExists,
        )
      )
    }
    return outExists[0]
  }

  public actual fun styleLayerType(layerId: String): String? {
    NativeAccess.ensureLoaded()
    val outFound = booleanArrayOf(false)
    MaplibreNativeC.mln_string_view().use { outType ->
      StringViewScope(layerId).use { nativeLayerId ->
        Status.check(
          MaplibreNativeC.mln_map_get_style_layer_type(
            map(requireLiveAddress()),
            nativeLayerId.view,
            outType,
            outFound,
          )
        )
      }
      return if (outFound[0]) stringView(outType) else null
    }
  }

  public actual fun styleLayerIds(): List<String> {
    NativeAccess.ensureLoaded()
    PointerPointer<MaplibreNativeC.mln_style_id_list>(1).use { outList ->
      outList.put(0, null as Pointer?)
      Status.check(MaplibreNativeC.mln_map_list_style_layer_ids(map(requireLiveAddress()), outList))
      val list = outList.get(MaplibreNativeC.mln_style_id_list::class.java, 0)
      return styleIdList(requireNotNull(list))
    }
  }

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
      destroy = { MaplibreNativeC.mln_map_destroy(map(handleAddress)) },
      afterSuccess = { runtimeRetention.close() },
    )
  }

  public actual companion object {
    public actual fun create(runtime: RuntimeHandle, options: MapOptions): MapHandle {
      NativeAccess.ensureLoaded()
      MapOptionsScope(options).use { nativeOptions ->
        PointerPointer<MaplibreNativeC.mln_map>(1).use { outMap ->
          outMap.put(0, null as Pointer?)
          Status.check(
            MaplibreNativeC.mln_map_create(
              runtime(runtime.nativeAddress()),
              nativeOptions.options,
              outMap,
            )
          )
          val map = outMap.get(MaplibreNativeC.mln_map::class.java, 0)
          val address = if (map == null || map.isNull) 0L else map.address()
          require(address != 0L) { "mln_map_create returned a null map" }
          return MapHandle(runtime, address)
        }
      }
    }
  }

  private fun requireLiveAddress(): Long {
    core.requireLive()
    return handleAddress
  }
}

private fun map(address: Long): MaplibreNativeC.mln_map =
  MaplibreNativeC.mln_map(AddressPointer(address))

private fun runtime(address: Long): MaplibreNativeC.mln_runtime =
  MaplibreNativeC.mln_runtime(AddressPointer(address))

private fun optionalCString(value: String): BytePointer {
  require('\u0000' !in value) { "C string inputs must not contain embedded NUL characters" }
  return BytePointer(value, java.nio.charset.StandardCharsets.UTF_8)
}

private fun stringView(view: MaplibreNativeC.mln_string_view): String {
  if (view.size() == 0L || view.data() == null || view.data().isNull) return ""
  val bytes = ByteArray(Math.toIntExact(view.size()))
  view.data().get(bytes, 0, bytes.size)
  return String(bytes, java.nio.charset.StandardCharsets.UTF_8)
}

private fun styleIdList(list: MaplibreNativeC.mln_style_id_list): List<String> =
  try {
    SizeTPointer(1).use { outCount ->
      Status.check(MaplibreNativeC.mln_style_id_list_count(list, outCount))
      List(Math.toIntExact(outCount.get())) { index ->
        MaplibreNativeC.mln_string_view().use { outId ->
          Status.check(MaplibreNativeC.mln_style_id_list_get(list, index.toLong(), outId))
          stringView(outId)
        }
      }
    }
  } finally {
    MaplibreNativeC.mln_style_id_list_destroy(list)
  }

private fun copyStyleSourceAttribution(
  mapAddress: Long,
  sourceId: StringViewScope,
  info: MaplibreNativeC.mln_style_source_info,
): String {
  val attributionSize = Math.toIntExact(info.attribution_size())
  if (attributionSize == 0) return ""
  BytePointer(attributionSize.toLong()).use { outAttribution ->
    SizeTPointer(1).use { outSize ->
      val outFound = booleanArrayOf(false)
      Status.check(
        MaplibreNativeC.mln_map_copy_style_source_attribution(
          map(mapAddress),
          sourceId.view,
          outAttribution,
          attributionSize.toLong(),
          outSize,
          outFound,
        )
      )
      if (!outFound[0]) return ""
      val bytes = ByteArray(Math.toIntExact(outSize.get()))
      outAttribution.get(bytes, 0, bytes.size)
      return String(bytes, java.nio.charset.StandardCharsets.UTF_8)
    }
  }
}

private class StringViewScope(value: String) : AutoCloseable {
  private val bytes: BytePointer
  val view: MaplibreNativeC.mln_string_view = MaplibreNativeC.mln_string_view()

  init {
    val utf8 = value.toByteArray(java.nio.charset.StandardCharsets.UTF_8)
    bytes = BytePointer(Math.max(utf8.size, 1).toLong())
    if (utf8.isNotEmpty()) bytes.put(utf8, 0, utf8.size)
    view.data(if (utf8.isEmpty()) null else bytes)
    view.size(utf8.size.toLong())
  }

  override fun close() {
    view.close()
    bytes.close()
  }
}

private class JsonScope(value: JsonValue) : AutoCloseable {
  private val owned = mutableListOf<Pointer>()
  private val strings = mutableListOf<StringViewScope>()
  val value: MaplibreNativeC.mln_json_value = jsonValue(value)

  override fun close() {
    owned.asReversed().forEach(Pointer::close)
    strings.asReversed().forEach(StringViewScope::close)
  }

  private fun <T : Pointer> own(pointer: T): T {
    owned += pointer
    return pointer
  }

  private fun string(value: String): MaplibreNativeC.mln_string_view {
    val scope = StringViewScope(value)
    strings += scope
    return scope.view
  }

  private fun jsonValue(value: JsonValue, depth: Int = 0): MaplibreNativeC.mln_json_value {
    require(depth <= JsonValue.MAX_DESCRIPTOR_DEPTH) {
      "JSON descriptor depth exceeds ${JsonValue.MAX_DESCRIPTOR_DEPTH}"
    }
    val out = own(MaplibreNativeC.mln_json_value())
    out.size(out.sizeof())
    when (value) {
      JsonValue.Null -> out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_NULL)
      is JsonValue.Bool ->
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_BOOL).data_bool_value(value.value)
      is JsonValue.UInt ->
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_UINT).data_uint_value(value.value)
      is JsonValue.Int ->
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_INT).data_int_value(value.value)
      is JsonValue.DoubleValue ->
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_DOUBLE).data_double_value(value.value)
      is JsonValue.StringValue ->
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_STRING).data_string_value(string(value.value))
      is JsonValue.Array -> {
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_ARRAY)
        val array = own(MaplibreNativeC.mln_json_array())
        if (value.values.isNotEmpty()) {
          val nativeValues = own(MaplibreNativeC.mln_json_value(value.values.size.toLong()))
          value.values.forEachIndexed { index, child ->
            nativeValues
              .position(index.toLong())
              .put<MaplibreNativeC.mln_json_value>(jsonValue(child, depth + 1))
          }
          nativeValues.position(0)
          array.values(nativeValues)
        }
        array.value_count(value.values.size.toLong())
        out.data_array_value(array)
      }
      is JsonValue.ObjectValue -> {
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_OBJECT)
        val obj = own(MaplibreNativeC.mln_json_object())
        if (value.members.isNotEmpty()) {
          val nativeMembers = own(MaplibreNativeC.mln_json_member(value.members.size.toLong()))
          value.members.forEachIndexed { index, member ->
            nativeMembers.position(index.toLong())
            nativeMembers.key(string(member.key))
            nativeMembers.value(jsonValue(member.value, depth + 1))
          }
          nativeMembers.position(0)
          obj.members(nativeMembers)
        }
        obj.member_count(value.members.size.toLong())
        out.data_object_value(obj)
      }
      is JsonValue.Unknown ->
        throw IllegalArgumentException("unknown JSON values cannot be used as input")
    }
    return out
  }
}

private class MapOptionsScope(value: MapOptions) : AutoCloseable {
  val options: MaplibreNativeC.mln_map_options = MaplibreNativeC.mln_map_options_default()

  init {
    value.width?.let { options.width(it) }
    value.height?.let { options.height(it) }
    value.scaleFactor?.let { options.scale_factor(it) }
    value.mapMode?.let {
      require(it.isKnown) { "Unknown map mode cannot be used as input: ${it.nativeValue}" }
      options.map_mode(it.nativeValue)
    }
  }

  override fun close() {
    options.close()
  }
}

private class AddressPointer(address: Long) : Pointer(null as Pointer?) {
  init {
    this.address = address
  }
}

private fun unsupportedMapHandle(): Nothing =
  throw UnsupportedOperationException(
    "MapHandle is not available until the Android map bridge is implemented"
  )

package org.maplibre.nativeffi.map

import org.bytedeco.javacpp.BoolPointer
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
    NativeAccess.ensureLoaded()
    StringViewScope(imageId).use { nativeImageId ->
      PremultipliedImageScope(image).use { nativeImage ->
        StyleImageOptionsScope(options).use { nativeOptions ->
          Status.check(
            MaplibreNativeC.mln_map_set_style_image(
              map(requireLiveAddress()),
              nativeImageId.view,
              nativeImage.image,
              nativeOptions.options,
            )
          )
        }
      }
    }
  }

  public actual fun removeStyleImage(imageId: String): Boolean {
    NativeAccess.ensureLoaded()
    val outRemoved = booleanArrayOf(false)
    StringViewScope(imageId).use { nativeImageId ->
      Status.check(
        MaplibreNativeC.mln_map_remove_style_image(
          map(requireLiveAddress()),
          nativeImageId.view,
          outRemoved,
        )
      )
    }
    return outRemoved[0]
  }

  public actual fun styleImageExists(imageId: String): Boolean {
    NativeAccess.ensureLoaded()
    val outExists = booleanArrayOf(false)
    StringViewScope(imageId).use { nativeImageId ->
      Status.check(
        MaplibreNativeC.mln_map_style_image_exists(
          map(requireLiveAddress()),
          nativeImageId.view,
          outExists,
        )
      )
    }
    return outExists[0]
  }

  public actual fun styleImageInfo(imageId: String): StyleImageInfo? {
    NativeAccess.ensureLoaded()
    StringViewScope(imageId).use { nativeImageId ->
      MaplibreNativeC.mln_style_image_info_default().use { outInfo ->
        val outFound = booleanArrayOf(false)
        Status.check(
          MaplibreNativeC.mln_map_get_style_image_info(
            map(requireLiveAddress()),
            nativeImageId.view,
            outInfo,
            outFound,
          )
        )
        return if (outFound[0]) styleImageInfo(outInfo) else null
      }
    }
  }

  public actual fun copyStyleImagePremultipliedRgba8(imageId: String): StyleImage? {
    NativeAccess.ensureLoaded()
    val info = styleImageInfo(imageId) ?: return null
    val outPixels = ByteArray(Math.toIntExact(info.byteLength))
    val outFound = booleanArrayOf(false)
    StringViewScope(imageId).use { nativeImageId ->
      SizeTPointer(1).use { outByteLength ->
        Status.check(
          MaplibreNativeC.mln_map_copy_style_image_premultiplied_rgba8(
            map(requireLiveAddress()),
            nativeImageId.view,
            outPixels,
            outPixels.size.toLong(),
            outByteLength,
            outFound,
          )
        )
      }
    }
    return if (outFound[0]) {
      StyleImage(
        PremultipliedRgba8Image(info.width, info.height, info.stride, outPixels),
        info.pixelRatio,
        info.sdf,
      )
    } else null
  }

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
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      StringViewScope(sourceId).use { nativeSourceId ->
        StringViewScope(beforeLayerId).use { nativeBeforeLayerId ->
          Status.check(
            MaplibreNativeC.mln_map_add_hillshade_layer(
              map(requireLiveAddress()),
              nativeLayerId.view,
              nativeSourceId.view,
              nativeBeforeLayerId.view,
            )
          )
        }
      }
    }
  }

  public actual fun addColorReliefLayer(layerId: String, sourceId: String, beforeLayerId: String) {
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      StringViewScope(sourceId).use { nativeSourceId ->
        StringViewScope(beforeLayerId).use { nativeBeforeLayerId ->
          Status.check(
            MaplibreNativeC.mln_map_add_color_relief_layer(
              map(requireLiveAddress()),
              nativeLayerId.view,
              nativeSourceId.view,
              nativeBeforeLayerId.view,
            )
          )
        }
      }
    }
  }

  public actual fun addLocationIndicatorLayer(layerId: String, beforeLayerId: String) {
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      StringViewScope(beforeLayerId).use { nativeBeforeLayerId ->
        Status.check(
          MaplibreNativeC.mln_map_add_location_indicator_layer(
            map(requireLiveAddress()),
            nativeLayerId.view,
            nativeBeforeLayerId.view,
          )
        )
      }
    }
  }

  public actual fun setLocationIndicatorLocation(
    layerId: String,
    coordinate: LatLng,
    altitude: Double,
  ) {
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      latLng(coordinate).use { nativeCoordinate ->
        Status.check(
          MaplibreNativeC.mln_map_set_location_indicator_location(
            map(requireLiveAddress()),
            nativeLayerId.view,
            nativeCoordinate,
            altitude,
          )
        )
      }
    }
  }

  public actual fun setLocationIndicatorBearing(layerId: String, bearing: Double) {
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      Status.check(
        MaplibreNativeC.mln_map_set_location_indicator_bearing(
          map(requireLiveAddress()),
          nativeLayerId.view,
          bearing,
        )
      )
    }
  }

  public actual fun setLocationIndicatorAccuracyRadius(layerId: String, radius: Double) {
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      Status.check(
        MaplibreNativeC.mln_map_set_location_indicator_accuracy_radius(
          map(requireLiveAddress()),
          nativeLayerId.view,
          radius,
        )
      )
    }
  }

  public actual fun setLocationIndicatorImageName(
    layerId: String,
    imageKind: LocationIndicatorImageKind,
    imageId: String,
  ) {
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      StringViewScope(imageId).use { nativeImageId ->
        Status.check(
          MaplibreNativeC.mln_map_set_location_indicator_image_name(
            map(requireLiveAddress()),
            nativeLayerId.view,
            imageKind.nativeValue,
            nativeImageId.view,
          )
        )
      }
    }
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
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      StringViewScope(beforeLayerId).use { nativeBeforeLayerId ->
        Status.check(
          MaplibreNativeC.mln_map_move_style_layer(
            map(requireLiveAddress()),
            nativeLayerId.view,
            nativeBeforeLayerId.view,
          )
        )
      }
    }
  }

  public actual fun styleLayerJson(layerId: String): JsonValue? {
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      PointerPointer<Pointer>(1).use { outSnapshot ->
        BoolPointer(1).use { outFound ->
          outSnapshot.put(0, null as Pointer?)
          Status.check(
            MaplibreNativeC.mln_map_get_style_layer_json(
              map(requireLiveAddress()),
              nativeLayerId.view,
              outSnapshot,
              outFound,
            )
          )
          return if (outFound.get()) jsonSnapshot(outSnapshot) else null
        }
      }
    }
  }

  public actual fun setStyleLightJson(lightJson: JsonValue) {
    NativeAccess.ensureLoaded()
    JsonScope(lightJson).use { nativeLightJson ->
      Status.check(
        MaplibreNativeC.mln_map_set_style_light_json(
          map(requireLiveAddress()),
          nativeLightJson.value,
        )
      )
    }
  }

  public actual fun setStyleLightProperty(propertyName: String, value: JsonValue) {
    NativeAccess.ensureLoaded()
    StringViewScope(propertyName).use { nativePropertyName ->
      JsonScope(value).use { nativeValue ->
        Status.check(
          MaplibreNativeC.mln_map_set_style_light_property(
            map(requireLiveAddress()),
            nativePropertyName.view,
            nativeValue.value,
          )
        )
      }
    }
  }

  public actual fun styleLightProperty(propertyName: String): JsonValue? {
    NativeAccess.ensureLoaded()
    StringViewScope(propertyName).use { nativePropertyName ->
      PointerPointer<Pointer>(1).use { outSnapshot ->
        outSnapshot.put(0, null as Pointer?)
        Status.check(
          MaplibreNativeC.mln_map_get_style_light_property(
            map(requireLiveAddress()),
            nativePropertyName.view,
            outSnapshot,
          )
        )
        return jsonSnapshot(outSnapshot)
      }
    }
  }

  public actual fun setLayerProperty(layerId: String, propertyName: String, value: JsonValue) {
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      StringViewScope(propertyName).use { nativePropertyName ->
        JsonScope(value).use { nativeValue ->
          Status.check(
            MaplibreNativeC.mln_map_set_layer_property(
              map(requireLiveAddress()),
              nativeLayerId.view,
              nativePropertyName.view,
              nativeValue.value,
            )
          )
        }
      }
    }
  }

  public actual fun layerProperty(layerId: String, propertyName: String): JsonValue? {
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      StringViewScope(propertyName).use { nativePropertyName ->
        PointerPointer<Pointer>(1).use { outSnapshot ->
          outSnapshot.put(0, null as Pointer?)
          Status.check(
            MaplibreNativeC.mln_map_get_layer_property(
              map(requireLiveAddress()),
              nativeLayerId.view,
              nativePropertyName.view,
              outSnapshot,
            )
          )
          return jsonSnapshot(outSnapshot)
        }
      }
    }
  }

  public actual fun setLayerFilter(layerId: String, filter: JsonValue) {
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      JsonScope(filter).use { nativeFilter ->
        Status.check(
          MaplibreNativeC.mln_map_set_layer_filter(
            map(requireLiveAddress()),
            nativeLayerId.view,
            nativeFilter.value,
          )
        )
      }
    }
  }

  public actual fun clearLayerFilter(layerId: String) {
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      Status.check(
        MaplibreNativeC.mln_map_set_layer_filter(
          map(requireLiveAddress()),
          nativeLayerId.view,
          null,
        )
      )
    }
  }

  public actual fun layerFilter(layerId: String): JsonValue? {
    NativeAccess.ensureLoaded()
    StringViewScope(layerId).use { nativeLayerId ->
      PointerPointer<Pointer>(1).use { outSnapshot ->
        outSnapshot.put(0, null as Pointer?)
        Status.check(
          MaplibreNativeC.mln_map_get_layer_filter(
            map(requireLiveAddress()),
            nativeLayerId.view,
            outSnapshot,
          )
        )
        return jsonSnapshot(outSnapshot)
      }
    }
  }

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

private fun latLng(value: LatLng): MaplibreNativeC.mln_lat_lng =
  MaplibreNativeC.mln_lat_lng().latitude(value.latitude).longitude(value.longitude)

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

private fun jsonSnapshot(outSnapshot: PointerPointer<Pointer>): JsonValue? {
  val snapshotPointer = outSnapshot.get(Pointer::class.java, 0) ?: return null
  if (snapshotPointer.isNull) return null
  val snapshot = MaplibreNativeC.mln_json_snapshot(snapshotPointer)
  return try {
    PointerPointer<Pointer>(1).use { outValue ->
      outValue.put(0, null as Pointer?)
      Status.check(MaplibreNativeC.mln_json_snapshot_get(snapshot, outValue))
      val valuePointer = outValue.get(Pointer::class.java, 0) ?: return null
      if (valuePointer.isNull) null else jsonValue(MaplibreNativeC.mln_json_value(valuePointer))
    }
  } finally {
    MaplibreNativeC.mln_json_snapshot_destroy(snapshot)
  }
}

private fun jsonValue(value: MaplibreNativeC.mln_json_value): JsonValue =
  when (value.type()) {
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_NULL -> JsonValue.Null
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_BOOL -> JsonValue.Bool(value.data_bool_value())
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_UINT -> JsonValue.UInt(value.data_uint_value())
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_INT -> JsonValue.Int(value.data_int_value())
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_DOUBLE -> JsonValue.DoubleValue(value.data_double_value())
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_STRING ->
      JsonValue.StringValue(stringView(value.data_string_value()))
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_ARRAY -> jsonArray(value.data_array_value())
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_OBJECT -> jsonObject(value.data_object_value())
    else -> JsonValue.Unknown(value.type(), value.size())
  }

private fun jsonArray(array: MaplibreNativeC.mln_json_array): JsonValue.Array {
  val nativeValues = array.values()
  return JsonValue.Array(
    List(Math.toIntExact(array.value_count())) { index ->
      jsonValue(nativeValues.getPointer(index.toLong()))
    }
  )
}

private fun jsonObject(obj: MaplibreNativeC.mln_json_object): JsonValue.ObjectValue {
  val nativeMembers = obj.members()
  return JsonValue.ObjectValue(
    List(Math.toIntExact(obj.member_count())) { index ->
      val member = nativeMembers.getPointer(index.toLong())
      JsonValue.Member(stringView(member.key()), jsonValue(member.value()))
    }
  )
}

private fun styleImageInfo(info: MaplibreNativeC.mln_style_image_info): StyleImageInfo =
  StyleImageInfo(
    info.width(),
    info.height(),
    info.stride(),
    info.byte_length(),
    info.pixel_ratio(),
    info.sdf(),
  )

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

private class PremultipliedImageScope(value: PremultipliedRgba8Image) : AutoCloseable {
  private val pixels: BytePointer
  val image: MaplibreNativeC.mln_premultiplied_rgba8_image =
    MaplibreNativeC.mln_premultiplied_rgba8_image_default()

  init {
    val bytes = value.pixels
    pixels = BytePointer(bytes.size.toLong())
    pixels.put(bytes, 0, bytes.size)
    image.width(value.width)
    image.height(value.height)
    image.stride(value.stride)
    image.pixels(pixels)
    image.byte_length(bytes.size.toLong())
  }

  override fun close() {
    image.close()
    pixels.close()
  }
}

private class StyleImageOptionsScope(value: StyleImageOptions) : AutoCloseable {
  val options: MaplibreNativeC.mln_style_image_options =
    MaplibreNativeC.mln_style_image_options_default()

  init {
    var fields = 0
    value.pixelRatio?.let {
      fields = fields or MaplibreNativeC.MLN_STYLE_IMAGE_OPTION_PIXEL_RATIO
      options.pixel_ratio(it)
    }
    value.sdf?.let {
      fields = fields or MaplibreNativeC.MLN_STYLE_IMAGE_OPTION_SDF
      options.sdf(it)
    }
    options.fields(fields)
  }

  override fun close() {
    options.close()
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

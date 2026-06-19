package org.maplibre.nativeffi.internal.loader

import java.lang.foreign.Arena
import java.lang.foreign.FunctionDescriptor
import java.lang.foreign.Linker
import java.lang.foreign.MemoryLayout
import java.lang.foreign.MemorySegment
import java.lang.foreign.SymbolLookup
import java.lang.foreign.ValueLayout
import java.lang.invoke.MethodHandle
import java.nio.charset.StandardCharsets
import java.nio.file.Path
import java.util.NoSuchElementException
import org.maplibre.nativeffi.camera.EdgeInsets
import org.maplibre.nativeffi.error.AbiVersionMismatchException
import org.maplibre.nativeffi.geo.LatLng
import org.maplibre.nativeffi.geo.LatLngBounds
import org.maplibre.nativeffi.geo.ProjectedMeters
import org.maplibre.nativeffi.geo.TileId
import org.maplibre.nativeffi.internal.status.Status
import org.maplibre.nativeffi.json.JsonValue
import org.maplibre.nativeffi.map.ConstrainMode
import org.maplibre.nativeffi.map.DebugOption
import org.maplibre.nativeffi.map.MapMode
import org.maplibre.nativeffi.map.MapOptions
import org.maplibre.nativeffi.map.NorthOrientation
import org.maplibre.nativeffi.map.ProjectionModeOptions
import org.maplibre.nativeffi.map.RenderingStats
import org.maplibre.nativeffi.map.TileLodMode
import org.maplibre.nativeffi.map.TileOperation
import org.maplibre.nativeffi.map.TileOptions
import org.maplibre.nativeffi.map.ViewportMode
import org.maplibre.nativeffi.map.ViewportOptions
import org.maplibre.nativeffi.offline.OfflineRegionDefinition
import org.maplibre.nativeffi.offline.OfflineRegionDownloadState
import org.maplibre.nativeffi.offline.OfflineRegionInfo
import org.maplibre.nativeffi.offline.OfflineRegionStatus
import org.maplibre.nativeffi.render.PremultipliedRgba8Image
import org.maplibre.nativeffi.render.RenderMode
import org.maplibre.nativeffi.resource.ResourceErrorReason
import org.maplibre.nativeffi.resource.ResourceKind
import org.maplibre.nativeffi.resource.ResourceLoadingMethod
import org.maplibre.nativeffi.resource.ResourcePriority
import org.maplibre.nativeffi.resource.ResourceRequest
import org.maplibre.nativeffi.resource.ResourceResponse
import org.maplibre.nativeffi.resource.ResourceStoragePolicy
import org.maplibre.nativeffi.resource.ResourceUsage
import org.maplibre.nativeffi.runtime.OfflineOperationKind
import org.maplibre.nativeffi.runtime.OfflineOperationResultKind
import org.maplibre.nativeffi.runtime.RuntimeEventPayload
import org.maplibre.nativeffi.runtime.RuntimeOptions
import org.maplibre.nativeffi.style.LocationIndicatorImageKind
import org.maplibre.nativeffi.style.SourceInfo
import org.maplibre.nativeffi.style.SourceType
import org.maplibre.nativeffi.style.StyleImage
import org.maplibre.nativeffi.style.StyleImageInfo
import org.maplibre.nativeffi.style.StyleImageOptions
import org.maplibre.nativeffi.style.TileSourceOptions

/** Ensures the native library is loaded before JVM FFM downcalls run. */
internal object NativeAccess {
  const val EXPECTED_C_ABI_VERSION: Long = 0L
  const val DEFAULT_LOG_SEVERITY_MASK: Int = (1 shl 1) or (1 shl 2)

  private val lock = Any()

  @Volatile private var initialized = false

  fun ensureLoaded() {
    if (initialized) {
      return
    }

    synchronized(lock) {
      if (initialized) {
        return
      }

      NativeLibrary.load()
      checkNativeAccessAndAbi()
      initialized = true
    }
  }

  fun load(libraryPath: Path) {
    synchronized(lock) {
      NativeLibrary.load(libraryPath)
      checkNativeAccessAndAbi()
      initialized = true
    }
  }

  internal fun checkAbiVersion(version: Long) {
    if (version != EXPECTED_C_ABI_VERSION) {
      throw AbiVersionMismatchException(version, EXPECTED_C_ABI_VERSION)
    }
  }

  internal fun checkNativeAccessAndAbi(cVersion: () -> Long) {
    val version =
      try {
        cVersion()
      } catch (error: IllegalCallerException) {
        throw nativeAccessFailure(error)
      } catch (error: NoSuchElementException) {
        throw missingSymbols(error)
      } catch (error: UnsatisfiedLinkError) {
        throw missingSymbols(error)
      }

    checkAbiVersion(version)
  }

  private fun checkNativeAccessAndAbi() {
    checkNativeAccessAndAbi(::cVersion)
  }

  internal fun cVersion(): Long =
    intFunction("mln_c_version").invokeWithArguments().let { Integer.toUnsignedLong(it as Int) }

  internal fun supportedRenderBackendMask(): Int =
    intFunction("mln_supported_render_backend_mask").invokeWithArguments() as Int

  internal fun supportedOpenGLContextProviderMask(): Int =
    intFunction("mln_opengl_supported_context_provider_mask").invokeWithArguments() as Int

  internal fun networkStatus(): Int =
    Arena.ofConfined().use { arena ->
      val outStatus = arena.allocate(ValueLayout.JAVA_INT)
      Status.check(
        statusOutFunction("mln_network_status_get").invokeWithArguments(outStatus) as Int
      )
      outStatus.get(ValueLayout.JAVA_INT, 0)
    }

  internal fun setNetworkStatus(status: Int) {
    Status.check(statusInFunction("mln_network_status_set").invokeWithArguments(status) as Int)
  }

  internal fun setAsyncLogSeverityMask(mask: Int) {
    Status.check(
      statusInFunction("mln_log_set_async_severity_mask").invokeWithArguments(mask) as Int
    )
  }

  internal fun setLogCallback(callback: MemorySegment): Int =
    logSetCallbackFunction().invokeWithArguments(callback, MemorySegment.NULL) as Int

  internal fun clearLogCallback(): Int =
    intFunction("mln_log_clear_callback").invokeWithArguments() as Int

  internal fun projectedMetersForLatLng(coordinate: LatLng): ProjectedMeters =
    Arena.ofConfined().use { arena ->
      val nativeCoordinate = arena.allocate(latLngLayout)
      nativeCoordinate.set(ValueLayout.JAVA_DOUBLE, 0, coordinate.latitude)
      nativeCoordinate.set(
        ValueLayout.JAVA_DOUBLE,
        Double.SIZE_BYTES.toLong(),
        coordinate.longitude,
      )
      val outMeters = arena.allocate(projectedMetersLayout)
      Status.check(
        projectedMetersForLatLngFunction().invokeWithArguments(nativeCoordinate, outMeters) as Int
      )
      ProjectedMeters(
        outMeters.get(ValueLayout.JAVA_DOUBLE, 0),
        outMeters.get(ValueLayout.JAVA_DOUBLE, Double.SIZE_BYTES.toLong()),
      )
    }

  internal fun latLngForProjectedMeters(meters: ProjectedMeters): LatLng =
    Arena.ofConfined().use { arena ->
      val nativeMeters = arena.allocate(projectedMetersLayout)
      nativeMeters.set(ValueLayout.JAVA_DOUBLE, 0, meters.northing)
      nativeMeters.set(ValueLayout.JAVA_DOUBLE, Double.SIZE_BYTES.toLong(), meters.easting)
      val outCoordinate = arena.allocate(latLngLayout)
      Status.check(
        latLngForProjectedMetersFunction().invokeWithArguments(nativeMeters, outCoordinate) as Int
      )
      LatLng(
        outCoordinate.get(ValueLayout.JAVA_DOUBLE, 0),
        outCoordinate.get(ValueLayout.JAVA_DOUBLE, Double.SIZE_BYTES.toLong()),
      )
    }

  internal fun createRuntime(options: RuntimeOptions): MemorySegment =
    Arena.ofConfined().use { arena ->
      val nativeOptions = runtimeOptions(options, arena)
      val outRuntime = arena.allocate(ValueLayout.ADDRESS)
      outRuntime.set(ValueLayout.ADDRESS, 0, MemorySegment.NULL)
      Status.check(runtimeCreateFunction().invokeWithArguments(nativeOptions, outRuntime) as Int)
      outRuntime.get(ValueLayout.ADDRESS, 0).also { runtime ->
        require(runtime != MemorySegment.NULL) { "mln_runtime_create returned a null runtime" }
      }
    }

  internal fun runRuntimeOnce(runtime: MemorySegment) {
    Status.check(runtimeStatusFunction("mln_runtime_run_once").invokeWithArguments(runtime) as Int)
  }

  internal fun destroyRuntime(runtime: MemorySegment): Int =
    runtimeStatusFunction("mln_runtime_destroy").invokeWithArguments(runtime) as Int

  internal fun setResourceProvider(runtime: MemorySegment, provider: MemorySegment): Int =
    runtimeSetResourceProviderFunction().invokeWithArguments(runtime, provider) as Int

  internal fun startAmbientCacheOperation(runtime: MemorySegment, operation: Int): Long =
    Arena.ofConfined().use { arena ->
      val outOperationId = arena.allocate(ValueLayout.JAVA_LONG)
      Status.check(
        runtimeAmbientCacheOperationStartFunction()
          .invokeWithArguments(runtime, operation, outOperationId) as Int
      )
      outOperationId.get(ValueLayout.JAVA_LONG, 0)
    }

  internal fun startCreateOfflineRegion(
    runtime: MemorySegment,
    definition: OfflineRegionDefinition,
    metadata: ByteArray,
  ): Long =
    Arena.ofConfined().use { arena ->
      val outOperationId = arena.allocate(ValueLayout.JAVA_LONG)
      Status.check(
        runtimeOfflineRegionCreateStartFunction()
          .invokeWithArguments(
            runtime,
            offlineRegionDefinition(definition, arena),
            nativeBytes(arena, metadata),
            metadata.size.toLong(),
            outOperationId,
          ) as Int
      )
      outOperationId.get(ValueLayout.JAVA_LONG, 0)
    }

  internal fun startOfflineRegion(runtime: MemorySegment, regionId: Long): Long =
    startRuntimeLongOperation("mln_runtime_offline_region_get_start", runtime, regionId)

  internal fun startOfflineRegions(runtime: MemorySegment): Long =
    startRuntimeOperation("mln_runtime_offline_regions_list_start", runtime)

  internal fun startMergeOfflineRegionsDatabase(runtime: MemorySegment, path: String): Long =
    Arena.ofConfined().use { arena ->
      val outOperationId = arena.allocate(ValueLayout.JAVA_LONG)
      Status.check(
        runtimeAddressOperationStartFunction("mln_runtime_offline_regions_merge_database_start")
          .invokeWithArguments(runtime, cString(arena, path), outOperationId) as Int
      )
      outOperationId.get(ValueLayout.JAVA_LONG, 0)
    }

  internal fun startUpdateOfflineRegionMetadata(
    runtime: MemorySegment,
    regionId: Long,
    metadata: ByteArray,
  ): Long =
    Arena.ofConfined().use { arena ->
      val outOperationId = arena.allocate(ValueLayout.JAVA_LONG)
      Status.check(
        runtimeLongAddressLongOperationStartFunction(
            "mln_runtime_offline_region_update_metadata_start"
          )
          .invokeWithArguments(
            runtime,
            regionId,
            nativeBytes(arena, metadata),
            metadata.size.toLong(),
            outOperationId,
          ) as Int
      )
      outOperationId.get(ValueLayout.JAVA_LONG, 0)
    }

  internal fun startOfflineRegionStatus(runtime: MemorySegment, regionId: Long): Long =
    startRuntimeLongOperation("mln_runtime_offline_region_get_status_start", runtime, regionId)

  internal fun startSetOfflineRegionObserved(
    runtime: MemorySegment,
    regionId: Long,
    observed: Boolean,
  ): Long =
    Arena.ofConfined().use { arena ->
      val outOperationId = arena.allocate(ValueLayout.JAVA_LONG)
      Status.check(
        runtimeLongBooleanOperationStartFunction("mln_runtime_offline_region_set_observed_start")
          .invokeWithArguments(runtime, regionId, observed, outOperationId) as Int
      )
      outOperationId.get(ValueLayout.JAVA_LONG, 0)
    }

  internal fun startSetOfflineRegionDownloadState(
    runtime: MemorySegment,
    regionId: Long,
    downloadState: Int,
  ): Long =
    Arena.ofConfined().use { arena ->
      val outOperationId = arena.allocate(ValueLayout.JAVA_LONG)
      Status.check(
        runtimeLongIntOperationStartFunction("mln_runtime_offline_region_set_download_state_start")
          .invokeWithArguments(runtime, regionId, downloadState, outOperationId) as Int
      )
      outOperationId.get(ValueLayout.JAVA_LONG, 0)
    }

  internal fun startInvalidateOfflineRegion(runtime: MemorySegment, regionId: Long): Long =
    startRuntimeLongOperation("mln_runtime_offline_region_invalidate_start", runtime, regionId)

  internal fun startDeleteOfflineRegion(runtime: MemorySegment, regionId: Long): Long =
    startRuntimeLongOperation("mln_runtime_offline_region_delete_start", runtime, regionId)

  internal fun discardOfflineOperation(runtime: MemorySegment, operationId: Long): Int =
    runtimeOfflineOperationDiscardFunction().invokeWithArguments(runtime, operationId) as Int

  internal fun setResourceTransform(runtime: MemorySegment, descriptor: MemorySegment): Int =
    runtimeSetResourceTransformFunction().invokeWithArguments(runtime, descriptor) as Int

  internal fun clearResourceTransform(runtime: MemorySegment): Int =
    runtimeClearResourceTransformFunction().invokeWithArguments(runtime) as Int

  internal fun completeResourceRequest(handle: MemorySegment, response: ResourceResponse): Int =
    Arena.ofConfined().use { arena ->
      resourceRequestCompleteFunction()
        .invokeWithArguments(handle, resourceResponse(response, arena)) as Int
    }

  internal fun isResourceRequestCancelled(handle: MemorySegment): Boolean =
    Arena.ofConfined().use { arena ->
      val outCancelled = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        resourceRequestCancelledFunction().invokeWithArguments(handle, outCancelled) as Int
      )
      outCancelled.get(ValueLayout.JAVA_BOOLEAN, 0)
    }

  internal fun releaseResourceRequest(handle: MemorySegment) {
    resourceRequestReleaseFunction().invokeWithArguments(handle)
  }

  internal fun createMap(runtime: MemorySegment, options: MapOptions): MemorySegment =
    Arena.ofConfined().use { arena ->
      val outMap = arena.allocate(ValueLayout.ADDRESS)
      outMap.set(ValueLayout.ADDRESS, 0, MemorySegment.NULL)
      Status.check(
        mapCreateFunction().invokeWithArguments(runtime, mapOptions(options, arena), outMap) as Int
      )
      outMap.get(ValueLayout.ADDRESS, 0).also { map ->
        require(map != MemorySegment.NULL) { "mln_map_create returned a null map" }
      }
    }

  internal fun destroyMap(map: MemorySegment): Int =
    mapStatusFunction("mln_map_destroy").invokeWithArguments(map) as Int

  internal fun setMapStyleUrl(map: MemorySegment, url: String) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapAddressStatusFunction("mln_map_set_style_url")
          .invokeWithArguments(map, cString(arena, url)) as Int
      )
    }
  }

  internal fun setMapStyleJson(map: MemorySegment, json: String) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapAddressStatusFunction("mln_map_set_style_json")
          .invokeWithArguments(map, cString(arena, json)) as Int
      )
    }
  }

  internal fun addStyleSourceJson(map: MemorySegment, sourceId: String, sourceJson: JsonValue) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_add_style_source_json")
          .invokeWithArguments(map, stringView(arena, sourceId), jsonValue(arena, sourceJson))
          as Int
      )
    }
  }

  internal fun removeStyleSource(map: MemorySegment, sourceId: String): Boolean =
    Arena.ofConfined().use { arena ->
      val outRemoved = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_remove_style_source")
          .invokeWithArguments(map, stringView(arena, sourceId), outRemoved) as Int
      )
      outRemoved.get(ValueLayout.JAVA_BOOLEAN, 0)
    }

  internal fun styleSourceExists(map: MemorySegment, sourceId: String): Boolean =
    Arena.ofConfined().use { arena ->
      val outExists = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_style_source_exists")
          .invokeWithArguments(map, stringView(arena, sourceId), outExists) as Int
      )
      outExists.get(ValueLayout.JAVA_BOOLEAN, 0)
    }

  internal fun styleSourceType(map: MemorySegment, sourceId: String): SourceType? =
    Arena.ofConfined().use { arena ->
      val outType = arena.allocate(ValueLayout.JAVA_INT)
      val outFound = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapStringViewTwoAddressStatusFunction("mln_map_get_style_source_type")
          .invokeWithArguments(map, stringView(arena, sourceId), outType, outFound) as Int
      )
      if (outFound.get(ValueLayout.JAVA_BOOLEAN, 0)) {
        SourceType.fromNative(outType.get(ValueLayout.JAVA_INT, 0))
      } else null
    }

  internal fun styleSourceInfo(map: MemorySegment, sourceId: String): SourceInfo? =
    Arena.ofConfined().use { arena ->
      val sourceIdView = stringView(arena, sourceId)
      val outInfo = arena.allocate(STYLE_SOURCE_INFO_SIZE)
      outInfo.set(
        ValueLayout.JAVA_INT,
        STYLE_SOURCE_INFO_SIZE_OFFSET,
        STYLE_SOURCE_INFO_SIZE.toInt(),
      )
      val outFound = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapStringViewTwoAddressStatusFunction("mln_map_get_style_source_info")
          .invokeWithArguments(map, sourceIdView, outInfo, outFound) as Int
      )
      if (!outFound.get(ValueLayout.JAVA_BOOLEAN, 0)) {
        return@use null
      }
      val attribution =
        if (outInfo.get(ValueLayout.JAVA_BOOLEAN, STYLE_SOURCE_INFO_HAS_ATTRIBUTION_OFFSET)) {
          copyStyleSourceAttribution(
            map,
            sourceIdView,
            outInfo.get(ValueLayout.JAVA_LONG, STYLE_SOURCE_INFO_ATTRIBUTION_SIZE_OFFSET),
            arena,
          ) ?: return@use null
        } else null
      SourceInfo(
        SourceType.fromNative(outInfo.get(ValueLayout.JAVA_INT, STYLE_SOURCE_INFO_TYPE_OFFSET)),
        outInfo.get(ValueLayout.JAVA_BOOLEAN, STYLE_SOURCE_INFO_IS_VOLATILE_OFFSET),
        attribution,
      )
    }

  internal fun styleSourceIds(map: MemorySegment): List<String> =
    Arena.ofConfined().use { arena ->
      val outList = arena.allocate(ValueLayout.ADDRESS)
      outList.set(ValueLayout.ADDRESS, 0, MemorySegment.NULL)
      Status.check(mapListStyleSourceIdsFunction().invokeWithArguments(map, outList) as Int)
      styleIdList(outList.get(ValueLayout.ADDRESS, 0))
    }

  internal fun addVectorSourceUrl(
    map: MemorySegment,
    sourceId: String,
    url: String,
    options: TileSourceOptions?,
  ) {
    addTileSourceUrl("mln_map_add_vector_source_url", map, sourceId, url, options)
  }

  internal fun addVectorSourceTiles(
    map: MemorySegment,
    sourceId: String,
    tiles: List<String>,
    options: TileSourceOptions?,
  ) {
    addTileSourceTiles("mln_map_add_vector_source_tiles", map, sourceId, tiles, options)
  }

  internal fun addRasterSourceUrl(
    map: MemorySegment,
    sourceId: String,
    url: String,
    options: TileSourceOptions?,
  ) {
    addTileSourceUrl("mln_map_add_raster_source_url", map, sourceId, url, options)
  }

  internal fun addRasterSourceTiles(
    map: MemorySegment,
    sourceId: String,
    tiles: List<String>,
    options: TileSourceOptions?,
  ) {
    addTileSourceTiles("mln_map_add_raster_source_tiles", map, sourceId, tiles, options)
  }

  internal fun addRasterDemSourceUrl(
    map: MemorySegment,
    sourceId: String,
    url: String,
    options: TileSourceOptions?,
  ) {
    addTileSourceUrl("mln_map_add_raster_dem_source_url", map, sourceId, url, options)
  }

  internal fun addRasterDemSourceTiles(
    map: MemorySegment,
    sourceId: String,
    tiles: List<String>,
    options: TileSourceOptions?,
  ) {
    addTileSourceTiles("mln_map_add_raster_dem_source_tiles", map, sourceId, tiles, options)
  }

  internal fun setStyleImage(
    map: MemorySegment,
    imageId: String,
    image: PremultipliedRgba8Image,
    options: StyleImageOptions,
  ) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewTwoAddressStatusFunction("mln_map_set_style_image")
          .invokeWithArguments(
            map,
            stringView(arena, imageId),
            premultipliedRgba8Image(arena, image),
            styleImageOptions(arena, options),
          ) as Int
      )
    }
  }

  internal fun removeStyleImage(map: MemorySegment, imageId: String): Boolean =
    Arena.ofConfined().use { arena ->
      val outRemoved = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_remove_style_image")
          .invokeWithArguments(map, stringView(arena, imageId), outRemoved) as Int
      )
      outRemoved.get(ValueLayout.JAVA_BOOLEAN, 0)
    }

  internal fun styleImageExists(map: MemorySegment, imageId: String): Boolean =
    Arena.ofConfined().use { arena ->
      val outExists = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_style_image_exists")
          .invokeWithArguments(map, stringView(arena, imageId), outExists) as Int
      )
      outExists.get(ValueLayout.JAVA_BOOLEAN, 0)
    }

  internal fun styleImageInfo(map: MemorySegment, imageId: String): StyleImageInfo? =
    Arena.ofConfined().use { arena ->
      val outInfo = styleImageInfoDefault(arena)
      val outFound = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapStringViewTwoAddressStatusFunction("mln_map_get_style_image_info")
          .invokeWithArguments(map, stringView(arena, imageId), outInfo, outFound) as Int
      )
      if (outFound.get(ValueLayout.JAVA_BOOLEAN, 0)) styleImageInfo(outInfo) else null
    }

  internal fun copyStyleImagePremultipliedRgba8(map: MemorySegment, imageId: String): StyleImage? {
    val info = styleImageInfo(map, imageId) ?: return null
    return Arena.ofConfined().use { arena ->
      val outPixels = arena.allocate(info.byteLength)
      val outByteLength = arena.allocate(ValueLayout.JAVA_LONG)
      val outFound = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapStringViewAddressLongTwoAddressStatusFunction(
            "mln_map_copy_style_image_premultiplied_rgba8"
          )
          .invokeWithArguments(
            map,
            stringView(arena, imageId),
            outPixels,
            info.byteLength,
            outByteLength,
            outFound,
          ) as Int
      )
      if (!outFound.get(ValueLayout.JAVA_BOOLEAN, 0)) {
        return@use null
      }
      val byteLength = outByteLength.get(ValueLayout.JAVA_LONG, 0)
      StyleImage(
        PremultipliedRgba8Image(
          info.width,
          info.height,
          info.stride,
          copyBytes(outPixels, byteLength),
        ),
        info.pixelRatio,
        info.sdf,
      )
    }
  }

  internal fun addImageSourceUrl(
    map: MemorySegment,
    sourceId: String,
    coordinates: List<LatLng>,
    url: String,
  ) {
    val coordinateSnapshot = coordinates.toList()
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewAddressLongStringViewStatusFunction("mln_map_add_image_source_url")
          .invokeWithArguments(
            map,
            stringView(arena, sourceId),
            latLngArray(arena, coordinateSnapshot),
            coordinateSnapshot.size.toLong(),
            stringView(arena, url),
          ) as Int
      )
    }
  }

  internal fun addImageSourceImage(
    map: MemorySegment,
    sourceId: String,
    coordinates: List<LatLng>,
    image: PremultipliedRgba8Image,
  ) {
    val coordinateSnapshot = coordinates.toList()
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewAddressLongAddressStatusFunction("mln_map_add_image_source_image")
          .invokeWithArguments(
            map,
            stringView(arena, sourceId),
            latLngArray(arena, coordinateSnapshot),
            coordinateSnapshot.size.toLong(),
            premultipliedRgba8Image(arena, image),
          ) as Int
      )
    }
  }

  internal fun setImageSourceUrl(map: MemorySegment, sourceId: String, url: String) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapTwoStringViewsStatusFunction("mln_map_set_image_source_url")
          .invokeWithArguments(map, stringView(arena, sourceId), stringView(arena, url)) as Int
      )
    }
  }

  internal fun setImageSourceImage(
    map: MemorySegment,
    sourceId: String,
    image: PremultipliedRgba8Image,
  ) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_set_image_source_image")
          .invokeWithArguments(
            map,
            stringView(arena, sourceId),
            premultipliedRgba8Image(arena, image),
          ) as Int
      )
    }
  }

  internal fun setImageSourceCoordinates(
    map: MemorySegment,
    sourceId: String,
    coordinates: List<LatLng>,
  ) {
    val coordinateSnapshot = coordinates.toList()
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewAddressLongStatusFunction("mln_map_set_image_source_coordinates")
          .invokeWithArguments(
            map,
            stringView(arena, sourceId),
            latLngArray(arena, coordinateSnapshot),
            coordinateSnapshot.size.toLong(),
          ) as Int
      )
    }
  }

  internal fun imageSourceCoordinates(map: MemorySegment, sourceId: String): List<LatLng>? =
    Arena.ofConfined().use { arena ->
      val outCoordinates = arena.allocate(latLngLayout.byteSize() * IMAGE_SOURCE_COORDINATE_COUNT)
      val outCoordinateCount = arena.allocate(ValueLayout.JAVA_LONG)
      val outFound = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapStringViewAddressLongTwoAddressStatusFunction("mln_map_get_image_source_coordinates")
          .invokeWithArguments(
            map,
            stringView(arena, sourceId),
            outCoordinates,
            IMAGE_SOURCE_COORDINATE_COUNT.toLong(),
            outCoordinateCount,
            outFound,
          ) as Int
      )
      if (outFound.get(ValueLayout.JAVA_BOOLEAN, 0)) {
        latLngArray(
          outCoordinates,
          Math.toIntExact(outCoordinateCount.get(ValueLayout.JAVA_LONG, 0)),
        )
      } else null
    }

  internal fun addStyleLayerJson(map: MemorySegment, layerJson: JsonValue, beforeLayerId: String) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapAddressStringViewStatusFunction("mln_map_add_style_layer_json")
          .invokeWithArguments(map, jsonValue(arena, layerJson), stringView(arena, beforeLayerId))
          as Int
      )
    }
  }

  internal fun removeStyleLayer(map: MemorySegment, layerId: String): Boolean =
    Arena.ofConfined().use { arena ->
      val outRemoved = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_remove_style_layer")
          .invokeWithArguments(map, stringView(arena, layerId), outRemoved) as Int
      )
      outRemoved.get(ValueLayout.JAVA_BOOLEAN, 0)
    }

  internal fun styleLayerExists(map: MemorySegment, layerId: String): Boolean =
    Arena.ofConfined().use { arena ->
      val outExists = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_style_layer_exists")
          .invokeWithArguments(map, stringView(arena, layerId), outExists) as Int
      )
      outExists.get(ValueLayout.JAVA_BOOLEAN, 0)
    }

  internal fun styleLayerType(map: MemorySegment, layerId: String): String? =
    Arena.ofConfined().use { arena ->
      val outType = arena.allocate(STRING_VIEW_SIZE)
      val outFound = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapStringViewTwoAddressStatusFunction("mln_map_get_style_layer_type")
          .invokeWithArguments(map, stringView(arena, layerId), outType, outFound) as Int
      )
      if (outFound.get(ValueLayout.JAVA_BOOLEAN, 0)) stringView(outType) else null
    }

  internal fun styleLayerIds(map: MemorySegment): List<String> =
    Arena.ofConfined().use { arena ->
      val outList = arena.allocate(ValueLayout.ADDRESS)
      outList.set(ValueLayout.ADDRESS, 0, MemorySegment.NULL)
      Status.check(mapListStyleLayerIdsFunction().invokeWithArguments(map, outList) as Int)
      styleIdList(outList.get(ValueLayout.ADDRESS, 0))
    }

  internal fun addHillshadeLayer(
    map: MemorySegment,
    layerId: String,
    sourceId: String,
    beforeLayerId: String,
  ) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapThreeStringViewsStatusFunction("mln_map_add_hillshade_layer")
          .invokeWithArguments(
            map,
            stringView(arena, layerId),
            stringView(arena, sourceId),
            stringView(arena, beforeLayerId),
          ) as Int
      )
    }
  }

  internal fun addColorReliefLayer(
    map: MemorySegment,
    layerId: String,
    sourceId: String,
    beforeLayerId: String,
  ) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapThreeStringViewsStatusFunction("mln_map_add_color_relief_layer")
          .invokeWithArguments(
            map,
            stringView(arena, layerId),
            stringView(arena, sourceId),
            stringView(arena, beforeLayerId),
          ) as Int
      )
    }
  }

  internal fun addLocationIndicatorLayer(
    map: MemorySegment,
    layerId: String,
    beforeLayerId: String,
  ) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapTwoStringViewsStatusFunction("mln_map_add_location_indicator_layer")
          .invokeWithArguments(map, stringView(arena, layerId), stringView(arena, beforeLayerId))
          as Int
      )
    }
  }

  internal fun setLocationIndicatorLocation(
    map: MemorySegment,
    layerId: String,
    coordinate: LatLng,
    altitude: Double,
  ) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewLatLngDoubleStatusFunction("mln_map_set_location_indicator_location")
          .invokeWithArguments(map, stringView(arena, layerId), latLng(coordinate, arena), altitude)
          as Int
      )
    }
  }

  internal fun setLocationIndicatorBearing(map: MemorySegment, layerId: String, bearing: Double) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewDoubleStatusFunction("mln_map_set_location_indicator_bearing")
          .invokeWithArguments(map, stringView(arena, layerId), bearing) as Int
      )
    }
  }

  internal fun setLocationIndicatorAccuracyRadius(
    map: MemorySegment,
    layerId: String,
    radius: Double,
  ) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewDoubleStatusFunction("mln_map_set_location_indicator_accuracy_radius")
          .invokeWithArguments(map, stringView(arena, layerId), radius) as Int
      )
    }
  }

  internal fun setLocationIndicatorImageName(
    map: MemorySegment,
    layerId: String,
    imageKind: LocationIndicatorImageKind,
    imageId: String,
  ) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewIntStringViewStatusFunction("mln_map_set_location_indicator_image_name")
          .invokeWithArguments(
            map,
            stringView(arena, layerId),
            imageKind.nativeValue,
            stringView(arena, imageId),
          ) as Int
      )
    }
  }

  internal fun moveStyleLayer(map: MemorySegment, layerId: String, beforeLayerId: String) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapTwoStringViewsStatusFunction("mln_map_move_style_layer")
          .invokeWithArguments(map, stringView(arena, layerId), stringView(arena, beforeLayerId))
          as Int
      )
    }
  }

  internal fun styleLayerJson(map: MemorySegment, layerId: String): JsonValue? =
    Arena.ofConfined().use { arena ->
      val outSnapshot = arena.allocate(ValueLayout.ADDRESS)
      val outFound = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      outSnapshot.set(ValueLayout.ADDRESS, 0, MemorySegment.NULL)
      Status.check(
        mapStringViewTwoAddressStatusFunction("mln_map_get_style_layer_json")
          .invokeWithArguments(map, stringView(arena, layerId), outSnapshot, outFound) as Int
      )
      if (outFound.get(ValueLayout.JAVA_BOOLEAN, 0)) {
        jsonSnapshot(outSnapshot.get(ValueLayout.ADDRESS, 0))
      } else null
    }

  internal fun setStyleLightJson(map: MemorySegment, lightJson: JsonValue) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapAddressStatusFunction("mln_map_set_style_light_json")
          .invokeWithArguments(map, jsonValue(arena, lightJson)) as Int
      )
    }
  }

  internal fun setStyleLightProperty(map: MemorySegment, propertyName: String, value: JsonValue) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_set_style_light_property")
          .invokeWithArguments(map, stringView(arena, propertyName), jsonValue(arena, value)) as Int
      )
    }
  }

  internal fun styleLightProperty(map: MemorySegment, propertyName: String): JsonValue? =
    Arena.ofConfined().use { arena ->
      val outSnapshot = arena.allocate(ValueLayout.ADDRESS)
      outSnapshot.set(ValueLayout.ADDRESS, 0, MemorySegment.NULL)
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_get_style_light_property")
          .invokeWithArguments(map, stringView(arena, propertyName), outSnapshot) as Int
      )
      jsonSnapshot(outSnapshot.get(ValueLayout.ADDRESS, 0))
    }

  internal fun setLayerProperty(
    map: MemorySegment,
    layerId: String,
    propertyName: String,
    value: JsonValue,
  ) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapTwoStringViewsAddressStatusFunction("mln_map_set_layer_property")
          .invokeWithArguments(
            map,
            stringView(arena, layerId),
            stringView(arena, propertyName),
            jsonValue(arena, value),
          ) as Int
      )
    }
  }

  internal fun layerProperty(
    map: MemorySegment,
    layerId: String,
    propertyName: String,
  ): JsonValue? =
    Arena.ofConfined().use { arena ->
      val outSnapshot = arena.allocate(ValueLayout.ADDRESS)
      outSnapshot.set(ValueLayout.ADDRESS, 0, MemorySegment.NULL)
      Status.check(
        mapTwoStringViewsAddressStatusFunction("mln_map_get_layer_property")
          .invokeWithArguments(
            map,
            stringView(arena, layerId),
            stringView(arena, propertyName),
            outSnapshot,
          ) as Int
      )
      jsonSnapshot(outSnapshot.get(ValueLayout.ADDRESS, 0))
    }

  internal fun setLayerFilter(map: MemorySegment, layerId: String, filter: JsonValue) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_set_layer_filter")
          .invokeWithArguments(map, stringView(arena, layerId), jsonValue(arena, filter)) as Int
      )
    }
  }

  internal fun clearLayerFilter(map: MemorySegment, layerId: String) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_set_layer_filter")
          .invokeWithArguments(map, stringView(arena, layerId), MemorySegment.NULL) as Int
      )
    }
  }

  internal fun layerFilter(map: MemorySegment, layerId: String): JsonValue? =
    Arena.ofConfined().use { arena ->
      val outSnapshot = arena.allocate(ValueLayout.ADDRESS)
      outSnapshot.set(ValueLayout.ADDRESS, 0, MemorySegment.NULL)
      Status.check(
        mapStringViewAddressStatusFunction("mln_map_get_layer_filter")
          .invokeWithArguments(map, stringView(arena, layerId), outSnapshot) as Int
      )
      jsonSnapshot(outSnapshot.get(ValueLayout.ADDRESS, 0))
    }

  internal fun requestRepaint(map: MemorySegment) {
    Status.check(mapStatusFunction("mln_map_request_repaint").invokeWithArguments(map) as Int)
  }

  internal fun requestStillImage(map: MemorySegment) {
    Status.check(mapStatusFunction("mln_map_request_still_image").invokeWithArguments(map) as Int)
  }

  internal fun setDebugOptions(map: MemorySegment, options: Set<DebugOption>) {
    val mask = options.fold(0) { acc, option -> acc or option.nativeMask }
    Status.check(
      mapIntStatusFunction("mln_map_set_debug_options").invokeWithArguments(map, mask) as Int
    )
  }

  internal fun debugOptions(map: MemorySegment): Set<DebugOption> =
    Arena.ofConfined().use { arena ->
      val outOptions = arena.allocate(ValueLayout.JAVA_INT)
      Status.check(
        mapAddressStatusFunction("mln_map_get_debug_options").invokeWithArguments(map, outOptions)
          as Int
      )
      debugOptions(outOptions.get(ValueLayout.JAVA_INT, 0))
    }

  internal fun setRenderingStatsViewEnabled(map: MemorySegment, enabled: Boolean) {
    Status.check(
      mapBooleanStatusFunction("mln_map_set_rendering_stats_view_enabled")
        .invokeWithArguments(map, enabled) as Int
    )
  }

  internal fun renderingStatsViewEnabled(map: MemorySegment): Boolean =
    Arena.ofConfined().use { arena ->
      val outEnabled = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapAddressStatusFunction("mln_map_get_rendering_stats_view_enabled")
          .invokeWithArguments(map, outEnabled) as Int
      )
      outEnabled.get(ValueLayout.JAVA_BOOLEAN, 0)
    }

  internal fun isFullyLoaded(map: MemorySegment): Boolean =
    Arena.ofConfined().use { arena ->
      val outLoaded = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      Status.check(
        mapAddressStatusFunction("mln_map_is_fully_loaded").invokeWithArguments(map, outLoaded)
          as Int
      )
      outLoaded.get(ValueLayout.JAVA_BOOLEAN, 0)
    }

  internal fun dumpDebugLogs(map: MemorySegment) {
    Status.check(mapStatusFunction("mln_map_dump_debug_logs").invokeWithArguments(map) as Int)
  }

  internal fun viewportOptions(map: MemorySegment): ViewportOptions =
    Arena.ofConfined().use { arena ->
      val outOptions = viewportOptionsDefault(arena)
      Status.check(
        mapAddressStatusFunction("mln_map_get_viewport_options")
          .invokeWithArguments(map, outOptions) as Int
      )
      readViewportOptions(outOptions)
    }

  internal fun setViewportOptions(map: MemorySegment, options: ViewportOptions) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapAddressStatusFunction("mln_map_set_viewport_options")
          .invokeWithArguments(map, viewportOptions(arena, options)) as Int
      )
    }
  }

  internal fun tileOptions(map: MemorySegment): TileOptions =
    Arena.ofConfined().use { arena ->
      val outOptions = tileOptionsDefault(arena)
      Status.check(
        mapAddressStatusFunction("mln_map_get_tile_options").invokeWithArguments(map, outOptions)
          as Int
      )
      readTileOptions(outOptions)
    }

  internal fun setTileOptions(map: MemorySegment, options: TileOptions) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapAddressStatusFunction("mln_map_set_tile_options")
          .invokeWithArguments(map, tileOptions(arena, options)) as Int
      )
    }
  }

  internal fun projectionMode(map: MemorySegment): ProjectionModeOptions =
    Arena.ofConfined().use { arena ->
      val outMode = projectionModeDefault(arena)
      Status.check(
        mapAddressStatusFunction("mln_map_get_projection_mode").invokeWithArguments(map, outMode)
          as Int
      )
      projectionModeOptions(outMode)
    }

  internal fun setProjectionMode(map: MemorySegment, mode: ProjectionModeOptions) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapAddressStatusFunction("mln_map_set_projection_mode")
          .invokeWithArguments(map, projectionModeOptions(arena, mode)) as Int
      )
    }
  }

  internal fun setResourceTransformResponseUrl(response: MemorySegment, value: String): Int =
    Arena.ofConfined().use { arena ->
      val bytes = value.toByteArray(StandardCharsets.UTF_8)
      resourceTransformResponseSetUrlFunction()
        .invokeWithArguments(response, nativeBytes(arena, bytes), bytes.size.toLong()) as Int
    }

  internal fun takeOfflineRegionStatusResult(
    runtime: MemorySegment,
    operationId: Long,
  ): OfflineRegionStatus =
    Arena.ofConfined().use { arena ->
      val status = arena.allocate(OFFLINE_REGION_STATUS_SIZE)
      status.set(
        ValueLayout.JAVA_INT,
        OFFLINE_REGION_STATUS_SIZE_OFFSET,
        OFFLINE_REGION_STATUS_SIZE.toInt(),
      )
      Status.check(
        runtimeOfflineRegionStatusTakeResultFunction()
          .invokeWithArguments(runtime, operationId, status) as Int
      )
      offlineRegionStatus(status)
    }

  internal fun takeCreateOfflineRegionResult(
    runtime: MemorySegment,
    operationId: Long,
  ): OfflineRegionInfo =
    takeOfflineRegionSnapshot(runtime, operationId, runtimeOfflineRegionCreateTakeResultFunction())

  internal fun takeOfflineRegionResult(
    runtime: MemorySegment,
    operationId: Long,
  ): OfflineRegionInfo? =
    Arena.ofConfined().use { arena ->
      val outSnapshot = arena.allocate(ValueLayout.ADDRESS)
      val outFound = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      outSnapshot.set(ValueLayout.ADDRESS, 0, MemorySegment.NULL)
      outFound.set(ValueLayout.JAVA_BOOLEAN, 0, false)
      Status.check(
        runtimeOfflineRegionGetTakeResultFunction()
          .invokeWithArguments(runtime, operationId, outSnapshot, outFound) as Int
      )
      if (!outFound.get(ValueLayout.JAVA_BOOLEAN, 0)) {
        return@use null
      }
      offlineRegionSnapshot(outSnapshot.get(ValueLayout.ADDRESS, 0))
    }

  internal fun takeOfflineRegionsResult(
    runtime: MemorySegment,
    operationId: Long,
  ): List<OfflineRegionInfo> =
    takeOfflineRegionList(runtime, operationId, runtimeOfflineRegionsListTakeResultFunction())

  internal fun takeMergeOfflineRegionsDatabaseResult(
    runtime: MemorySegment,
    operationId: Long,
  ): List<OfflineRegionInfo> =
    takeOfflineRegionList(
      runtime,
      operationId,
      runtimeOfflineRegionsMergeDatabaseTakeResultFunction(),
    )

  internal fun takeUpdateOfflineRegionMetadataResult(
    runtime: MemorySegment,
    operationId: Long,
  ): OfflineRegionInfo =
    takeOfflineRegionSnapshot(
      runtime,
      operationId,
      runtimeOfflineRegionUpdateMetadataTakeResultFunction(),
    )

  internal fun pollRuntimeEvent(runtime: MemorySegment): NativeRuntimeEvent? =
    Arena.ofConfined().use { arena ->
      val event = arena.allocate(RUNTIME_EVENT_SIZE)
      event.set(ValueLayout.JAVA_INT, RUNTIME_EVENT_SIZE_OFFSET, RUNTIME_EVENT_SIZE.toInt())
      val hasEvent = arena.allocate(ValueLayout.JAVA_BOOLEAN)
      hasEvent.set(ValueLayout.JAVA_BOOLEAN, 0, false)
      Status.check(runtimePollEventFunction().invokeWithArguments(runtime, event, hasEvent) as Int)
      if (!hasEvent.get(ValueLayout.JAVA_BOOLEAN, 0)) {
        return@use null
      }
      val payload = event.get(ValueLayout.ADDRESS, RUNTIME_EVENT_PAYLOAD_OFFSET)
      val payloadSize = event.get(ValueLayout.JAVA_LONG, RUNTIME_EVENT_PAYLOAD_SIZE_OFFSET)
      val payloadType = event.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_PAYLOAD_TYPE_OFFSET)
      val message = event.get(ValueLayout.ADDRESS, RUNTIME_EVENT_MESSAGE_OFFSET)
      val messageSize = event.get(ValueLayout.JAVA_LONG, RUNTIME_EVENT_MESSAGE_SIZE_OFFSET)
      NativeRuntimeEvent(
        type = event.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_TYPE_OFFSET),
        sourceType = event.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_SOURCE_TYPE_OFFSET),
        sourceAddress = event.get(ValueLayout.ADDRESS, RUNTIME_EVENT_SOURCE_OFFSET).address(),
        code = event.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_CODE_OFFSET),
        payload = runtimeEventPayload(payloadType, payload, payloadSize),
        message = copyString(message, messageSize),
      )
    }

  private fun intFunction(name: String): MethodHandle =
    downcall(name, FunctionDescriptor.of(ValueLayout.JAVA_INT))

  private fun statusOutFunction(name: String): MethodHandle =
    downcall(name, FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS))

  private fun statusInFunction(name: String): MethodHandle =
    downcall(name, FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.JAVA_INT))

  private fun logSetCallbackFunction(): MethodHandle =
    downcall(
      "mln_log_set_callback",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun projectedMetersForLatLngFunction(): MethodHandle =
    downcall(
      "mln_projected_meters_for_lat_lng",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, latLngLayout, ValueLayout.ADDRESS),
    )

  private fun latLngForProjectedMetersFunction(): MethodHandle =
    downcall(
      "mln_lat_lng_for_projected_meters",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, projectedMetersLayout, ValueLayout.ADDRESS),
    )

  private fun runtimeCreateFunction(): MethodHandle =
    downcall(
      "mln_runtime_create",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun runtimeStatusFunction(name: String): MethodHandle =
    downcall(name, FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS))

  private fun runtimeAmbientCacheOperationStartFunction(): MethodHandle =
    downcall(
      "mln_runtime_run_ambient_cache_operation_start",
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
      ),
    )

  private fun runtimeOfflineRegionCreateStartFunction(): MethodHandle =
    downcall(
      "mln_runtime_offline_region_create_start",
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.ADDRESS,
      ),
    )

  private fun runtimeOfflineOperationDiscardFunction(): MethodHandle =
    downcall(
      "mln_runtime_offline_operation_discard",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG),
    )

  private fun runtimeSetResourceProviderFunction(): MethodHandle =
    downcall(
      "mln_runtime_set_resource_provider",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun runtimeSetResourceTransformFunction(): MethodHandle =
    downcall(
      "mln_runtime_set_resource_transform",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun runtimeClearResourceTransformFunction(): MethodHandle =
    downcall(
      "mln_runtime_clear_resource_transform",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS),
    )

  private fun resourceTransformResponseSetUrlFunction(): MethodHandle =
    downcall(
      "mln_resource_transform_response_set_url",
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
      ),
    )

  private fun resourceRequestCompleteFunction(): MethodHandle =
    downcall(
      "mln_resource_request_complete",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun resourceRequestCancelledFunction(): MethodHandle =
    downcall(
      "mln_resource_request_cancelled",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun resourceRequestReleaseFunction(): MethodHandle =
    downcall("mln_resource_request_release", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS))

  private fun mapCreateFunction(): MethodHandle =
    downcall(
      "mln_map_create",
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
      ),
    )

  private fun mapStatusFunction(name: String): MethodHandle =
    downcall(name, FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS))

  private fun mapIntStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.JAVA_INT),
    )

  private fun mapBooleanStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.JAVA_BOOLEAN),
    )

  private fun mapAddressStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun mapStringViewAddressStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        ValueLayout.ADDRESS,
      ),
    )

  private fun mapAddressStringViewStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
        stringViewLayout,
      ),
    )

  private fun mapTwoStringViewsStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        stringViewLayout,
      ),
    )

  private fun mapThreeStringViewsStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        stringViewLayout,
        stringViewLayout,
      ),
    )

  private fun mapTwoStringViewsAddressStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        stringViewLayout,
        ValueLayout.ADDRESS,
      ),
    )

  private fun mapStringViewLatLngDoubleStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        latLngLayout,
        ValueLayout.JAVA_DOUBLE,
      ),
    )

  private fun mapStringViewDoubleStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        ValueLayout.JAVA_DOUBLE,
      ),
    )

  private fun mapStringViewIntStringViewStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        ValueLayout.JAVA_INT,
        stringViewLayout,
      ),
    )

  private fun mapStringViewTwoAddressStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
      ),
    )

  private fun mapStringViewAddressLongTwoAddressStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
      ),
    )

  private fun mapStringViewAddressLongStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
      ),
    )

  private fun mapStringViewAddressLongStringViewStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        stringViewLayout,
      ),
    )

  private fun mapStringViewAddressLongAddressStatusFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.ADDRESS,
      ),
    )

  private fun mapListStyleSourceIdsFunction(): MethodHandle =
    downcall(
      "mln_map_list_style_source_ids",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun mapListStyleLayerIdsFunction(): MethodHandle =
    downcall(
      "mln_map_list_style_layer_ids",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun styleIdListCountFunction(): MethodHandle =
    downcall(
      "mln_style_id_list_count",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun styleIdListGetFunction(): MethodHandle =
    downcall(
      "mln_style_id_list_get",
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.ADDRESS,
      ),
    )

  private fun styleIdListDestroyFunction(): MethodHandle =
    downcall("mln_style_id_list_destroy", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS))

  private fun copyStyleSourceAttributionFunction(): MethodHandle =
    downcall(
      "mln_map_copy_style_source_attribution",
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        stringViewLayout,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
      ),
    )

  private fun jsonSnapshotGetFunction(): MethodHandle =
    downcall(
      "mln_json_snapshot_get",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun jsonSnapshotDestroyFunction(): MethodHandle =
    downcall("mln_json_snapshot_destroy", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS))

  private fun runtimeOfflineRegionStatusTakeResultFunction(): MethodHandle =
    downcall(
      "mln_runtime_offline_region_get_status_take_result",
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.ADDRESS,
      ),
    )

  private fun runtimeOfflineRegionCreateTakeResultFunction(): MethodHandle =
    runtimeOfflineOperationSnapshotTakeResultFunction(
      "mln_runtime_offline_region_create_take_result"
    )

  private fun runtimeOfflineRegionGetTakeResultFunction(): MethodHandle =
    downcall(
      "mln_runtime_offline_region_get_take_result",
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
      ),
    )

  private fun runtimeOfflineRegionsListTakeResultFunction(): MethodHandle =
    runtimeOfflineOperationListTakeResultFunction("mln_runtime_offline_regions_list_take_result")

  private fun runtimeOfflineRegionsMergeDatabaseTakeResultFunction(): MethodHandle =
    runtimeOfflineOperationListTakeResultFunction(
      "mln_runtime_offline_regions_merge_database_take_result"
    )

  private fun runtimeOfflineRegionUpdateMetadataTakeResultFunction(): MethodHandle =
    runtimeOfflineOperationSnapshotTakeResultFunction(
      "mln_runtime_offline_region_update_metadata_take_result"
    )

  private fun runtimeOfflineOperationSnapshotTakeResultFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.ADDRESS,
      ),
    )

  private fun runtimeOfflineOperationListTakeResultFunction(name: String): MethodHandle =
    runtimeOfflineOperationSnapshotTakeResultFunction(name)

  private fun offlineRegionSnapshotGetFunction(): MethodHandle =
    downcall(
      "mln_offline_region_snapshot_get",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun offlineRegionSnapshotDestroyFunction(): MethodHandle =
    downcall("mln_offline_region_snapshot_destroy", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS))

  private fun offlineRegionListCountFunction(): MethodHandle =
    downcall(
      "mln_offline_region_list_count",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun offlineRegionListGetFunction(): MethodHandle =
    downcall(
      "mln_offline_region_list_get",
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.ADDRESS,
      ),
    )

  private fun offlineRegionListDestroyFunction(): MethodHandle =
    downcall("mln_offline_region_list_destroy", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS))

  private fun runtimePollEventFunction(): MethodHandle =
    downcall(
      "mln_runtime_poll_event",
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
      ),
    )

  private fun addTileSourceUrl(
    functionName: String,
    map: MemorySegment,
    sourceId: String,
    url: String,
    options: TileSourceOptions?,
  ) {
    Arena.ofConfined().use { arena ->
      Status.check(
        mapTwoStringViewsAddressStatusFunction(functionName)
          .invokeWithArguments(
            map,
            stringView(arena, sourceId),
            stringView(arena, url),
            tileSourceOptions(arena, options),
          ) as Int
      )
    }
  }

  private fun addTileSourceTiles(
    functionName: String,
    map: MemorySegment,
    sourceId: String,
    tiles: List<String>,
    options: TileSourceOptions?,
  ) {
    val tileSnapshot = tiles.toList()
    Arena.ofConfined().use { arena ->
      Status.check(
        mapStringViewAddressLongAddressStatusFunction(functionName)
          .invokeWithArguments(
            map,
            stringView(arena, sourceId),
            stringViewArray(arena, tileSnapshot),
            tileSnapshot.size.toLong(),
            tileSourceOptions(arena, options),
          ) as Int
      )
    }
  }

  private fun startRuntimeOperation(name: String, runtime: MemorySegment): Long =
    Arena.ofConfined().use { arena ->
      val outOperationId = arena.allocate(ValueLayout.JAVA_LONG)
      Status.check(
        runtimeOperationStartFunction(name).invokeWithArguments(runtime, outOperationId) as Int
      )
      outOperationId.get(ValueLayout.JAVA_LONG, 0)
    }

  private fun startRuntimeLongOperation(name: String, runtime: MemorySegment, value: Long): Long =
    Arena.ofConfined().use { arena ->
      val outOperationId = arena.allocate(ValueLayout.JAVA_LONG)
      Status.check(
        runtimeLongOperationStartFunction(name).invokeWithArguments(runtime, value, outOperationId)
          as Int
      )
      outOperationId.get(ValueLayout.JAVA_LONG, 0)
    }

  private fun runtimeOperationStartFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS),
    )

  private fun runtimeLongOperationStartFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.ADDRESS,
      ),
    )

  private fun runtimeAddressOperationStartFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
      ),
    )

  private fun runtimeLongAddressLongOperationStartFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.ADDRESS,
      ),
    )

  private fun runtimeLongBooleanOperationStartFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.JAVA_BOOLEAN,
        ValueLayout.ADDRESS,
      ),
    )

  private fun runtimeLongIntOperationStartFunction(name: String): MethodHandle =
    downcall(
      name,
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.JAVA_LONG,
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
      ),
    )

  private fun runtimeOptions(options: RuntimeOptions, arena: Arena): MemorySegment {
    val nativeOptions = arena.allocate(runtimeOptionsLayout)
    nativeOptions.set(ValueLayout.JAVA_INT, 0, runtimeOptionsLayout.byteSize().toInt())
    var flags = 0
    nativeOptions.set(
      ValueLayout.ADDRESS,
      RUNTIME_OPTIONS_ASSET_PATH,
      optionalCString(arena, options.assetPath),
    )
    nativeOptions.set(
      ValueLayout.ADDRESS,
      RUNTIME_OPTIONS_CACHE_PATH,
      optionalCString(arena, options.cachePath),
    )
    options.maximumCacheSize?.let { maximumCacheSize ->
      flags = flags or RUNTIME_OPTION_MAXIMUM_CACHE_SIZE
      nativeOptions.set(ValueLayout.JAVA_LONG, RUNTIME_OPTIONS_MAXIMUM_CACHE_SIZE, maximumCacheSize)
    }
    nativeOptions.set(ValueLayout.JAVA_INT, Int.SIZE_BYTES.toLong(), flags)
    return nativeOptions
  }

  private fun optionalCString(arena: Arena, value: String?): MemorySegment =
    value?.let { cString(arena, it) } ?: MemorySegment.NULL

  private fun cString(arena: Arena, value: String): MemorySegment {
    require('\u0000' !in value) { "C string inputs must not contain embedded NUL characters" }
    return arena.allocateFrom(value)
  }

  private fun stringView(arena: Arena, value: String): MemorySegment {
    val bytes = value.toByteArray(StandardCharsets.UTF_8)
    val segment = arena.allocate(STRING_VIEW_SIZE)
    segment.set(ValueLayout.ADDRESS, STRING_VIEW_DATA_OFFSET, nativeBytes(arena, bytes))
    segment.set(ValueLayout.JAVA_LONG, STRING_VIEW_SIZE_OFFSET, bytes.size.toLong())
    return segment
  }

  private fun stringViewArray(arena: Arena, values: List<String>): MemorySegment {
    if (values.isEmpty()) {
      return MemorySegment.NULL
    }
    val array = arena.allocate(STRING_VIEW_SIZE * values.size)
    values.forEachIndexed { index, value ->
      array.asSlice(index * STRING_VIEW_SIZE, STRING_VIEW_SIZE).copyFrom(stringView(arena, value))
    }
    return array
  }

  private fun tileSourceOptions(arena: Arena, value: TileSourceOptions?): MemorySegment {
    if (value == null) {
      return MemorySegment.NULL
    }
    val segment = arena.allocate(TILE_SOURCE_OPTIONS_SIZE)
    var fields = 0
    segment.set(
      ValueLayout.JAVA_INT,
      TILE_SOURCE_OPTIONS_SIZE_OFFSET,
      TILE_SOURCE_OPTIONS_SIZE.toInt(),
    )
    value.minZoom?.let {
      fields = fields or TILE_SOURCE_OPTION_MIN_ZOOM
      segment.set(ValueLayout.JAVA_DOUBLE, TILE_SOURCE_OPTIONS_MIN_ZOOM_OFFSET, it)
    }
    value.maxZoom?.let {
      fields = fields or TILE_SOURCE_OPTION_MAX_ZOOM
      segment.set(ValueLayout.JAVA_DOUBLE, TILE_SOURCE_OPTIONS_MAX_ZOOM_OFFSET, it)
    }
    value.attribution?.let {
      fields = fields or TILE_SOURCE_OPTION_ATTRIBUTION
      segment
        .asSlice(TILE_SOURCE_OPTIONS_ATTRIBUTION_OFFSET, STRING_VIEW_SIZE)
        .copyFrom(stringView(arena, it))
    }
    value.scheme?.let {
      fields = fields or TILE_SOURCE_OPTION_SCHEME
      segment.set(ValueLayout.JAVA_INT, TILE_SOURCE_OPTIONS_SCHEME_OFFSET, it.nativeValue)
    }
    value.bounds?.let {
      fields = fields or TILE_SOURCE_OPTION_BOUNDS
      latLngBounds(it, segment.asSlice(TILE_SOURCE_OPTIONS_BOUNDS_OFFSET))
    }
    value.tileSize?.let {
      fields = fields or TILE_SOURCE_OPTION_TILE_SIZE
      segment.set(ValueLayout.JAVA_INT, TILE_SOURCE_OPTIONS_TILE_SIZE_OFFSET, it)
    }
    value.vectorEncoding?.let {
      fields = fields or TILE_SOURCE_OPTION_VECTOR_ENCODING
      segment.set(ValueLayout.JAVA_INT, TILE_SOURCE_OPTIONS_VECTOR_ENCODING_OFFSET, it.nativeValue)
    }
    value.rasterDemEncoding?.let {
      fields = fields or TILE_SOURCE_OPTION_RASTER_ENCODING
      segment.set(ValueLayout.JAVA_INT, TILE_SOURCE_OPTIONS_RASTER_ENCODING_OFFSET, it.nativeValue)
    }
    segment.set(ValueLayout.JAVA_INT, TILE_SOURCE_OPTIONS_FIELDS_OFFSET, fields)
    return segment
  }

  private fun edgeInsets(segment: MemorySegment): EdgeInsets =
    EdgeInsets(
      segment.get(ValueLayout.JAVA_DOUBLE, EDGE_INSETS_TOP_OFFSET),
      segment.get(ValueLayout.JAVA_DOUBLE, EDGE_INSETS_LEFT_OFFSET),
      segment.get(ValueLayout.JAVA_DOUBLE, EDGE_INSETS_BOTTOM_OFFSET),
      segment.get(ValueLayout.JAVA_DOUBLE, EDGE_INSETS_RIGHT_OFFSET),
    )

  private fun writeEdgeInsets(segment: MemorySegment, value: EdgeInsets) {
    segment.set(ValueLayout.JAVA_DOUBLE, EDGE_INSETS_TOP_OFFSET, value.top)
    segment.set(ValueLayout.JAVA_DOUBLE, EDGE_INSETS_LEFT_OFFSET, value.left)
    segment.set(ValueLayout.JAVA_DOUBLE, EDGE_INSETS_BOTTOM_OFFSET, value.bottom)
    segment.set(ValueLayout.JAVA_DOUBLE, EDGE_INSETS_RIGHT_OFFSET, value.right)
  }

  private fun viewportOptionsDefault(arena: Arena): MemorySegment {
    val segment = arena.allocate(VIEWPORT_OPTIONS_SIZE)
    segment.set(ValueLayout.JAVA_INT, VIEWPORT_OPTIONS_SIZE_OFFSET, VIEWPORT_OPTIONS_SIZE.toInt())
    return segment
  }

  private fun viewportOptions(arena: Arena, value: ViewportOptions): MemorySegment {
    val segment = viewportOptionsDefault(arena)
    var fields = 0
    value.northOrientation?.let {
      require(it.isKnown) { "Unknown north orientation cannot be used as input: ${it.nativeValue}" }
      fields = fields or VIEWPORT_OPTION_NORTH_ORIENTATION
      segment.set(ValueLayout.JAVA_INT, VIEWPORT_OPTIONS_NORTH_ORIENTATION_OFFSET, it.nativeValue)
    }
    value.constrainMode?.let {
      require(it.isKnown) { "Unknown constrain mode cannot be used as input: ${it.nativeValue}" }
      fields = fields or VIEWPORT_OPTION_CONSTRAIN_MODE
      segment.set(ValueLayout.JAVA_INT, VIEWPORT_OPTIONS_CONSTRAIN_MODE_OFFSET, it.nativeValue)
    }
    value.viewportMode?.let {
      require(it.isKnown) { "Unknown viewport mode cannot be used as input: ${it.nativeValue}" }
      fields = fields or VIEWPORT_OPTION_VIEWPORT_MODE
      segment.set(ValueLayout.JAVA_INT, VIEWPORT_OPTIONS_VIEWPORT_MODE_OFFSET, it.nativeValue)
    }
    value.frustumOffset?.let {
      fields = fields or VIEWPORT_OPTION_FRUSTUM_OFFSET
      writeEdgeInsets(segment.asSlice(VIEWPORT_OPTIONS_FRUSTUM_OFFSET_OFFSET, EDGE_INSETS_SIZE), it)
    }
    segment.set(ValueLayout.JAVA_INT, VIEWPORT_OPTIONS_FIELDS_OFFSET, fields)
    return segment
  }

  private fun readViewportOptions(segment: MemorySegment): ViewportOptions {
    val fields = segment.get(ValueLayout.JAVA_INT, VIEWPORT_OPTIONS_FIELDS_OFFSET)
    return ViewportOptions().apply {
      if ((fields and VIEWPORT_OPTION_NORTH_ORIENTATION) != 0) {
        northOrientation =
          NorthOrientation.fromNative(
            segment.get(ValueLayout.JAVA_INT, VIEWPORT_OPTIONS_NORTH_ORIENTATION_OFFSET)
          )
      }
      if ((fields and VIEWPORT_OPTION_CONSTRAIN_MODE) != 0) {
        constrainMode =
          ConstrainMode.fromNative(
            segment.get(ValueLayout.JAVA_INT, VIEWPORT_OPTIONS_CONSTRAIN_MODE_OFFSET)
          )
      }
      if ((fields and VIEWPORT_OPTION_VIEWPORT_MODE) != 0) {
        viewportMode =
          ViewportMode.fromNative(
            segment.get(ValueLayout.JAVA_INT, VIEWPORT_OPTIONS_VIEWPORT_MODE_OFFSET)
          )
      }
      if ((fields and VIEWPORT_OPTION_FRUSTUM_OFFSET) != 0) {
        frustumOffset =
          edgeInsets(segment.asSlice(VIEWPORT_OPTIONS_FRUSTUM_OFFSET_OFFSET, EDGE_INSETS_SIZE))
      }
    }
  }

  private fun tileOptionsDefault(arena: Arena): MemorySegment {
    val segment = arena.allocate(TILE_OPTIONS_SIZE)
    segment.set(ValueLayout.JAVA_INT, TILE_OPTIONS_SIZE_OFFSET, TILE_OPTIONS_SIZE.toInt())
    return segment
  }

  private fun tileOptions(arena: Arena, value: TileOptions): MemorySegment {
    val segment = tileOptionsDefault(arena)
    var fields = 0
    value.prefetchZoomDelta?.let {
      fields = fields or TILE_OPTION_PREFETCH_ZOOM_DELTA
      segment.set(ValueLayout.JAVA_INT, TILE_OPTIONS_PREFETCH_ZOOM_DELTA_OFFSET, it)
    }
    value.lodMinRadius?.let {
      fields = fields or TILE_OPTION_LOD_MIN_RADIUS
      segment.set(ValueLayout.JAVA_DOUBLE, TILE_OPTIONS_LOD_MIN_RADIUS_OFFSET, it)
    }
    value.lodScale?.let {
      fields = fields or TILE_OPTION_LOD_SCALE
      segment.set(ValueLayout.JAVA_DOUBLE, TILE_OPTIONS_LOD_SCALE_OFFSET, it)
    }
    value.lodPitchThreshold?.let {
      fields = fields or TILE_OPTION_LOD_PITCH_THRESHOLD
      segment.set(ValueLayout.JAVA_DOUBLE, TILE_OPTIONS_LOD_PITCH_THRESHOLD_OFFSET, it)
    }
    value.lodZoomShift?.let {
      fields = fields or TILE_OPTION_LOD_ZOOM_SHIFT
      segment.set(ValueLayout.JAVA_DOUBLE, TILE_OPTIONS_LOD_ZOOM_SHIFT_OFFSET, it)
    }
    value.lodMode?.let {
      require(it.isKnown) { "Unknown tile LOD mode cannot be used as input: ${it.nativeValue}" }
      fields = fields or TILE_OPTION_LOD_MODE
      segment.set(ValueLayout.JAVA_INT, TILE_OPTIONS_LOD_MODE_OFFSET, it.nativeValue)
    }
    segment.set(ValueLayout.JAVA_INT, TILE_OPTIONS_FIELDS_OFFSET, fields)
    return segment
  }

  private fun readTileOptions(segment: MemorySegment): TileOptions {
    val fields = segment.get(ValueLayout.JAVA_INT, TILE_OPTIONS_FIELDS_OFFSET)
    return TileOptions().apply {
      if ((fields and TILE_OPTION_PREFETCH_ZOOM_DELTA) != 0) {
        prefetchZoomDelta =
          segment.get(ValueLayout.JAVA_INT, TILE_OPTIONS_PREFETCH_ZOOM_DELTA_OFFSET)
      }
      if ((fields and TILE_OPTION_LOD_MIN_RADIUS) != 0) {
        lodMinRadius = segment.get(ValueLayout.JAVA_DOUBLE, TILE_OPTIONS_LOD_MIN_RADIUS_OFFSET)
      }
      if ((fields and TILE_OPTION_LOD_SCALE) != 0) {
        lodScale = segment.get(ValueLayout.JAVA_DOUBLE, TILE_OPTIONS_LOD_SCALE_OFFSET)
      }
      if ((fields and TILE_OPTION_LOD_PITCH_THRESHOLD) != 0) {
        lodPitchThreshold =
          segment.get(ValueLayout.JAVA_DOUBLE, TILE_OPTIONS_LOD_PITCH_THRESHOLD_OFFSET)
      }
      if ((fields and TILE_OPTION_LOD_ZOOM_SHIFT) != 0) {
        lodZoomShift = segment.get(ValueLayout.JAVA_DOUBLE, TILE_OPTIONS_LOD_ZOOM_SHIFT_OFFSET)
      }
      if ((fields and TILE_OPTION_LOD_MODE) != 0) {
        lodMode =
          TileLodMode.fromNative(segment.get(ValueLayout.JAVA_INT, TILE_OPTIONS_LOD_MODE_OFFSET))
      }
    }
  }

  private fun projectionModeDefault(arena: Arena): MemorySegment {
    val segment = arena.allocate(PROJECTION_MODE_SIZE)
    segment.set(ValueLayout.JAVA_INT, PROJECTION_MODE_SIZE_OFFSET, PROJECTION_MODE_SIZE.toInt())
    return segment
  }

  private fun projectionModeOptions(arena: Arena, value: ProjectionModeOptions): MemorySegment {
    val segment = projectionModeDefault(arena)
    var fields = 0
    value.axonometric?.let {
      fields = fields or PROJECTION_MODE_AXONOMETRIC
      segment.set(ValueLayout.JAVA_BOOLEAN, PROJECTION_MODE_AXONOMETRIC_OFFSET, it)
    }
    value.xSkew?.let {
      fields = fields or PROJECTION_MODE_X_SKEW
      segment.set(ValueLayout.JAVA_DOUBLE, PROJECTION_MODE_X_SKEW_OFFSET, it)
    }
    value.ySkew?.let {
      fields = fields or PROJECTION_MODE_Y_SKEW
      segment.set(ValueLayout.JAVA_DOUBLE, PROJECTION_MODE_Y_SKEW_OFFSET, it)
    }
    segment.set(ValueLayout.JAVA_INT, PROJECTION_MODE_FIELDS_OFFSET, fields)
    return segment
  }

  private fun projectionModeOptions(segment: MemorySegment): ProjectionModeOptions {
    val fields = segment.get(ValueLayout.JAVA_INT, PROJECTION_MODE_FIELDS_OFFSET)
    return ProjectionModeOptions().apply {
      if ((fields and PROJECTION_MODE_AXONOMETRIC) != 0) {
        axonometric = segment.get(ValueLayout.JAVA_BOOLEAN, PROJECTION_MODE_AXONOMETRIC_OFFSET)
      }
      if ((fields and PROJECTION_MODE_X_SKEW) != 0) {
        xSkew = segment.get(ValueLayout.JAVA_DOUBLE, PROJECTION_MODE_X_SKEW_OFFSET)
      }
      if ((fields and PROJECTION_MODE_Y_SKEW) != 0) {
        ySkew = segment.get(ValueLayout.JAVA_DOUBLE, PROJECTION_MODE_Y_SKEW_OFFSET)
      }
    }
  }

  private fun jsonValue(arena: Arena, value: JsonValue): MemorySegment {
    val segment = arena.allocate(JSON_VALUE_SIZE)
    writeJson(segment, value, arena, 0)
    return segment
  }

  private fun premultipliedRgba8Image(arena: Arena, value: PremultipliedRgba8Image): MemorySegment {
    val pixels = value.pixels
    val segment = arena.allocate(PREMULTIPLIED_RGBA8_IMAGE_SIZE)
    segment.set(
      ValueLayout.JAVA_INT,
      PREMULTIPLIED_RGBA8_IMAGE_SIZE_OFFSET,
      PREMULTIPLIED_RGBA8_IMAGE_SIZE.toInt(),
    )
    segment.set(ValueLayout.JAVA_INT, PREMULTIPLIED_RGBA8_IMAGE_WIDTH_OFFSET, value.width)
    segment.set(ValueLayout.JAVA_INT, PREMULTIPLIED_RGBA8_IMAGE_HEIGHT_OFFSET, value.height)
    segment.set(ValueLayout.JAVA_INT, PREMULTIPLIED_RGBA8_IMAGE_STRIDE_OFFSET, value.stride)
    segment.set(
      ValueLayout.ADDRESS,
      PREMULTIPLIED_RGBA8_IMAGE_PIXELS_OFFSET,
      nativeBytes(arena, pixels),
    )
    segment.set(
      ValueLayout.JAVA_LONG,
      PREMULTIPLIED_RGBA8_IMAGE_BYTE_LENGTH_OFFSET,
      pixels.size.toLong(),
    )
    return segment
  }

  private fun styleImageOptions(arena: Arena, value: StyleImageOptions): MemorySegment {
    val segment = arena.allocate(STYLE_IMAGE_OPTIONS_SIZE)
    var fields = 0
    segment.set(
      ValueLayout.JAVA_INT,
      STYLE_IMAGE_OPTIONS_SIZE_OFFSET,
      STYLE_IMAGE_OPTIONS_SIZE.toInt(),
    )
    segment.set(ValueLayout.JAVA_FLOAT, STYLE_IMAGE_OPTIONS_PIXEL_RATIO_OFFSET, DEFAULT_PIXEL_RATIO)
    segment.set(ValueLayout.JAVA_BOOLEAN, STYLE_IMAGE_OPTIONS_SDF_OFFSET, false)
    value.pixelRatio?.let {
      fields = fields or STYLE_IMAGE_OPTION_PIXEL_RATIO
      segment.set(ValueLayout.JAVA_FLOAT, STYLE_IMAGE_OPTIONS_PIXEL_RATIO_OFFSET, it)
    }
    value.sdf?.let {
      fields = fields or STYLE_IMAGE_OPTION_SDF
      segment.set(ValueLayout.JAVA_BOOLEAN, STYLE_IMAGE_OPTIONS_SDF_OFFSET, it)
    }
    segment.set(ValueLayout.JAVA_INT, STYLE_IMAGE_OPTIONS_FIELDS_OFFSET, fields)
    return segment
  }

  private fun styleImageInfoDefault(arena: Arena): MemorySegment {
    val segment = arena.allocate(STYLE_IMAGE_INFO_SIZE)
    segment.set(ValueLayout.JAVA_INT, STYLE_IMAGE_INFO_SIZE_OFFSET, STYLE_IMAGE_INFO_SIZE.toInt())
    segment.set(ValueLayout.JAVA_FLOAT, STYLE_IMAGE_INFO_PIXEL_RATIO_OFFSET, DEFAULT_PIXEL_RATIO)
    return segment
  }

  private fun styleImageInfo(segment: MemorySegment): StyleImageInfo =
    StyleImageInfo(
      segment.get(ValueLayout.JAVA_INT, STYLE_IMAGE_INFO_WIDTH_OFFSET),
      segment.get(ValueLayout.JAVA_INT, STYLE_IMAGE_INFO_HEIGHT_OFFSET),
      segment.get(ValueLayout.JAVA_INT, STYLE_IMAGE_INFO_STRIDE_OFFSET),
      segment.get(ValueLayout.JAVA_LONG, STYLE_IMAGE_INFO_BYTE_LENGTH_OFFSET),
      segment.get(ValueLayout.JAVA_FLOAT, STYLE_IMAGE_INFO_PIXEL_RATIO_OFFSET),
      segment.get(ValueLayout.JAVA_BOOLEAN, STYLE_IMAGE_INFO_SDF_OFFSET),
    )

  private fun debugOptions(mask: Int): Set<DebugOption> =
    DebugOption.entries.filterTo(mutableSetOf()) { option -> (mask and option.nativeMask) != 0 }

  private fun writeJson(segment: MemorySegment, value: JsonValue, arena: Arena, depth: Int) {
    require(depth <= JsonValue.MAX_DESCRIPTOR_DEPTH) {
      "JSON descriptor depth exceeds ${JsonValue.MAX_DESCRIPTOR_DEPTH}"
    }
    segment.set(ValueLayout.JAVA_INT, JSON_VALUE_SIZE_OFFSET, JSON_VALUE_SIZE.toInt())
    when (value) {
      JsonValue.Null -> segment.set(ValueLayout.JAVA_INT, JSON_VALUE_TYPE_OFFSET, JSON_NULL)
      is JsonValue.Bool -> {
        segment.set(ValueLayout.JAVA_INT, JSON_VALUE_TYPE_OFFSET, JSON_BOOL)
        segment.set(ValueLayout.JAVA_BOOLEAN, JSON_VALUE_DATA_OFFSET, value.value)
      }
      is JsonValue.UInt -> {
        segment.set(ValueLayout.JAVA_INT, JSON_VALUE_TYPE_OFFSET, JSON_UINT)
        segment.set(ValueLayout.JAVA_LONG, JSON_VALUE_DATA_OFFSET, value.value)
      }
      is JsonValue.Int -> {
        segment.set(ValueLayout.JAVA_INT, JSON_VALUE_TYPE_OFFSET, JSON_INT)
        segment.set(ValueLayout.JAVA_LONG, JSON_VALUE_DATA_OFFSET, value.value)
      }
      is JsonValue.DoubleValue -> {
        segment.set(ValueLayout.JAVA_INT, JSON_VALUE_TYPE_OFFSET, JSON_DOUBLE)
        segment.set(ValueLayout.JAVA_DOUBLE, JSON_VALUE_DATA_OFFSET, value.value)
      }
      is JsonValue.StringValue -> {
        segment.set(ValueLayout.JAVA_INT, JSON_VALUE_TYPE_OFFSET, JSON_STRING)
        segment
          .asSlice(JSON_VALUE_DATA_OFFSET, STRING_VIEW_SIZE)
          .copyFrom(stringView(arena, value.value))
      }
      is JsonValue.Array -> {
        segment.set(ValueLayout.JAVA_INT, JSON_VALUE_TYPE_OFFSET, JSON_ARRAY)
        val nativeValues =
          if (value.values.isEmpty()) MemorySegment.NULL
          else arena.allocate(JSON_VALUE_SIZE * value.values.size)
        value.values.forEachIndexed { index, child ->
          writeJson(
            nativeValues.asSlice(index * JSON_VALUE_SIZE, JSON_VALUE_SIZE),
            child,
            arena,
            depth + 1,
          )
        }
        segment.set(ValueLayout.ADDRESS, JSON_VALUE_DATA_OFFSET, nativeValues)
        segment.set(
          ValueLayout.JAVA_LONG,
          JSON_VALUE_DATA_OFFSET + Long.SIZE_BYTES,
          value.values.size.toLong(),
        )
      }
      is JsonValue.ObjectValue -> {
        segment.set(ValueLayout.JAVA_INT, JSON_VALUE_TYPE_OFFSET, JSON_OBJECT)
        val nativeMembers =
          if (value.members.isEmpty()) MemorySegment.NULL
          else arena.allocate(JSON_MEMBER_SIZE * value.members.size)
        value.members.forEachIndexed { index, member ->
          val memberSegment = nativeMembers.asSlice(index * JSON_MEMBER_SIZE, JSON_MEMBER_SIZE)
          memberSegment
            .asSlice(JSON_MEMBER_KEY_OFFSET, STRING_VIEW_SIZE)
            .copyFrom(stringView(arena, member.key))
          val nativeValue = arena.allocate(JSON_VALUE_SIZE)
          writeJson(nativeValue, member.value, arena, depth + 1)
          memberSegment.set(ValueLayout.ADDRESS, JSON_MEMBER_VALUE_OFFSET, nativeValue)
        }
        segment.set(ValueLayout.ADDRESS, JSON_VALUE_DATA_OFFSET, nativeMembers)
        segment.set(
          ValueLayout.JAVA_LONG,
          JSON_VALUE_DATA_OFFSET + Long.SIZE_BYTES,
          value.members.size.toLong(),
        )
      }
      is JsonValue.Unknown ->
        throw IllegalArgumentException("unknown JSON values cannot be used as input")
    }
  }

  private fun jsonSnapshot(snapshot: MemorySegment): JsonValue? {
    if (snapshot == MemorySegment.NULL) {
      return null
    }
    return try {
      Arena.ofConfined().use { arena ->
        val outValue = arena.allocate(ValueLayout.ADDRESS)
        outValue.set(ValueLayout.ADDRESS, 0, MemorySegment.NULL)
        Status.check(jsonSnapshotGetFunction().invokeWithArguments(snapshot, outValue) as Int)
        val value = outValue.get(ValueLayout.ADDRESS, 0)
        if (value == MemorySegment.NULL) null else readJson(value.reinterpret(JSON_VALUE_SIZE), 0)
      }
    } finally {
      jsonSnapshotDestroyFunction().invokeWithArguments(snapshot)
    }
  }

  private fun readJson(segment: MemorySegment, depth: Int): JsonValue {
    require(depth <= JsonValue.MAX_DESCRIPTOR_DEPTH) {
      "JSON descriptor depth exceeds ${JsonValue.MAX_DESCRIPTOR_DEPTH}"
    }
    return when (val type = segment.get(ValueLayout.JAVA_INT, JSON_VALUE_TYPE_OFFSET)) {
      JSON_NULL -> JsonValue.Null
      JSON_BOOL -> JsonValue.Bool(segment.get(ValueLayout.JAVA_BOOLEAN, JSON_VALUE_DATA_OFFSET))
      JSON_UINT -> JsonValue.UInt(segment.get(ValueLayout.JAVA_LONG, JSON_VALUE_DATA_OFFSET))
      JSON_INT -> JsonValue.Int(segment.get(ValueLayout.JAVA_LONG, JSON_VALUE_DATA_OFFSET))
      JSON_DOUBLE ->
        JsonValue.DoubleValue(segment.get(ValueLayout.JAVA_DOUBLE, JSON_VALUE_DATA_OFFSET))
      JSON_STRING ->
        JsonValue.StringValue(stringView(segment.asSlice(JSON_VALUE_DATA_OFFSET, STRING_VIEW_SIZE)))
      JSON_ARRAY -> {
        val count =
          Math.toIntExact(
            segment.get(ValueLayout.JAVA_LONG, JSON_VALUE_DATA_OFFSET + Long.SIZE_BYTES)
          )
        val values = segment.get(ValueLayout.ADDRESS, JSON_VALUE_DATA_OFFSET)
        JsonValue.Array(
          List(count) { index ->
            readJson(
              values
                .reinterpret(JSON_VALUE_SIZE * count)
                .asSlice(index * JSON_VALUE_SIZE, JSON_VALUE_SIZE),
              depth + 1,
            )
          }
        )
      }
      JSON_OBJECT -> {
        val count =
          Math.toIntExact(
            segment.get(ValueLayout.JAVA_LONG, JSON_VALUE_DATA_OFFSET + Long.SIZE_BYTES)
          )
        val members = segment.get(ValueLayout.ADDRESS, JSON_VALUE_DATA_OFFSET)
        JsonValue.ObjectValue(
          List(count) { index ->
            val member =
              members
                .reinterpret(JSON_MEMBER_SIZE * count)
                .asSlice(index * JSON_MEMBER_SIZE, JSON_MEMBER_SIZE)
            val value = member.get(ValueLayout.ADDRESS, JSON_MEMBER_VALUE_OFFSET)
            JsonValue.Member(
              stringView(member.asSlice(JSON_MEMBER_KEY_OFFSET, STRING_VIEW_SIZE)),
              readJson(value.reinterpret(JSON_VALUE_SIZE), depth + 1),
            )
          }
        )
      }
      else -> JsonValue.Unknown(type, segment.get(ValueLayout.JAVA_INT, JSON_VALUE_SIZE_OFFSET))
    }
  }

  private fun latLng(value: LatLng, arena: Arena): MemorySegment {
    val segment = arena.allocate(latLngLayout)
    segment.set(ValueLayout.JAVA_DOUBLE, 0, value.latitude)
    segment.set(ValueLayout.JAVA_DOUBLE, Double.SIZE_BYTES.toLong(), value.longitude)
    return segment
  }

  private fun latLngArray(arena: Arena, values: List<LatLng>): MemorySegment {
    if (values.isEmpty()) {
      return MemorySegment.NULL
    }
    val array = arena.allocate(latLngLayout.byteSize() * values.size)
    values.forEachIndexed { index, value ->
      array
        .asSlice(latLngLayout.byteSize() * index, latLngLayout.byteSize())
        .copyFrom(latLng(value, arena))
    }
    return array
  }

  private fun latLngArray(segment: MemorySegment, count: Int): List<LatLng> =
    List(count) { index ->
      val coordinate =
        segment
          .reinterpret(latLngLayout.byteSize() * count)
          .asSlice(latLngLayout.byteSize() * index, latLngLayout.byteSize())
      LatLng(
        coordinate.get(ValueLayout.JAVA_DOUBLE, 0),
        coordinate.get(ValueLayout.JAVA_DOUBLE, Double.SIZE_BYTES.toLong()),
      )
    }

  private fun nativeBytes(arena: Arena, bytes: ByteArray): MemorySegment {
    if (bytes.isEmpty()) {
      return MemorySegment.NULL
    }
    val segment = arena.allocate(bytes.size.toLong())
    MemorySegment.copy(bytes, 0, segment, ValueLayout.JAVA_BYTE, 0, bytes.size)
    return segment
  }

  private fun copyBytes(address: MemorySegment, byteCount: Long): ByteArray {
    if (address == MemorySegment.NULL || byteCount == 0L) {
      return ByteArray(0)
    }
    return address.reinterpret(byteCount).toArray(ValueLayout.JAVA_BYTE)
  }

  private fun copyString(address: MemorySegment, byteCount: Long): String =
    String(copyBytes(address, byteCount), StandardCharsets.UTF_8)

  private fun stringView(segment: MemorySegment): String =
    copyString(
      segment.get(ValueLayout.ADDRESS, STRING_VIEW_DATA_OFFSET),
      segment.get(ValueLayout.JAVA_LONG, STRING_VIEW_SIZE_OFFSET),
    )

  internal fun resourceRequest(request: MemorySegment): ResourceRequest =
    ResourceRequest(
      copyCString(request.get(ValueLayout.ADDRESS, RESOURCE_REQUEST_URL_OFFSET)),
      ResourceKind.fromNative(request.get(ValueLayout.JAVA_INT, RESOURCE_REQUEST_KIND_OFFSET)),
      ResourceLoadingMethod.fromNative(
        request.get(ValueLayout.JAVA_INT, RESOURCE_REQUEST_LOADING_METHOD_OFFSET)
      ),
      ResourcePriority.fromNative(
        request.get(ValueLayout.JAVA_INT, RESOURCE_REQUEST_PRIORITY_OFFSET)
      ),
      ResourceUsage.fromNative(request.get(ValueLayout.JAVA_INT, RESOURCE_REQUEST_USAGE_OFFSET)),
      ResourceStoragePolicy.fromNative(
        request.get(ValueLayout.JAVA_INT, RESOURCE_REQUEST_STORAGE_POLICY_OFFSET)
      ),
      if (request.get(ValueLayout.JAVA_BOOLEAN, RESOURCE_REQUEST_HAS_RANGE_OFFSET))
        ResourceRequest.ByteRange(
          request.get(ValueLayout.JAVA_LONG, RESOURCE_REQUEST_RANGE_START_OFFSET),
          request.get(ValueLayout.JAVA_LONG, RESOURCE_REQUEST_RANGE_END_OFFSET),
        )
      else null,
      if (request.get(ValueLayout.JAVA_BOOLEAN, RESOURCE_REQUEST_HAS_PRIOR_MODIFIED_OFFSET))
        request.get(ValueLayout.JAVA_LONG, RESOURCE_REQUEST_PRIOR_MODIFIED_OFFSET)
      else null,
      if (request.get(ValueLayout.JAVA_BOOLEAN, RESOURCE_REQUEST_HAS_PRIOR_EXPIRES_OFFSET))
        request.get(ValueLayout.JAVA_LONG, RESOURCE_REQUEST_PRIOR_EXPIRES_OFFSET)
      else null,
      optionalCString(request.get(ValueLayout.ADDRESS, RESOURCE_REQUEST_PRIOR_ETAG_OFFSET)),
      copyBytes(
        request.get(ValueLayout.ADDRESS, RESOURCE_REQUEST_PRIOR_DATA_OFFSET),
        request.get(ValueLayout.JAVA_LONG, RESOURCE_REQUEST_PRIOR_DATA_SIZE_OFFSET),
      ),
    )

  private fun resourceResponse(response: ResourceResponse, arena: Arena): MemorySegment {
    val segment = arena.allocate(RESOURCE_RESPONSE_SIZE)
    val bytes = response.bytes
    require(response.errorReason.isKnown) {
      "Unknown resource error reason cannot be used as input: ${response.errorReason.nativeValue}"
    }
    segment.set(ValueLayout.JAVA_INT, RESOURCE_RESPONSE_SIZE_OFFSET, RESOURCE_RESPONSE_SIZE.toInt())
    segment.set(ValueLayout.JAVA_INT, RESOURCE_RESPONSE_STATUS_OFFSET, response.status.nativeValue)
    segment.set(
      ValueLayout.JAVA_INT,
      RESOURCE_RESPONSE_ERROR_REASON_OFFSET,
      response.errorReason.nativeValue,
    )
    if (bytes.isNotEmpty()) {
      segment.set(ValueLayout.ADDRESS, RESOURCE_RESPONSE_BYTES_OFFSET, nativeBytes(arena, bytes))
      segment.set(ValueLayout.JAVA_LONG, RESOURCE_RESPONSE_BYTE_COUNT_OFFSET, bytes.size.toLong())
    }
    segment.set(
      ValueLayout.ADDRESS,
      RESOURCE_RESPONSE_ERROR_MESSAGE_OFFSET,
      optionalCString(arena, response.errorMessage),
    )
    segment.set(
      ValueLayout.JAVA_BOOLEAN,
      RESOURCE_RESPONSE_MUST_REVALIDATE_OFFSET,
      response.mustRevalidate,
    )
    response.modifiedUnixMs?.let {
      segment.set(ValueLayout.JAVA_BOOLEAN, RESOURCE_RESPONSE_HAS_MODIFIED_OFFSET, true)
      segment.set(ValueLayout.JAVA_LONG, RESOURCE_RESPONSE_MODIFIED_OFFSET, it)
    }
    response.expiresUnixMs?.let {
      segment.set(ValueLayout.JAVA_BOOLEAN, RESOURCE_RESPONSE_HAS_EXPIRES_OFFSET, true)
      segment.set(ValueLayout.JAVA_LONG, RESOURCE_RESPONSE_EXPIRES_OFFSET, it)
    }
    segment.set(
      ValueLayout.ADDRESS,
      RESOURCE_RESPONSE_ETAG_OFFSET,
      optionalCString(arena, response.etag),
    )
    response.retryAfterUnixMs?.let {
      segment.set(ValueLayout.JAVA_BOOLEAN, RESOURCE_RESPONSE_HAS_RETRY_AFTER_OFFSET, true)
      segment.set(ValueLayout.JAVA_LONG, RESOURCE_RESPONSE_RETRY_AFTER_OFFSET, it)
    }
    return segment
  }

  private fun optionalCString(address: MemorySegment): String? =
    if (address == MemorySegment.NULL) null else copyCString(address)

  private fun copyCString(address: MemorySegment): String {
    if (address == MemorySegment.NULL) {
      return ""
    }
    var length = 0L
    while (address.reinterpret(length + 1).get(ValueLayout.JAVA_BYTE, length) != 0.toByte()) {
      length++
    }
    return copyString(address, length)
  }

  private fun mapOptions(options: MapOptions, arena: Arena): MemorySegment {
    val segment = arena.allocate(MAP_OPTIONS_SIZE)
    segment.set(ValueLayout.JAVA_INT, MAP_OPTIONS_SIZE_OFFSET, MAP_OPTIONS_SIZE.toInt())
    segment.set(ValueLayout.JAVA_INT, MAP_OPTIONS_WIDTH_OFFSET, DEFAULT_MAP_WIDTH)
    segment.set(ValueLayout.JAVA_INT, MAP_OPTIONS_HEIGHT_OFFSET, DEFAULT_MAP_HEIGHT)
    segment.set(ValueLayout.JAVA_DOUBLE, MAP_OPTIONS_SCALE_FACTOR_OFFSET, DEFAULT_SCALE_FACTOR)
    segment.set(ValueLayout.JAVA_INT, MAP_OPTIONS_MAP_MODE_OFFSET, MapMode.CONTINUOUS.nativeValue)
    options.width?.let {
      require(it >= 0) { "width must be non-negative" }
      segment.set(ValueLayout.JAVA_INT, MAP_OPTIONS_WIDTH_OFFSET, it)
    }
    options.height?.let {
      require(it >= 0) { "height must be non-negative" }
      segment.set(ValueLayout.JAVA_INT, MAP_OPTIONS_HEIGHT_OFFSET, it)
    }
    options.scaleFactor?.let {
      segment.set(ValueLayout.JAVA_DOUBLE, MAP_OPTIONS_SCALE_FACTOR_OFFSET, it)
    }
    options.mapMode?.let {
      require(it.isKnown) { "Unknown map mode cannot be used as input: ${it.nativeValue}" }
      segment.set(ValueLayout.JAVA_INT, MAP_OPTIONS_MAP_MODE_OFFSET, it.nativeValue)
    }
    return segment
  }

  private fun offlineRegionStatus(status: MemorySegment): OfflineRegionStatus =
    OfflineRegionStatus(
      OfflineRegionDownloadState.fromNative(
        status.get(ValueLayout.JAVA_INT, OFFLINE_REGION_STATUS_DOWNLOAD_STATE_OFFSET)
      ),
      status.get(ValueLayout.JAVA_LONG, OFFLINE_REGION_STATUS_COMPLETED_RESOURCE_COUNT_OFFSET),
      status.get(ValueLayout.JAVA_LONG, OFFLINE_REGION_STATUS_COMPLETED_RESOURCE_SIZE_OFFSET),
      status.get(ValueLayout.JAVA_LONG, OFFLINE_REGION_STATUS_COMPLETED_TILE_COUNT_OFFSET),
      status.get(ValueLayout.JAVA_LONG, OFFLINE_REGION_STATUS_REQUIRED_TILE_COUNT_OFFSET),
      status.get(ValueLayout.JAVA_LONG, OFFLINE_REGION_STATUS_COMPLETED_TILE_SIZE_OFFSET),
      status.get(ValueLayout.JAVA_LONG, OFFLINE_REGION_STATUS_REQUIRED_RESOURCE_COUNT_OFFSET),
      status.get(
        ValueLayout.JAVA_BOOLEAN,
        OFFLINE_REGION_STATUS_REQUIRED_RESOURCE_COUNT_IS_PRECISE_OFFSET,
      ),
      status.get(ValueLayout.JAVA_BOOLEAN, OFFLINE_REGION_STATUS_COMPLETE_OFFSET),
    )

  private fun offlineRegionDefinition(value: OfflineRegionDefinition, arena: Arena): MemorySegment {
    val segment = arena.allocate(OFFLINE_REGION_DEFINITION_SIZE)
    segment.set(
      ValueLayout.JAVA_INT,
      OFFLINE_REGION_DEFINITION_SIZE_OFFSET,
      OFFLINE_REGION_DEFINITION_SIZE.toInt(),
    )
    when (value) {
      is OfflineRegionDefinition.TilePyramid -> {
        segment.set(
          ValueLayout.JAVA_INT,
          OFFLINE_REGION_DEFINITION_TYPE_OFFSET,
          OFFLINE_REGION_DEFINITION_TYPE_TILE_PYRAMID,
        )
        offlineTilePyramidDefinition(
          value,
          segment.asSlice(OFFLINE_REGION_DEFINITION_DATA_OFFSET),
          arena,
        )
      }
      is OfflineRegionDefinition.GeometryRegion ->
        throw UnsupportedOperationException(
          "Geometry offline region definitions are not available until the JVM geometry bridge is migrated"
        )
      is OfflineRegionDefinition.Unknown ->
        throw IllegalArgumentException("unknown offline region definitions cannot be used as input")
    }
    return segment
  }

  private fun offlineTilePyramidDefinition(
    value: OfflineRegionDefinition.TilePyramid,
    segment: MemorySegment,
    arena: Arena,
  ) {
    segment.set(
      ValueLayout.JAVA_INT,
      OFFLINE_TILE_PYRAMID_DEFINITION_SIZE_OFFSET,
      OFFLINE_TILE_PYRAMID_DEFINITION_SIZE.toInt(),
    )
    segment.set(
      ValueLayout.ADDRESS,
      OFFLINE_TILE_PYRAMID_DEFINITION_STYLE_URL_OFFSET,
      cString(arena, value.styleUrl),
    )
    latLngBounds(value.bounds, segment.asSlice(OFFLINE_TILE_PYRAMID_DEFINITION_BOUNDS_OFFSET))
    segment.set(
      ValueLayout.JAVA_DOUBLE,
      OFFLINE_TILE_PYRAMID_DEFINITION_MIN_ZOOM_OFFSET,
      value.minZoom,
    )
    segment.set(
      ValueLayout.JAVA_DOUBLE,
      OFFLINE_TILE_PYRAMID_DEFINITION_MAX_ZOOM_OFFSET,
      value.maxZoom,
    )
    segment.set(
      ValueLayout.JAVA_FLOAT,
      OFFLINE_TILE_PYRAMID_DEFINITION_PIXEL_RATIO_OFFSET,
      value.pixelRatio,
    )
    segment.set(
      ValueLayout.JAVA_BOOLEAN,
      OFFLINE_TILE_PYRAMID_DEFINITION_INCLUDE_IDEOGRAPHS_OFFSET,
      value.includeIdeographs,
    )
  }

  private fun latLngBounds(bounds: LatLngBounds, segment: MemorySegment) {
    segment.set(
      ValueLayout.JAVA_DOUBLE,
      LAT_LNG_BOUNDS_SOUTHWEST_LATITUDE_OFFSET,
      bounds.southwest.latitude,
    )
    segment.set(
      ValueLayout.JAVA_DOUBLE,
      LAT_LNG_BOUNDS_SOUTHWEST_LONGITUDE_OFFSET,
      bounds.southwest.longitude,
    )
    segment.set(
      ValueLayout.JAVA_DOUBLE,
      LAT_LNG_BOUNDS_NORTHEAST_LATITUDE_OFFSET,
      bounds.northeast.latitude,
    )
    segment.set(
      ValueLayout.JAVA_DOUBLE,
      LAT_LNG_BOUNDS_NORTHEAST_LONGITUDE_OFFSET,
      bounds.northeast.longitude,
    )
  }

  private fun latLngBounds(segment: MemorySegment): LatLngBounds =
    LatLngBounds(
      LatLng(
        segment.get(ValueLayout.JAVA_DOUBLE, LAT_LNG_BOUNDS_SOUTHWEST_LATITUDE_OFFSET),
        segment.get(ValueLayout.JAVA_DOUBLE, LAT_LNG_BOUNDS_SOUTHWEST_LONGITUDE_OFFSET),
      ),
      LatLng(
        segment.get(ValueLayout.JAVA_DOUBLE, LAT_LNG_BOUNDS_NORTHEAST_LATITUDE_OFFSET),
        segment.get(ValueLayout.JAVA_DOUBLE, LAT_LNG_BOUNDS_NORTHEAST_LONGITUDE_OFFSET),
      ),
    )

  private fun offlineRegionSnapshot(snapshot: MemorySegment): OfflineRegionInfo =
    try {
      Arena.ofConfined().use { arena ->
        val info = arena.allocate(OFFLINE_REGION_INFO_SIZE)
        info.set(
          ValueLayout.JAVA_INT,
          OFFLINE_REGION_INFO_SIZE_OFFSET,
          OFFLINE_REGION_INFO_SIZE.toInt(),
        )
        Status.check(offlineRegionSnapshotGetFunction().invokeWithArguments(snapshot, info) as Int)
        offlineRegionInfo(info)
      }
    } finally {
      offlineRegionSnapshotDestroyFunction().invokeWithArguments(snapshot)
    }

  private fun offlineRegionList(list: MemorySegment): List<OfflineRegionInfo> =
    try {
      Arena.ofConfined().use { arena ->
        val outCount = arena.allocate(ValueLayout.JAVA_LONG)
        Status.check(offlineRegionListCountFunction().invokeWithArguments(list, outCount) as Int)
        val count = Math.toIntExact(outCount.get(ValueLayout.JAVA_LONG, 0))
        List(count) { index ->
          val info = arena.allocate(OFFLINE_REGION_INFO_SIZE)
          info.set(
            ValueLayout.JAVA_INT,
            OFFLINE_REGION_INFO_SIZE_OFFSET,
            OFFLINE_REGION_INFO_SIZE.toInt(),
          )
          Status.check(
            offlineRegionListGetFunction().invokeWithArguments(list, index.toLong(), info) as Int
          )
          offlineRegionInfo(info)
        }
      }
    } finally {
      offlineRegionListDestroyFunction().invokeWithArguments(list)
    }

  private fun styleIdList(list: MemorySegment): List<String> =
    try {
      Arena.ofConfined().use { arena ->
        val outCount = arena.allocate(ValueLayout.JAVA_LONG)
        Status.check(styleIdListCountFunction().invokeWithArguments(list, outCount) as Int)
        val count = Math.toIntExact(outCount.get(ValueLayout.JAVA_LONG, 0))
        List(count) { index ->
          val outId = arena.allocate(STRING_VIEW_SIZE)
          Status.check(
            styleIdListGetFunction().invokeWithArguments(list, index.toLong(), outId) as Int
          )
          stringView(outId)
        }
      }
    } finally {
      styleIdListDestroyFunction().invokeWithArguments(list)
    }

  private fun copyStyleSourceAttribution(
    map: MemorySegment,
    sourceId: MemorySegment,
    attributionSize: Long,
    arena: Arena,
  ): String? {
    if (attributionSize == 0L) {
      return ""
    }
    val outAttribution = arena.allocate(attributionSize)
    val outAttributionSize = arena.allocate(ValueLayout.JAVA_LONG)
    val outFound = arena.allocate(ValueLayout.JAVA_BOOLEAN)
    Status.check(
      copyStyleSourceAttributionFunction()
        .invokeWithArguments(
          map,
          sourceId,
          outAttribution,
          attributionSize,
          outAttributionSize,
          outFound,
        ) as Int
    )
    if (!outFound.get(ValueLayout.JAVA_BOOLEAN, 0)) {
      return null
    }
    return copyString(outAttribution, outAttributionSize.get(ValueLayout.JAVA_LONG, 0))
  }

  private fun takeOfflineRegionSnapshot(
    runtime: MemorySegment,
    operationId: Long,
    function: MethodHandle,
  ): OfflineRegionInfo =
    Arena.ofConfined().use { arena ->
      val outSnapshot = arena.allocate(ValueLayout.ADDRESS)
      outSnapshot.set(ValueLayout.ADDRESS, 0, MemorySegment.NULL)
      Status.check(function.invokeWithArguments(runtime, operationId, outSnapshot) as Int)
      offlineRegionSnapshot(outSnapshot.get(ValueLayout.ADDRESS, 0))
    }

  private fun takeOfflineRegionList(
    runtime: MemorySegment,
    operationId: Long,
    function: MethodHandle,
  ): List<OfflineRegionInfo> =
    Arena.ofConfined().use { arena ->
      val outList = arena.allocate(ValueLayout.ADDRESS)
      outList.set(ValueLayout.ADDRESS, 0, MemorySegment.NULL)
      Status.check(function.invokeWithArguments(runtime, operationId, outList) as Int)
      offlineRegionList(outList.get(ValueLayout.ADDRESS, 0))
    }

  private fun offlineRegionInfo(info: MemorySegment): OfflineRegionInfo =
    OfflineRegionInfo(
      info.get(ValueLayout.JAVA_LONG, OFFLINE_REGION_INFO_ID_OFFSET),
      offlineRegionDefinition(info.asSlice(OFFLINE_REGION_INFO_DEFINITION_OFFSET)),
      copyBytes(
        info.get(ValueLayout.ADDRESS, OFFLINE_REGION_INFO_METADATA_OFFSET),
        info.get(ValueLayout.JAVA_LONG, OFFLINE_REGION_INFO_METADATA_SIZE_OFFSET),
      ),
    )

  private fun offlineRegionDefinition(segment: MemorySegment): OfflineRegionDefinition =
    when (val type = segment.get(ValueLayout.JAVA_INT, OFFLINE_REGION_DEFINITION_TYPE_OFFSET)) {
      OFFLINE_REGION_DEFINITION_TYPE_TILE_PYRAMID ->
        offlineTilePyramidDefinition(segment.asSlice(OFFLINE_REGION_DEFINITION_DATA_OFFSET))
      else ->
        OfflineRegionDefinition.Unknown(
          type,
          segment.get(ValueLayout.JAVA_INT, OFFLINE_REGION_DEFINITION_SIZE_OFFSET),
        )
    }

  private fun offlineTilePyramidDefinition(
    segment: MemorySegment
  ): OfflineRegionDefinition.TilePyramid =
    OfflineRegionDefinition.TilePyramid(
      copyString(
        segment.get(ValueLayout.ADDRESS, OFFLINE_TILE_PYRAMID_DEFINITION_STYLE_URL_OFFSET),
        cStringLength(
          segment.get(ValueLayout.ADDRESS, OFFLINE_TILE_PYRAMID_DEFINITION_STYLE_URL_OFFSET)
        ),
      ),
      latLngBounds(segment.asSlice(OFFLINE_TILE_PYRAMID_DEFINITION_BOUNDS_OFFSET)),
      segment.get(ValueLayout.JAVA_DOUBLE, OFFLINE_TILE_PYRAMID_DEFINITION_MIN_ZOOM_OFFSET),
      segment.get(ValueLayout.JAVA_DOUBLE, OFFLINE_TILE_PYRAMID_DEFINITION_MAX_ZOOM_OFFSET),
      segment.get(ValueLayout.JAVA_FLOAT, OFFLINE_TILE_PYRAMID_DEFINITION_PIXEL_RATIO_OFFSET),
      segment.get(
        ValueLayout.JAVA_BOOLEAN,
        OFFLINE_TILE_PYRAMID_DEFINITION_INCLUDE_IDEOGRAPHS_OFFSET,
      ),
    )

  private fun cStringLength(address: MemorySegment): Long {
    if (address == MemorySegment.NULL) {
      return 0
    }
    var length = 0L
    while (address.reinterpret(length + 1).get(ValueLayout.JAVA_BYTE, length) != 0.toByte()) {
      length++
    }
    return length
  }

  private fun runtimeEventPayload(
    payloadType: Int,
    payload: MemorySegment,
    payloadSize: Long,
  ): RuntimeEventPayload =
    when (payloadType) {
      PAYLOAD_NONE -> RuntimeEventPayload.None
      PAYLOAD_RENDER_FRAME ->
        if (hasPayloadSize(payload, payloadSize, RUNTIME_EVENT_RENDER_FRAME_SIZE)) {
          renderFramePayload(payload)
        } else unknownPayload(payloadType, payload, payloadSize)
      PAYLOAD_RENDER_MAP ->
        if (hasPayloadSize(payload, payloadSize, RUNTIME_EVENT_RENDER_MAP_SIZE)) {
          renderMapPayload(payload)
        } else unknownPayload(payloadType, payload, payloadSize)
      PAYLOAD_STYLE_IMAGE_MISSING ->
        if (hasPayloadSize(payload, payloadSize, RUNTIME_EVENT_STYLE_IMAGE_MISSING_SIZE)) {
          styleImageMissingPayload(payload)
        } else unknownPayload(payloadType, payload, payloadSize)
      PAYLOAD_TILE_ACTION ->
        if (hasPayloadSize(payload, payloadSize, RUNTIME_EVENT_TILE_ACTION_SIZE)) {
          tileActionPayload(payload)
        } else unknownPayload(payloadType, payload, payloadSize)
      PAYLOAD_OFFLINE_REGION_STATUS ->
        if (hasPayloadSize(payload, payloadSize, RUNTIME_EVENT_OFFLINE_REGION_STATUS_SIZE)) {
          offlineRegionStatusPayload(payload)
        } else unknownPayload(payloadType, payload, payloadSize)
      PAYLOAD_OFFLINE_REGION_RESPONSE_ERROR ->
        if (
          hasPayloadSize(payload, payloadSize, RUNTIME_EVENT_OFFLINE_REGION_RESPONSE_ERROR_SIZE)
        ) {
          offlineRegionResponseErrorPayload(payload)
        } else unknownPayload(payloadType, payload, payloadSize)
      PAYLOAD_OFFLINE_REGION_TILE_COUNT_LIMIT ->
        if (
          hasPayloadSize(payload, payloadSize, RUNTIME_EVENT_OFFLINE_REGION_TILE_COUNT_LIMIT_SIZE)
        ) {
          offlineRegionTileCountLimitPayload(payload)
        } else unknownPayload(payloadType, payload, payloadSize)
      PAYLOAD_OFFLINE_OPERATION_COMPLETED ->
        if (hasPayloadSize(payload, payloadSize, RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED_SIZE)) {
          offlineOperationCompletedPayload(payload)
        } else unknownPayload(payloadType, payload, payloadSize)
      else -> unknownPayload(payloadType, payload, payloadSize)
    }

  private fun hasPayloadSize(
    payload: MemorySegment,
    payloadSize: Long,
    requiredSize: Long,
  ): Boolean = payload != MemorySegment.NULL && payloadSize >= requiredSize

  private fun unknownPayload(
    payloadType: Int,
    payload: MemorySegment,
    payloadSize: Long,
  ): RuntimeEventPayload.Unknown =
    RuntimeEventPayload.Unknown(payloadType, payloadSize, copyBytes(payload, payloadSize))

  private fun renderFramePayload(payload: MemorySegment): RuntimeEventPayload.RenderFrame =
    RuntimeEventPayload.RenderFrame(
      RenderMode.fromNative(
        payload.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_RENDER_FRAME_MODE_OFFSET)
      ),
      payload.get(ValueLayout.JAVA_BOOLEAN, RUNTIME_EVENT_RENDER_FRAME_NEEDS_REPAINT_OFFSET),
      payload.get(ValueLayout.JAVA_BOOLEAN, RUNTIME_EVENT_RENDER_FRAME_PLACEMENT_CHANGED_OFFSET),
      RenderingStats(
        payload.get(ValueLayout.JAVA_DOUBLE, RUNTIME_EVENT_RENDER_FRAME_ENCODING_TIME_OFFSET),
        payload.get(ValueLayout.JAVA_DOUBLE, RUNTIME_EVENT_RENDER_FRAME_RENDERING_TIME_OFFSET),
        payload.get(ValueLayout.JAVA_LONG, RUNTIME_EVENT_RENDER_FRAME_FRAME_COUNT_OFFSET),
        payload.get(ValueLayout.JAVA_LONG, RUNTIME_EVENT_RENDER_FRAME_DRAW_CALL_COUNT_OFFSET),
        payload.get(ValueLayout.JAVA_LONG, RUNTIME_EVENT_RENDER_FRAME_TOTAL_DRAW_CALL_COUNT_OFFSET),
      ),
    )

  private fun renderMapPayload(payload: MemorySegment): RuntimeEventPayload.RenderMap =
    RuntimeEventPayload.RenderMap(
      RenderMode.fromNative(payload.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_RENDER_MAP_MODE_OFFSET))
    )

  private fun styleImageMissingPayload(
    payload: MemorySegment
  ): RuntimeEventPayload.StyleImageMissing =
    RuntimeEventPayload.StyleImageMissing(
      copyString(
        payload.get(ValueLayout.ADDRESS, RUNTIME_EVENT_STYLE_IMAGE_MISSING_IMAGE_ID_OFFSET),
        payload.get(ValueLayout.JAVA_LONG, RUNTIME_EVENT_STYLE_IMAGE_MISSING_IMAGE_ID_SIZE_OFFSET),
      )
    )

  private fun tileActionPayload(payload: MemorySegment): RuntimeEventPayload.TileAction =
    RuntimeEventPayload.TileAction(
      TileOperation.fromNative(
        payload.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_TILE_ACTION_OPERATION_OFFSET)
      ),
      TileId(
        Integer.toUnsignedLong(
          payload.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_TILE_ACTION_OVERSCALED_Z_OFFSET)
        ),
        payload.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_TILE_ACTION_WRAP_OFFSET),
        Integer.toUnsignedLong(
          payload.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_TILE_ACTION_CANONICAL_Z_OFFSET)
        ),
        Integer.toUnsignedLong(
          payload.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_TILE_ACTION_CANONICAL_X_OFFSET)
        ),
        Integer.toUnsignedLong(
          payload.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_TILE_ACTION_CANONICAL_Y_OFFSET)
        ),
      ),
      copyString(
        payload.get(ValueLayout.ADDRESS, RUNTIME_EVENT_TILE_ACTION_SOURCE_ID_OFFSET),
        payload.get(ValueLayout.JAVA_LONG, RUNTIME_EVENT_TILE_ACTION_SOURCE_ID_SIZE_OFFSET),
      ),
    )

  private fun offlineRegionStatusPayload(
    payload: MemorySegment
  ): RuntimeEventPayload.OfflineRegionStatusChanged =
    RuntimeEventPayload.OfflineRegionStatusChanged(
      payload.get(ValueLayout.JAVA_LONG, RUNTIME_EVENT_OFFLINE_REGION_STATUS_REGION_ID_OFFSET),
      offlineRegionStatus(payload.asSlice(RUNTIME_EVENT_OFFLINE_REGION_STATUS_STATUS_OFFSET)),
    )

  private fun offlineRegionResponseErrorPayload(
    payload: MemorySegment
  ): RuntimeEventPayload.OfflineRegionResponseError =
    RuntimeEventPayload.OfflineRegionResponseError(
      payload.get(
        ValueLayout.JAVA_LONG,
        RUNTIME_EVENT_OFFLINE_REGION_RESPONSE_ERROR_REGION_ID_OFFSET,
      ),
      ResourceErrorReason.fromNative(
        payload.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_OFFLINE_REGION_RESPONSE_ERROR_REASON_OFFSET)
      ),
    )

  private fun offlineRegionTileCountLimitPayload(
    payload: MemorySegment
  ): RuntimeEventPayload.OfflineRegionTileCountLimit =
    RuntimeEventPayload.OfflineRegionTileCountLimit(
      payload.get(
        ValueLayout.JAVA_LONG,
        RUNTIME_EVENT_OFFLINE_REGION_TILE_COUNT_LIMIT_REGION_ID_OFFSET,
      ),
      payload.get(ValueLayout.JAVA_LONG, RUNTIME_EVENT_OFFLINE_REGION_TILE_COUNT_LIMIT_LIMIT_OFFSET),
    )

  private fun offlineOperationCompletedPayload(
    payload: MemorySegment
  ): RuntimeEventPayload.OfflineOperationCompleted =
    RuntimeEventPayload.OfflineOperationCompleted(
      payload.get(
        ValueLayout.JAVA_LONG,
        RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED_OPERATION_ID_OFFSET,
      ),
      OfflineOperationKind.fromNative(
        payload.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED_KIND_OFFSET)
      ),
      OfflineOperationResultKind.fromNative(
        payload.get(
          ValueLayout.JAVA_INT,
          RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED_RESULT_KIND_OFFSET,
        )
      ),
      payload.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED_STATUS_OFFSET),
      payload.get(ValueLayout.JAVA_BOOLEAN, RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED_FOUND_OFFSET),
    )

  private fun downcall(name: String, descriptor: FunctionDescriptor): MethodHandle {
    val symbol = SymbolLookup.loaderLookup().find(name).orElseThrow { NoSuchElementException(name) }
    return Linker.nativeLinker().downcallHandle(symbol, descriptor)
  }

  private fun nativeAccessFailure(cause: Throwable): IllegalStateException =
    IllegalStateException(
      "Java FFM native access is not enabled. Run the JVM with " +
        "--enable-native-access=ALL-UNNAMED for this classpath build.",
      cause,
    )

  private fun missingSymbols(cause: Throwable): UnsatisfiedLinkError {
    val missing =
      UnsatisfiedLinkError("Loaded native library does not expose the Maplibre C ABI symbols.")
    missing.addSuppressed(cause)
    return missing
  }

  private val latLngLayout =
    MemoryLayout.structLayout(
      ValueLayout.JAVA_DOUBLE.withName("latitude"),
      ValueLayout.JAVA_DOUBLE.withName("longitude"),
    )

  private val projectedMetersLayout =
    MemoryLayout.structLayout(
      ValueLayout.JAVA_DOUBLE.withName("northing"),
      ValueLayout.JAVA_DOUBLE.withName("easting"),
    )

  private val stringViewLayout =
    MemoryLayout.structLayout(
      ValueLayout.ADDRESS.withName("data"),
      ValueLayout.JAVA_LONG.withName("size"),
    )

  private const val STRING_VIEW_SIZE: Long = 16
  private const val STRING_VIEW_DATA_OFFSET: Long = 0
  private const val STRING_VIEW_SIZE_OFFSET: Long = 8

  private const val JSON_VALUE_SIZE: Long = 24
  private const val JSON_VALUE_SIZE_OFFSET: Long = 0
  private const val JSON_VALUE_TYPE_OFFSET: Long = 4
  private const val JSON_VALUE_DATA_OFFSET: Long = 8
  private const val JSON_MEMBER_SIZE: Long = 24
  private const val JSON_MEMBER_KEY_OFFSET: Long = 0
  private const val JSON_MEMBER_VALUE_OFFSET: Long = 16
  private const val JSON_NULL: Int = 0
  private const val JSON_BOOL: Int = 1
  private const val JSON_UINT: Int = 2
  private const val JSON_INT: Int = 3
  private const val JSON_DOUBLE: Int = 4
  private const val JSON_STRING: Int = 5
  private const val JSON_ARRAY: Int = 6
  private const val JSON_OBJECT: Int = 7

  private const val STYLE_SOURCE_INFO_SIZE: Long = 32
  private const val STYLE_SOURCE_INFO_SIZE_OFFSET: Long = 0
  private const val STYLE_SOURCE_INFO_TYPE_OFFSET: Long = 4
  private const val STYLE_SOURCE_INFO_IS_VOLATILE_OFFSET: Long = 16
  private const val STYLE_SOURCE_INFO_HAS_ATTRIBUTION_OFFSET: Long = 17
  private const val STYLE_SOURCE_INFO_ATTRIBUTION_SIZE_OFFSET: Long = 24

  private const val IMAGE_SOURCE_COORDINATE_COUNT: Int = 4

  private const val EDGE_INSETS_SIZE: Long = 32
  private const val EDGE_INSETS_TOP_OFFSET: Long = 0
  private const val EDGE_INSETS_LEFT_OFFSET: Long = 8
  private const val EDGE_INSETS_BOTTOM_OFFSET: Long = 16
  private const val EDGE_INSETS_RIGHT_OFFSET: Long = 24

  private const val VIEWPORT_OPTION_NORTH_ORIENTATION: Int = 1 shl 0
  private const val VIEWPORT_OPTION_CONSTRAIN_MODE: Int = 1 shl 1
  private const val VIEWPORT_OPTION_VIEWPORT_MODE: Int = 1 shl 2
  private const val VIEWPORT_OPTION_FRUSTUM_OFFSET: Int = 1 shl 3

  private const val VIEWPORT_OPTIONS_SIZE: Long = 56
  private const val VIEWPORT_OPTIONS_SIZE_OFFSET: Long = 0
  private const val VIEWPORT_OPTIONS_FIELDS_OFFSET: Long = 4
  private const val VIEWPORT_OPTIONS_NORTH_ORIENTATION_OFFSET: Long = 8
  private const val VIEWPORT_OPTIONS_CONSTRAIN_MODE_OFFSET: Long = 12
  private const val VIEWPORT_OPTIONS_VIEWPORT_MODE_OFFSET: Long = 16
  private const val VIEWPORT_OPTIONS_FRUSTUM_OFFSET_OFFSET: Long = 24

  private const val TILE_OPTION_PREFETCH_ZOOM_DELTA: Int = 1 shl 0
  private const val TILE_OPTION_LOD_MIN_RADIUS: Int = 1 shl 1
  private const val TILE_OPTION_LOD_SCALE: Int = 1 shl 2
  private const val TILE_OPTION_LOD_PITCH_THRESHOLD: Int = 1 shl 3
  private const val TILE_OPTION_LOD_ZOOM_SHIFT: Int = 1 shl 4
  private const val TILE_OPTION_LOD_MODE: Int = 1 shl 5

  private const val TILE_OPTIONS_SIZE: Long = 56
  private const val TILE_OPTIONS_SIZE_OFFSET: Long = 0
  private const val TILE_OPTIONS_FIELDS_OFFSET: Long = 4
  private const val TILE_OPTIONS_PREFETCH_ZOOM_DELTA_OFFSET: Long = 8
  private const val TILE_OPTIONS_LOD_MIN_RADIUS_OFFSET: Long = 16
  private const val TILE_OPTIONS_LOD_SCALE_OFFSET: Long = 24
  private const val TILE_OPTIONS_LOD_PITCH_THRESHOLD_OFFSET: Long = 32
  private const val TILE_OPTIONS_LOD_ZOOM_SHIFT_OFFSET: Long = 40
  private const val TILE_OPTIONS_LOD_MODE_OFFSET: Long = 48

  private const val PROJECTION_MODE_AXONOMETRIC: Int = 1 shl 0
  private const val PROJECTION_MODE_X_SKEW: Int = 1 shl 1
  private const val PROJECTION_MODE_Y_SKEW: Int = 1 shl 2

  private const val PROJECTION_MODE_SIZE: Long = 32
  private const val PROJECTION_MODE_SIZE_OFFSET: Long = 0
  private const val PROJECTION_MODE_FIELDS_OFFSET: Long = 4
  private const val PROJECTION_MODE_AXONOMETRIC_OFFSET: Long = 8
  private const val PROJECTION_MODE_X_SKEW_OFFSET: Long = 16
  private const val PROJECTION_MODE_Y_SKEW_OFFSET: Long = 24

  private const val TILE_SOURCE_OPTION_MIN_ZOOM: Int = 1 shl 0
  private const val TILE_SOURCE_OPTION_MAX_ZOOM: Int = 1 shl 1
  private const val TILE_SOURCE_OPTION_ATTRIBUTION: Int = 1 shl 2
  private const val TILE_SOURCE_OPTION_SCHEME: Int = 1 shl 3
  private const val TILE_SOURCE_OPTION_BOUNDS: Int = 1 shl 4
  private const val TILE_SOURCE_OPTION_TILE_SIZE: Int = 1 shl 5
  private const val TILE_SOURCE_OPTION_VECTOR_ENCODING: Int = 1 shl 6
  private const val TILE_SOURCE_OPTION_RASTER_ENCODING: Int = 1 shl 7

  private const val TILE_SOURCE_OPTIONS_SIZE: Long = 96
  private const val TILE_SOURCE_OPTIONS_SIZE_OFFSET: Long = 0
  private const val TILE_SOURCE_OPTIONS_FIELDS_OFFSET: Long = 4
  private const val TILE_SOURCE_OPTIONS_MIN_ZOOM_OFFSET: Long = 8
  private const val TILE_SOURCE_OPTIONS_MAX_ZOOM_OFFSET: Long = 16
  private const val TILE_SOURCE_OPTIONS_ATTRIBUTION_OFFSET: Long = 24
  private const val TILE_SOURCE_OPTIONS_SCHEME_OFFSET: Long = 40
  private const val TILE_SOURCE_OPTIONS_BOUNDS_OFFSET: Long = 48
  private const val TILE_SOURCE_OPTIONS_TILE_SIZE_OFFSET: Long = 80
  private const val TILE_SOURCE_OPTIONS_VECTOR_ENCODING_OFFSET: Long = 84
  private const val TILE_SOURCE_OPTIONS_RASTER_ENCODING_OFFSET: Long = 88

  private const val PREMULTIPLIED_RGBA8_IMAGE_SIZE: Long = 32
  private const val PREMULTIPLIED_RGBA8_IMAGE_SIZE_OFFSET: Long = 0
  private const val PREMULTIPLIED_RGBA8_IMAGE_WIDTH_OFFSET: Long = 4
  private const val PREMULTIPLIED_RGBA8_IMAGE_HEIGHT_OFFSET: Long = 8
  private const val PREMULTIPLIED_RGBA8_IMAGE_STRIDE_OFFSET: Long = 12
  private const val PREMULTIPLIED_RGBA8_IMAGE_PIXELS_OFFSET: Long = 16
  private const val PREMULTIPLIED_RGBA8_IMAGE_BYTE_LENGTH_OFFSET: Long = 24

  private const val STYLE_IMAGE_OPTION_PIXEL_RATIO: Int = 1 shl 0
  private const val STYLE_IMAGE_OPTION_SDF: Int = 1 shl 1

  private const val DEFAULT_PIXEL_RATIO: Float = 1.0f

  private const val STYLE_IMAGE_OPTIONS_SIZE: Long = 16
  private const val STYLE_IMAGE_OPTIONS_SIZE_OFFSET: Long = 0
  private const val STYLE_IMAGE_OPTIONS_FIELDS_OFFSET: Long = 4
  private const val STYLE_IMAGE_OPTIONS_PIXEL_RATIO_OFFSET: Long = 8
  private const val STYLE_IMAGE_OPTIONS_SDF_OFFSET: Long = 12

  private const val STYLE_IMAGE_INFO_SIZE: Long = 32
  private const val STYLE_IMAGE_INFO_SIZE_OFFSET: Long = 0
  private const val STYLE_IMAGE_INFO_WIDTH_OFFSET: Long = 4
  private const val STYLE_IMAGE_INFO_HEIGHT_OFFSET: Long = 8
  private const val STYLE_IMAGE_INFO_STRIDE_OFFSET: Long = 12
  private const val STYLE_IMAGE_INFO_BYTE_LENGTH_OFFSET: Long = 16
  private const val STYLE_IMAGE_INFO_PIXEL_RATIO_OFFSET: Long = 24
  private const val STYLE_IMAGE_INFO_SDF_OFFSET: Long = 28

  private const val RUNTIME_OPTION_MAXIMUM_CACHE_SIZE: Int = 1 shl 0
  private const val RUNTIME_OPTIONS_ASSET_PATH: Long = 8
  private const val RUNTIME_OPTIONS_CACHE_PATH: Long = 16
  private const val RUNTIME_OPTIONS_MAXIMUM_CACHE_SIZE: Long = 24

  private val runtimeOptionsLayout =
    MemoryLayout.structLayout(
      ValueLayout.JAVA_INT.withName("size"),
      ValueLayout.JAVA_INT.withName("flags"),
      ValueLayout.ADDRESS.withName("asset_path"),
      ValueLayout.ADDRESS.withName("cache_path"),
      ValueLayout.JAVA_LONG.withName("maximum_cache_size"),
    )

  private const val RUNTIME_EVENT_SIZE: Long = 64
  private const val RUNTIME_EVENT_SIZE_OFFSET: Long = 0
  private const val RUNTIME_EVENT_TYPE_OFFSET: Long = 4
  private const val RUNTIME_EVENT_SOURCE_TYPE_OFFSET: Long = 8
  private const val RUNTIME_EVENT_SOURCE_OFFSET: Long = 16
  private const val RUNTIME_EVENT_CODE_OFFSET: Long = 24
  private const val RUNTIME_EVENT_PAYLOAD_TYPE_OFFSET: Long = 28
  private const val RUNTIME_EVENT_PAYLOAD_OFFSET: Long = 32
  private const val RUNTIME_EVENT_PAYLOAD_SIZE_OFFSET: Long = 40
  private const val RUNTIME_EVENT_MESSAGE_OFFSET: Long = 48
  private const val RUNTIME_EVENT_MESSAGE_SIZE_OFFSET: Long = 56

  private const val RESOURCE_REQUEST_URL_OFFSET: Long = 8
  private const val RESOURCE_REQUEST_KIND_OFFSET: Long = 16
  private const val RESOURCE_REQUEST_LOADING_METHOD_OFFSET: Long = 20
  private const val RESOURCE_REQUEST_PRIORITY_OFFSET: Long = 24
  private const val RESOURCE_REQUEST_USAGE_OFFSET: Long = 28
  private const val RESOURCE_REQUEST_STORAGE_POLICY_OFFSET: Long = 32
  private const val RESOURCE_REQUEST_HAS_RANGE_OFFSET: Long = 36
  private const val RESOURCE_REQUEST_RANGE_START_OFFSET: Long = 40
  private const val RESOURCE_REQUEST_RANGE_END_OFFSET: Long = 48
  private const val RESOURCE_REQUEST_HAS_PRIOR_MODIFIED_OFFSET: Long = 56
  private const val RESOURCE_REQUEST_PRIOR_MODIFIED_OFFSET: Long = 64
  private const val RESOURCE_REQUEST_HAS_PRIOR_EXPIRES_OFFSET: Long = 72
  private const val RESOURCE_REQUEST_PRIOR_EXPIRES_OFFSET: Long = 80
  private const val RESOURCE_REQUEST_PRIOR_ETAG_OFFSET: Long = 88
  private const val RESOURCE_REQUEST_PRIOR_DATA_OFFSET: Long = 96
  private const val RESOURCE_REQUEST_PRIOR_DATA_SIZE_OFFSET: Long = 104

  private const val RESOURCE_RESPONSE_SIZE: Long = 96
  private const val RESOURCE_RESPONSE_SIZE_OFFSET: Long = 0
  private const val RESOURCE_RESPONSE_STATUS_OFFSET: Long = 4
  private const val RESOURCE_RESPONSE_ERROR_REASON_OFFSET: Long = 8
  private const val RESOURCE_RESPONSE_BYTES_OFFSET: Long = 16
  private const val RESOURCE_RESPONSE_BYTE_COUNT_OFFSET: Long = 24
  private const val RESOURCE_RESPONSE_ERROR_MESSAGE_OFFSET: Long = 32
  private const val RESOURCE_RESPONSE_MUST_REVALIDATE_OFFSET: Long = 40
  private const val RESOURCE_RESPONSE_HAS_MODIFIED_OFFSET: Long = 41
  private const val RESOURCE_RESPONSE_MODIFIED_OFFSET: Long = 48
  private const val RESOURCE_RESPONSE_HAS_EXPIRES_OFFSET: Long = 56
  private const val RESOURCE_RESPONSE_EXPIRES_OFFSET: Long = 64
  private const val RESOURCE_RESPONSE_ETAG_OFFSET: Long = 72
  private const val RESOURCE_RESPONSE_HAS_RETRY_AFTER_OFFSET: Long = 80
  private const val RESOURCE_RESPONSE_RETRY_AFTER_OFFSET: Long = 88

  private const val DEFAULT_MAP_WIDTH: Int = 256
  private const val DEFAULT_MAP_HEIGHT: Int = 256
  private const val DEFAULT_SCALE_FACTOR: Double = 1.0

  private const val MAP_OPTIONS_SIZE: Long = 32
  private const val MAP_OPTIONS_SIZE_OFFSET: Long = 0
  private const val MAP_OPTIONS_WIDTH_OFFSET: Long = 4
  private const val MAP_OPTIONS_HEIGHT_OFFSET: Long = 8
  private const val MAP_OPTIONS_SCALE_FACTOR_OFFSET: Long = 16
  private const val MAP_OPTIONS_MAP_MODE_OFFSET: Long = 24

  private const val PAYLOAD_NONE: Int = 0
  private const val PAYLOAD_RENDER_FRAME: Int = 1
  private const val PAYLOAD_RENDER_MAP: Int = 2
  private const val PAYLOAD_STYLE_IMAGE_MISSING: Int = 3
  private const val PAYLOAD_TILE_ACTION: Int = 4
  private const val PAYLOAD_OFFLINE_REGION_STATUS: Int = 5
  private const val PAYLOAD_OFFLINE_REGION_RESPONSE_ERROR: Int = 6
  private const val PAYLOAD_OFFLINE_REGION_TILE_COUNT_LIMIT: Int = 7
  private const val PAYLOAD_OFFLINE_OPERATION_COMPLETED: Int = 8

  private const val OFFLINE_REGION_STATUS_SIZE: Long = 64
  private const val OFFLINE_REGION_STATUS_SIZE_OFFSET: Long = 0
  private const val OFFLINE_REGION_STATUS_DOWNLOAD_STATE_OFFSET: Long = 4
  private const val OFFLINE_REGION_STATUS_COMPLETED_RESOURCE_COUNT_OFFSET: Long = 8
  private const val OFFLINE_REGION_STATUS_COMPLETED_RESOURCE_SIZE_OFFSET: Long = 16
  private const val OFFLINE_REGION_STATUS_COMPLETED_TILE_COUNT_OFFSET: Long = 24
  private const val OFFLINE_REGION_STATUS_REQUIRED_TILE_COUNT_OFFSET: Long = 32
  private const val OFFLINE_REGION_STATUS_COMPLETED_TILE_SIZE_OFFSET: Long = 40
  private const val OFFLINE_REGION_STATUS_REQUIRED_RESOURCE_COUNT_OFFSET: Long = 48
  private const val OFFLINE_REGION_STATUS_REQUIRED_RESOURCE_COUNT_IS_PRECISE_OFFSET: Long = 56
  private const val OFFLINE_REGION_STATUS_COMPLETE_OFFSET: Long = 57

  private const val RUNTIME_EVENT_RENDER_FRAME_SIZE: Long = 64
  private const val RUNTIME_EVENT_RENDER_FRAME_MODE_OFFSET: Long = 4
  private const val RUNTIME_EVENT_RENDER_FRAME_NEEDS_REPAINT_OFFSET: Long = 8
  private const val RUNTIME_EVENT_RENDER_FRAME_PLACEMENT_CHANGED_OFFSET: Long = 9
  private const val RUNTIME_EVENT_RENDER_FRAME_ENCODING_TIME_OFFSET: Long = 24
  private const val RUNTIME_EVENT_RENDER_FRAME_RENDERING_TIME_OFFSET: Long = 32
  private const val RUNTIME_EVENT_RENDER_FRAME_FRAME_COUNT_OFFSET: Long = 40
  private const val RUNTIME_EVENT_RENDER_FRAME_DRAW_CALL_COUNT_OFFSET: Long = 48
  private const val RUNTIME_EVENT_RENDER_FRAME_TOTAL_DRAW_CALL_COUNT_OFFSET: Long = 56

  private const val RUNTIME_EVENT_RENDER_MAP_SIZE: Long = 8
  private const val RUNTIME_EVENT_RENDER_MAP_MODE_OFFSET: Long = 4

  private const val RUNTIME_EVENT_STYLE_IMAGE_MISSING_SIZE: Long = 24
  private const val RUNTIME_EVENT_STYLE_IMAGE_MISSING_IMAGE_ID_OFFSET: Long = 8
  private const val RUNTIME_EVENT_STYLE_IMAGE_MISSING_IMAGE_ID_SIZE_OFFSET: Long = 16

  private const val RUNTIME_EVENT_TILE_ACTION_SIZE: Long = 48
  private const val RUNTIME_EVENT_TILE_ACTION_OPERATION_OFFSET: Long = 4
  private const val RUNTIME_EVENT_TILE_ACTION_OVERSCALED_Z_OFFSET: Long = 8
  private const val RUNTIME_EVENT_TILE_ACTION_WRAP_OFFSET: Long = 12
  private const val RUNTIME_EVENT_TILE_ACTION_CANONICAL_Z_OFFSET: Long = 16
  private const val RUNTIME_EVENT_TILE_ACTION_CANONICAL_X_OFFSET: Long = 20
  private const val RUNTIME_EVENT_TILE_ACTION_CANONICAL_Y_OFFSET: Long = 24
  private const val RUNTIME_EVENT_TILE_ACTION_SOURCE_ID_OFFSET: Long = 32
  private const val RUNTIME_EVENT_TILE_ACTION_SOURCE_ID_SIZE_OFFSET: Long = 40

  private const val RUNTIME_EVENT_OFFLINE_REGION_STATUS_SIZE: Long = 80
  private const val RUNTIME_EVENT_OFFLINE_REGION_STATUS_REGION_ID_OFFSET: Long = 8
  private const val RUNTIME_EVENT_OFFLINE_REGION_STATUS_STATUS_OFFSET: Long = 16

  private const val RUNTIME_EVENT_OFFLINE_REGION_RESPONSE_ERROR_SIZE: Long = 24
  private const val RUNTIME_EVENT_OFFLINE_REGION_RESPONSE_ERROR_REGION_ID_OFFSET: Long = 8
  private const val RUNTIME_EVENT_OFFLINE_REGION_RESPONSE_ERROR_REASON_OFFSET: Long = 16

  private const val RUNTIME_EVENT_OFFLINE_REGION_TILE_COUNT_LIMIT_SIZE: Long = 24
  private const val RUNTIME_EVENT_OFFLINE_REGION_TILE_COUNT_LIMIT_REGION_ID_OFFSET: Long = 8
  private const val RUNTIME_EVENT_OFFLINE_REGION_TILE_COUNT_LIMIT_LIMIT_OFFSET: Long = 16

  private const val RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED_SIZE: Long = 32
  private const val RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED_OPERATION_ID_OFFSET: Long = 8
  private const val RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED_KIND_OFFSET: Long = 16
  private const val RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED_RESULT_KIND_OFFSET: Long = 20
  private const val RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED_STATUS_OFFSET: Long = 24
  private const val RUNTIME_EVENT_OFFLINE_OPERATION_COMPLETED_FOUND_OFFSET: Long = 28

  private const val OFFLINE_REGION_DEFINITION_TYPE_TILE_PYRAMID: Int = 0
  private const val OFFLINE_REGION_DEFINITION_TYPE_GEOMETRY: Int = 1

  private const val LAT_LNG_BOUNDS_SOUTHWEST_LATITUDE_OFFSET: Long = 0
  private const val LAT_LNG_BOUNDS_SOUTHWEST_LONGITUDE_OFFSET: Long = 8
  private const val LAT_LNG_BOUNDS_NORTHEAST_LATITUDE_OFFSET: Long = 16
  private const val LAT_LNG_BOUNDS_NORTHEAST_LONGITUDE_OFFSET: Long = 24

  private const val OFFLINE_TILE_PYRAMID_DEFINITION_SIZE: Long = 72
  private const val OFFLINE_TILE_PYRAMID_DEFINITION_SIZE_OFFSET: Long = 0
  private const val OFFLINE_TILE_PYRAMID_DEFINITION_STYLE_URL_OFFSET: Long = 8
  private const val OFFLINE_TILE_PYRAMID_DEFINITION_BOUNDS_OFFSET: Long = 16
  private const val OFFLINE_TILE_PYRAMID_DEFINITION_MIN_ZOOM_OFFSET: Long = 48
  private const val OFFLINE_TILE_PYRAMID_DEFINITION_MAX_ZOOM_OFFSET: Long = 56
  private const val OFFLINE_TILE_PYRAMID_DEFINITION_PIXEL_RATIO_OFFSET: Long = 64
  private const val OFFLINE_TILE_PYRAMID_DEFINITION_INCLUDE_IDEOGRAPHS_OFFSET: Long = 68

  private const val OFFLINE_REGION_DEFINITION_SIZE: Long = 80
  private const val OFFLINE_REGION_DEFINITION_SIZE_OFFSET: Long = 0
  private const val OFFLINE_REGION_DEFINITION_TYPE_OFFSET: Long = 4
  private const val OFFLINE_REGION_DEFINITION_DATA_OFFSET: Long = 8

  private const val OFFLINE_REGION_INFO_SIZE: Long = 112
  private const val OFFLINE_REGION_INFO_SIZE_OFFSET: Long = 0
  private const val OFFLINE_REGION_INFO_ID_OFFSET: Long = 8
  private const val OFFLINE_REGION_INFO_DEFINITION_OFFSET: Long = 16
  private const val OFFLINE_REGION_INFO_METADATA_OFFSET: Long = 96
  private const val OFFLINE_REGION_INFO_METADATA_SIZE_OFFSET: Long = 104

  internal data class NativeRuntimeEvent(
    val type: Int,
    val sourceType: Int,
    val sourceAddress: Long,
    val code: Int,
    val payload: RuntimeEventPayload,
    val message: String,
  )
}

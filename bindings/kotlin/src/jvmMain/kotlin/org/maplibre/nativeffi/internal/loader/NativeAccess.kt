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
import org.maplibre.nativeffi.error.AbiVersionMismatchException
import org.maplibre.nativeffi.geo.LatLng
import org.maplibre.nativeffi.geo.LatLngBounds
import org.maplibre.nativeffi.geo.ProjectedMeters
import org.maplibre.nativeffi.geo.TileId
import org.maplibre.nativeffi.internal.status.Status
import org.maplibre.nativeffi.map.RenderingStats
import org.maplibre.nativeffi.map.TileOperation
import org.maplibre.nativeffi.offline.OfflineRegionDefinition
import org.maplibre.nativeffi.offline.OfflineRegionDownloadState
import org.maplibre.nativeffi.offline.OfflineRegionInfo
import org.maplibre.nativeffi.offline.OfflineRegionStatus
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

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
import org.maplibre.nativeffi.geo.ProjectedMeters
import org.maplibre.nativeffi.internal.status.Status
import org.maplibre.nativeffi.offline.OfflineRegionDownloadState
import org.maplibre.nativeffi.offline.OfflineRegionStatus
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

  internal fun startAmbientCacheOperation(runtime: MemorySegment, operation: Int): Long =
    Arena.ofConfined().use { arena ->
      val outOperationId = arena.allocate(ValueLayout.JAVA_LONG)
      Status.check(
        runtimeAmbientCacheOperationStartFunction()
          .invokeWithArguments(runtime, operation, outOperationId) as Int
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
      val message = event.get(ValueLayout.ADDRESS, RUNTIME_EVENT_MESSAGE_OFFSET)
      val messageSize = event.get(ValueLayout.JAVA_LONG, RUNTIME_EVENT_MESSAGE_SIZE_OFFSET)
      NativeRuntimeEvent(
        type = event.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_TYPE_OFFSET),
        sourceType = event.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_SOURCE_TYPE_OFFSET),
        sourceAddress = event.get(ValueLayout.ADDRESS, RUNTIME_EVENT_SOURCE_OFFSET).address(),
        code = event.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_CODE_OFFSET),
        payloadType = event.get(ValueLayout.JAVA_INT, RUNTIME_EVENT_PAYLOAD_TYPE_OFFSET),
        payloadSize = payloadSize,
        payloadBytes = copyBytes(payload, payloadSize),
        message = copyString(message, messageSize),
      )
    }

  private fun intFunction(name: String): MethodHandle =
    downcall(name, FunctionDescriptor.of(ValueLayout.JAVA_INT))

  private fun statusOutFunction(name: String): MethodHandle =
    downcall(name, FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS))

  private fun statusInFunction(name: String): MethodHandle =
    downcall(name, FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.JAVA_INT))

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

  private fun runtimeOfflineOperationDiscardFunction(): MethodHandle =
    downcall(
      "mln_runtime_offline_operation_discard",
      FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG),
    )

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

  internal data class NativeRuntimeEvent(
    val type: Int,
    val sourceType: Int,
    val sourceAddress: Long,
    val code: Int,
    val payloadType: Int,
    val payloadSize: Long,
    val payloadBytes: ByteArray,
    val message: String,
  )
}

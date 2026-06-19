package org.maplibre.nativeffi.runtime

import java.nio.charset.StandardCharsets
import org.bytedeco.javacpp.BytePointer
import org.bytedeco.javacpp.Pointer
import org.bytedeco.javacpp.PointerPointer
import org.maplibre.nativeffi.NativeAccess
import org.maplibre.nativeffi.error.InvalidStateException
import org.maplibre.nativeffi.geo.TileId
import org.maplibre.nativeffi.internal.javacpp.MaplibreNativeC
import org.maplibre.nativeffi.internal.lifecycle.HandleStateCore
import org.maplibre.nativeffi.internal.status.Status
import org.maplibre.nativeffi.map.RenderingStats
import org.maplibre.nativeffi.map.TileOperation
import org.maplibre.nativeffi.offline.OfflineRegionDefinition
import org.maplibre.nativeffi.offline.OfflineRegionDownloadState
import org.maplibre.nativeffi.offline.OfflineRegionInfo
import org.maplibre.nativeffi.offline.OfflineRegionStatus
import org.maplibre.nativeffi.render.RenderMode
import org.maplibre.nativeffi.resource.ResourceErrorReason
import org.maplibre.nativeffi.resource.ResourceProviderCallback
import org.maplibre.nativeffi.resource.ResourceTransformCallback

/** Owned runtime handle backed by the Android JNI bridge. */
public actual class RuntimeHandle private constructor(private val handleAddress: Long) :
  AutoCloseable {
  private val core = HandleStateCore("RuntimeHandle", handleAddress)

  public actual val isClosed: Boolean
    get() = core.isReleased()

  public actual fun runOnce() {
    NativeAccess.ensureLoaded()
    Status.check(MaplibreNativeC.mln_runtime_run_once(runtime(requireLiveAddress())))
  }

  public actual fun startAmbientCacheOperation(
    operation: AmbientCacheOperation
  ): OfflineOperationHandle<Unit> {
    NativeAccess.ensureLoaded()
    val outOperationId = longArrayOf(0L)
    Status.check(
      MaplibreNativeC.mln_runtime_run_ambient_cache_operation_start(
        runtime(requireLiveAddress()),
        operation.nativeValue,
        outOperationId,
      )
    )
    return offlineOperation(
      outOperationId[0],
      OfflineOperationKind.AMBIENT_CACHE,
      OfflineOperationResultKind.NONE,
    )
  }

  public actual fun startCreateOfflineRegion(
    definition: OfflineRegionDefinition,
    metadata: ByteArray,
  ): OfflineOperationHandle<OfflineRegionInfo> = unsupportedRuntimeHandle()

  public actual fun startOfflineRegion(id: Long): OfflineOperationHandle<OfflineRegionInfo?> =
    offlineOperation(
      startOfflineLongOperation(id, MaplibreNativeC::mln_runtime_offline_region_get_start),
      OfflineOperationKind.REGION_GET,
      OfflineOperationResultKind.OPTIONAL_REGION,
    )

  public actual fun startOfflineRegions(): OfflineOperationHandle<List<OfflineRegionInfo>> =
    offlineOperation(
      startOfflineOperation(MaplibreNativeC::mln_runtime_offline_regions_list_start),
      OfflineOperationKind.REGIONS_LIST,
      OfflineOperationResultKind.REGION_LIST,
    )

  public actual fun startMergeOfflineRegionsDatabase(
    path: String
  ): OfflineOperationHandle<List<OfflineRegionInfo>> {
    require('\u0000' !in path) { "C string inputs must not contain embedded NUL characters" }
    val outOperationId = longArrayOf(0L)
    Status.check(
      MaplibreNativeC.mln_runtime_offline_regions_merge_database_start(
        runtime(requireLiveAddress()),
        path,
        outOperationId,
      )
    )
    return offlineOperation(
      outOperationId[0],
      OfflineOperationKind.REGIONS_MERGE_DATABASE,
      OfflineOperationResultKind.REGION_LIST,
    )
  }

  public actual fun startUpdateOfflineRegionMetadata(
    id: Long,
    metadata: ByteArray,
  ): OfflineOperationHandle<OfflineRegionInfo> {
    val outOperationId = longArrayOf(0L)
    Status.check(
      MaplibreNativeC.mln_runtime_offline_region_update_metadata_start(
        runtime(requireLiveAddress()),
        id,
        metadata,
        metadata.size.toLong(),
        outOperationId,
      )
    )
    return offlineOperation(
      outOperationId[0],
      OfflineOperationKind.REGION_UPDATE_METADATA,
      OfflineOperationResultKind.REGION,
    )
  }

  public actual fun startOfflineRegionStatus(
    id: Long
  ): OfflineOperationHandle<OfflineRegionStatus> =
    offlineOperation(
      startOfflineLongOperation(id, MaplibreNativeC::mln_runtime_offline_region_get_status_start),
      OfflineOperationKind.REGION_GET_STATUS,
      OfflineOperationResultKind.REGION_STATUS,
    )

  public actual fun startSetOfflineRegionObserved(
    id: Long,
    observed: Boolean,
  ): OfflineOperationHandle<Unit> {
    val outOperationId = longArrayOf(0L)
    Status.check(
      MaplibreNativeC.mln_runtime_offline_region_set_observed_start(
        runtime(requireLiveAddress()),
        id,
        observed,
        outOperationId,
      )
    )
    return offlineOperation(
      outOperationId[0],
      OfflineOperationKind.REGION_SET_OBSERVED,
      OfflineOperationResultKind.NONE,
    )
  }

  public actual fun startSetOfflineRegionDownloadState(
    id: Long,
    downloadState: OfflineRegionDownloadState,
  ): OfflineOperationHandle<Unit> {
    val outOperationId = longArrayOf(0L)
    Status.check(
      MaplibreNativeC.mln_runtime_offline_region_set_download_state_start(
        runtime(requireLiveAddress()),
        id,
        downloadState.nativeValue,
        outOperationId,
      )
    )
    return offlineOperation(
      outOperationId[0],
      OfflineOperationKind.REGION_SET_DOWNLOAD_STATE,
      OfflineOperationResultKind.NONE,
    )
  }

  public actual fun startInvalidateOfflineRegion(id: Long): OfflineOperationHandle<Unit> =
    offlineOperation(
      startOfflineLongOperation(id, MaplibreNativeC::mln_runtime_offline_region_invalidate_start),
      OfflineOperationKind.REGION_INVALIDATE,
      OfflineOperationResultKind.NONE,
    )

  public actual fun startDeleteOfflineRegion(id: Long): OfflineOperationHandle<Unit> =
    offlineOperation(
      startOfflineLongOperation(id, MaplibreNativeC::mln_runtime_offline_region_delete_start),
      OfflineOperationKind.REGION_DELETE,
      OfflineOperationResultKind.NONE,
    )

  public actual fun takeCreateOfflineRegionResult(
    operation: OfflineOperationHandle<OfflineRegionInfo>
  ): OfflineRegionInfo = unsupportedRuntimeHandle()

  public actual fun takeOfflineRegionResult(
    operation: OfflineOperationHandle<OfflineRegionInfo?>
  ): OfflineRegionInfo? = unsupportedRuntimeHandle()

  public actual fun takeOfflineRegionsResult(
    operation: OfflineOperationHandle<List<OfflineRegionInfo>>
  ): List<OfflineRegionInfo> = unsupportedRuntimeHandle()

  public actual fun takeMergeOfflineRegionsDatabaseResult(
    operation: OfflineOperationHandle<List<OfflineRegionInfo>>
  ): List<OfflineRegionInfo> = unsupportedRuntimeHandle()

  public actual fun takeUpdateOfflineRegionMetadataResult(
    operation: OfflineOperationHandle<OfflineRegionInfo>
  ): OfflineRegionInfo = unsupportedRuntimeHandle()

  public actual fun takeOfflineRegionStatusResult(
    operation: OfflineOperationHandle<OfflineRegionStatus>
  ): OfflineRegionStatus {
    val operationId =
      operation.requireLive(
        this,
        OfflineOperationKind.REGION_GET_STATUS,
        OfflineOperationResultKind.REGION_STATUS,
      )
    MaplibreNativeC.mln_offline_region_status().use { status ->
      status.size(status.sizeof())
      Status.check(
        MaplibreNativeC.mln_runtime_offline_region_get_status_take_result(
          runtime(requireLiveAddress()),
          operationId,
          status,
        )
      )
      operation.markConsumed()
      return offlineRegionStatus(status)
    }
  }

  public actual fun setResourceProvider(callback: ResourceProviderCallback) {
    unsupportedRuntimeHandle()
  }

  public actual fun setResourceTransform(callback: ResourceTransformCallback) {
    unsupportedRuntimeHandle()
  }

  public actual fun clearResourceTransform() {
    unsupportedRuntimeHandle()
  }

  public actual fun pollEvent(): RuntimeEvent? {
    NativeAccess.ensureLoaded()
    MaplibreNativeC.mln_runtime_event().use { event ->
      event.size(event.sizeof())
      val hasEvent = booleanArrayOf(false)
      Status.check(
        MaplibreNativeC.mln_runtime_poll_event(runtime(requireLiveAddress()), event, hasEvent)
      )
      return if (hasEvent[0]) runtimeEvent(event) else null
    }
  }

  public actual override fun close() {
    core.closeOnce(destroy = { MaplibreNativeC.mln_runtime_destroy(runtime(handleAddress)) })
  }

  public actual companion object {
    public actual fun create(options: RuntimeOptions): RuntimeHandle {
      NativeAccess.ensureLoaded()
      RuntimeOptionsScope(options).use { nativeOptions ->
        val outRuntime = PointerPointer<MaplibreNativeC.mln_runtime>(1)
        outRuntime.put(0, null as Pointer?)
        Status.check(MaplibreNativeC.mln_runtime_create(nativeOptions.options, outRuntime))
        val runtime = outRuntime.get(MaplibreNativeC.mln_runtime::class.java, 0)
        val address = if (runtime == null || runtime.isNull) 0L else runtime.address()
        require(address != 0L) { "mln_runtime_create returned a null runtime" }
        return RuntimeHandle(address)
      }
    }
  }

  private fun <T> offlineOperation(
    operationId: Long,
    kind: OfflineOperationKind,
    resultKind: OfflineOperationResultKind,
  ): OfflineOperationHandle<T> = OfflineOperationHandle(this, operationId, kind, resultKind)

  private fun startOfflineOperation(start: (MaplibreNativeC.mln_runtime, LongArray) -> Int): Long {
    val outOperationId = longArrayOf(0L)
    Status.check(start(runtime(requireLiveAddress()), outOperationId))
    return outOperationId[0]
  }

  private fun startOfflineLongOperation(
    value: Long,
    start: (MaplibreNativeC.mln_runtime, Long, LongArray) -> Int,
  ): Long {
    val outOperationId = longArrayOf(0L)
    Status.check(start(runtime(requireLiveAddress()), value, outOperationId))
    return outOperationId[0]
  }

  internal fun discardOfflineOperation(operation: OfflineOperationHandle<*>) {
    if (operation.isClosed) return
    val operationId = operation.requireLive(this)
    val runtimeAddress =
      try {
        requireLiveAddress()
      } catch (error: InvalidStateException) {
        operation.markConsumed()
        throw error
      }
    Status.check(
      MaplibreNativeC.mln_runtime_offline_operation_discard(runtime(runtimeAddress), operationId)
    )
    operation.markConsumed()
  }

  internal fun retainChild(): HandleStateCore.ChildRetention = core.retainChild()

  private fun requireLiveAddress(): Long {
    core.requireLive()
    return handleAddress
  }

  private fun runtimeEvent(event: MaplibreNativeC.mln_runtime_event): RuntimeEvent {
    val sourceType = RuntimeEventSourceType.fromNative(event.source_type())
    return RuntimeEvent(
      RuntimeEventType.fromNative(event.type()),
      sourceType,
      if (sourceType == RuntimeEventSourceType.RUNTIME) this else null,
      null,
      event.code(),
      runtimeEventPayload(event),
      byteString(event.message(), event.message_size()),
    )
  }

  private fun runtimeEventPayload(event: MaplibreNativeC.mln_runtime_event): RuntimeEventPayload {
    val payloadType = event.payload_type()
    val payloadBytes = payloadBytes(event)
    return when (payloadType) {
      MaplibreNativeC.MLN_RUNTIME_EVENT_PAYLOAD_NONE -> RuntimeEventPayload.None
      MaplibreNativeC.MLN_RUNTIME_EVENT_PAYLOAD_RENDER_FRAME ->
        if (hasPayloadSize(event, PayloadSizes.RENDER_FRAME)) {
          renderFramePayload(event.payload())
        } else {
          RuntimeEventPayload.Unknown(payloadType, event.payload_size(), payloadBytes)
        }
      MaplibreNativeC.MLN_RUNTIME_EVENT_PAYLOAD_RENDER_MAP ->
        if (hasPayloadSize(event, PayloadSizes.RENDER_MAP)) {
          renderMapPayload(event.payload())
        } else {
          RuntimeEventPayload.Unknown(payloadType, event.payload_size(), payloadBytes)
        }
      MaplibreNativeC.MLN_RUNTIME_EVENT_PAYLOAD_STYLE_IMAGE_MISSING ->
        if (hasPayloadSize(event, PayloadSizes.STYLE_IMAGE_MISSING)) {
          styleImageMissingPayload(event.payload())
        } else {
          RuntimeEventPayload.Unknown(payloadType, event.payload_size(), payloadBytes)
        }
      MaplibreNativeC.MLN_RUNTIME_EVENT_PAYLOAD_TILE_ACTION ->
        if (hasPayloadSize(event, PayloadSizes.TILE_ACTION)) {
          tileActionPayload(event.payload())
        } else {
          RuntimeEventPayload.Unknown(payloadType, event.payload_size(), payloadBytes)
        }
      MaplibreNativeC.MLN_RUNTIME_EVENT_PAYLOAD_OFFLINE_REGION_STATUS ->
        if (hasPayloadSize(event, PayloadSizes.OFFLINE_REGION_STATUS)) {
          offlineRegionStatusPayload(event.payload())
        } else {
          RuntimeEventPayload.Unknown(payloadType, event.payload_size(), payloadBytes)
        }
      MaplibreNativeC.MLN_RUNTIME_EVENT_PAYLOAD_OFFLINE_REGION_RESPONSE_ERROR ->
        if (hasPayloadSize(event, PayloadSizes.OFFLINE_REGION_RESPONSE_ERROR)) {
          offlineRegionResponseErrorPayload(event.payload())
        } else {
          RuntimeEventPayload.Unknown(payloadType, event.payload_size(), payloadBytes)
        }
      MaplibreNativeC.MLN_RUNTIME_EVENT_PAYLOAD_OFFLINE_REGION_TILE_COUNT_LIMIT ->
        if (hasPayloadSize(event, PayloadSizes.OFFLINE_REGION_TILE_COUNT_LIMIT)) {
          offlineRegionTileCountLimitPayload(event.payload())
        } else {
          RuntimeEventPayload.Unknown(payloadType, event.payload_size(), payloadBytes)
        }
      MaplibreNativeC.MLN_RUNTIME_EVENT_PAYLOAD_OFFLINE_OPERATION_COMPLETED ->
        if (hasPayloadSize(event, PayloadSizes.OFFLINE_OPERATION_COMPLETED)) {
          offlineOperationCompletedPayload(event.payload())
        } else {
          RuntimeEventPayload.Unknown(payloadType, event.payload_size(), payloadBytes)
        }
      else -> RuntimeEventPayload.Unknown(payloadType, event.payload_size(), payloadBytes)
    }
  }
}

private fun hasPayloadSize(event: MaplibreNativeC.mln_runtime_event, requiredSize: Long): Boolean =
  event.payload() != null && !event.payload().isNull && event.payload_size() >= requiredSize

private fun payloadBytes(event: MaplibreNativeC.mln_runtime_event): ByteArray =
  byteArray(event.payload(), event.payload_size())

private fun byteString(pointer: BytePointer?, byteCount: Long): String =
  String(byteArray(pointer, byteCount), StandardCharsets.UTF_8)

private fun byteArray(pointer: Pointer?, byteCount: Long): ByteArray {
  if (pointer == null || pointer.isNull || byteCount == 0L) {
    return ByteArray(0)
  }
  val bytes = ByteArray(Math.toIntExact(byteCount))
  BytePointer(pointer).get(bytes, 0, bytes.size)
  return bytes
}

private fun renderFramePayload(payload: Pointer): RuntimeEventPayload.RenderFrame {
  val frame = MaplibreNativeC.mln_runtime_event_render_frame(payload)
  return RuntimeEventPayload.RenderFrame(
    RenderMode.fromNative(frame.mode()),
    frame.needs_repaint(),
    frame.placement_changed(),
    RenderingStats(
      frame.stats().encoding_time(),
      frame.stats().rendering_time(),
      frame.stats().frame_count(),
      frame.stats().draw_call_count(),
      frame.stats().total_draw_call_count(),
    ),
  )
}

private fun renderMapPayload(payload: Pointer): RuntimeEventPayload.RenderMap =
  RuntimeEventPayload.RenderMap(
    RenderMode.fromNative(MaplibreNativeC.mln_runtime_event_render_map(payload).mode())
  )

private fun styleImageMissingPayload(payload: Pointer): RuntimeEventPayload.StyleImageMissing {
  val missing = MaplibreNativeC.mln_runtime_event_style_image_missing(payload)
  return RuntimeEventPayload.StyleImageMissing(
    byteString(missing.image_id(), missing.image_id_size())
  )
}

private fun tileActionPayload(payload: Pointer): RuntimeEventPayload.TileAction {
  val action = MaplibreNativeC.mln_runtime_event_tile_action(payload)
  return RuntimeEventPayload.TileAction(
    TileOperation.fromNative(action.operation()),
    TileId(
      Integer.toUnsignedLong(action.tile_id().overscaled_z()),
      action.tile_id().wrap(),
      Integer.toUnsignedLong(action.tile_id().canonical_z()),
      Integer.toUnsignedLong(action.tile_id().canonical_x()),
      Integer.toUnsignedLong(action.tile_id().canonical_y()),
    ),
    byteString(action.source_id(), action.source_id_size()),
  )
}

private fun offlineRegionStatus(
  status: MaplibreNativeC.mln_offline_region_status
): OfflineRegionStatus =
  OfflineRegionStatus(
    OfflineRegionDownloadState.fromNative(status.download_state()),
    status.completed_resource_count(),
    status.completed_resource_size(),
    status.completed_tile_count(),
    status.required_tile_count(),
    status.completed_tile_size(),
    status.required_resource_count(),
    status.required_resource_count_is_precise(),
    status.complete(),
  )

private fun offlineRegionStatusPayload(
  payload: Pointer
): RuntimeEventPayload.OfflineRegionStatusChanged {
  val status = MaplibreNativeC.mln_runtime_event_offline_region_status(payload)
  return RuntimeEventPayload.OfflineRegionStatusChanged(
    status.region_id(),
    offlineRegionStatus(status.status()),
  )
}

private fun offlineRegionResponseErrorPayload(
  payload: Pointer
): RuntimeEventPayload.OfflineRegionResponseError {
  val error = MaplibreNativeC.mln_runtime_event_offline_region_response_error(payload)
  return RuntimeEventPayload.OfflineRegionResponseError(
    error.region_id(),
    ResourceErrorReason.fromNative(error.reason()),
  )
}

private fun offlineRegionTileCountLimitPayload(
  payload: Pointer
): RuntimeEventPayload.OfflineRegionTileCountLimit {
  val limit = MaplibreNativeC.mln_runtime_event_offline_region_tile_count_limit(payload)
  return RuntimeEventPayload.OfflineRegionTileCountLimit(limit.region_id(), limit.limit())
}

private fun offlineOperationCompletedPayload(
  payload: Pointer
): RuntimeEventPayload.OfflineOperationCompleted {
  val operation = MaplibreNativeC.mln_runtime_event_offline_operation_completed(payload)
  return RuntimeEventPayload.OfflineOperationCompleted(
    operation.operation_id(),
    OfflineOperationKind.fromNative(operation.operation_kind()),
    OfflineOperationResultKind.fromNative(operation.result_kind()),
    operation.result_status(),
    operation.found(),
  )
}

private object PayloadSizes {
  val RENDER_FRAME: Long =
    MaplibreNativeC.mln_runtime_event_render_frame().use { it.sizeof().toLong() }
  val RENDER_MAP: Long = MaplibreNativeC.mln_runtime_event_render_map().use { it.sizeof().toLong() }
  val STYLE_IMAGE_MISSING: Long =
    MaplibreNativeC.mln_runtime_event_style_image_missing().use { it.sizeof().toLong() }
  val TILE_ACTION: Long =
    MaplibreNativeC.mln_runtime_event_tile_action().use { it.sizeof().toLong() }
  val OFFLINE_REGION_STATUS: Long =
    MaplibreNativeC.mln_runtime_event_offline_region_status().use { it.sizeof().toLong() }
  val OFFLINE_REGION_RESPONSE_ERROR: Long =
    MaplibreNativeC.mln_runtime_event_offline_region_response_error().use { it.sizeof().toLong() }
  val OFFLINE_REGION_TILE_COUNT_LIMIT: Long =
    MaplibreNativeC.mln_runtime_event_offline_region_tile_count_limit().use { it.sizeof().toLong() }
  val OFFLINE_OPERATION_COMPLETED: Long =
    MaplibreNativeC.mln_runtime_event_offline_operation_completed().use { it.sizeof().toLong() }
}

private fun runtime(address: Long): MaplibreNativeC.mln_runtime =
  MaplibreNativeC.mln_runtime(AddressPointer(address))

private class AddressPointer(address: Long) : Pointer(null as Pointer?) {
  init {
    this.address = address
  }
}

private class RuntimeOptionsScope(options: RuntimeOptions) : AutoCloseable {
  val options: MaplibreNativeC.mln_runtime_options = MaplibreNativeC.mln_runtime_options_default()

  private val assetPath = optionalCString(options.assetPath)
  private val cachePath = optionalCString(options.cachePath)

  init {
    this.options.asset_path(assetPath)
    this.options.cache_path(cachePath)
    options.maximumCacheSize?.let { maximumCacheSize ->
      this.options.flags(
        this.options.flags() or MaplibreNativeC.MLN_RUNTIME_OPTION_MAXIMUM_CACHE_SIZE
      )
      this.options.maximum_cache_size(maximumCacheSize)
    }
  }

  override fun close() {
    assetPath?.close()
    cachePath?.close()
    options.close()
  }
}

private fun optionalCString(value: String?): BytePointer? = value?.let {
  require('\u0000' !in it) { "C string inputs must not contain embedded NUL characters" }
  BytePointer(it, StandardCharsets.UTF_8)
}

private fun unsupportedRuntimeHandle(): Nothing =
  throw UnsupportedOperationException(
    "RuntimeHandle is not available until the Android runtime bridge is implemented"
  )

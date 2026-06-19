package org.maplibre.nativeffi.runtime

import java.lang.foreign.MemorySegment
import java.nio.ByteBuffer
import java.nio.ByteOrder
import org.maplibre.nativeffi.error.InvalidStateException
import org.maplibre.nativeffi.internal.lifecycle.HandleStateCore
import org.maplibre.nativeffi.internal.loader.NativeAccess
import org.maplibre.nativeffi.internal.loader.NativeAccess.NativeRuntimeEvent
import org.maplibre.nativeffi.internal.status.Status
import org.maplibre.nativeffi.offline.OfflineRegionDefinition
import org.maplibre.nativeffi.offline.OfflineRegionDownloadState
import org.maplibre.nativeffi.offline.OfflineRegionInfo
import org.maplibre.nativeffi.offline.OfflineRegionStatus
import org.maplibre.nativeffi.resource.ResourceProviderCallback
import org.maplibre.nativeffi.resource.ResourceTransformCallback

/** Owned runtime handle backed by the JVM FFM bridge. */
public actual class RuntimeHandle private constructor(private val handle: MemorySegment) :
  AutoCloseable {
  private val core = HandleStateCore("RuntimeHandle", handle.address())

  public actual val isClosed: Boolean
    get() = core.isReleased()

  public actual fun runOnce() {
    NativeAccess.ensureLoaded()
    core.requireLive()
    NativeAccess.runRuntimeOnce(handle)
  }

  public actual fun startAmbientCacheOperation(
    operation: AmbientCacheOperation
  ): OfflineOperationHandle<Unit> {
    NativeAccess.ensureLoaded()
    val operationId =
      NativeAccess.startAmbientCacheOperation(requireLiveHandle(), operation.nativeValue)
    return offlineOperation(
      operationId,
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
      NativeAccess.startOfflineRegion(requireLiveHandle(), id),
      OfflineOperationKind.REGION_GET,
      OfflineOperationResultKind.OPTIONAL_REGION,
    )

  public actual fun startOfflineRegions(): OfflineOperationHandle<List<OfflineRegionInfo>> =
    offlineOperation(
      NativeAccess.startOfflineRegions(requireLiveHandle()),
      OfflineOperationKind.REGIONS_LIST,
      OfflineOperationResultKind.REGION_LIST,
    )

  public actual fun startMergeOfflineRegionsDatabase(
    path: String
  ): OfflineOperationHandle<List<OfflineRegionInfo>> =
    offlineOperation(
      NativeAccess.startMergeOfflineRegionsDatabase(requireLiveHandle(), path),
      OfflineOperationKind.REGIONS_MERGE_DATABASE,
      OfflineOperationResultKind.REGION_LIST,
    )

  public actual fun startUpdateOfflineRegionMetadata(
    id: Long,
    metadata: ByteArray,
  ): OfflineOperationHandle<OfflineRegionInfo> =
    offlineOperation(
      NativeAccess.startUpdateOfflineRegionMetadata(requireLiveHandle(), id, metadata),
      OfflineOperationKind.REGION_UPDATE_METADATA,
      OfflineOperationResultKind.REGION,
    )

  public actual fun startOfflineRegionStatus(
    id: Long
  ): OfflineOperationHandle<OfflineRegionStatus> =
    offlineOperation(
      NativeAccess.startOfflineRegionStatus(requireLiveHandle(), id),
      OfflineOperationKind.REGION_GET_STATUS,
      OfflineOperationResultKind.REGION_STATUS,
    )

  public actual fun startSetOfflineRegionObserved(
    id: Long,
    observed: Boolean,
  ): OfflineOperationHandle<Unit> =
    offlineOperation(
      NativeAccess.startSetOfflineRegionObserved(requireLiveHandle(), id, observed),
      OfflineOperationKind.REGION_SET_OBSERVED,
      OfflineOperationResultKind.NONE,
    )

  public actual fun startSetOfflineRegionDownloadState(
    id: Long,
    downloadState: OfflineRegionDownloadState,
  ): OfflineOperationHandle<Unit> =
    offlineOperation(
      NativeAccess.startSetOfflineRegionDownloadState(
        requireLiveHandle(),
        id,
        downloadState.nativeValue,
      ),
      OfflineOperationKind.REGION_SET_DOWNLOAD_STATE,
      OfflineOperationResultKind.NONE,
    )

  public actual fun startInvalidateOfflineRegion(id: Long): OfflineOperationHandle<Unit> =
    offlineOperation(
      NativeAccess.startInvalidateOfflineRegion(requireLiveHandle(), id),
      OfflineOperationKind.REGION_INVALIDATE,
      OfflineOperationResultKind.NONE,
    )

  public actual fun startDeleteOfflineRegion(id: Long): OfflineOperationHandle<Unit> =
    offlineOperation(
      NativeAccess.startDeleteOfflineRegion(requireLiveHandle(), id),
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
    val status = NativeAccess.takeOfflineRegionStatusResult(requireLiveHandle(), operationId)
    operation.markConsumed()
    return status
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
    return NativeAccess.pollRuntimeEvent(requireLiveHandle())?.toRuntimeEvent()
  }

  public actual override fun close() {
    core.closeOnce(destroy = { NativeAccess.destroyRuntime(handle) })
  }

  public actual companion object {
    public actual fun create(options: RuntimeOptions): RuntimeHandle {
      NativeAccess.ensureLoaded()
      return RuntimeHandle(NativeAccess.createRuntime(options))
    }
  }

  private fun <T> offlineOperation(
    operationId: Long,
    kind: OfflineOperationKind,
    resultKind: OfflineOperationResultKind,
  ): OfflineOperationHandle<T> = OfflineOperationHandle(this, operationId, kind, resultKind)

  internal fun discardOfflineOperation(operation: OfflineOperationHandle<*>) {
    if (operation.isClosed) return
    val operationId = operation.requireLive(this)
    val runtime =
      try {
        requireLiveHandle()
      } catch (error: InvalidStateException) {
        operation.markConsumed()
        throw error
      }
    Status.check(NativeAccess.discardOfflineOperation(runtime, operationId))
    operation.markConsumed()
  }

  internal fun retainChild(): HandleStateCore.ChildRetention = core.retainChild()

  private fun requireLiveHandle(): MemorySegment {
    core.requireLive()
    return handle
  }

  private fun NativeRuntimeEvent.toRuntimeEvent(): RuntimeEvent {
    val sourceType = RuntimeEventSourceType.fromNative(sourceType)
    return RuntimeEvent(
      RuntimeEventType.fromNative(type),
      sourceType,
      if (sourceType == RuntimeEventSourceType.RUNTIME) this@RuntimeHandle else null,
      null,
      code,
      runtimeEventPayload(),
      message,
    )
  }

  private fun NativeRuntimeEvent.runtimeEventPayload(): RuntimeEventPayload =
    when (payloadType) {
      PAYLOAD_NONE -> RuntimeEventPayload.None
      PAYLOAD_OFFLINE_OPERATION_COMPLETED ->
        if (payloadBytes.size >= OFFLINE_OPERATION_COMPLETED_SIZE) {
          offlineOperationCompletedPayload(payloadBytes)
        } else unknownPayload()
      else -> unknownPayload()
    }

  private fun NativeRuntimeEvent.unknownPayload(): RuntimeEventPayload =
    RuntimeEventPayload.Unknown(payloadType, payloadSize, payloadBytes)
}

private fun offlineOperationCompletedPayload(payload: ByteArray): RuntimeEventPayload {
  val buffer = ByteBuffer.wrap(payload).order(ByteOrder.nativeOrder())
  return RuntimeEventPayload.OfflineOperationCompleted(
    buffer.getLong(OFFLINE_OPERATION_COMPLETED_OPERATION_ID_OFFSET),
    OfflineOperationKind.fromNative(buffer.getInt(OFFLINE_OPERATION_COMPLETED_KIND_OFFSET)),
    OfflineOperationResultKind.fromNative(
      buffer.getInt(OFFLINE_OPERATION_COMPLETED_RESULT_KIND_OFFSET)
    ),
    buffer.getInt(OFFLINE_OPERATION_COMPLETED_STATUS_OFFSET),
    buffer.get(OFFLINE_OPERATION_COMPLETED_FOUND_OFFSET) != 0.toByte(),
  )
}

private const val PAYLOAD_NONE: Int = 0
private const val PAYLOAD_OFFLINE_OPERATION_COMPLETED: Int = 8
private const val OFFLINE_OPERATION_COMPLETED_SIZE: Int = 32
private const val OFFLINE_OPERATION_COMPLETED_OPERATION_ID_OFFSET: Int = 8
private const val OFFLINE_OPERATION_COMPLETED_KIND_OFFSET: Int = 16
private const val OFFLINE_OPERATION_COMPLETED_RESULT_KIND_OFFSET: Int = 20
private const val OFFLINE_OPERATION_COMPLETED_STATUS_OFFSET: Int = 24
private const val OFFLINE_OPERATION_COMPLETED_FOUND_OFFSET: Int = 28

private fun unsupportedRuntimeHandle(): Nothing =
  throw UnsupportedOperationException(
    "RuntimeHandle is not available until the JVM runtime bridge is implemented"
  )

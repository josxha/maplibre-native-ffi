package org.maplibre.nativeffi.runtime

import java.lang.foreign.MemorySegment
import org.maplibre.nativeffi.error.InvalidStateException
import org.maplibre.nativeffi.internal.lifecycle.HandleStateCore
import org.maplibre.nativeffi.internal.loader.NativeAccess
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
    unsupportedRuntimeHandle()

  public actual fun startOfflineRegions(): OfflineOperationHandle<List<OfflineRegionInfo>> =
    unsupportedRuntimeHandle()

  public actual fun startMergeOfflineRegionsDatabase(
    path: String
  ): OfflineOperationHandle<List<OfflineRegionInfo>> = unsupportedRuntimeHandle()

  public actual fun startUpdateOfflineRegionMetadata(
    id: Long,
    metadata: ByteArray,
  ): OfflineOperationHandle<OfflineRegionInfo> = unsupportedRuntimeHandle()

  public actual fun startOfflineRegionStatus(
    id: Long
  ): OfflineOperationHandle<OfflineRegionStatus> = unsupportedRuntimeHandle()

  public actual fun startSetOfflineRegionObserved(
    id: Long,
    observed: Boolean,
  ): OfflineOperationHandle<Unit> = unsupportedRuntimeHandle()

  public actual fun startSetOfflineRegionDownloadState(
    id: Long,
    downloadState: OfflineRegionDownloadState,
  ): OfflineOperationHandle<Unit> = unsupportedRuntimeHandle()

  public actual fun startInvalidateOfflineRegion(id: Long): OfflineOperationHandle<Unit> =
    unsupportedRuntimeHandle()

  public actual fun startDeleteOfflineRegion(id: Long): OfflineOperationHandle<Unit> =
    unsupportedRuntimeHandle()

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
  ): OfflineRegionStatus = unsupportedRuntimeHandle()

  public actual fun setResourceProvider(callback: ResourceProviderCallback) {
    unsupportedRuntimeHandle()
  }

  public actual fun setResourceTransform(callback: ResourceTransformCallback) {
    unsupportedRuntimeHandle()
  }

  public actual fun clearResourceTransform() {
    unsupportedRuntimeHandle()
  }

  public actual fun pollEvent(): RuntimeEvent? = unsupportedRuntimeHandle()

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
}

private fun unsupportedRuntimeHandle(): Nothing =
  throw UnsupportedOperationException(
    "RuntimeHandle is not available until the JVM runtime bridge is implemented"
  )

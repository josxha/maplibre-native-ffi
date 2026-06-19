package org.maplibre.nativeffi.runtime

import org.maplibre.nativeffi.offline.OfflineRegionDefinition
import org.maplibre.nativeffi.offline.OfflineRegionDownloadState
import org.maplibre.nativeffi.offline.OfflineRegionInfo
import org.maplibre.nativeffi.offline.OfflineRegionStatus
import org.maplibre.nativeffi.resource.ResourceProviderCallback
import org.maplibre.nativeffi.resource.ResourceTransformCallback

/** Android actual placeholder until the JNI runtime bridge is migrated. */
public actual class RuntimeHandle private constructor() : AutoCloseable {
  public actual val isClosed: Boolean
    get() = unsupportedRuntimeHandle()

  public actual fun runOnce() {
    unsupportedRuntimeHandle()
  }

  public actual fun startAmbientCacheOperation(
    operation: AmbientCacheOperation
  ): OfflineOperationHandle<Unit> = unsupportedRuntimeHandle()

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
    unsupportedRuntimeHandle()
  }

  public actual companion object {
    public actual fun create(options: RuntimeOptions): RuntimeHandle = unsupportedRuntimeHandle()
  }
}

private fun unsupportedRuntimeHandle(): Nothing =
  throw UnsupportedOperationException(
    "RuntimeHandle is not available until the Android runtime bridge is implemented"
  )

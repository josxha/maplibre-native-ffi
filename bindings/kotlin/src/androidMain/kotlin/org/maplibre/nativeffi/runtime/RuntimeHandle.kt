package org.maplibre.nativeffi.runtime

import java.nio.charset.StandardCharsets
import org.bytedeco.javacpp.BytePointer
import org.bytedeco.javacpp.Pointer
import org.bytedeco.javacpp.PointerPointer
import org.maplibre.nativeffi.NativeAccess
import org.maplibre.nativeffi.error.InvalidStateException
import org.maplibre.nativeffi.internal.javacpp.MaplibreNativeC
import org.maplibre.nativeffi.internal.lifecycle.HandleStateCore
import org.maplibre.nativeffi.internal.status.Status
import org.maplibre.nativeffi.offline.OfflineRegionDefinition
import org.maplibre.nativeffi.offline.OfflineRegionDownloadState
import org.maplibre.nativeffi.offline.OfflineRegionInfo
import org.maplibre.nativeffi.offline.OfflineRegionStatus
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

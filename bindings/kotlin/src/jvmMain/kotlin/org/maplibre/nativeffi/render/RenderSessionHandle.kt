package org.maplibre.nativeffi.render

import java.lang.foreign.MemorySegment
import org.maplibre.nativeffi.geo.Feature
import org.maplibre.nativeffi.internal.lifecycle.HandleStateCore
import org.maplibre.nativeffi.internal.loader.NativeAccess
import org.maplibre.nativeffi.json.JsonValue
import org.maplibre.nativeffi.map.MapHandle
import org.maplibre.nativeffi.query.FeatureExtensionResult
import org.maplibre.nativeffi.query.FeatureStateSelector
import org.maplibre.nativeffi.query.QueriedFeature
import org.maplibre.nativeffi.query.RenderedFeatureQueryOptions
import org.maplibre.nativeffi.query.RenderedQueryGeometry
import org.maplibre.nativeffi.query.SourceFeatureQueryOptions

/** Owned JVM FFM render session handle. Close it on the map owner thread. */
public actual class RenderSessionHandle
internal constructor(private val map: MapHandle, private val handle: MemorySegment) :
  AutoCloseable {
  private val mapRetention = map.retainChild()
  private val core = HandleStateCore("RenderSessionHandle", handle.address(), map)

  public actual val isClosed: Boolean
    get() = core.isReleased()

  public actual fun map(): MapHandle = map

  public actual fun resize(width: Int, height: Int, scaleFactor: Double) {
    NativeAccess.ensureLoaded()
    NativeAccess.resizeRenderSession(requireLiveHandle(), width, height, scaleFactor)
  }

  public actual fun renderUpdate() {
    NativeAccess.ensureLoaded()
    NativeAccess.renderUpdate(requireLiveHandle())
  }

  public actual fun detach() {
    NativeAccess.ensureLoaded()
    NativeAccess.detachRenderSession(requireLiveHandle())
  }

  public actual fun reduceMemoryUse() {
    NativeAccess.ensureLoaded()
    NativeAccess.reduceRenderSessionMemoryUse(requireLiveHandle())
  }

  public actual fun clearData() {
    NativeAccess.ensureLoaded()
    NativeAccess.clearRenderSessionData(requireLiveHandle())
  }

  public actual fun dumpDebugLogs() {
    NativeAccess.ensureLoaded()
    NativeAccess.dumpRenderSessionDebugLogs(requireLiveHandle())
  }

  public actual fun setFeatureState(selector: FeatureStateSelector, value: JsonValue) {
    NativeAccess.ensureLoaded()
    NativeAccess.setFeatureState(requireLiveHandle(), selector, value)
  }

  public actual fun getFeatureState(selector: FeatureStateSelector): JsonValue {
    NativeAccess.ensureLoaded()
    return NativeAccess.getFeatureState(requireLiveHandle(), selector)
  }

  public actual fun removeFeatureState(selector: FeatureStateSelector) {
    NativeAccess.ensureLoaded()
    NativeAccess.removeFeatureState(requireLiveHandle(), selector)
  }

  public actual fun queryRenderedFeatures(
    geometry: RenderedQueryGeometry,
    options: RenderedFeatureQueryOptions?,
  ): List<QueriedFeature> = unsupportedRenderSessionHandle()

  public actual fun querySourceFeatures(
    sourceId: String,
    options: SourceFeatureQueryOptions?,
  ): List<QueriedFeature> = unsupportedRenderSessionHandle()

  public actual fun queryFeatureExtension(
    sourceId: String,
    feature: Feature,
    extension: String,
    extensionField: String,
    arguments: JsonValue?,
  ): FeatureExtensionResult = unsupportedRenderSessionHandle()

  public actual fun textureImageInfo(): TextureImageInfo {
    NativeAccess.ensureLoaded()
    return NativeAccess.textureImageInfo(requireLiveHandle())
  }

  public actual fun readPremultipliedRgba8(buffer: NativeBuffer): TextureImageInfo {
    NativeAccess.ensureLoaded()
    return NativeAccess.readPremultipliedRgba8(requireLiveHandle(), buffer)
  }

  public actual fun acquireMetalOwnedTextureFrame(): MetalOwnedTextureFrameHandle =
    unsupportedRenderSessionHandle()

  public actual fun acquireVulkanOwnedTextureFrame(): VulkanOwnedTextureFrameHandle =
    unsupportedRenderSessionHandle()

  public actual fun acquireOpenGLOwnedTextureFrame(): OpenGLOwnedTextureFrameHandle =
    unsupportedRenderSessionHandle()

  public actual override fun close() {
    NativeAccess.ensureLoaded()
    core.closeOnce(
      destroy = { NativeAccess.destroyRenderSession(handle) },
      afterSuccess = { mapRetention.close() },
    )
  }

  internal fun nativeAddress(): Long = handle.address()

  private fun requireLiveHandle(): MemorySegment {
    core.requireLive()
    return handle
  }
}

private fun unsupportedRenderSessionHandle(): Nothing =
  throw UnsupportedOperationException(
    "This RenderSessionHandle operation is not available until the JVM render bridge is completed"
  )

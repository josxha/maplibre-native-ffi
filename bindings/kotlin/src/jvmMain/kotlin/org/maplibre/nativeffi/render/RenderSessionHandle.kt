package org.maplibre.nativeffi.render

import org.maplibre.nativeffi.geo.Feature
import org.maplibre.nativeffi.json.JsonValue
import org.maplibre.nativeffi.map.MapHandle
import org.maplibre.nativeffi.query.FeatureExtensionResult
import org.maplibre.nativeffi.query.FeatureStateSelector
import org.maplibre.nativeffi.query.QueriedFeature
import org.maplibre.nativeffi.query.RenderedFeatureQueryOptions
import org.maplibre.nativeffi.query.RenderedQueryGeometry
import org.maplibre.nativeffi.query.SourceFeatureQueryOptions

/** JVM actual placeholder until the FFM render session bridge is migrated. */
public actual class RenderSessionHandle private constructor() : AutoCloseable {
  public actual val isClosed: Boolean
    get() = unsupportedRenderSessionHandle()

  public actual fun map(): MapHandle = unsupportedRenderSessionHandle()

  public actual fun resize(width: Int, height: Int, scaleFactor: Double) {
    unsupportedRenderSessionHandle()
  }

  public actual fun renderUpdate() {
    unsupportedRenderSessionHandle()
  }

  public actual fun detach() {
    unsupportedRenderSessionHandle()
  }

  public actual fun reduceMemoryUse() {
    unsupportedRenderSessionHandle()
  }

  public actual fun clearData() {
    unsupportedRenderSessionHandle()
  }

  public actual fun dumpDebugLogs() {
    unsupportedRenderSessionHandle()
  }

  public actual fun setFeatureState(selector: FeatureStateSelector, value: JsonValue) {
    unsupportedRenderSessionHandle()
  }

  public actual fun getFeatureState(selector: FeatureStateSelector): JsonValue =
    unsupportedRenderSessionHandle()

  public actual fun removeFeatureState(selector: FeatureStateSelector) {
    unsupportedRenderSessionHandle()
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

  public actual fun textureImageInfo(): TextureImageInfo = unsupportedRenderSessionHandle()

  public actual fun readPremultipliedRgba8(buffer: NativeBuffer): TextureImageInfo =
    unsupportedRenderSessionHandle()

  public actual fun acquireMetalOwnedTextureFrame(): MetalOwnedTextureFrameHandle =
    unsupportedRenderSessionHandle()

  public actual fun acquireVulkanOwnedTextureFrame(): VulkanOwnedTextureFrameHandle =
    unsupportedRenderSessionHandle()

  public actual fun acquireOpenGLOwnedTextureFrame(): OpenGLOwnedTextureFrameHandle =
    unsupportedRenderSessionHandle()

  public actual override fun close() {
    unsupportedRenderSessionHandle()
  }
}

private fun unsupportedRenderSessionHandle(): Nothing =
  throw UnsupportedOperationException(
    "RenderSessionHandle is not available until the JVM render bridge is implemented"
  )

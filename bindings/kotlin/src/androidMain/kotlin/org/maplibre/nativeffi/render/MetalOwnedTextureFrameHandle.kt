package org.maplibre.nativeffi.render

/** Android actual placeholder until the JNI Metal owned texture frame bridge is migrated. */
public actual class MetalOwnedTextureFrameHandle private constructor() : AutoCloseable {
  public actual fun frame(): MetalOwnedTextureFrame = unsupportedMetalOwnedTextureFrameHandle()

  public actual val isClosed: Boolean
    get() = unsupportedMetalOwnedTextureFrameHandle()

  public actual override fun close() {
    unsupportedMetalOwnedTextureFrameHandle()
  }
}

private fun unsupportedMetalOwnedTextureFrameHandle(): Nothing =
  throw UnsupportedOperationException(
    "MetalOwnedTextureFrameHandle is not available until the Android render bridge is implemented"
  )

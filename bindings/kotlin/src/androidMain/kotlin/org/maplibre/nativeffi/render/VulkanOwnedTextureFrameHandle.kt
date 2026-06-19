package org.maplibre.nativeffi.render

/** Android actual placeholder until the JNI Vulkan owned texture frame bridge is migrated. */
public actual class VulkanOwnedTextureFrameHandle private constructor() : AutoCloseable {
  public actual fun frame(): VulkanOwnedTextureFrame = unsupportedVulkanOwnedTextureFrameHandle()

  public actual val isClosed: Boolean
    get() = unsupportedVulkanOwnedTextureFrameHandle()

  public actual override fun close() {
    unsupportedVulkanOwnedTextureFrameHandle()
  }
}

private fun unsupportedVulkanOwnedTextureFrameHandle(): Nothing =
  throw UnsupportedOperationException(
    "VulkanOwnedTextureFrameHandle is not available until the Android render bridge is implemented"
  )

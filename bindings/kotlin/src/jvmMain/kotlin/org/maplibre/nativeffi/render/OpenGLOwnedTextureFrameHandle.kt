package org.maplibre.nativeffi.render

/** JVM actual placeholder until the FFM OpenGL owned texture frame bridge is migrated. */
public actual class OpenGLOwnedTextureFrameHandle private constructor() : AutoCloseable {
  public actual fun frame(): OpenGLOwnedTextureFrame = unsupportedOpenGLOwnedTextureFrameHandle()

  public actual val isClosed: Boolean
    get() = unsupportedOpenGLOwnedTextureFrameHandle()

  public actual override fun close() {
    unsupportedOpenGLOwnedTextureFrameHandle()
  }
}

private fun unsupportedOpenGLOwnedTextureFrameHandle(): Nothing =
  throw UnsupportedOperationException(
    "OpenGLOwnedTextureFrameHandle is not available until the JVM render bridge is implemented"
  )

package org.maplibre.nativeffi.render

import org.maplibre.nativeffi.map.MapHandle

/** Android actual placeholder until the JNI render session bridge is migrated. */
public actual class RenderSessionHandle private constructor() : AutoCloseable {
  public actual val isClosed: Boolean
    get() = unsupportedRenderSessionHandle()

  public actual fun map(): MapHandle = unsupportedRenderSessionHandle()

  public actual override fun close() {
    unsupportedRenderSessionHandle()
  }
}

private fun unsupportedRenderSessionHandle(): Nothing =
  throw UnsupportedOperationException(
    "RenderSessionHandle is not available until the Android render bridge is implemented"
  )

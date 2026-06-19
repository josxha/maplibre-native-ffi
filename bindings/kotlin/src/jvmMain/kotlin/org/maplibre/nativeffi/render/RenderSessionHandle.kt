package org.maplibre.nativeffi.render

import org.maplibre.nativeffi.map.MapHandle

/** JVM actual placeholder until the FFM render session bridge is migrated. */
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
    "RenderSessionHandle is not available until the JVM render bridge is implemented"
  )

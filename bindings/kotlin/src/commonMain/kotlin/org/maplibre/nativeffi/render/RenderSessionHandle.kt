package org.maplibre.nativeffi.render

import org.maplibre.nativeffi.map.MapHandle

/** Owned render session handle. Platform actuals own the native render session carrier. */
public expect class RenderSessionHandle : AutoCloseable {
  public val isClosed: Boolean

  public fun map(): MapHandle

  override fun close()
}

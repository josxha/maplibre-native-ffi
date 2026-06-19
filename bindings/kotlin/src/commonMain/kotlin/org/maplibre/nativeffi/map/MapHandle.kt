package org.maplibre.nativeffi.map

import org.maplibre.nativeffi.runtime.RuntimeHandle

/** Owned map handle. Platform actuals own the native map carrier. */
public expect class MapHandle : AutoCloseable {
  public val isClosed: Boolean

  public fun runtime(): RuntimeHandle

  override fun close()
}

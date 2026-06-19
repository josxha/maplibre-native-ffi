package org.maplibre.nativeffi.runtime

/** Owned runtime handle. Platform actuals own the native runtime carrier. */
public expect class RuntimeHandle : AutoCloseable {
  public val isClosed: Boolean

  override fun close()
}

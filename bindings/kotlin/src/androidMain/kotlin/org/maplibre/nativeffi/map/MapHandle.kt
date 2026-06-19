package org.maplibre.nativeffi.map

import org.maplibre.nativeffi.runtime.RuntimeHandle

/** Android actual placeholder until the JNI map bridge is migrated. */
public actual class MapHandle private constructor() : AutoCloseable {
  public actual val isClosed: Boolean
    get() = unsupportedMapHandle()

  public actual fun runtime(): RuntimeHandle = unsupportedMapHandle()

  public actual override fun close() {
    unsupportedMapHandle()
  }
}

private fun unsupportedMapHandle(): Nothing =
  throw UnsupportedOperationException(
    "MapHandle is not available until the Android map bridge is implemented"
  )

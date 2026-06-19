package org.maplibre.nativeffi.runtime

/** JVM actual placeholder until the FFM runtime bridge is migrated. */
public actual class RuntimeHandle private constructor() : AutoCloseable {
  public actual val isClosed: Boolean
    get() = unsupportedRuntimeHandle()

  public actual override fun close() {
    unsupportedRuntimeHandle()
  }
}

private fun unsupportedRuntimeHandle(): Nothing =
  throw UnsupportedOperationException(
    "RuntimeHandle is not available until the JVM runtime bridge is implemented"
  )

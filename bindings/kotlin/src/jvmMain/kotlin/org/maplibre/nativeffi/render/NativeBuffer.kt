package org.maplibre.nativeffi.render

/** JVM actual placeholder until the FFM render readback bridge is migrated. */
public actual class NativeBuffer private constructor() : AutoCloseable {
  public actual fun byteLength(): Long = unsupportedNativeBuffer()

  public actual fun toByteArray(): ByteArray = unsupportedNativeBuffer()

  public actual override fun close() {
    unsupportedNativeBuffer()
  }

  public actual companion object {
    public actual fun allocate(byteLength: Long): NativeBuffer = unsupportedNativeBuffer()
  }
}

private fun unsupportedNativeBuffer(): Nothing =
  throw UnsupportedOperationException(
    "NativeBuffer is not available until the JVM render bridge is implemented"
  )

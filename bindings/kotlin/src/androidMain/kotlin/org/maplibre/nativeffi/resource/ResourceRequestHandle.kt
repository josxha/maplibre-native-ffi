package org.maplibre.nativeffi.resource

/** Android actual placeholder until the JNI resource provider bridge is migrated. */
public actual class ResourceRequestHandle private constructor() : AutoCloseable {
  public actual fun complete(response: ResourceResponse) {
    unsupportedResourceRequestHandle()
  }

  public actual fun isCancelled(): Boolean = unsupportedResourceRequestHandle()

  public actual override fun close() {
    unsupportedResourceRequestHandle()
  }
}

private fun unsupportedResourceRequestHandle(): Nothing =
  throw UnsupportedOperationException(
    "ResourceRequestHandle is not available until the Android resource provider bridge is implemented"
  )

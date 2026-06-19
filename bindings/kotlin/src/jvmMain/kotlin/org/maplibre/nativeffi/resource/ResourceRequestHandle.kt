package org.maplibre.nativeffi.resource

/** JVM actual placeholder until the FFM resource provider bridge is migrated. */
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
    "ResourceRequestHandle is not available until the JVM resource provider bridge is implemented"
  )

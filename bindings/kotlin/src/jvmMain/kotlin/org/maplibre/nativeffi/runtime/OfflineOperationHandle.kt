package org.maplibre.nativeffi.runtime

/** JVM actual placeholder until the FFM offline bridge is migrated. */
public actual class OfflineOperationHandle<T> private constructor() : AutoCloseable {
  public actual val id: Long
    get() = unsupportedOfflineOperationHandle()

  public actual val kind: OfflineOperationKind
    get() = unsupportedOfflineOperationHandle()

  public actual val resultKind: OfflineOperationResultKind
    get() = unsupportedOfflineOperationHandle()

  public actual val isClosed: Boolean
    get() = unsupportedOfflineOperationHandle()

  public actual override fun close() {
    unsupportedOfflineOperationHandle()
  }
}

private fun unsupportedOfflineOperationHandle(): Nothing =
  throw UnsupportedOperationException(
    "OfflineOperationHandle is not available until the JVM offline bridge is implemented"
  )

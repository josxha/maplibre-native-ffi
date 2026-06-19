package org.maplibre.nativeffi

/** Process-global entry points for the MapLibre Native FFI binding. */
public expect object Maplibre {
  /** C ABI contract version expected by this binding. */
  public val EXPECTED_C_ABI_VERSION: Long

  /** Loads or verifies access to the native library for the current platform. */
  public fun loadNativeLibrary()

  /** Returns the native C ABI contract version. */
  public fun cVersion(): Long
}

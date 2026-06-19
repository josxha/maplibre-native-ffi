package org.maplibre.nativeffi

import java.nio.file.Path
import org.maplibre.nativeffi.internal.loader.NativeAccess

/** Process-global entry points for the Kotlin/JVM FFM bridge. */
public object Maplibre {
  /** C ABI contract version expected by this Kotlin/JVM binding. */
  public const val EXPECTED_C_ABI_VERSION: Long = NativeAccess.EXPECTED_C_ABI_VERSION

  /** Loads the native library using the binding's standard lookup order. */
  public fun loadNativeLibrary() {
    NativeAccess.ensureLoaded()
  }

  /** Loads the native library from an exact file path. */
  public fun loadNativeLibrary(libraryPath: Path) {
    NativeAccess.load(libraryPath)
  }

  /** Returns the native C ABI contract version. */
  public fun cVersion(): Long {
    NativeAccess.ensureLoaded()
    return NativeAccess.cVersion()
  }
}

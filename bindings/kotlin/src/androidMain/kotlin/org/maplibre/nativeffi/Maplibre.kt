package org.maplibre.nativeffi

import org.maplibre.nativeffi.internal.javacpp.MaplibreNativeC

/** Process-global entry points for the Android JNI bridge. */
public actual object Maplibre {
  /** C ABI contract version expected by this Android binding. */
  public actual const val EXPECTED_C_ABI_VERSION: Long = 0L

  /** Loads the Android JNI bridge library. */
  public actual fun loadNativeLibrary() {
    NativeAccess.ensureLoaded()
  }

  /** Returns the native C ABI contract version. */
  public actual fun cVersion(): Long {
    NativeAccess.ensureLoaded()
    return NativeAccess.cVersion()
  }
}

private object NativeAccess {
  private val lock = Any()

  @Volatile private var loaded = false

  fun ensureLoaded() {
    if (loaded) {
      return
    }

    synchronized(lock) {
      if (loaded) {
        return
      }

      Maplibre.checkCompatibleCAbi(cVersion())
      loaded = true
    }
  }

  fun cVersion(): Long = Integer.toUnsignedLong(MaplibreNativeC.mln_c_version())
}

private fun Maplibre.checkCompatibleCAbi(actualVersion: Long) {
  if (actualVersion != EXPECTED_C_ABI_VERSION) {
    throw org.maplibre.nativeffi.error.AbiVersionMismatchException(
      actualVersion,
      EXPECTED_C_ABI_VERSION,
    )
  }
}

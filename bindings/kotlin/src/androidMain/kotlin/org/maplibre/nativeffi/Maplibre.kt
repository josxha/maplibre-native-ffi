package org.maplibre.nativeffi

import org.maplibre.nativeffi.internal.javacpp.MaplibreNativeC
import org.maplibre.nativeffi.internal.status.Status
import org.maplibre.nativeffi.render.OpenGLContextProvider
import org.maplibre.nativeffi.render.RenderBackend
import org.maplibre.nativeffi.runtime.NetworkStatus

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

  /** Returns the render backends compiled into the loaded native library. */
  public actual fun supportedRenderBackends(): Set<RenderBackend> {
    NativeAccess.ensureLoaded()
    return RenderBackend.fromMask(NativeAccess.supportedRenderBackendMask())
  }

  /** Returns the OpenGL context providers compiled into the loaded native library. */
  public actual fun supportedOpenGLContextProviders(): Set<OpenGLContextProvider> {
    NativeAccess.ensureLoaded()
    return OpenGLContextProvider.fromMask(NativeAccess.supportedOpenGLContextProviderMask())
  }

  /** Reads Maplibre Native's process-global network status. */
  public actual val networkStatus: NetworkStatus
    get() {
      NativeAccess.ensureLoaded()
      return NetworkStatus.fromNative(NativeAccess.networkStatus())
    }

  /** Sets Maplibre Native's process-global network status. */
  public actual fun setNetworkStatus(status: NetworkStatus) {
    require(status.isKnown) {
      "Unknown network status cannot be used as input: ${status.nativeValue}"
    }
    NativeAccess.setNetworkStatus(status.nativeValue)
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

  fun supportedRenderBackendMask(): Int = MaplibreNativeC.mln_supported_render_backend_mask()

  fun supportedOpenGLContextProviderMask(): Int =
    MaplibreNativeC.mln_opengl_supported_context_provider_mask()

  fun networkStatus(): Int {
    val outStatus = intArrayOf(0)
    Status.check(MaplibreNativeC.mln_network_status_get(outStatus))
    return outStatus[0]
  }

  fun setNetworkStatus(status: Int) {
    Status.check(MaplibreNativeC.mln_network_status_set(status))
  }
}

private fun Maplibre.checkCompatibleCAbi(actualVersion: Long) {
  if (actualVersion != EXPECTED_C_ABI_VERSION) {
    throw org.maplibre.nativeffi.error.AbiVersionMismatchException(
      actualVersion,
      EXPECTED_C_ABI_VERSION,
    )
  }
}

package org.maplibre.nativeffi.internal.loader

import java.lang.foreign.Arena
import java.lang.foreign.FunctionDescriptor
import java.lang.foreign.Linker
import java.lang.foreign.SymbolLookup
import java.lang.foreign.ValueLayout
import java.lang.invoke.MethodHandle
import java.nio.file.Path
import java.util.NoSuchElementException
import org.maplibre.nativeffi.error.AbiVersionMismatchException
import org.maplibre.nativeffi.internal.status.Status

/** Ensures the native library is loaded before JVM FFM downcalls run. */
internal object NativeAccess {
  const val EXPECTED_C_ABI_VERSION: Long = 0L

  private val lock = Any()

  @Volatile private var initialized = false

  fun ensureLoaded() {
    if (initialized) {
      return
    }

    synchronized(lock) {
      if (initialized) {
        return
      }

      NativeLibrary.load()
      checkNativeAccessAndAbi()
      initialized = true
    }
  }

  fun load(libraryPath: Path) {
    synchronized(lock) {
      NativeLibrary.load(libraryPath)
      checkNativeAccessAndAbi()
      initialized = true
    }
  }

  internal fun checkAbiVersion(version: Long) {
    if (version != EXPECTED_C_ABI_VERSION) {
      throw AbiVersionMismatchException(version, EXPECTED_C_ABI_VERSION)
    }
  }

  internal fun checkNativeAccessAndAbi(cVersion: () -> Long) {
    val version =
      try {
        cVersion()
      } catch (error: IllegalCallerException) {
        throw nativeAccessFailure(error)
      } catch (error: NoSuchElementException) {
        throw missingSymbols(error)
      } catch (error: UnsatisfiedLinkError) {
        throw missingSymbols(error)
      }

    checkAbiVersion(version)
  }

  private fun checkNativeAccessAndAbi() {
    checkNativeAccessAndAbi(::cVersion)
  }

  internal fun cVersion(): Long =
    intFunction("mln_c_version").invokeWithArguments().let { Integer.toUnsignedLong(it as Int) }

  internal fun supportedRenderBackendMask(): Int =
    intFunction("mln_supported_render_backend_mask").invokeWithArguments() as Int

  internal fun supportedOpenGLContextProviderMask(): Int =
    intFunction("mln_opengl_supported_context_provider_mask").invokeWithArguments() as Int

  internal fun networkStatus(): Int =
    Arena.ofConfined().use { arena ->
      val outStatus = arena.allocate(ValueLayout.JAVA_INT)
      Status.check(
        statusOutFunction("mln_network_status_get").invokeWithArguments(outStatus) as Int
      )
      outStatus.get(ValueLayout.JAVA_INT, 0)
    }

  internal fun setNetworkStatus(status: Int) {
    Status.check(statusInFunction("mln_network_status_set").invokeWithArguments(status) as Int)
  }

  private fun intFunction(name: String): MethodHandle =
    downcall(name, FunctionDescriptor.of(ValueLayout.JAVA_INT))

  private fun statusOutFunction(name: String): MethodHandle =
    downcall(name, FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS))

  private fun statusInFunction(name: String): MethodHandle =
    downcall(name, FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.JAVA_INT))

  private fun downcall(name: String, descriptor: FunctionDescriptor): MethodHandle {
    val symbol = SymbolLookup.loaderLookup().find(name).orElseThrow { NoSuchElementException(name) }
    return Linker.nativeLinker().downcallHandle(symbol, descriptor)
  }

  private fun nativeAccessFailure(cause: Throwable): IllegalStateException =
    IllegalStateException(
      "Java FFM native access is not enabled. Run the JVM with " +
        "--enable-native-access=ALL-UNNAMED for this classpath build.",
      cause,
    )

  private fun missingSymbols(cause: Throwable): UnsatisfiedLinkError {
    val missing =
      UnsatisfiedLinkError("Loaded native library does not expose the Maplibre C ABI symbols.")
    missing.addSuppressed(cause)
    return missing
  }
}

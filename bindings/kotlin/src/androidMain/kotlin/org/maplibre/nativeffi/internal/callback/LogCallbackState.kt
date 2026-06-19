package org.maplibre.nativeffi.internal.callback

import java.nio.charset.StandardCharsets
import kotlin.concurrent.atomics.AtomicInt
import kotlin.concurrent.atomics.AtomicReference
import kotlin.concurrent.atomics.ExperimentalAtomicApi
import org.bytedeco.javacpp.BytePointer
import org.bytedeco.javacpp.Pointer
import org.maplibre.nativeffi.NativeAccess
import org.maplibre.nativeffi.internal.javacpp.MaplibreNativeC
import org.maplibre.nativeffi.internal.status.Status
import org.maplibre.nativeffi.log.LogCallback
import org.maplibre.nativeffi.log.LogEvent
import org.maplibre.nativeffi.log.LogRecord
import org.maplibre.nativeffi.log.LogSeverity

/** Owns process-global Android JNI logging callback state. */
@OptIn(ExperimentalAtomicApi::class)
internal class LogCallbackState private constructor(private val callback: LogCallback) :
  AutoCloseable {
  private val closed = AtomicInt(0)
  private val nativeCallback =
    object : MaplibreNativeC.mln_log_callback() {
      override fun call(
        userData: Pointer?,
        severity: Int,
        event: Int,
        code: Long,
        message: BytePointer?,
      ): Int = invoke(severity, event, code, message)
    }

  fun invoke(rawSeverity: Int, rawEvent: Int, code: Long, message: BytePointer?): Int {
    if (closed.load() != 0) return 0
    return try {
      val record =
        LogRecord(
          LogSeverity.fromNative(rawSeverity),
          LogEvent.fromNative(rawEvent),
          code,
          cString(message),
        )
      if (callback.log(record)) 1 else 0
    } catch (_: Throwable) {
      0
    }
  }

  override fun close() {
    if (closed.compareAndSet(0, 1)) {
      nativeCallback.close()
    }
  }

  private fun cString(pointer: BytePointer?): String {
    if (pointer == null || pointer.isNull) {
      return ""
    }
    var length = 0L
    while (pointer.get(length) != 0.toByte()) {
      length++
    }
    val bytes = ByteArray(Math.toIntExact(length))
    pointer.get(bytes, 0, bytes.size)
    return String(bytes, StandardCharsets.UTF_8)
  }

  internal companion object {
    private val updateLock = AtomicInt(0)
    private val current = AtomicReference<LogCallbackState?>(null)

    fun set(callback: LogCallback) {
      NativeAccess.ensureLoaded()
      val replacement = LogCallbackState(callback)
      var previous: LogCallbackState? = null
      try {
        withUpdateLock {
          Status.check(MaplibreNativeC.mln_log_set_callback(replacement.nativeCallback, null))
          previous = current.exchange(replacement)
        }
      } catch (error: Throwable) {
        replacement.close()
        throw error
      }
      previous?.close()
    }

    fun clear() {
      NativeAccess.ensureLoaded()
      var previous: LogCallbackState? = null
      withUpdateLock {
        Status.check(MaplibreNativeC.mln_log_clear_callback())
        previous = current.exchange(null)
      }
      previous?.close()
    }

    fun currentForTesting(): LogCallbackState? = current.load()

    private inline fun <R> withUpdateLock(block: () -> R): R {
      while (!updateLock.compareAndSet(0, 1)) {
        // Spin briefly; log callback registration is process-global and infrequent.
      }
      try {
        return block()
      } finally {
        updateLock.store(0)
      }
    }
  }
}

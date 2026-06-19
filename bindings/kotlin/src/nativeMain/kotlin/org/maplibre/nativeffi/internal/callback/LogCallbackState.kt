package org.maplibre.nativeffi.internal.callback

import kotlin.concurrent.atomics.AtomicInt
import kotlin.concurrent.atomics.ExperimentalAtomicApi
import kotlinx.cinterop.ByteVar
import kotlinx.cinterop.COpaquePointer
import kotlinx.cinterop.CPointer
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.staticCFunction
import org.maplibre.nativeffi.internal.c.mln_log_clear_callback
import org.maplibre.nativeffi.internal.c.mln_log_set_callback
import org.maplibre.nativeffi.internal.memory.MemoryUtil
import org.maplibre.nativeffi.log.LogCallback
import org.maplibre.nativeffi.log.LogEvent
import org.maplibre.nativeffi.log.LogRecord
import org.maplibre.nativeffi.log.LogSeverity

/** Owns process-global logging callback state. */
@OptIn(ExperimentalForeignApi::class, ExperimentalAtomicApi::class)
internal class LogCallbackState private constructor(private val callback: LogCallback) :
  AutoCloseable {
  private val closed = AtomicInt(0)

  fun invoke(severity: UInt, event: UInt, code: Long, message: CPointer<ByteVar>?): UInt {
    if (closed.load() != 0) return 0U
    return try {
      val record =
        LogRecord(
          LogSeverity.fromNative(severity),
          LogEvent.fromNative(event),
          code,
          MemoryUtil.copyCString(message),
        )
      if (callback.log(record)) 1U else 0U
    } catch (_: Throwable) {
      0U
    }
  }

  override fun close() {
    // Native logging can dispatch from worker threads. The C API stops future callbacks after
    // replacement or clear, but it does not guarantee that an already-entered upcall has finished
    // running. Keep callback state independent from user_data and gate dispatch instead.
    closed.store(1)
  }

  internal fun isClosedForTesting(): Boolean = closed.load() != 0

  internal companion object {
    private val registry = LogCallbackRegistry<LogCallbackState>()

    fun set(callback: LogCallback) {
      registry.set(LogCallbackState(callback)) {
        mln_log_set_callback(staticCFunction(::logCallback), null)
      }
    }

    fun setForTesting(
      callback: LogCallback,
      install: () -> Int,
      captureReplacement: (LogCallbackState) -> Unit,
    ) {
      registry.set(LogCallbackState(callback).also(captureReplacement), install)
    }

    fun clear() {
      registry.clear(::mln_log_clear_callback)
    }

    fun currentForTesting(): LogCallbackState? = registry.current()

    fun invokeCurrent(severity: UInt, event: UInt, code: Long, message: CPointer<ByteVar>?): UInt =
      registry.current()?.invoke(severity, event, code, message) ?: 0U
  }
}

@Suppress("UNUSED_PARAMETER")
@OptIn(ExperimentalForeignApi::class)
private fun logCallback(
  userData: COpaquePointer?,
  severity: UInt,
  event: UInt,
  code: Long,
  message: CPointer<ByteVar>?,
): UInt = LogCallbackState.invokeCurrent(severity, event, code, message)

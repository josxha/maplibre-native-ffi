package org.maplibre.nativeffi.internal.callback

import kotlin.concurrent.atomics.AtomicInt
import kotlin.concurrent.atomics.ExperimentalAtomicApi

/** Platform-neutral callback entry gate with close-after-last-callback cleanup. */
@OptIn(ExperimentalAtomicApi::class)
internal class CallbackGate(private val name: String, private val closeNative: () -> Unit = {}) :
  AutoCloseable {
  private val state = AtomicInt(0)
  private val nativeClosed = AtomicInt(0)

  fun enter(): Lease? {
    while (true) {
      val current = state.load()
      if (current and (CLOSING_FLAG or CLOSED_FLAG) != 0) return null
      val activeCallbacks = current and ACTIVE_MASK
      check(activeCallbacks < ACTIVE_MASK) { "too many active $name" }
      if (state.compareAndSet(current, current + 1)) return Lease(this)
    }
  }

  override fun close() {
    while (true) {
      val current = state.load()
      if (current and CLOSED_FLAG != 0) return
      val activeCallbacks = current and ACTIVE_MASK
      val next = if (activeCallbacks == 0) CLOSED_FLAG else current or CLOSING_FLAG
      if (state.compareAndSet(current, next)) {
        if (next and CLOSED_FLAG != 0) closeNativeOnce()
        return
      }
    }
  }

  fun isClosedForTesting(): Boolean = state.load() and CLOSED_FLAG != 0

  private fun exit() {
    while (true) {
      val current = state.load()
      val activeCallbacks = current and ACTIVE_MASK
      check(activeCallbacks > 0) { "$name count underflow" }
      val next =
        if (activeCallbacks == 1 && current and CLOSING_FLAG != 0) {
          CLOSED_FLAG
        } else {
          current - 1
        }
      if (state.compareAndSet(current, next)) {
        if (next and CLOSED_FLAG != 0) closeNativeOnce()
        return
      }
    }
  }

  private fun closeNativeOnce() {
    if (nativeClosed.compareAndSet(0, 1)) closeNative()
  }

  internal class Lease(private val gate: CallbackGate) : AutoCloseable {
    private val closed = AtomicInt(0)

    override fun close() {
      if (closed.compareAndSet(0, 1)) gate.exit()
    }
  }

  private companion object {
    private const val CLOSED_FLAG = Int.MIN_VALUE
    private const val CLOSING_FLAG = 1 shl 30
    private const val ACTIVE_MASK = CLOSING_FLAG - 1
  }
}

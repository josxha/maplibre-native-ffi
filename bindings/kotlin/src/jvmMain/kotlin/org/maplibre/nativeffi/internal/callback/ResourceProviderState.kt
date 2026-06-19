package org.maplibre.nativeffi.internal.callback

import java.lang.foreign.Arena
import java.lang.foreign.FunctionDescriptor
import java.lang.foreign.Linker
import java.lang.foreign.MemorySegment
import java.lang.foreign.ValueLayout
import java.lang.invoke.MethodHandles
import java.lang.invoke.MethodType
import org.maplibre.nativeffi.internal.loader.NativeAccess
import org.maplibre.nativeffi.resource.ResourceProviderCallback
import org.maplibre.nativeffi.resource.ResourceRequestHandle

/** Owns runtime-scoped JVM FFM resource provider callback state. */
internal class ResourceProviderState(private val callback: ResourceProviderCallback) :
  AutoCloseable {
  private val arena = Arena.ofShared()
  private val gate = CallbackGate("resource provider callbacks") { arena.close() }
  private val stub: MemorySegment
  private val descriptor: MemorySegment

  init {
    val method =
      MethodHandles.lookup()
        .findVirtual(
          ResourceProviderState::class.java,
          "invoke",
          MethodType.methodType(
            Int::class.javaPrimitiveType,
            MemorySegment::class.java,
            MemorySegment::class.java,
            MemorySegment::class.java,
          ),
        )
        .bindTo(this)
    stub = Linker.nativeLinker().upcallStub(method, callbackDescriptor, arena)
    descriptor = arena.allocate(RESOURCE_PROVIDER_SIZE)
    descriptor.set(
      ValueLayout.JAVA_INT,
      RESOURCE_PROVIDER_SIZE_OFFSET,
      RESOURCE_PROVIDER_SIZE.toInt(),
    )
    descriptor.set(ValueLayout.ADDRESS, RESOURCE_PROVIDER_CALLBACK_OFFSET, stub)
    descriptor.set(ValueLayout.ADDRESS, RESOURCE_PROVIDER_USER_DATA_OFFSET, MemorySegment.NULL)
  }

  fun descriptor(): MemorySegment = descriptor

  @Suppress("UNUSED_PARAMETER")
  fun invoke(userData: MemorySegment, request: MemorySegment, handle: MemorySegment): Int {
    if (request == MemorySegment.NULL || handle == MemorySegment.NULL) return UNKNOWN_DECISION
    val lease = gate.enter() ?: return UNKNOWN_DECISION
    var requestHandle: ResourceRequestHandle? = null
    return try {
      requestHandle = ResourceRequestHandle(handle)
      val decision = callback.handle(NativeAccess.resourceRequest(request), requestHandle)
      requestHandle.finishProviderDecision(decision)
    } catch (_: Throwable) {
      requestHandle?.finishProviderException() ?: UNKNOWN_DECISION
    } finally {
      lease.close()
    }
  }

  override fun close() = gate.close()

  private companion object {
    private const val UNKNOWN_DECISION: Int = -1

    private val callbackDescriptor =
      FunctionDescriptor.of(
        ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
        ValueLayout.ADDRESS,
      )

    private const val RESOURCE_PROVIDER_SIZE: Long = 24
    private const val RESOURCE_PROVIDER_SIZE_OFFSET: Long = 0
    private const val RESOURCE_PROVIDER_CALLBACK_OFFSET: Long = 8
    private const val RESOURCE_PROVIDER_USER_DATA_OFFSET: Long = 16
  }
}

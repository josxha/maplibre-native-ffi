package org.maplibre.nativeffi.runtime

import kotlin.test.Test
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import org.maplibre.nativeffi.error.InvalidStateException

class RuntimeHandleTest {
  @Test
  fun runtimeRunsOnceAndCloses() {
    val runtime = RuntimeHandle.create(RuntimeOptions())

    assertFalse(runtime.isClosed)
    runtime.runOnce()
    runtime.close()
    runtime.close()

    assertTrue(runtime.isClosed)
    assertFailsWith<InvalidStateException> { runtime.runOnce() }
  }
}

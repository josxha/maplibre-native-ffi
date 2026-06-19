package org.maplibre.nativeffi.internal.callback

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class CallbackGateTest {
  @Test
  fun closeWithoutEnteredCallbacksClosesNativeStateOnce() {
    var closes = 0
    val gate = CallbackGate("test callbacks") { closes++ }

    gate.close()
    gate.close()

    assertTrue(gate.isClosedForTesting())
    assertEquals(1, closes)
    assertNull(gate.enter())
  }

  @Test
  fun closeDuringEnteredCallbackDefersNativeCloseUntilCallbackExits() {
    var closes = 0
    val gate = CallbackGate("test callbacks") { closes++ }
    val lease = assertNotNull(gate.enter())

    gate.close()

    assertFalse(gate.isClosedForTesting())
    assertEquals(0, closes)
    assertNull(gate.enter())

    lease.close()
    lease.close()

    assertTrue(gate.isClosedForTesting())
    assertEquals(1, closes)
  }

  @Test
  fun multipleEnteredCallbacksDelayNativeCloseUntilLastExit() {
    var closes = 0
    val gate = CallbackGate("test callbacks") { closes++ }
    val first = assertNotNull(gate.enter())
    val second = assertNotNull(gate.enter())

    gate.close()
    first.close()

    assertFalse(gate.isClosedForTesting())
    assertEquals(0, closes)

    second.close()

    assertTrue(gate.isClosedForTesting())
    assertEquals(1, closes)
  }
}

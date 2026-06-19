package org.maplibre.nativeffi.map

import kotlin.test.Test
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertSame
import kotlin.test.assertTrue
import org.maplibre.nativeffi.error.InvalidStateException
import org.maplibre.nativeffi.runtime.RuntimeHandle
import org.maplibre.nativeffi.runtime.RuntimeOptions

class MapHandleTest {
  @Test
  fun mapCreateStyleAndCloseRetainsRuntime() {
    val runtime = RuntimeHandle.create(RuntimeOptions())
    val map =
      MapHandle.create(
        runtime,
        MapOptions().apply {
          width = 64
          height = 64
          scaleFactor = 1.0
          mapMode = MapMode.STATIC
        },
      )

    assertFalse(map.isClosed)
    assertSame(runtime, map.runtime())
    assertFailsWith<InvalidStateException> { runtime.close() }

    map.setStyleJson("""{"version":8,"sources":{},"layers":[]}""")
    map.setStyleUrl("https://example.com/style.json")
    map.close()
    map.close()

    assertTrue(map.isClosed)
    assertFailsWith<InvalidStateException> {
      map.setStyleJson("""{"version":8,"sources":{},"layers":[]}""")
    }
    runtime.close()
    assertTrue(runtime.isClosed)
  }
}

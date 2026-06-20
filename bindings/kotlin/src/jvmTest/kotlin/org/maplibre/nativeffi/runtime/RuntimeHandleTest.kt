package org.maplibre.nativeffi.runtime

import kotlin.test.Test
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue
import org.maplibre.nativeffi.error.InvalidArgumentException
import org.maplibre.nativeffi.error.InvalidStateException
import org.maplibre.nativeffi.geo.Geometry
import org.maplibre.nativeffi.geo.LatLng
import org.maplibre.nativeffi.offline.OfflineRegionDefinition

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

  @Test
  fun freshRuntimeHasNoQueuedEvent() {
    RuntimeHandle.create(RuntimeOptions()).use { runtime -> assertNull(runtime.pollEvent()) }
  }

  @Test
  fun ambientCacheOperationRetainsRuntimeUntilDiscarded() {
    val runtime = RuntimeHandle.create(RuntimeOptions())
    val operation = runtime.startAmbientCacheOperation(AmbientCacheOperation.INVALIDATE)

    assertFalse(operation.isClosed)
    assertFailsWith<InvalidStateException> { runtime.close() }

    operation.close()
    operation.close()

    assertTrue(operation.isClosed)
    runtime.close()
    assertTrue(runtime.isClosed)
  }

  @Test
  fun geometryOfflineRegionDefinitionReachesNativeValidation() {
    RuntimeHandle.create(RuntimeOptions()).use { runtime ->
      assertFailsWith<InvalidArgumentException> {
        runtime.startCreateOfflineRegion(
          OfflineRegionDefinition.GeometryRegion(
            "asset://style.json",
            Geometry.Point(LatLng(1.0, 2.0)),
            0.0,
            1.0,
            1.0f,
            false,
          ),
          ByteArray(0),
        )
      }
    }
  }
}

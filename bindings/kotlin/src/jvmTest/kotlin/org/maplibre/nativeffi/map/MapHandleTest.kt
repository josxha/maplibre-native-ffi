package org.maplibre.nativeffi.map

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertSame
import kotlin.test.assertTrue
import org.maplibre.nativeffi.error.InvalidStateException
import org.maplibre.nativeffi.geo.LatLng
import org.maplibre.nativeffi.json.JsonValue
import org.maplibre.nativeffi.render.PremultipliedRgba8Image
import org.maplibre.nativeffi.runtime.RuntimeHandle
import org.maplibre.nativeffi.runtime.RuntimeOptions
import org.maplibre.nativeffi.style.SourceType
import org.maplibre.nativeffi.style.StyleImageOptions

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

  @Test
  fun styleSourceJsonCanBeAddedInspectedListedAndRemoved() {
    val runtime = RuntimeHandle.create(RuntimeOptions())
    val map =
      MapHandle.create(
        runtime,
        MapOptions().apply {
          width = 64
          height = 64
          mapMode = MapMode.STATIC
        },
      )

    try {
      map.setStyleJson("""{"version":8,"sources":{},"layers":[]}""")
      map.addStyleSourceJson("places", geoJsonSource())

      assertTrue(map.styleSourceExists("places"))
      assertEquals(SourceType.GEOJSON, map.styleSourceType("places"))
      assertEquals(SourceType.GEOJSON, map.styleSourceInfo("places")?.type)
      assertTrue(map.styleSourceIds().contains("places"))
      assertTrue(map.removeStyleSource("places"))
      assertFalse(map.styleSourceExists("places"))
      assertFalse(map.removeStyleSource("places"))
    } finally {
      map.close()
      runtime.close()
    }
  }

  @Test
  fun styleLayerJsonCanBeAddedInspectedListedAndRemoved() {
    val runtime = RuntimeHandle.create(RuntimeOptions())
    val map =
      MapHandle.create(
        runtime,
        MapOptions().apply {
          width = 64
          height = 64
          mapMode = MapMode.STATIC
        },
      )

    try {
      map.setStyleJson("""{"version":8,"sources":{},"layers":[]}""")
      map.addStyleLayerJson(backgroundLayer(), "")
      map.addLocationIndicatorLayer("puck", "")
      map.setLocationIndicatorLocation("puck", LatLng(12.0, 34.0), 56.0)
      map.setLocationIndicatorBearing("puck", 78.0)
      map.setLocationIndicatorAccuracyRadius("puck", 9.0)
      map.moveStyleLayer("puck", "background")

      assertTrue(map.styleLayerExists("background"))
      assertEquals("background", map.styleLayerType("background"))
      assertTrue(map.styleLayerExists("puck"))
      assertTrue(map.styleLayerIds().contains("background"))
      assertTrue(map.styleLayerIds().contains("puck"))
      assertEquals("background", map.styleLayerJson("background")?.objectMember("type"))
      map.setLayerProperty("background", "background-opacity", JsonValue.DoubleValue(0.5))
      assertEquals(
        JsonValue.DoubleValue(0.5),
        map.layerProperty("background", "background-opacity"),
      )
      map.setStyleLightProperty("anchor", JsonValue.StringValue("viewport"))
      assertEquals(JsonValue.StringValue("viewport"), map.styleLightProperty("anchor"))
      assertTrue(map.removeStyleLayer("background"))
      assertTrue(map.removeStyleLayer("puck"))
      assertFalse(map.styleLayerExists("background"))
      assertFalse(map.removeStyleLayer("background"))
    } finally {
      map.close()
      runtime.close()
    }
  }

  @Test
  fun styleImageCanBeSetCopiedInspectedAndRemoved() {
    val runtime = RuntimeHandle.create(RuntimeOptions())
    val map =
      MapHandle.create(
        runtime,
        MapOptions().apply {
          width = 64
          height = 64
          mapMode = MapMode.STATIC
        },
      )

    try {
      val image = PremultipliedRgba8Image(1, 1, 4, byteArrayOf(1, 2, 3, 4))
      val options =
        StyleImageOptions().apply {
          pixelRatio = 2.0f
          sdf = true
        }

      map.setStyleJson("""{"version":8,"sources":{},"layers":[]}""")
      map.setStyleImage("dot", image, options)

      assertTrue(map.styleImageExists("dot"))
      val info = map.styleImageInfo("dot")
      assertEquals(1, info?.width)
      assertEquals(1, info?.height)
      assertEquals(4, info?.stride)
      assertEquals(4, info?.byteLength)
      assertEquals(2.0f, info?.pixelRatio)
      assertEquals(true, info?.sdf)
      assertEquals(image, map.copyStyleImagePremultipliedRgba8("dot")?.image)
      assertEquals(2.0f, map.copyStyleImagePremultipliedRgba8("dot")?.pixelRatio)
      assertEquals(true, map.copyStyleImagePremultipliedRgba8("dot")?.sdf)
      assertTrue(map.removeStyleImage("dot"))
      assertFalse(map.styleImageExists("dot"))
      assertFalse(map.removeStyleImage("dot"))
    } finally {
      map.close()
      runtime.close()
    }
  }

  private fun geoJsonSource(): JsonValue =
    JsonValue.ObjectValue(
      listOf(
        JsonValue.Member("type", JsonValue.StringValue("geojson")),
        JsonValue.Member(
          "data",
          JsonValue.ObjectValue(
            listOf(
              JsonValue.Member("type", JsonValue.StringValue("FeatureCollection")),
              JsonValue.Member("features", JsonValue.Array(emptyList())),
            )
          ),
        ),
      )
    )

  private fun backgroundLayer(): JsonValue =
    JsonValue.ObjectValue(
      listOf(
        JsonValue.Member("id", JsonValue.StringValue("background")),
        JsonValue.Member("type", JsonValue.StringValue("background")),
      )
    )

  private fun JsonValue.objectMember(key: String): String? =
    (this as? JsonValue.ObjectValue)
      ?.members
      ?.firstOrNull { it.key == key }
      ?.value
      ?.let { it as? JsonValue.StringValue }
      ?.value
}

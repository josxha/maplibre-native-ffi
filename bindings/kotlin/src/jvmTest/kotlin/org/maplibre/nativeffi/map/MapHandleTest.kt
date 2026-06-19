package org.maplibre.nativeffi.map

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertSame
import kotlin.test.assertTrue
import org.maplibre.nativeffi.camera.EdgeInsets
import org.maplibre.nativeffi.error.InvalidStateException
import org.maplibre.nativeffi.geo.LatLng
import org.maplibre.nativeffi.json.JsonValue
import org.maplibre.nativeffi.render.PremultipliedRgba8Image
import org.maplibre.nativeffi.runtime.RuntimeHandle
import org.maplibre.nativeffi.runtime.RuntimeOptions
import org.maplibre.nativeffi.style.RasterDemEncoding
import org.maplibre.nativeffi.style.SourceType
import org.maplibre.nativeffi.style.StyleImageOptions
import org.maplibre.nativeffi.style.TileScheme
import org.maplibre.nativeffi.style.TileSourceOptions
import org.maplibre.nativeffi.style.VectorTileEncoding

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
  fun tileSourcesCanBeAddedAndInspected() {
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
      map.addVectorSourceUrl(
        "roads",
        "https://example.com/vector.json",
        TileSourceOptions().apply {
          minZoom = 1.0
          maxZoom = 12.0
          attribution = "vector attribution"
          scheme = TileScheme.XYZ
          vectorEncoding = VectorTileEncoding.MVT
        },
      )
      map.addRasterSourceTiles(
        "satellite",
        listOf("https://example.com/raster/{z}/{x}/{y}.png"),
        TileSourceOptions().apply { tileSize = 256 },
      )
      map.addRasterDemSourceTiles(
        "terrain",
        listOf("https://example.com/terrain/{z}/{x}/{y}.png"),
        TileSourceOptions().apply {
          tileSize = 512
          rasterDemEncoding = RasterDemEncoding.TERRARIUM
        },
      )

      assertEquals(SourceType.VECTOR, map.styleSourceType("roads"))
      assertEquals(SourceType.RASTER, map.styleSourceInfo("satellite")?.type)
      assertEquals(SourceType.RASTER_DEM, map.styleSourceType("terrain"))
      assertTrue(map.styleSourceIds().containsAll(listOf("roads", "satellite", "terrain")))
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

  @Test
  fun imageSourcesCanBeAddedUpdatedAndInspected() {
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
      val coordinates = imageCoordinates()
      val moved = listOf(LatLng(1.0, 0.0), LatLng(1.0, 1.0), LatLng(0.0, 1.0), LatLng(0.0, 0.0))

      map.setStyleJson("""{"version":8,"sources":{},"layers":[]}""")
      map.addImageSourceUrl("overlay", coordinates, "https://example.com/image.png")

      assertEquals(SourceType.IMAGE, map.styleSourceType("overlay"))
      assertEquals(coordinates, map.imageSourceCoordinates("overlay"))
      map.setImageSourceUrl("overlay", "https://example.com/updated-image.png")
      map.setImageSourceImage("overlay", image)
      map.setImageSourceCoordinates("overlay", moved)
      assertEquals(moved, map.imageSourceCoordinates("overlay"))
      assertEquals(null, map.imageSourceCoordinates("missing-overlay"))

      map.addImageSourceImage("inline-overlay", coordinates, image)
      assertEquals(SourceType.IMAGE, map.styleSourceInfo("inline-overlay")?.type)
    } finally {
      map.close()
      runtime.close()
    }
  }

  @Test
  fun mapDebugControlsCanBeReadAndWritten() {
    val runtime = RuntimeHandle.create(RuntimeOptions())
    val map =
      MapHandle.create(
        runtime,
        MapOptions().apply {
          width = 64
          height = 64
          mapMode = MapMode.CONTINUOUS
        },
      )

    try {
      assertTrue(map.debugOptions.isEmpty())
      map.debugOptions = setOf(DebugOption.TILE_BORDERS, DebugOption.TIMESTAMPS)
      assertEquals(setOf(DebugOption.TILE_BORDERS, DebugOption.TIMESTAMPS), map.debugOptions)
      map.debugOptions = emptySet()
      assertTrue(map.debugOptions.isEmpty())

      assertFalse(map.isRenderingStatsViewEnabled)
      map.isRenderingStatsViewEnabled = true
      assertTrue(map.isRenderingStatsViewEnabled)
      map.isRenderingStatsViewEnabled = false
      assertFalse(map.isRenderingStatsViewEnabled)

      map.requestRepaint()
      map.isFullyLoaded
      map.dumpDebugLogs()
    } finally {
      map.close()
      runtime.close()
    }
  }

  @Test
  fun mapOptionPropertiesCanBeRoundTripped() {
    val runtime = RuntimeHandle.create(RuntimeOptions())
    val map =
      MapHandle.create(
        runtime,
        MapOptions().apply {
          width = 128
          height = 128
          mapMode = MapMode.STATIC
        },
      )

    try {
      map.viewportOptions =
        ViewportOptions().apply {
          northOrientation = NorthOrientation.UP
          constrainMode = ConstrainMode.HEIGHT_ONLY
          viewportMode = ViewportMode.DEFAULT
          frustumOffset = EdgeInsets.ZERO
        }
      val viewport = map.viewportOptions
      assertEquals(NorthOrientation.UP, viewport.northOrientation)
      assertEquals(ConstrainMode.HEIGHT_ONLY, viewport.constrainMode)
      assertEquals(ViewportMode.DEFAULT, viewport.viewportMode)
      assertEquals(EdgeInsets.ZERO, viewport.frustumOffset)

      map.tileOptions =
        TileOptions().apply {
          prefetchZoomDelta = 2
          lodMinRadius = 1.5
          lodScale = 2.5
          lodPitchThreshold = 30.0
          lodZoomShift = 1.0
          lodMode = TileLodMode.DEFAULT
        }
      val tile = map.tileOptions
      assertEquals(2, tile.prefetchZoomDelta)
      assertEquals(1.5, tile.lodMinRadius)
      assertEquals(2.5, tile.lodScale)
      assertEquals(30.0, tile.lodPitchThreshold)
      assertEquals(1.0, tile.lodZoomShift)
      assertEquals(TileLodMode.DEFAULT, tile.lodMode)

      map.projectionMode =
        ProjectionModeOptions().apply {
          axonometric = false
          xSkew = 0.0
          ySkew = 0.0
        }
      val projectionMode = map.projectionMode
      assertEquals(false, projectionMode.axonometric)
      assertEquals(0.0, projectionMode.xSkew)
      assertEquals(0.0, projectionMode.ySkew)
    } finally {
      map.close()
      runtime.close()
    }
  }

  @Test
  fun mapCoordinateConversionsCanBeRoundTripped() {
    val runtime = RuntimeHandle.create(RuntimeOptions())
    val map =
      MapHandle.create(
        runtime,
        MapOptions().apply {
          width = 128
          height = 128
          mapMode = MapMode.STATIC
        },
      )

    try {
      val coordinate = LatLng(0.0, 0.0)
      val point = map.pixelForLatLng(coordinate)
      val roundTrip = map.latLngForPixel(point)
      assertNear(coordinate, roundTrip)

      val coordinates = listOf(LatLng(0.0, 0.0), LatLng(10.0, 20.0))
      val points = map.pixelsForLatLngs(coordinates)
      assertEquals(coordinates.size, points.size)
      val coordinateRoundTrips = map.latLngsForPixels(points)
      assertEquals(coordinates.size, coordinateRoundTrips.size)
      coordinates.zip(coordinateRoundTrips).forEach { (expected, actual) ->
        assertNear(expected, actual)
      }

      assertEquals(emptyList(), map.pixelsForLatLngs(emptyList()))
      assertEquals(emptyList(), map.latLngsForPixels(emptyList()))
    } finally {
      map.close()
      runtime.close()
    }
  }

  @Test
  fun mapProjectionCoordinateConversionsCanBeRoundTrippedAndClosed() {
    val runtime = RuntimeHandle.create(RuntimeOptions())
    val map =
      MapHandle.create(
        runtime,
        MapOptions().apply {
          width = 128
          height = 128
          mapMode = MapMode.STATIC
        },
      )

    try {
      val projection = map.createProjection()
      assertFalse(projection.isClosed)
      val coordinate = LatLng(0.0, 0.0)
      val point = projection.pixelForLatLng(coordinate)
      val roundTrip = projection.latLngForPixel(point)
      assertNear(coordinate, roundTrip)

      map.close()
      val detachedPoint = projection.pixelForLatLng(coordinate)
      assertNear(coordinate, projection.latLngForPixel(detachedPoint))

      projection.close()
      projection.close()
      assertTrue(projection.isClosed)
      assertFailsWith<InvalidStateException> { projection.pixelForLatLng(coordinate) }
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

  private fun imageCoordinates(): List<LatLng> =
    listOf(LatLng(0.0, 0.0), LatLng(0.0, 1.0), LatLng(1.0, 1.0), LatLng(1.0, 0.0))

  private fun assertNear(expected: LatLng, actual: LatLng) {
    assertEquals(expected.latitude, actual.latitude, 1e-6)
    assertEquals(expected.longitude, actual.longitude, 1e-6)
  }

  private fun JsonValue.objectMember(key: String): String? =
    (this as? JsonValue.ObjectValue)
      ?.members
      ?.firstOrNull { it.key == key }
      ?.value
      ?.let { it as? JsonValue.StringValue }
      ?.value
}

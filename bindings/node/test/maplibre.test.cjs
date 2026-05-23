const assert = require("node:assert/strict");
const path = require("node:path");
const test = require("node:test");
const { Worker } = require("node:worker_threads");

const {
  clearLogCallback,
  cVersion,
  InvalidArgumentError,
  InvalidStateError,
  MaplibreError,
  MapHandle,
  MapProjectionHandle,
  latLngForProjectedMeters,
  MaplibreStatus,
  NativeBuffer,
  NativePointer,
  networkStatus,
  projectedMetersForLatLng,
  restoreDefaultAsyncLogSeverities,
  RuntimeHandle,
  setAsyncLogSeverities,
  setLogCallback,
  setNetworkStatus,
  supportedRenderBackends,
} = require("..");

test("process-global proof slice crosses the native add-on", () => {
  assert.equal(cVersion(), 0);

  const backends = supportedRenderBackends();
  assert.equal(typeof backends.rawMask, "number");
  assert.equal(typeof backends.metal, "boolean");
  assert.equal(typeof backends.vulkan, "boolean");

  const original = networkStatus();
  assert.match(original.kind, /^(online|offline|unknown)$/);

  setNetworkStatus("online");
  assert.equal(networkStatus().kind, "online");

  setNetworkStatus("offline");
  assert.equal(networkStatus().kind, "offline");

  if (original.kind === "online" || original.kind === "offline") {
    setNetworkStatus(original.kind);
  }
});

test("projection helpers round trip copied coordinate values", () => {
  const coordinate = { latitude: 45, longitude: -122 };
  const meters = projectedMetersForLatLng(coordinate);
  const roundTripped = latLngForProjectedMeters(meters);

  assert.equal(typeof meters.northing, "number");
  assert.equal(typeof meters.easting, "number");
  assert.ok(Math.abs(roundTripped.latitude - coordinate.latitude) < 1e-9);
  assert.ok(Math.abs(roundTripped.longitude - coordinate.longitude) < 1e-9);
});

test("log callback copies records through the Node event loop", async () => {
  /** @type {import("..").LogRecord[]} */
  const records = [];
  setLogCallback((record) => records.push(record));
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 16, height: 16 });

  try {
    map.dumpDebugLogs();
    await eventually(() => records.length > 0);
    assert.equal(typeof records[0].message, "string");
    assert.equal(typeof records[0].rawSeverity, "number");
    assert.equal(typeof records[0].rawEvent, "number");
  } finally {
    map.close();
    runtime.close();
    clearLogCallback();
  }
});

test("async log severities map string values and reject unknown values", () => {
  setAsyncLogSeverities(["info", "warning"]);
  setAsyncLogSeverities(new Set(["error"]));
  restoreDefaultAsyncLogSeverities();

  assert.throws(
    () => setAsyncLogSeverities([/** @type {any} */ ("debug")]),
    InvalidArgumentError,
  );
});

test("native pointer is a borrowed opaque address value", () => {
  const pointer = NativePointer.unsafeFromAddress(0x1234n);

  assert.equal(pointer.address, 0x1234n);
  assert.equal(pointer.isNull, false);
  assert.equal(pointer.equals(NativePointer.unsafeFromAddress(0x1234n)), true);
  assert.equal(pointer.equals(NativePointer.null), false);
  assert.equal(NativePointer.null.isNull, true);
  assert.equal(pointer.toString(), "NativePointer[address=0x1234]");
  assert.throws(
    () => NativePointer.unsafeFromAddress(-1n),
    InvalidArgumentError,
  );
});

test("native buffer owns byte storage for render interop", () => {
  const allocated = NativeBuffer.allocate(4);
  allocated.asUint8Array().set([1, 2, 3, 4]);

  assert.equal(allocated.byteLength, 4);
  assert.deepEqual([...allocated.asUint8Array()], [1, 2, 3, 4]);
  assert.equal(allocated.asArrayBuffer() instanceof ArrayBuffer, true);
  assert.equal(
    Object.prototype.toString.call(allocated),
    "[object NativeBuffer]",
  );

  const copied = NativeBuffer.from(allocated.asUint8Array());
  allocated.asUint8Array()[0] = 9;
  assert.deepEqual([...copied.asUint8Array()], [1, 2, 3, 4]);
  assert.throws(() => NativeBuffer.allocate(-1), InvalidArgumentError);
  assert.throws(
    () => NativeBuffer.from(/** @type {any} */ ("bytes")),
    InvalidArgumentError,
  );
});

test("handles stay local while workers create their own runtime", async () => {
  const worker = new Worker(
    `
      const { parentPort, workerData } = require("node:worker_threads");
      const { RuntimeHandle } = require(workerData.packageRoot);
      try {
        const runtime = new RuntimeHandle();
        runtime.close();
        parentPort.postMessage({ ok: true });
      } catch (error) {
        parentPort.postMessage({
          ok: false,
          name: error?.name,
          message: error?.message,
        });
      }
    `,
    {
      eval: true,
      workerData: { packageRoot: path.join(__dirname, "..") },
    },
  );
  const runtime = new RuntimeHandle();

  try {
    const clone = structuredClone(runtime);
    assert.equal(clone instanceof RuntimeHandle, false);
    assert.equal(typeof clone.close, "undefined");
  } finally {
    runtime.close();
  }

  const result = await new Promise((resolve, reject) => {
    worker.once("message", resolve);
    worker.once("error", reject);
    worker.once("exit", (code) => {
      if (code !== 0) {
        reject(new Error(`worker exited with code ${code}`));
      }
    });
  });
  assert.deepEqual(result, { ok: true });
});

test("runtime handle supports options, explicit close, and idempotent disposal", () => {
  const runtime = new RuntimeHandle({ maximumCacheSize: 1n });

  assert.equal(runtime.closed, false);
  runtime.runOnce();
  assert.equal(runtime.pollEvent(), null);
  runtime.close();
  assert.equal(runtime.closed, true);
  runtime.close();
  runtime[Symbol.dispose]();
});

test("map handle retains runtime parent and closes before runtime", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 32, height: 32, scaleFactor: 1 });

  assert.equal(map instanceof MapHandle, true);
  assert.equal(map.closed, false);
  assert.throws(() => runtime.close(), InvalidStateError);
  map.close();
  assert.equal(map.closed, true);
  map.close();
  runtime.close();
});

test("map utility methods expose copied booleans and native commands", () => {
  const runtime = new RuntimeHandle();
  const continuousMap = runtime.createMap({ width: 16, height: 16 });

  try {
    assert.equal(typeof continuousMap.isFullyLoaded(), "boolean");
    continuousMap.renderingStatsViewEnabled = true;
    assert.equal(continuousMap.renderingStatsViewEnabled, true);
    continuousMap.renderingStatsViewEnabled = false;
    assert.equal(continuousMap.renderingStatsViewEnabled, false);
    continuousMap.requestRepaint();
    continuousMap.dumpDebugLogs();
  } finally {
    continuousMap.close();
  }

  const staticMap = runtime.createMap({
    width: 16,
    height: 16,
    mapMode: "static",
  });
  try {
    staticMap.requestStillImage();
  } finally {
    staticMap.close();
    runtime.close();
  }
});

test("map viewport and tile options map descriptor fields", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 16, height: 16 });

  try {
    map.setViewportOptions({
      northOrientation: "right",
      constrainMode: "screen",
      viewportMode: "flippedY",
      frustumOffset: { top: 1, left: 2, bottom: 3, right: 4 },
    });
    const viewport = map.getViewportOptions();
    assert.equal(viewport.northOrientation, "right");
    assert.equal(viewport.constrainMode, "screen");
    assert.equal(viewport.viewportMode, "flippedY");
    assert.deepEqual(viewport.frustumOffset, {
      top: 1,
      left: 2,
      bottom: 3,
      right: 4,
    });

    map.setTileOptions({
      prefetchZoomDelta: 2,
      lodMinRadius: 1,
      lodScale: 1.5,
      lodPitchThreshold: 20,
      lodZoomShift: 0.5,
      lodMode: "distance",
    });
    const tile = map.getTileOptions();
    assert.equal(tile.prefetchZoomDelta, 2);
    assert.equal(tile.lodMode, "distance");
    assert.throws(
      () =>
        map.setViewportOptions({
          northOrientation: /** @type {any} */ ("north"),
        }),
      InvalidArgumentError,
    );
    assert.throws(
      () => map.setTileOptions({ lodMode: /** @type {any} */ ("nearest") }),
      InvalidArgumentError,
    );
  } finally {
    map.close();
    runtime.close();
  }
});

test("map projection mode maps optional descriptor fields", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 16, height: 16 });

  try {
    map.setProjectionMode({ axonometric: true, xSkew: 0.2, ySkew: 0.3 });
    const mode = map.getProjectionMode();
    assert.equal(mode.axonometric, true);
    assert.equal(mode.xSkew, 0.2);
    assert.equal(mode.ySkew, 0.3);
  } finally {
    map.close();
    runtime.close();
  }
});

test("map camera commands copy descriptor values", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 16, height: 16 });

  try {
    map.jumpTo({
      center: { latitude: 12.5, longitude: 34.5 },
      zoom: 3,
      bearing: 10,
      pitch: 20,
    });
    const camera = map.getCamera();
    assert.ok(camera.center);
    assert.ok(Math.abs(camera.center.latitude - 12.5) < 1e-9);
    assert.ok(Math.abs(camera.center.longitude - 34.5) < 1e-9);
    assert.equal(camera.zoom, 3);
    assert.equal(camera.bearing, 10);
    assert.equal(camera.pitch, 20);
  } finally {
    map.close();
    runtime.close();
  }
});

test("map camera movement commands adapt point descriptors", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 256, height: 256 });

  try {
    map.moveBy(1, 2);
    map.scaleBy(1.1, { x: 10, y: 10 });
    map.scaleBy(1.0);
    map.rotateBy({ x: 10, y: 10 }, { x: 12, y: 12 });
    map.pitchBy(1);
    map.cancelTransitions();
  } finally {
    map.close();
    runtime.close();
  }
});

test("map projection handle snapshots projection state", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 256, height: 256 });
  let projection;

  try {
    map.jumpTo({ center: { latitude: 5, longitude: 6 }, zoom: 2 });
    projection = map.createProjection();
    assert.equal(projection instanceof MapProjectionHandle, true);
    const point = projection.pixelForLatLng({ latitude: 5, longitude: 6 });
    const roundTripped = projection.latLngForPixel(point);
    assert.ok(Math.abs(roundTripped.latitude - 5) < 1e-9);
    assert.ok(Math.abs(roundTripped.longitude - 6) < 1e-9);
    projection.setCamera({ center: { latitude: 7, longitude: 8 }, zoom: 3 });
    assert.ok(projection.getCamera().center);
    projection.close();
    assert.equal(projection.closed, true);
    projection.close();
  } finally {
    projection?.close();
    map.close();
    runtime.close();
  }
});

test("map screen projection helpers copy point values", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 256, height: 256 });
  const coordinate = { latitude: 10, longitude: 20 };

  try {
    map.jumpTo({ center: coordinate, zoom: 2 });
    const point = map.pixelForLatLng(coordinate);
    const roundTripped = map.latLngForPixel(point);
    const points = map.pixelsForLatLngs([coordinate]);
    const coordinates = map.latLngsForPixels(points);
    assert.equal(typeof point.x, "number");
    assert.equal(typeof point.y, "number");
    assert.equal(points.length, 1);
    assert.equal(coordinates.length, 1);
    assert.ok(Math.abs(roundTripped.latitude - coordinate.latitude) < 1e-9);
    assert.ok(Math.abs(roundTripped.longitude - coordinate.longitude) < 1e-9);
    assert.ok(Math.abs(coordinates[0].latitude - coordinate.latitude) < 1e-9);
    assert.ok(Math.abs(coordinates[0].longitude - coordinate.longitude) < 1e-9);
  } finally {
    map.close();
    runtime.close();
  }
});

test("style JSON helpers serialize JavaScript values and copy booleans", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 16, height: 16 });

  try {
    map.setStyleJson('{"version":8,"sources":{},"layers":[]}');
    runtime.runOnce();
    map.addStyleSourceJson("empty-geojson", {
      type: "geojson",
      data: { type: "FeatureCollection", features: [] },
    });
    map.setStyleLight({ anchor: "viewport", color: "#ffffff", intensity: 0.5 });
    map.setStyleLightProperty("intensity", 0.75);
    assert.equal(map.getStyleLightProperty("intensity"), 0.75);
    assert.deepEqual(map.getStyleLightProperty("color"), [
      "rgba",
      255,
      255,
      255,
      1,
    ]);

    assert.equal(map.styleSourceExists("empty-geojson"), true);
    assert.equal(map.listStyleSourceIds().includes("empty-geojson"), true);
    assert.equal(map.getStyleSourceType("empty-geojson"), "geojson");
    assert.equal(
      map.getStyleSourceInfo("empty-geojson")?.sourceType,
      "geojson",
    );
    assert.equal(map.getStyleSourceInfo("missing-source"), null);

    const imageCoordinates = [
      { latitude: 1, longitude: 2 },
      { latitude: 1, longitude: 3 },
      { latitude: 0, longitude: 3 },
      { latitude: 0, longitude: 2 },
    ];
    map.addGeoJsonSourceUrl("geojson-url", "https://example.test/data.geojson");
    assert.equal(map.getStyleSourceType("geojson-url"), "geojson");
    map.setGeoJsonSourceUrl(
      "geojson-url",
      "https://example.test/updated.geojson",
    );
    map.addVectorSourceUrl("vector-url", "https://example.test/vector.json");
    assert.equal(map.getStyleSourceType("vector-url"), "vector");
    map.addRasterSourceUrl("raster-url", "https://example.test/raster.json");
    assert.equal(map.getStyleSourceType("raster-url"), "raster");
    map.addRasterDemSourceUrl(
      "raster-dem-url",
      "https://example.test/dem.json",
    );
    assert.equal(map.getStyleSourceType("raster-dem-url"), "raster-dem");

    map.addImageSourceUrl(
      "image-source",
      imageCoordinates,
      "https://example.test/image.png",
    );
    assert.equal(map.getStyleSourceType("image-source"), "image");
    map.setImageSourceUrl("image-source", "https://example.test/updated.png");
    map.setImageSourceCoordinates("image-source", imageCoordinates);
    assert.deepEqual(
      map.getImageSourceCoordinates("image-source"),
      imageCoordinates,
    );
    assert.equal(map.getImageSourceCoordinates("missing-source"), null);
    assert.equal(map.removeStyleSource("image-source"), true);
    assert.equal(map.removeStyleSource("raster-dem-url"), true);
    assert.equal(map.removeStyleSource("raster-url"), true);
    assert.equal(map.removeStyleSource("vector-url"), true);
    assert.equal(map.removeStyleSource("geojson-url"), true);
    assert.equal(map.removeStyleSource("empty-geojson"), true);

    map.addStyleLayerJson({
      id: "background",
      type: "background",
      paint: { "background-color": "#000000" },
    });
    map.addStyleLayerJson({ id: "background-2", type: "background" });
    map.moveStyleLayer("background-2", "background");
    map.setLayerProperty("background", "background-color", "#ff0000");
    const backgroundColor = map.getLayerProperty(
      "background",
      "background-color",
    );
    assert.deepEqual(backgroundColor, ["rgba", 255, 0, 0, 1]);
    assert.equal(
      map.getLayerProperty("background", "background-opacity"),
      null,
    );
    assert.equal(map.getLayerFilter("background"), null);
    assert.equal(map.styleLayerExists("background"), true);
    assert.equal(map.listStyleLayerIds().includes("background"), true);
    assert.equal(map.getStyleLayerType("background"), "background");
    const backgroundLayer = map.getStyleLayerJson("background");
    assert.ok(
      backgroundLayer &&
        typeof backgroundLayer === "object" &&
        !Array.isArray(backgroundLayer),
    );
    assert.equal(backgroundLayer.id, "background");
    assert.equal(map.getStyleLayerType("missing-layer"), null);
    assert.equal(map.getStyleLayerJson("missing-layer"), null);
    assert.equal(map.removeStyleLayer("background-2"), true);
    assert.equal(map.removeStyleLayer("background"), true);
    assert.throws(
      () => map.addStyleLayerJson(/** @type {any} */ (undefined)),
      InvalidArgumentError,
    );
  } finally {
    map.close();
    runtime.close();
  }
});

test("style existence and removal probes return copied booleans", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 16, height: 16 });

  try {
    map.setStyleJson('{"version":8,"sources":{},"layers":[]}');
    runtime.runOnce();
    assert.equal(map.styleSourceExists("missing-source"), false);
    assert.equal(map.removeStyleSource("missing-source"), false);
    assert.equal(map.styleLayerExists("missing-layer"), false);
    assert.equal(map.removeStyleLayer("missing-layer"), false);
  } finally {
    map.close();
    runtime.close();
  }
});

test("map debug options map stable strings to native bitmasks", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 16, height: 16 });

  try {
    map.setDebugOptions(["tileBorders", "collision"]);
    assert.deepEqual(map.getDebugOptions(), ["tileBorders", "collision"]);
    map.setDebugOptions([]);
    assert.deepEqual(map.getDebugOptions(), []);
    assert.throws(
      () => map.setDebugOptions([/** @type {any} */ ("wireframe")]),
      InvalidArgumentError,
    );
  } finally {
    map.close();
    runtime.close();
  }
});

test("map options reject unknown map modes", () => {
  const runtime = new RuntimeHandle();
  assert.throws(
    () => runtime.createMap({ mapMode: /** @type {any} */ ("globe") }),
    InvalidArgumentError,
  );
  runtime.close();
});

test("runtime options reject invalid bigint values", () => {
  assert.throws(
    () => new RuntimeHandle({ maximumCacheSize: -1n }),
    (error) => {
      if (!(error instanceof InvalidArgumentError)) {
        return false;
      }
      assert.equal(error.nativeStatusCode, null);
      assert.match(error.diagnostic, /maximumCacheSize/);
      return true;
    },
  );
});

/** @param {() => boolean} predicate */
async function eventually(predicate) {
  const deadline = Date.now() + 500;
  while (Date.now() < deadline) {
    if (predicate()) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
  assert.equal(predicate(), true);
}

test("binding-owned validation rejects unknown network status strings", () => {
  assert.throws(
    () => setNetworkStatus(/** @type {any} */ ("airplane")),
    (error) => {
      if (!(error instanceof InvalidArgumentError)) {
        return false;
      }
      assert.equal(error instanceof MaplibreError, true);
      assert.equal(error.status, MaplibreStatus.invalidArgument);
      assert.equal(error.nativeStatusCode, null);
      assert.match(error.diagnostic, /network status/);
      return true;
    },
  );
});

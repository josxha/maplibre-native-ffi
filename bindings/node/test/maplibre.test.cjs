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
  MetalOwnedTextureFrame,
  NativeBuffer,
  NativePointer,
  OfflineOperationHandle,
  ResourceRequestHandle,
  networkStatus,
  projectedMetersForLatLng,
  restoreDefaultAsyncLogSeverities,
  RenderSessionHandle,
  RuntimeHandle,
  setAsyncLogSeverities,
  VulkanOwnedTextureFrame,
  setLogCallback,
  setNetworkStatus,
  supportedRenderBackends,
  threadLastErrorMessage,
} = require("..");

if (false) {
  const session = /** @type {import("..").RenderSessionHandle} */ (
    /** @type {unknown} */ (null)
  );
  // @ts-expect-error texture frame callbacks release native frames synchronously.
  session.withMetalOwnedTextureFrame(async (frame) => frame.width);
  // @ts-expect-error texture frame callbacks release native frames synchronously.
  session.withVulkanOwnedTextureFrame(async (frame) => frame.width);
}

test("concept subpath modules expose curated public API groups", () => {
  const packageJson = require("../package.json");
  for (const subpath of Object.keys(packageJson.exports)) {
    require(
      subpath === "."
        ? "@maplibre/native-ffi-node"
        : `@maplibre/native-ffi-node${subpath.slice(1)}`,
    );
  }

  const runtimeModule = require("@maplibre/native-ffi-node/runtime");
  const renderModule = require("@maplibre/native-ffi-node/render");
  const errorModule = require("@maplibre/native-ffi-node/error");
  const geoModule = require("@maplibre/native-ffi-node/geo");
  const logModule = require("@maplibre/native-ffi-node/log");
  const mapModule = require("@maplibre/native-ffi-node/map");
  const offlineModule = require("@maplibre/native-ffi-node/offline");
  const resourceModule = require("@maplibre/native-ffi-node/resource");

  assert.equal(runtimeModule.RuntimeHandle, RuntimeHandle);
  assert.equal(runtimeModule.networkStatus, networkStatus);
  assert.equal(renderModule.RenderSessionHandle, RenderSessionHandle);
  assert.equal(renderModule.NativeBuffer, NativeBuffer);
  assert.equal(errorModule.InvalidArgumentError, InvalidArgumentError);
  assert.equal(geoModule.projectedMetersForLatLng, projectedMetersForLatLng);
  assert.equal(logModule.setLogCallback, setLogCallback);
  assert.equal(mapModule.MapHandle, MapHandle);
  assert.equal(offlineModule.OfflineOperationHandle, OfflineOperationHandle);
  assert.equal(resourceModule.ResourceRequestHandle, ResourceRequestHandle);
});

test("process-global APIs cross the native add-on", () => {
  assert.equal(cVersion(), 0);

  const backends = supportedRenderBackends();
  assert.equal(typeof backends.rawMask, "number");
  assert.equal(typeof backends.metal, "boolean");
  assert.equal(typeof backends.vulkan, "boolean");
  assert.equal(typeof threadLastErrorMessage(), "string");

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
  assert.throws(
    () => new /** @type {any} */ (NativePointer)(1n),
    InvalidArgumentError,
  );
});

test("texture frame scopes expose borrowed pointers only while active", () => {
  let released = false;
  /** @type {import("..").MetalOwnedTextureFrame | undefined} */
  let scopedFrame;
  /** @type {import("..").NativePointer | undefined} */
  let scopedTexture;
  const result = RenderSessionHandle.prototype.withMetalOwnedTextureFrame.call(
    {
      native: {
        acquireMetalOwnedTextureFrame() {
          return {
            generation: 1n,
            width: 2,
            height: 3,
            scaleFactor: 4,
            frameId: 5n,
            textureAddress: 0x10n,
            deviceAddress: 0x20n,
            pixelFormat: 80n,
          };
        },
        /** @param {any} raw */
        releaseMetalOwnedTextureFrame(raw) {
          assert.equal(raw.frameId, 5n);
          released = true;
        },
      },
    },
    (frame) => {
      scopedFrame = frame;
      assert.equal(frame instanceof MetalOwnedTextureFrame, true);
      assert.equal(frame.width, 2);
      scopedTexture = frame.texture;
      assert.equal(scopedTexture.address, 0x10n);
      assert.equal(frame.device.address, 0x20n);
      assert.equal(frame.pixelFormat, 80n);
      return "ok";
    },
  );

  assert.equal(result, "ok");
  assert.equal(released, true);
  const frameAfterScope = scopedFrame;
  assert.ok(frameAfterScope);
  assert.throws(() => frameAfterScope.width, InvalidStateError);
  const textureAfterScope = scopedTexture;
  assert.ok(textureAfterScope);
  assert.throws(() => textureAfterScope.address, InvalidStateError);

  assert.equal(typeof VulkanOwnedTextureFrame, "function");
  assert.throws(
    () =>
      RenderSessionHandle.prototype.withVulkanOwnedTextureFrame.call(
        { native: {} },
        /** @type {any} */ (null),
      ),
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
  const sourceBuffer = new ArrayBuffer(4);
  new Uint8Array(sourceBuffer).set([5, 6, 7, 8]);
  const constructorCopied = new NativeBuffer(sourceBuffer);
  new Uint8Array(sourceBuffer)[0] = 1;
  assert.deepEqual([...constructorCopied.asUint8Array()], [5, 6, 7, 8]);
  const sharedCopy = NativeBuffer.from(
    new Uint8Array(new SharedArrayBuffer(4)),
  );
  assert.equal(sharedCopy.asArrayBuffer() instanceof ArrayBuffer, true);
  assert.equal(sharedCopy.asArrayBuffer() instanceof SharedArrayBuffer, false);
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

    assert.throws(
      () =>
        ResourceRequestHandle.prototype.complete.call(
          { handleId: "detached", closed: false },
          {},
        ),
      InvalidStateError,
    );
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

test("offline operations expose discardable handles", () => {
  const runtime = new RuntimeHandle();

  try {
    const operation = runtime.runAmbientCacheOperation("clear");
    assert.equal(operation instanceof OfflineOperationHandle, true);
    assert.equal(typeof operation.operationId, "bigint");
    assert.equal(operation.closed, false);
    operation.close();
    assert.equal(operation.closed, true);
    operation.close();
    const list = runtime.offlineRegionsList();
    assert.equal(list instanceof OfflineOperationHandle, true);
    assert.throws(
      () => runtime.offlineRegionsListTakeResult(list),
      InvalidStateError,
    );
    list.close();
    const get = runtime.offlineRegionGet(1n);
    get.close();
    const status = runtime.offlineRegionGetStatus(1n);
    status.close();
    const updateMetadata = runtime.offlineRegionUpdateMetadata(
      1n,
      new Uint8Array([4, 5]),
    );
    updateMetadata.close();
    const create = runtime.offlineRegionCreate(
      {
        kind: "tilePyramid",
        styleUrl: "https://example.test/style.json",
        bounds: {
          southwest: { latitude: -1, longitude: -2 },
          northeast: { latitude: 1, longitude: 2 },
        },
        minZoom: 0,
        maxZoom: 1,
        pixelRatio: 1,
      },
      new Uint8Array([1, 2, 3]),
    );
    create.close();
    assert.throws(
      () =>
        runtime.offlineRegionSetDownloadState(
          1n,
          /** @type {any} */ ("paused"),
        ),
      InvalidArgumentError,
    );
    assert.throws(
      () => runtime.runAmbientCacheOperation(/** @type {any} */ ("vacuum")),
      InvalidArgumentError,
    );
  } finally {
    runtime.close();
  }
});

test("resource provider callback registration validates Node handoff shape", () => {
  const runtime = new RuntimeHandle();

  try {
    runtime.setResourceProvider((request) => {
      assert.equal(typeof request.url, "string");
      assert.equal(typeof request.rawKind, "number");
      assert.equal(request.handle instanceof ResourceRequestHandle, true);
      return "passThrough";
    });
    assert.throws(
      () => runtime.setResourceProvider(/** @type {any} */ (null)),
      InvalidArgumentError,
    );
  } finally {
    runtime.close();
  }
});

test("runtime handle supports options, resource transform, explicit close, and idempotent disposal", () => {
  const runtime = new RuntimeHandle({ maximumCacheSize: 1n });

  assert.equal(runtime.closed, false);
  runtime.runOnce();
  assert.equal(runtime.pollEvent(), null);
  runtime.setResourceTransform((request) => {
    assert.equal(typeof request.url, "string");
    assert.equal(typeof request.rawKind, "number");
    return null;
  });
  runtime.clearResourceTransform();
  assert.throws(
    () => runtime.setResourceTransform(/** @type {any} */ (null)),
    InvalidArgumentError,
  );
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
  const projection = map.createProjection();
  projection.close();
  assert.throws(
    () =>
      Reflect.construct(/** @type {any} */ (RenderSessionHandle), [
        { closed: false },
        map,
      ]),
    InvalidArgumentError,
  );
  assert.throws(
    () => new /** @type {any} */ (ResourceRequestHandle)("detached"),
    InvalidArgumentError,
  );
  assert.throws(() => runtime.close(), InvalidStateError);
  map.close();
  assert.equal(map.closed, true);
  map.close();
  runtime.close();
});

test("render session attach descriptors translate native failures", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 16, height: 16 });

  try {
    assert.throws(
      () =>
        map.attachMetalOwnedTexture({
          extent: { width: 16, height: 16, scaleFactor: 1 },
          context: { device: NativePointer.null },
        }),
      MaplibreError,
    );
    assert.throws(
      () =>
        map.attachMetalBorrowedTexture({
          extent: { width: 16, height: 16, scaleFactor: 1 },
          texture: /** @type {any} */ (0n),
        }),
      InvalidArgumentError,
    );
    assert.equal(
      typeof RenderSessionHandle.attachMetalOwnedTexture,
      "function",
    );
    assert.equal(
      typeof RenderSessionHandle.attachVulkanOwnedTexture,
      "function",
    );
    assert.equal(
      typeof RenderSessionHandle.prototype.setFeatureState,
      "function",
    );
    assert.equal(
      typeof RenderSessionHandle.prototype.queryRenderedFeatures,
      "function",
    );
    assert.equal(
      typeof RenderSessionHandle.prototype.queryFeatureExtension,
      "function",
    );
  } finally {
    map.close();
    runtime.close();
  }
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

test("map bounds options copy constraints", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 16, height: 16 });

  try {
    const bounds = {
      southwest: { latitude: -10, longitude: -20 },
      northeast: { latitude: 10, longitude: 20 },
    };
    map.setBounds({
      bounds,
      minZoom: 1,
      maxZoom: 10,
      minPitch: 0,
      maxPitch: 45,
    });
    const copied = map.getBounds();
    assert.deepEqual(copied.bounds, bounds);
    assert.equal(copied.minZoom, 1);
    assert.equal(copied.maxZoom, 10);
    assert.equal(copied.minPitch, 0);
    assert.equal(copied.maxPitch, 45);
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

test("map camera fitting helpers copy camera and bounds values", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 256, height: 256 });
  const bounds = {
    southwest: { latitude: -1, longitude: -2 },
    northeast: { latitude: 1, longitude: 2 },
  };

  try {
    const camera = map.cameraForLatLngBounds(bounds);
    assert.equal(typeof camera.zoom, "number");
    assert.equal(typeof camera.center?.latitude, "number");
    const cameraFromCoordinates = map.cameraForLatLngs([
      bounds.southwest,
      bounds.northeast,
    ]);
    assert.equal(typeof cameraFromCoordinates.zoom, "number");
    const cameraFromGeometry = map.cameraForGeometry({
      type: "LineString",
      coordinates: [
        [-2, -1],
        [2, 1],
      ],
    });
    assert.equal(typeof cameraFromGeometry.zoom, "number");
    const visibleBounds = map.latLngBoundsForCamera(camera);
    const unwrappedBounds = map.latLngBoundsForCameraUnwrapped(camera);
    assert.equal(typeof visibleBounds.southwest.latitude, "number");
    assert.equal(typeof unwrappedBounds.northeast.longitude, "number");
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
    map.easeTo(
      { center: { latitude: 13, longitude: 35 }, zoom: 4 },
      { durationMs: 0, easing: { x1: 0, y1: 0, x2: 1, y2: 1 } },
    );
    map.flyTo({ center: { latitude: 14, longitude: 36 }, zoom: 5 }, null);
    map.setFreeCameraOptions({ orientation: { x: 0, y: 0, z: 0, w: 1 } });
    assert.equal(typeof map.getFreeCameraOptions().orientation?.w, "number");
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
    map.moveByAnimated(1, 2, { durationMs: 0 });
    map.scaleByAnimated(1.1, { x: 10, y: 10 }, { durationMs: 0 });
    map.rotateByAnimated({ x: 10, y: 10 }, { x: 12, y: 12 }, { durationMs: 0 });
    map.pitchByAnimated(1, { durationMs: 0 });
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
    projection.setVisibleCoordinates(
      [
        { latitude: -1, longitude: -2 },
        { latitude: 1, longitude: 2 },
      ],
      { top: 0, left: 0, bottom: 0, right: 0 },
    );
    projection.setVisibleGeometry(
      {
        type: "LineString",
        coordinates: [
          [-2, -1],
          [2, 1],
        ],
      },
      { top: 0, left: 0, bottom: 0, right: 0 },
    );
    assert.equal(typeof projection.getCamera().zoom, "number");
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

    map.setStyleImage("red-pixel", {
      width: 1,
      height: 1,
      pixels: new Uint8Array([255, 0, 0, 255]),
      pixelRatio: 2,
      sdf: true,
    });
    assert.equal(map.styleImageExists("red-pixel"), true);
    assert.deepEqual(map.getStyleImageInfo("red-pixel"), {
      width: 1,
      height: 1,
      stride: 4,
      byteLength: 4,
      pixelRatio: 2,
      sdf: true,
    });
    const copiedImage = map.copyStyleImagePremultipliedRgba8("red-pixel");
    assert.ok(copiedImage);
    assert.deepEqual([...copiedImage.pixels], [255, 0, 0, 255]);
    assert.equal(map.copyStyleImagePremultipliedRgba8("missing-image"), null);
    assert.equal(map.removeStyleImage("red-pixel"), true);
    assert.equal(map.styleImageExists("red-pixel"), false);

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
    const geojsonData = {
      type: "FeatureCollection",
      features: [
        {
          type: "Feature",
          id: "one",
          properties: { name: "point" },
          geometry: { type: "Point", coordinates: [2, 1] },
        },
      ],
    };
    map.addGeoJsonSourceUrl("geojson-url", "https://example.test/data.geojson");
    assert.equal(map.getStyleSourceType("geojson-url"), "geojson");
    map.setGeoJsonSourceUrl(
      "geojson-url",
      "https://example.test/updated.geojson",
    );
    map.setGeoJsonSourceData("geojson-url", geojsonData);
    map.addGeoJsonSourceData("geojson-data", geojsonData);
    assert.equal(map.getStyleSourceType("geojson-data"), "geojson");
    map.addVectorSourceUrl("vector-url", "https://example.test/vector.json");
    assert.equal(map.getStyleSourceType("vector-url"), "vector");
    map.addRasterSourceUrl("raster-url", "https://example.test/raster.json");
    assert.equal(map.getStyleSourceType("raster-url"), "raster");
    map.addRasterDemSourceUrl(
      "raster-dem-url",
      "https://example.test/dem.json",
    );
    assert.equal(map.getStyleSourceType("raster-dem-url"), "raster-dem");
    map.addVectorSourceTiles("vector-tiles", [
      "https://example.test/vector/{z}/{x}/{y}.pbf",
    ]);
    assert.equal(map.getStyleSourceType("vector-tiles"), "vector");
    map.addRasterSourceTiles("raster-tiles", [
      "https://example.test/raster/{z}/{x}/{y}.png",
    ]);
    assert.equal(map.getStyleSourceType("raster-tiles"), "raster");
    map.addRasterDemSourceTiles("raster-dem-tiles", [
      "https://example.test/dem/{z}/{x}/{y}.png",
    ]);
    assert.equal(map.getStyleSourceType("raster-dem-tiles"), "raster-dem");
    map.addCustomGeometrySource("custom-geometry", {
      fetchTile() {},
      cancelTile() {},
      minZoom: 0,
      maxZoom: 14,
      tolerance: 0.375,
      tileSize: 512,
      buffer: 128,
      clip: true,
      wrap: true,
    });
    assert.equal(map.styleSourceExists("custom-geometry"), true);
    assert.throws(
      () =>
        map.addCustomGeometrySource("custom-geometry-invalid", {
          fetchTile: /** @type {any} */ ("nope"),
        }),
      InvalidArgumentError,
    );
    map.setCustomGeometrySourceTileData(
      "custom-geometry",
      { z: 0, x: 0, y: 0 },
      geojsonData,
    );
    map.invalidateCustomGeometrySourceTile("custom-geometry", {
      z: 0,
      x: 0,
      y: 0,
    });
    map.invalidateCustomGeometrySourceRegion("custom-geometry", {
      southwest: { latitude: -1, longitude: -2 },
      northeast: { latitude: 1, longitude: 2 },
    });
    assert.equal(map.removeStyleSource("custom-geometry"), true);
    map.addHillshadeLayer("hillshade", "raster-dem-url");
    map.addColorReliefLayer("color-relief", "raster-dem-tiles", "hillshade");
    assert.equal(map.getStyleLayerType("hillshade"), "hillshade");
    assert.equal(map.getStyleLayerType("color-relief"), "color-relief");
    assert.equal(map.removeStyleLayer("color-relief"), true);
    assert.equal(map.removeStyleLayer("hillshade"), true);

    const inlineImage = {
      width: 1,
      height: 1,
      pixels: new Uint8Array([0, 255, 0, 255]),
    };
    map.addImageSourceUrl(
      "image-source",
      imageCoordinates,
      "https://example.test/image.png",
    );
    assert.equal(map.getStyleSourceType("image-source"), "image");
    map.setImageSourceUrl("image-source", "https://example.test/updated.png");
    map.setImageSourceImage("image-source", inlineImage);
    map.setImageSourceCoordinates("image-source", imageCoordinates);
    assert.deepEqual(
      map.getImageSourceCoordinates("image-source"),
      imageCoordinates,
    );
    map.addImageSourceImage(
      "inline-image-source",
      imageCoordinates,
      inlineImage,
    );
    assert.equal(map.getStyleSourceType("inline-image-source"), "image");
    assert.equal(map.getImageSourceCoordinates("missing-source"), null);
    assert.equal(map.removeStyleSource("inline-image-source"), true);
    assert.equal(map.removeStyleSource("image-source"), true);
    assert.equal(map.removeStyleSource("raster-dem-tiles"), true);
    assert.equal(map.removeStyleSource("raster-tiles"), true);
    assert.equal(map.removeStyleSource("vector-tiles"), true);
    assert.equal(map.removeStyleSource("raster-dem-url"), true);
    assert.equal(map.removeStyleSource("raster-url"), true);
    assert.equal(map.removeStyleSource("vector-url"), true);
    assert.equal(map.removeStyleSource("geojson-data"), true);
    assert.equal(map.removeStyleSource("geojson-url"), true);
    assert.equal(map.removeStyleSource("empty-geojson"), true);

    map.addLocationIndicatorLayer("location");
    map.setLocationIndicatorLocation(
      "location",
      { latitude: 1, longitude: 2 },
      3,
    );
    map.setLocationIndicatorBearing("location", 45);
    map.setLocationIndicatorAccuracyRadius("location", 12);
    map.setLocationIndicatorImageName("location", "top", "red-pixel");
    assert.equal(map.getStyleLayerType("location"), "location-indicator");
    assert.equal(map.removeStyleLayer("location"), true);
    assert.throws(
      () =>
        map.setLocationIndicatorImageName(
          "location",
          /** @type {any} */ ("halo"),
          "x",
        ),
      InvalidArgumentError,
    );

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

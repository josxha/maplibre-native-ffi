const assert = require("node:assert/strict");
const test = require("node:test");

const {
  cVersion,
  InvalidArgumentError,
  InvalidStateError,
  MaplibreError,
  MapHandle,
  latLngForProjectedMeters,
  MaplibreStatus,
  NativeBuffer,
  NativePointer,
  networkStatus,
  projectedMetersForLatLng,
  restoreDefaultAsyncLogSeverities,
  RuntimeHandle,
  setAsyncLogSeverities,
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

test("map screen projection helpers copy point values", () => {
  const runtime = new RuntimeHandle();
  const map = runtime.createMap({ width: 256, height: 256 });
  const coordinate = { latitude: 10, longitude: 20 };

  try {
    map.jumpTo({ center: coordinate, zoom: 2 });
    const point = map.pixelForLatLng(coordinate);
    const roundTripped = map.latLngForPixel(point);
    assert.equal(typeof point.x, "number");
    assert.equal(typeof point.y, "number");
    assert.ok(Math.abs(roundTripped.latitude - coordinate.latitude) < 1e-9);
    assert.ok(Math.abs(roundTripped.longitude - coordinate.longitude) < 1e-9);
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

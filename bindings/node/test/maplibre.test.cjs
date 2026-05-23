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

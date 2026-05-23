const assert = require("node:assert/strict");
const test = require("node:test");

const {
  cVersion,
  InvalidArgumentError,
  MaplibreError,
  MaplibreStatus,
  networkStatus,
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

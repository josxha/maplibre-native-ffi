"use strict";

const native = require("./index.js");

const MaplibreStatus = Object.freeze({
  invalidArgument: "invalid-argument",
  invalidState: "invalid-state",
  wrongThread: "wrong-thread",
  unsupported: "unsupported",
  nativeError: "native-error",
  abiVersionMismatch: "abi-version-mismatch",
  unknownStatus: "unknown-status",
});

const NATIVE_ERROR_PREFIX = "MaplibreNativeError:";

class MaplibreError extends Error {
  constructor(status, nativeStatusCode, diagnostic, options) {
    const detail =
      diagnostic && diagnostic.trim()
        ? diagnostic
        : "No native diagnostic available.";
    super(
      `${status}${nativeStatusCode == null ? "" : ` (${nativeStatusCode})`}: ${detail}`,
      options,
    );
    this.name = "MaplibreError";
    this.status = status;
    this.nativeStatusCode = nativeStatusCode ?? null;
    this.diagnostic = diagnostic ?? "";
  }
}

class InvalidArgumentError extends MaplibreError {
  constructor(nativeStatusCode, diagnostic, options) {
    super(
      MaplibreStatus.invalidArgument,
      nativeStatusCode,
      diagnostic,
      options,
    );
    this.name = "InvalidArgumentError";
  }
}

class InvalidStateError extends MaplibreError {
  constructor(nativeStatusCode, diagnostic, options) {
    super(MaplibreStatus.invalidState, nativeStatusCode, diagnostic, options);
    this.name = "InvalidStateError";
  }
}

class WrongThreadError extends MaplibreError {
  constructor(nativeStatusCode, diagnostic, options) {
    super(MaplibreStatus.wrongThread, nativeStatusCode, diagnostic, options);
    this.name = "WrongThreadError";
  }
}

class UnsupportedFeatureError extends MaplibreError {
  constructor(nativeStatusCode, diagnostic, options) {
    super(MaplibreStatus.unsupported, nativeStatusCode, diagnostic, options);
    this.name = "UnsupportedFeatureError";
  }
}

class NativeError extends MaplibreError {
  constructor(nativeStatusCode, diagnostic, options) {
    super(MaplibreStatus.nativeError, nativeStatusCode, diagnostic, options);
    this.name = "NativeError";
  }
}

function cVersion() {
  return native.cVersion();
}

function supportedRenderBackends() {
  return native.supportedRenderBackends();
}

function networkStatus() {
  return translateNativeErrors(() => native.networkStatus());
}

function setNetworkStatus(status) {
  if (status !== "online" && status !== "offline") {
    throw new InvalidArgumentError(
      null,
      `network status must be 'online' or 'offline', got '${status}'`,
    );
  }
  return translateNativeErrors(() => native.setNetworkStatus(status));
}

function projectedMetersForLatLng(coordinate) {
  return translateNativeErrors(() =>
    native.nativeProjectedMetersForLatLng(coordinate),
  );
}

function latLngForProjectedMeters(meters) {
  return translateNativeErrors(() =>
    native.nativeLatLngForProjectedMeters(meters),
  );
}

function setAsyncLogSeverities(severities) {
  let mask = 0;
  for (const severity of severities) {
    mask |= translateNativeErrors(() =>
      native.nativeLogSeverityMaskBit(severity),
    );
  }
  return translateNativeErrors(() =>
    native.nativeSetAsyncLogSeverityMask(mask),
  );
}

function restoreDefaultAsyncLogSeverities() {
  return translateNativeErrors(() =>
    native.nativeSetAsyncLogSeverityMask(
      native.nativeDefaultAsyncLogSeverityMask(),
    ),
  );
}

class NativePointer {
  static null = new NativePointer(0n);

  static unsafeFromAddress(address) {
    return new NativePointer(address);
  }

  constructor(address) {
    if (typeof address !== "bigint") {
      throw new InvalidArgumentError(
        null,
        "native pointer address must be a bigint",
      );
    }
    if (address < 0n) {
      throw new InvalidArgumentError(
        null,
        "native pointer address must be non-negative",
      );
    }
    this.address = address;
    Object.freeze(this);
  }

  get isNull() {
    return this.address === 0n;
  }

  equals(other) {
    return other instanceof NativePointer && this.address === other.address;
  }

  toString() {
    return `NativePointer[address=0x${this.address.toString(16)}]`;
  }
}

class RuntimeHandle {
  constructor(options) {
    this.native = translateNativeErrors(() =>
      native.createNativeRuntimeHandle(options ?? {}),
    );
  }

  createMap(options) {
    return new MapHandle(this, options);
  }

  close() {
    return translateNativeErrors(() => this.native.close());
  }

  get closed() {
    return this.native.closed;
  }

  runOnce() {
    return translateNativeErrors(() => this.native.runOnce());
  }

  pollEvent() {
    return translateNativeErrors(() => this.native.pollEvent());
  }

  [Symbol.dispose]() {
    this.close();
  }
}

class MapHandle {
  constructor(runtime, options) {
    if (!(runtime instanceof RuntimeHandle)) {
      throw new InvalidArgumentError(null, "runtime must be a RuntimeHandle");
    }
    this.runtime = runtime;
    this.native = translateNativeErrors(() =>
      native.createNativeMapHandle(runtime.native, options ?? {}),
    );
  }

  close() {
    return translateNativeErrors(() => this.native.close());
  }

  get closed() {
    return this.native.closed;
  }

  requestRepaint() {
    return translateNativeErrors(() => this.native.requestRepaint());
  }

  requestStillImage() {
    return translateNativeErrors(() => this.native.requestStillImage());
  }

  isFullyLoaded() {
    return translateNativeErrors(() => this.native.isFullyLoaded());
  }

  dumpDebugLogs() {
    return translateNativeErrors(() => this.native.dumpDebugLogs());
  }

  get renderingStatsViewEnabled() {
    return translateNativeErrors(() => this.native.renderingStatsViewEnabled);
  }

  set renderingStatsViewEnabled(enabled) {
    translateNativeErrors(() => {
      this.native.renderingStatsViewEnabled = enabled;
    });
  }

  setStyleJson(json) {
    return translateNativeErrors(() => this.native.setStyleJson(json));
  }

  setStyleUrl(url) {
    return translateNativeErrors(() => this.native.setStyleUrl(url));
  }

  [Symbol.dispose]() {
    this.close();
  }
}

function translateNativeErrors(callback) {
  try {
    return callback();
  } catch (error) {
    throw mapNativeError(error);
  }
}

function mapNativeError(error) {
  if (!(error instanceof Error)) {
    return error;
  }

  const payload = parseNativePayload(error.message);
  if (!payload) {
    return error;
  }

  const options = { cause: error };
  switch (payload.kind) {
    case "InvalidArgument":
      return new InvalidArgumentError(
        payload.nativeStatusCode,
        payload.diagnostic,
        options,
      );
    case "InvalidState":
      return new InvalidStateError(
        payload.nativeStatusCode,
        payload.diagnostic,
        options,
      );
    case "WrongThread":
      return new WrongThreadError(
        payload.nativeStatusCode,
        payload.diagnostic,
        options,
      );
    case "Unsupported":
      return new UnsupportedFeatureError(
        payload.nativeStatusCode,
        payload.diagnostic,
        options,
      );
    case "NativeError":
      return new NativeError(
        payload.nativeStatusCode,
        payload.diagnostic,
        options,
      );
    case "AbiVersionMismatch":
      return new MaplibreError(
        MaplibreStatus.abiVersionMismatch,
        payload.nativeStatusCode,
        payload.diagnostic,
        options,
      );
    default:
      return new MaplibreError(
        MaplibreStatus.unknownStatus,
        payload.nativeStatusCode,
        payload.diagnostic,
        options,
      );
  }
}

function parseNativePayload(message) {
  if (typeof message !== "string" || !message.startsWith(NATIVE_ERROR_PREFIX)) {
    return null;
  }

  try {
    const payload = JSON.parse(message.slice(NATIVE_ERROR_PREFIX.length));
    if (
      typeof payload.kind !== "string" ||
      typeof payload.diagnostic !== "string"
    ) {
      return null;
    }
    return {
      kind: payload.kind,
      nativeStatusCode:
        typeof payload.nativeStatusCode === "number"
          ? payload.nativeStatusCode
          : null,
      diagnostic: payload.diagnostic,
    };
  } catch {
    return null;
  }
}

module.exports = {
  MaplibreError,
  InvalidArgumentError,
  InvalidStateError,
  WrongThreadError,
  UnsupportedFeatureError,
  NativeError,
  MaplibreStatus,
  RuntimeHandle,
  MapHandle,
  NativePointer,
  cVersion,
  supportedRenderBackends,
  networkStatus,
  setNetworkStatus,
  projectedMetersForLatLng,
  latLngForProjectedMeters,
  setAsyncLogSeverities,
  restoreDefaultAsyncLogSeverities,
};

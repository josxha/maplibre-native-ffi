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

function threadLastErrorMessage() {
  return native.threadLastErrorMessage();
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

function setLogCallback(callback) {
  if (typeof callback !== "function") {
    throw new InvalidArgumentError(null, "log callback must be a function");
  }
  return translateNativeErrors(() =>
    native.nativeSetLogCallback((error, record) => {
      if (error) {
        throw error;
      }
      callback(record);
    }),
  );
}

function clearLogCallback() {
  return translateNativeErrors(() => native.nativeClearLogCallback());
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

const MAP_DEBUG_OPTIONS = Object.freeze([
  "tileBorders",
  "parseStatus",
  "timestamps",
  "collision",
  "overdraw",
  "stencilClip",
  "depthBuffer",
]);

function mapDebugOptionMaskBit(option) {
  return translateNativeErrors(() =>
    native.nativeMapDebugOptionMaskBit(option),
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

class NativeBuffer {
  static allocate(byteLength) {
    return new NativeBuffer(new ArrayBuffer(validateByteLength(byteLength)));
  }

  static from(data) {
    if (data instanceof NativeBuffer) {
      return new NativeBuffer(data.asUint8Array());
    }
    if (data instanceof ArrayBuffer) {
      return new NativeBuffer(data.slice(0));
    }
    if (ArrayBuffer.isView(data)) {
      return new NativeBuffer(
        new Uint8Array(data.buffer, data.byteOffset, data.byteLength),
      );
    }
    throw new InvalidArgumentError(
      null,
      "native buffer data must be an ArrayBuffer or typed array view",
    );
  }

  constructor(data) {
    if (typeof data === "number") {
      this.buffer = new ArrayBuffer(validateByteLength(data));
    } else if (data instanceof ArrayBuffer) {
      this.buffer = data;
    } else if (ArrayBuffer.isView(data)) {
      this.buffer = data.buffer.slice(
        data.byteOffset,
        data.byteOffset + data.byteLength,
      );
    } else {
      throw new InvalidArgumentError(
        null,
        "native buffer constructor requires a byte length, ArrayBuffer, or typed array view",
      );
    }
  }

  get byteLength() {
    return this.buffer.byteLength;
  }

  asArrayBuffer() {
    return this.buffer;
  }

  asUint8Array() {
    return new Uint8Array(this.buffer);
  }

  get [Symbol.toStringTag]() {
    return "NativeBuffer";
  }
}

function validateByteLength(byteLength) {
  if (!Number.isSafeInteger(byteLength) || byteLength < 0) {
    throw new InvalidArgumentError(
      null,
      "native buffer byteLength must be a non-negative safe integer",
    );
  }
  return byteLength;
}

class ResourceRequestHandle {
  constructor(handleId) {
    this.handleId = handleId;
    this.closed = false;
  }

  complete(response = {}) {
    if (this.closed) {
      throw new InvalidStateError(null, "ResourceRequestHandle is closed");
    }
    translateNativeErrors(() =>
      native.nativeResourceRequestComplete(this.handleId, response),
    );
    this.closed = true;
  }

  cancelled() {
    return translateNativeErrors(() =>
      native.nativeResourceRequestCancelled(this.handleId),
    );
  }

  close() {
    if (this.closed) {
      return;
    }
    translateNativeErrors(() =>
      native.nativeResourceRequestClose(this.handleId),
    );
    this.closed = true;
  }

  [Symbol.dispose]() {
    this.close();
  }
}

class OfflineOperationHandle {
  constructor(runtime, operationId) {
    if (!(runtime instanceof RuntimeHandle)) {
      throw new InvalidArgumentError(null, "runtime must be a RuntimeHandle");
    }
    if (typeof operationId !== "bigint" || operationId <= 0n) {
      throw new InvalidArgumentError(
        null,
        "offline operation id must be a positive bigint",
      );
    }
    this.runtime = runtime;
    this.operationId = operationId;
    this.discarded = false;
  }

  close() {
    if (this.discarded) {
      return;
    }
    translateNativeErrors(() =>
      this.runtime.native.discardOfflineOperation(this.operationId),
    );
    this.discarded = true;
  }

  get closed() {
    return this.discarded;
  }

  [Symbol.dispose]() {
    this.close();
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

  setResourceTransform(callback) {
    if (typeof callback !== "function") {
      throw new InvalidArgumentError(
        null,
        "resource transform callback must be a function",
      );
    }
    return translateNativeErrors(() =>
      this.native.setResourceTransform(callback),
    );
  }

  setResourceProvider(callback) {
    if (typeof callback !== "function") {
      throw new InvalidArgumentError(
        null,
        "resource provider callback must be a function",
      );
    }
    return translateNativeErrors(() =>
      this.native.setResourceProvider((request) => {
        const wrapped = {
          ...request,
          handle: new ResourceRequestHandle(request.handleId),
        };
        delete wrapped.handleId;
        return callback(wrapped);
      }),
    );
  }

  clearResourceTransform() {
    return translateNativeErrors(() => this.native.clearResourceTransform());
  }

  runAmbientCacheOperation(operation) {
    const start = translateNativeErrors(() =>
      this.native.runAmbientCacheOperation(operation),
    );
    return new OfflineOperationHandle(this, BigInt(start.operationId));
  }

  pollEvent() {
    return translateNativeErrors(() => this.native.pollEvent());
  }

  [Symbol.dispose]() {
    this.close();
  }
}

class MapProjectionHandle {
  constructor(map) {
    if (!(map instanceof MapHandle)) {
      throw new InvalidArgumentError(null, "map must be a MapHandle");
    }
    this.native = translateNativeErrors(() =>
      native.createNativeMapProjectionHandle(map.native),
    );
  }

  close() {
    return translateNativeErrors(() => this.native.close());
  }

  get closed() {
    return this.native.closed;
  }

  getCamera() {
    return translateNativeErrors(() => this.native.getCamera());
  }

  setCamera(camera) {
    return translateNativeErrors(() => this.native.setCamera(camera));
  }

  setVisibleCoordinates(coordinates, padding) {
    return translateNativeErrors(() =>
      this.native.setVisibleCoordinates(coordinates, padding),
    );
  }

  setVisibleGeometry(geometry, padding) {
    return translateNativeErrors(() =>
      this.native.setVisibleGeometry(stringifyJson(geometry), padding),
    );
  }

  pixelForLatLng(coordinate) {
    return translateNativeErrors(() => this.native.pixelForLatLng(coordinate));
  }

  latLngForPixel(point) {
    return translateNativeErrors(() => this.native.latLngForPixel(point));
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

  createProjection() {
    return new MapProjectionHandle(this);
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

  getDebugOptions() {
    const mask = translateNativeErrors(() => this.native.getDebugOptionsRaw());
    return MAP_DEBUG_OPTIONS.filter((option) =>
      Boolean(mask & mapDebugOptionMaskBit(option)),
    );
  }

  setDebugOptions(options) {
    let mask = 0;
    for (const option of options) {
      mask |= mapDebugOptionMaskBit(option);
    }
    return translateNativeErrors(() => this.native.setDebugOptionsRaw(mask));
  }

  moveBy(deltaX, deltaY) {
    return translateNativeErrors(() => this.native.moveBy(deltaX, deltaY));
  }

  scaleBy(scale, anchor = null) {
    return translateNativeErrors(() => this.native.scaleBy(scale, anchor));
  }

  rotateBy(first, second) {
    return translateNativeErrors(() => this.native.rotateBy(first, second));
  }

  pitchBy(pitch) {
    return translateNativeErrors(() => this.native.pitchBy(pitch));
  }

  moveByAnimated(deltaX, deltaY, animation = null) {
    return translateNativeErrors(() =>
      this.native.moveByAnimated(deltaX, deltaY, animation),
    );
  }

  scaleByAnimated(scale, anchor = null, animation = null) {
    return translateNativeErrors(() =>
      this.native.scaleByAnimated(scale, anchor, animation),
    );
  }

  rotateByAnimated(first, second, animation = null) {
    return translateNativeErrors(() =>
      this.native.rotateByAnimated(first, second, animation),
    );
  }

  pitchByAnimated(pitch, animation = null) {
    return translateNativeErrors(() =>
      this.native.pitchByAnimated(pitch, animation),
    );
  }

  cancelTransitions() {
    return translateNativeErrors(() => this.native.cancelTransitions());
  }

  getViewportOptions() {
    return translateNativeErrors(() => this.native.getViewportOptions());
  }

  setViewportOptions(options) {
    return translateNativeErrors(() => this.native.setViewportOptions(options));
  }

  getTileOptions() {
    return translateNativeErrors(() => this.native.getTileOptions());
  }

  setTileOptions(options) {
    return translateNativeErrors(() => this.native.setTileOptions(options));
  }

  getBounds() {
    return translateNativeErrors(() => this.native.getBounds());
  }

  setBounds(options) {
    return translateNativeErrors(() => this.native.setBounds(options));
  }

  getFreeCameraOptions() {
    return translateNativeErrors(() => this.native.getFreeCameraOptions());
  }

  setFreeCameraOptions(options) {
    return translateNativeErrors(() =>
      this.native.setFreeCameraOptions(options),
    );
  }

  getProjectionMode() {
    return translateNativeErrors(() => this.native.getProjectionMode());
  }

  setProjectionMode(mode) {
    return translateNativeErrors(() => this.native.setProjectionMode(mode));
  }

  getCamera() {
    return translateNativeErrors(() => this.native.getCamera());
  }

  jumpTo(camera) {
    return translateNativeErrors(() => this.native.jumpTo(camera));
  }

  easeTo(camera, animation = null) {
    return translateNativeErrors(() => this.native.easeTo(camera, animation));
  }

  flyTo(camera, animation = null) {
    return translateNativeErrors(() => this.native.flyTo(camera, animation));
  }

  cameraForLatLngBounds(bounds) {
    return translateNativeErrors(() =>
      this.native.cameraForLatLngBounds(bounds),
    );
  }

  cameraForLatLngs(coordinates) {
    return translateNativeErrors(() =>
      this.native.cameraForLatLngs(coordinates),
    );
  }

  cameraForGeometry(geometry) {
    return translateNativeErrors(() =>
      this.native.cameraForGeometry(stringifyJson(geometry)),
    );
  }

  latLngBoundsForCamera(camera) {
    return translateNativeErrors(() =>
      this.native.latLngBoundsForCamera(camera),
    );
  }

  latLngBoundsForCameraUnwrapped(camera) {
    return translateNativeErrors(() =>
      this.native.latLngBoundsForCameraUnwrapped(camera),
    );
  }

  pixelForLatLng(coordinate) {
    return translateNativeErrors(() => this.native.pixelForLatLng(coordinate));
  }

  latLngForPixel(point) {
    return translateNativeErrors(() => this.native.latLngForPixel(point));
  }

  pixelsForLatLngs(coordinates) {
    return translateNativeErrors(() =>
      this.native.pixelsForLatLngs(coordinates),
    );
  }

  latLngsForPixels(points) {
    return translateNativeErrors(() => this.native.latLngsForPixels(points));
  }

  get renderingStatsViewEnabled() {
    return translateNativeErrors(() => this.native.renderingStatsViewEnabled);
  }

  set renderingStatsViewEnabled(enabled) {
    translateNativeErrors(() => {
      this.native.renderingStatsViewEnabled = enabled;
    });
  }

  addStyleSourceJson(sourceId, source) {
    return translateNativeErrors(() =>
      this.native.addStyleSourceJson(sourceId, stringifyJson(source)),
    );
  }

  styleSourceExists(sourceId) {
    return translateNativeErrors(() => this.native.styleSourceExists(sourceId));
  }

  removeStyleSource(sourceId) {
    return translateNativeErrors(() => this.native.removeStyleSource(sourceId));
  }

  listStyleSourceIds() {
    return translateNativeErrors(() => this.native.listStyleSourceIds());
  }

  getStyleSourceType(sourceId) {
    return translateNativeErrors(() =>
      this.native.getStyleSourceType(sourceId),
    );
  }

  getStyleSourceInfo(sourceId) {
    return translateNativeErrors(() =>
      this.native.getStyleSourceInfo(sourceId),
    );
  }

  addGeoJsonSourceUrl(sourceId, url) {
    return translateNativeErrors(() =>
      this.native.addGeoJsonSourceUrl(sourceId, url),
    );
  }

  addGeoJsonSourceData(sourceId, data) {
    return translateNativeErrors(() =>
      this.native.addGeoJsonSourceData(sourceId, stringifyJson(data)),
    );
  }

  setGeoJsonSourceUrl(sourceId, url) {
    return translateNativeErrors(() =>
      this.native.setGeoJsonSourceUrl(sourceId, url),
    );
  }

  setGeoJsonSourceData(sourceId, data) {
    return translateNativeErrors(() =>
      this.native.setGeoJsonSourceData(sourceId, stringifyJson(data)),
    );
  }

  addVectorSourceUrl(sourceId, url) {
    return translateNativeErrors(() =>
      this.native.addVectorSourceUrl(sourceId, url),
    );
  }

  addRasterSourceUrl(sourceId, url) {
    return translateNativeErrors(() =>
      this.native.addRasterSourceUrl(sourceId, url),
    );
  }

  addRasterDemSourceUrl(sourceId, url) {
    return translateNativeErrors(() =>
      this.native.addRasterDemSourceUrl(sourceId, url),
    );
  }

  addVectorSourceTiles(sourceId, tiles) {
    return translateNativeErrors(() =>
      this.native.addVectorSourceTiles(sourceId, Array.from(tiles)),
    );
  }

  addRasterSourceTiles(sourceId, tiles) {
    return translateNativeErrors(() =>
      this.native.addRasterSourceTiles(sourceId, Array.from(tiles)),
    );
  }

  addRasterDemSourceTiles(sourceId, tiles) {
    return translateNativeErrors(() =>
      this.native.addRasterDemSourceTiles(sourceId, Array.from(tiles)),
    );
  }

  addCustomGeometrySource(sourceId, options = null) {
    return translateNativeErrors(() =>
      this.native.addCustomGeometrySource(sourceId, options),
    );
  }

  setCustomGeometrySourceTileData(sourceId, tileId, data) {
    return translateNativeErrors(() =>
      this.native.setCustomGeometrySourceTileData(
        sourceId,
        tileId,
        stringifyJson(data),
      ),
    );
  }

  invalidateCustomGeometrySourceTile(sourceId, tileId) {
    return translateNativeErrors(() =>
      this.native.invalidateCustomGeometrySourceTile(sourceId, tileId),
    );
  }

  invalidateCustomGeometrySourceRegion(sourceId, bounds) {
    return translateNativeErrors(() =>
      this.native.invalidateCustomGeometrySourceRegion(sourceId, bounds),
    );
  }

  setStyleImage(imageId, image) {
    return translateNativeErrors(() =>
      this.native.setStyleImage(imageId, image),
    );
  }

  styleImageExists(imageId) {
    return translateNativeErrors(() => this.native.styleImageExists(imageId));
  }

  removeStyleImage(imageId) {
    return translateNativeErrors(() => this.native.removeStyleImage(imageId));
  }

  getStyleImageInfo(imageId) {
    return translateNativeErrors(() => this.native.getStyleImageInfo(imageId));
  }

  copyStyleImagePremultipliedRgba8(imageId) {
    return translateNativeErrors(() =>
      this.native.copyStyleImagePremultipliedRgba8(imageId),
    );
  }

  addImageSourceUrl(sourceId, coordinates, url) {
    return translateNativeErrors(() =>
      this.native.addImageSourceUrl(sourceId, coordinates, url),
    );
  }

  addImageSourceImage(sourceId, coordinates, image) {
    return translateNativeErrors(() =>
      this.native.addImageSourceImage(sourceId, coordinates, image),
    );
  }

  setImageSourceUrl(sourceId, url) {
    return translateNativeErrors(() =>
      this.native.setImageSourceUrl(sourceId, url),
    );
  }

  setImageSourceImage(sourceId, image) {
    return translateNativeErrors(() =>
      this.native.setImageSourceImage(sourceId, image),
    );
  }

  setImageSourceCoordinates(sourceId, coordinates) {
    return translateNativeErrors(() =>
      this.native.setImageSourceCoordinates(sourceId, coordinates),
    );
  }

  getImageSourceCoordinates(sourceId) {
    return translateNativeErrors(() =>
      this.native.getImageSourceCoordinates(sourceId),
    );
  }

  addHillshadeLayer(layerId, sourceId, beforeLayerId = null) {
    return translateNativeErrors(() =>
      this.native.addHillshadeLayer(layerId, sourceId, beforeLayerId),
    );
  }

  addColorReliefLayer(layerId, sourceId, beforeLayerId = null) {
    return translateNativeErrors(() =>
      this.native.addColorReliefLayer(layerId, sourceId, beforeLayerId),
    );
  }

  addLocationIndicatorLayer(layerId, beforeLayerId = null) {
    return translateNativeErrors(() =>
      this.native.addLocationIndicatorLayer(layerId, beforeLayerId),
    );
  }

  setLocationIndicatorLocation(layerId, coordinate, altitude = 0) {
    return translateNativeErrors(() =>
      this.native.setLocationIndicatorLocation(layerId, coordinate, altitude),
    );
  }

  setLocationIndicatorBearing(layerId, bearing) {
    return translateNativeErrors(() =>
      this.native.setLocationIndicatorBearing(layerId, bearing),
    );
  }

  setLocationIndicatorAccuracyRadius(layerId, radius) {
    return translateNativeErrors(() =>
      this.native.setLocationIndicatorAccuracyRadius(layerId, radius),
    );
  }

  setLocationIndicatorImageName(layerId, imageKind, imageId) {
    return translateNativeErrors(() =>
      this.native.setLocationIndicatorImageName(layerId, imageKind, imageId),
    );
  }

  addStyleLayerJson(layer, beforeLayerId = null) {
    return translateNativeErrors(() =>
      this.native.addStyleLayerJson(stringifyJson(layer), beforeLayerId),
    );
  }

  styleLayerExists(layerId) {
    return translateNativeErrors(() => this.native.styleLayerExists(layerId));
  }

  removeStyleLayer(layerId) {
    return translateNativeErrors(() => this.native.removeStyleLayer(layerId));
  }

  listStyleLayerIds() {
    return translateNativeErrors(() => this.native.listStyleLayerIds());
  }

  getStyleLayerType(layerId) {
    return translateNativeErrors(() => this.native.getStyleLayerType(layerId));
  }

  getStyleLayerJson(layerId) {
    const json = translateNativeErrors(() =>
      this.native.getStyleLayerJson(layerId),
    );
    return json === null ? null : JSON.parse(json);
  }

  moveStyleLayer(layerId, beforeLayerId = null) {
    return translateNativeErrors(() =>
      this.native.moveStyleLayer(layerId, beforeLayerId),
    );
  }

  setLayerProperty(layerId, propertyName, value) {
    return translateNativeErrors(() =>
      this.native.setLayerPropertyJson(
        layerId,
        propertyName,
        stringifyJson(value),
      ),
    );
  }

  getLayerProperty(layerId, propertyName) {
    const json = translateNativeErrors(() =>
      this.native.getLayerPropertyJson(layerId, propertyName),
    );
    return json === null ? null : JSON.parse(json);
  }

  setLayerFilter(layerId, filter) {
    return translateNativeErrors(() =>
      this.native.setLayerFilterJson(
        layerId,
        filter === null ? null : stringifyJson(filter),
      ),
    );
  }

  getLayerFilter(layerId) {
    const json = translateNativeErrors(() =>
      this.native.getLayerFilterJson(layerId),
    );
    return json === null ? null : JSON.parse(json);
  }

  setStyleLight(light) {
    return translateNativeErrors(() =>
      this.native.setStyleLightJson(stringifyJson(light)),
    );
  }

  setStyleLightProperty(propertyName, value) {
    return translateNativeErrors(() =>
      this.native.setStyleLightPropertyJson(propertyName, stringifyJson(value)),
    );
  }

  getStyleLightProperty(propertyName) {
    const json = translateNativeErrors(() =>
      this.native.getStyleLightPropertyJson(propertyName),
    );
    return json === null ? null : JSON.parse(json);
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

function stringifyJson(value) {
  const json = JSON.stringify(value);
  if (json === undefined) {
    throw new InvalidArgumentError(
      null,
      "JSON value must be serializable as an object, array, string, number, boolean, or null",
    );
  }
  return json;
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
  ResourceRequestHandle,
  OfflineOperationHandle,
  MapHandle,
  MapProjectionHandle,
  NativePointer,
  NativeBuffer,
  cVersion,
  supportedRenderBackends,
  threadLastErrorMessage,
  networkStatus,
  setNetworkStatus,
  projectedMetersForLatLng,
  latLngForProjectedMeters,
  setLogCallback,
  clearLogCallback,
  setAsyncLogSeverities,
  restoreDefaultAsyncLogSeverities,
};

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

function supportedOpenGLContextProviders() {
  return native.supportedOpenGLContextProviders();
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

const CONSTRUCTION_TOKEN = Symbol("constructionToken");

class NativePointer {
  static null = new NativePointer(CONSTRUCTION_TOKEN, 0n);

  static unsafeFromAddress(address) {
    return new NativePointer(CONSTRUCTION_TOKEN, address);
  }

  constructor(token, address, isValid = () => true) {
    if (token !== CONSTRUCTION_TOKEN) {
      throw new InvalidArgumentError(
        null,
        "use NativePointer.unsafeFromAddress() to construct native pointers",
      );
    }
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
    Object.defineProperties(this, {
      _address: { value: address },
      _isValid: { value: isValid },
    });
    Object.freeze(this);
  }

  get address() {
    this.#assertValid();
    return this._address;
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

  #assertValid() {
    if (!this._isValid()) {
      throw new InvalidStateError(null, "native pointer scope is closed");
    }
  }
}

const HANDLE_ENVIRONMENT = Symbol("handleEnvironment");
const ENVIRONMENT_TOKEN = Object.freeze({});
const TEXTURE_FRAME_RAW = Symbol("textureFrameRaw");
const TEXTURE_FRAME_DEACTIVATE = Symbol("textureFrameDeactivate");
const NATIVE_HANDLES = new WeakMap();

function recordHandleEnvironment(handle) {
  Object.defineProperty(handle, HANDLE_ENVIRONMENT, {
    value: ENVIRONMENT_TOKEN,
  });
}

function assertHandleEnvironment(handle) {
  if (handle?.[HANDLE_ENVIRONMENT] !== ENVIRONMENT_TOKEN) {
    throw new InvalidStateError(
      null,
      "handle belongs to a different N-API environment",
    );
  }
}

function defineCheckedNative(owner, nativeHandle) {
  assertHandleEnvironment(owner);
  NATIVE_HANDLES.set(owner, nativeHandle);
}

function nativeOf(owner) {
  if (NATIVE_HANDLES.has(owner)) {
    assertHandleEnvironment(owner);
    return NATIVE_HANDLES.get(owner);
  }
  if (owner != null && Object.prototype.hasOwnProperty.call(owner, "native")) {
    return owner.native;
  }
  assertHandleEnvironment(owner);
  throw new InvalidStateError(null, "native handle is not initialized");
}

function liveNativeOf(owner) {
  const nativeHandle = nativeOf(owner);
  if (nativeHandle.closed) {
    throw new InvalidStateError(null, "handle is closed");
  }
  return nativeHandle;
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
      return new NativeBuffer(data);
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
      this.buffer = data.slice(0);
    } else if (ArrayBuffer.isView(data)) {
      const copy = new Uint8Array(data.byteLength);
      copy.set(new Uint8Array(data.buffer, data.byteOffset, data.byteLength));
      this.buffer = copy.buffer;
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

class MetalOwnedTextureFrame {
  #active = true;
  #raw;

  constructor(token, raw) {
    if (token !== CONSTRUCTION_TOKEN) {
      throw new InvalidArgumentError(
        null,
        "texture frames are only available inside render-session frame scopes",
      );
    }
    this.#raw = raw;
    Object.freeze(this);
  }

  get generation() {
    return this.#read("generation");
  }

  get width() {
    return this.#read("width");
  }

  get height() {
    return this.#read("height");
  }

  get scaleFactor() {
    return this.#read("scaleFactor");
  }

  get frameId() {
    return this.#read("frameId");
  }

  get texture() {
    return new NativePointer(
      CONSTRUCTION_TOKEN,
      this.#read("textureAddress"),
      () => this.#active,
    );
  }

  get device() {
    return new NativePointer(
      CONSTRUCTION_TOKEN,
      this.#read("deviceAddress"),
      () => this.#active,
    );
  }

  get pixelFormat() {
    return this.#read("pixelFormat");
  }

  #read(field) {
    if (!this.#active) {
      throw new InvalidStateError(null, "texture frame scope is closed");
    }
    return this.#raw[field];
  }

  [TEXTURE_FRAME_RAW]() {
    return this.#raw;
  }

  [TEXTURE_FRAME_DEACTIVATE]() {
    this.#active = false;
  }
}

class OpenGLOwnedTextureFrame {
  #active = true;
  #raw;

  constructor(token, raw) {
    if (token !== CONSTRUCTION_TOKEN) {
      throw new InvalidArgumentError(
        null,
        "texture frames are only available inside render-session frame scopes",
      );
    }
    this.#raw = raw;
    Object.freeze(this);
  }

  get generation() {
    return this.#read("generation");
  }
  get width() {
    return this.#read("width");
  }
  get height() {
    return this.#read("height");
  }
  get scaleFactor() {
    return this.#read("scaleFactor");
  }
  get frameId() {
    return this.#read("frameId");
  }
  get texture() {
    return this.#read("texture");
  }
  get target() {
    return this.#read("target");
  }
  get internalFormat() {
    return this.#read("internalFormat");
  }
  get format() {
    return this.#read("format");
  }
  get type() {
    return this.#read("type");
  }

  #read(field) {
    if (!this.#active) {
      throw new InvalidStateError(null, "texture frame scope is closed");
    }
    return this.#raw[field];
  }

  [TEXTURE_FRAME_RAW]() {
    return this.#raw;
  }
  [TEXTURE_FRAME_DEACTIVATE]() {
    this.#active = false;
  }
}

class VulkanOwnedTextureFrame {
  #active = true;
  #raw;

  constructor(token, raw) {
    if (token !== CONSTRUCTION_TOKEN) {
      throw new InvalidArgumentError(
        null,
        "texture frames are only available inside render-session frame scopes",
      );
    }
    this.#raw = raw;
    Object.freeze(this);
  }

  get generation() {
    return this.#read("generation");
  }

  get width() {
    return this.#read("width");
  }

  get height() {
    return this.#read("height");
  }

  get scaleFactor() {
    return this.#read("scaleFactor");
  }

  get frameId() {
    return this.#read("frameId");
  }

  get image() {
    return new NativePointer(
      CONSTRUCTION_TOKEN,
      this.#read("imageAddress"),
      () => this.#active,
    );
  }

  get imageView() {
    return new NativePointer(
      CONSTRUCTION_TOKEN,
      this.#read("imageViewAddress"),
      () => this.#active,
    );
  }

  get device() {
    return new NativePointer(
      CONSTRUCTION_TOKEN,
      this.#read("deviceAddress"),
      () => this.#active,
    );
  }

  get format() {
    return this.#read("format");
  }

  get layout() {
    return this.#read("layout");
  }

  #read(field) {
    if (!this.#active) {
      throw new InvalidStateError(null, "texture frame scope is closed");
    }
    return this.#raw[field];
  }

  [TEXTURE_FRAME_RAW]() {
    return this.#raw;
  }

  [TEXTURE_FRAME_DEACTIVATE]() {
    this.#active = false;
  }
}

function operationIdOf(operation) {
  if (operation instanceof OfflineOperationHandle) {
    if (operation.closed) {
      throw new InvalidStateError(null, "offline operation handle is closed");
    }
    return operation.operationId;
  }
  if (typeof operation === "bigint") {
    return operation;
  }
  throw new InvalidArgumentError(
    null,
    "offline operation must be an OfflineOperationHandle or bigint",
  );
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

const resourceRequestFinalizer =
  typeof FinalizationRegistry === "function"
    ? new FinalizationRegistry((handleId) => {
        try {
          native.nativeResourceRequestClose(handleId);
        } catch {
          // Finalizers are best-effort cleanup only.
        }
      })
    : null;

function resourceProviderErrorResponse(error) {
  const message =
    error && typeof error.message === "string"
      ? error.message
      : "resource provider callback failed";
  return {
    status: "error",
    errorReason: "other",
    errorMessage: message,
  };
}

function completeResourceRequestWithProviderError(handle, error) {
  if (handle.closed) {
    return;
  }
  try {
    handle.complete(resourceProviderErrorResponse(error));
  } catch {
    try {
      handle.close();
    } catch {
      // The request may have been cancelled or completed already.
    }
  }
}

class ResourceRequestHandle {
  #handleId;
  #closed = false;

  constructor(token, handleId) {
    if (token !== CONSTRUCTION_TOKEN) {
      throw new InvalidArgumentError(
        null,
        "resource request handles are created by resource provider callbacks",
      );
    }
    recordHandleEnvironment(this);
    this.#handleId = handleId;
    resourceRequestFinalizer?.register(this, handleId, this);
    Object.preventExtensions(this);
  }

  get closed() {
    assertHandleEnvironment(this);
    return this.#closed;
  }

  complete(response = {}) {
    assertHandleEnvironment(this);
    if (this.#closed) {
      throw new InvalidStateError(null, "ResourceRequestHandle is closed");
    }
    translateNativeErrors(() =>
      native.nativeResourceRequestComplete(this.#handleId, response),
    );
    this.#closed = true;
    resourceRequestFinalizer?.unregister(this);
  }

  cancelled() {
    assertHandleEnvironment(this);
    return translateNativeErrors(() =>
      native.nativeResourceRequestCancelled(this.#handleId),
    );
  }

  close() {
    assertHandleEnvironment(this);
    if (this.#closed) {
      return;
    }
    translateNativeErrors(() =>
      native.nativeResourceRequestClose(this.#handleId),
    );
    this.#closed = true;
    resourceRequestFinalizer?.unregister(this);
  }

  [Symbol.dispose]() {
    this.close();
  }
}

class OfflineOperationHandle {
  constructor(runtime, operationId) {
    recordHandleEnvironment(this);
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
    assertHandleEnvironment(this);
    if (this.discarded) {
      return;
    }
    translateNativeErrors(() =>
      liveNativeOf(this.runtime).discardOfflineOperation(this.operationId),
    );
    this.discarded = true;
  }

  get closed() {
    assertHandleEnvironment(this);
    return this.discarded;
  }

  [Symbol.dispose]() {
    this.close();
  }
}

class RuntimeHandle {
  constructor(options) {
    recordHandleEnvironment(this);
    defineCheckedNative(
      this,
      translateNativeErrors(() =>
        native.createNativeRuntimeHandle(options ?? {}),
      ),
    );
  }

  createMap(options) {
    return new MapHandle(this, options);
  }

  close() {
    return translateNativeErrors(() => nativeOf(this).close());
  }

  get closed() {
    return nativeOf(this).closed;
  }

  runOnce() {
    return translateNativeErrors(() => liveNativeOf(this).runOnce());
  }

  setResourceTransformRules(rules) {
    if (!Array.isArray(rules)) {
      throw new InvalidArgumentError(
        null,
        "resource transform rules must be an array",
      );
    }
    return translateNativeErrors(() =>
      liveNativeOf(this).setResourceTransformRules(rules),
    );
  }

  setResourceProviderRoutes(routes, callback) {
    if (!Array.isArray(routes)) {
      throw new InvalidArgumentError(
        null,
        "resource provider routes must be an array",
      );
    }
    if (typeof callback !== "function") {
      throw new InvalidArgumentError(
        null,
        "resource provider callback must be a function",
      );
    }
    return translateNativeErrors(() =>
      liveNativeOf(this).setResourceProviderRoutes(routes, (error, request) => {
        if (error) {
          throw error;
        }
        const handle = new ResourceRequestHandle(
          CONSTRUCTION_TOKEN,
          request.handleId,
        );
        const wrapped = {
          ...request,
          handle,
        };
        delete wrapped.handleId;
        try {
          const result = callback(wrapped);
          if (result && typeof result.then === "function") {
            Promise.resolve(result).catch((error) => {
              completeResourceRequestWithProviderError(handle, error);
            });
          }
        } catch (error) {
          completeResourceRequestWithProviderError(handle, error);
        }
      }),
    );
  }

  clearResourceTransform() {
    return translateNativeErrors(() =>
      liveNativeOf(this).clearResourceTransform(),
    );
  }

  runAmbientCacheOperation(operation) {
    const start = translateNativeErrors(() =>
      liveNativeOf(this).runAmbientCacheOperation(operation),
    );
    return new OfflineOperationHandle(this, BigInt(start.operationId));
  }

  offlineRegionsList() {
    return this.#offlineOperation(() =>
      liveNativeOf(this).offlineRegionsList(),
    );
  }

  offlineRegionGet(regionId) {
    return this.#offlineOperation(() =>
      liveNativeOf(this).offlineRegionGet(regionId),
    );
  }

  offlineRegionsMergeDatabase(path) {
    return this.#offlineOperation(() =>
      liveNativeOf(this).offlineRegionsMergeDatabase(path),
    );
  }

  offlineRegionUpdateMetadata(regionId, metadata = null) {
    return this.#offlineOperation(() =>
      liveNativeOf(this).offlineRegionUpdateMetadata(regionId, metadata),
    );
  }

  offlineRegionGetStatus(regionId) {
    return this.#offlineOperation(() =>
      liveNativeOf(this).offlineRegionGetStatus(regionId),
    );
  }

  offlineRegionSetObserved(regionId, observed) {
    return this.#offlineOperation(() =>
      liveNativeOf(this).offlineRegionSetObserved(regionId, observed),
    );
  }

  offlineRegionSetDownloadState(regionId, state) {
    return this.#offlineOperation(() =>
      liveNativeOf(this).offlineRegionSetDownloadState(regionId, state),
    );
  }

  offlineRegionInvalidate(regionId) {
    return this.#offlineOperation(() =>
      liveNativeOf(this).offlineRegionInvalidate(regionId),
    );
  }

  offlineRegionDelete(regionId) {
    return this.#offlineOperation(() =>
      liveNativeOf(this).offlineRegionDelete(regionId),
    );
  }

  offlineRegionCreate(definition, metadata = null) {
    const nativeDefinition = { ...definition };
    if (definition?.geometry == null) {
      delete nativeDefinition.geometry;
    } else {
      nativeDefinition.geometry = stringifyJson(definition.geometry);
    }
    return this.#offlineOperation(() =>
      liveNativeOf(this).offlineRegionCreate(nativeDefinition, metadata),
    );
  }

  offlineRegionCreateTakeResult(operation) {
    return translateNativeErrors(() =>
      liveNativeOf(this).offlineRegionCreateTakeResult(
        operationIdOf(operation),
      ),
    );
  }

  offlineRegionGetTakeResult(operation) {
    return translateNativeErrors(() =>
      liveNativeOf(this).offlineRegionGetTakeResult(operationIdOf(operation)),
    );
  }

  offlineRegionsListTakeResult(operation) {
    return translateNativeErrors(() =>
      liveNativeOf(this).offlineRegionsListTakeResult(operationIdOf(operation)),
    );
  }

  offlineRegionsMergeDatabaseTakeResult(operation) {
    return translateNativeErrors(() =>
      liveNativeOf(this).offlineRegionsMergeDatabaseTakeResult(
        operationIdOf(operation),
      ),
    );
  }

  offlineRegionUpdateMetadataTakeResult(operation) {
    return translateNativeErrors(() =>
      liveNativeOf(this).offlineRegionUpdateMetadataTakeResult(
        operationIdOf(operation),
      ),
    );
  }

  offlineRegionGetStatusTakeResult(operation) {
    return translateNativeErrors(() =>
      liveNativeOf(this).offlineRegionGetStatusTakeResult(
        operationIdOf(operation),
      ),
    );
  }

  #offlineOperation(startOperation) {
    const start = translateNativeErrors(startOperation);
    return new OfflineOperationHandle(this, BigInt(start.operationId));
  }

  pollEvent() {
    return translateNativeErrors(() => liveNativeOf(this).pollEvent());
  }

  [Symbol.dispose]() {
    this.close();
  }
}

class MapProjectionHandle {
  constructor(map) {
    recordHandleEnvironment(this);
    if (!(map instanceof MapHandle)) {
      throw new InvalidArgumentError(null, "map must be a MapHandle");
    }
    defineCheckedNative(
      this,
      translateNativeErrors(() =>
        native.createNativeMapProjectionHandle(liveNativeOf(map)),
      ),
    );
  }

  close() {
    return translateNativeErrors(() => nativeOf(this).close());
  }

  get closed() {
    return nativeOf(this).closed;
  }

  getCamera() {
    return translateNativeErrors(() => liveNativeOf(this).getCamera());
  }

  setCamera(camera) {
    return translateNativeErrors(() => liveNativeOf(this).setCamera(camera));
  }

  setVisibleCoordinates(coordinates, padding) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setVisibleCoordinates(coordinates, padding),
    );
  }

  setVisibleGeometry(geometry, padding) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setVisibleGeometry(stringifyJson(geometry), padding),
    );
  }

  pixelForLatLng(coordinate) {
    return translateNativeErrors(() =>
      liveNativeOf(this).pixelForLatLng(coordinate),
    );
  }

  latLngForPixel(point) {
    return translateNativeErrors(() =>
      liveNativeOf(this).latLngForPixel(point),
    );
  }

  [Symbol.dispose]() {
    this.close();
  }
}

function nativePointerAddress(value, fieldName) {
  if (!(value instanceof NativePointer)) {
    throw new InvalidArgumentError(
      null,
      `${fieldName} must be a NativePointer`,
    );
  }
  return value.address;
}

function nullableNativePointerAddress(value, fieldName) {
  return value == null ? null : nativePointerAddress(value, fieldName);
}

function normalizeMetalContext(context = {}) {
  return {
    deviceAddress: nullableNativePointerAddress(context.device, "device"),
  };
}

function normalizeVulkanContext(context) {
  return {
    instanceAddress: nativePointerAddress(context?.instance, "instance"),
    physicalDeviceAddress: nativePointerAddress(
      context?.physicalDevice,
      "physicalDevice",
    ),
    deviceAddress: nativePointerAddress(context?.device, "device"),
    graphicsQueueAddress: nativePointerAddress(
      context?.graphicsQueue,
      "graphicsQueue",
    ),
    graphicsQueueFamilyIndex: context?.graphicsQueueFamilyIndex,
    getInstanceProcAddrAddress: nullableNativePointerAddress(
      context?.getInstanceProcAddr,
      "getInstanceProcAddr",
    ),
    getDeviceProcAddrAddress: nullableNativePointerAddress(
      context?.getDeviceProcAddr,
      "getDeviceProcAddr",
    ),
  };
}

function normalizeOpenGLContext(context) {
  const platform = context?.platform;
  if (platform === "wgl") {
    return {
      platform,
      wgl: {
        deviceContextAddress: nativePointerAddress(
          context.deviceContext,
          "deviceContext",
        ),
        shareContextAddress: nativePointerAddress(
          context.shareContext,
          "shareContext",
        ),
        getProcAddressAddress: nullableNativePointerAddress(
          context.getProcAddress,
          "getProcAddress",
        ),
      },
      egl: null,
    };
  }
  if (platform === "egl") {
    return {
      platform,
      wgl: null,
      egl: {
        displayAddress: nativePointerAddress(context.display, "display"),
        configAddress: nativePointerAddress(context.config, "config"),
        shareContextAddress: nativePointerAddress(
          context.shareContext,
          "shareContext",
        ),
        getProcAddressAddress: nullableNativePointerAddress(
          context.getProcAddress,
          "getProcAddress",
        ),
      },
    };
  }
  throw new InvalidArgumentError(
    null,
    "OpenGL context platform must be 'wgl' or 'egl'",
  );
}

function normalizeMetalOwnedTextureDescriptor(descriptor) {
  return {
    extent: descriptor?.extent,
    context: normalizeMetalContext(descriptor?.context),
  };
}

function normalizeMetalBorrowedTextureDescriptor(descriptor) {
  return {
    extent: descriptor?.extent,
    textureAddress: nativePointerAddress(descriptor?.texture, "texture"),
  };
}

function normalizeMetalSurfaceDescriptor(descriptor) {
  return {
    extent: descriptor?.extent,
    context: normalizeMetalContext(descriptor?.context),
    layerAddress: nativePointerAddress(descriptor?.layer, "layer"),
  };
}

function normalizeVulkanOwnedTextureDescriptor(descriptor) {
  return {
    extent: descriptor?.extent,
    context: normalizeVulkanContext(descriptor?.context),
  };
}

function normalizeVulkanBorrowedTextureDescriptor(descriptor) {
  return {
    extent: descriptor?.extent,
    context: normalizeVulkanContext(descriptor?.context),
    imageAddress: nativePointerAddress(descriptor?.image, "image"),
    imageViewAddress: nativePointerAddress(descriptor?.imageView, "imageView"),
    format: descriptor?.format,
    initialLayout: descriptor?.initialLayout,
    finalLayout: descriptor?.finalLayout,
  };
}

function normalizeVulkanSurfaceDescriptor(descriptor) {
  return {
    extent: descriptor?.extent,
    context: normalizeVulkanContext(descriptor?.context),
    surfaceAddress: nativePointerAddress(descriptor?.surface, "surface"),
  };
}

function normalizeOpenGLOwnedTextureDescriptor(descriptor) {
  return {
    extent: descriptor?.extent,
    context: normalizeOpenGLContext(descriptor?.context),
  };
}

function normalizeOpenGLBorrowedTextureDescriptor(descriptor) {
  return {
    extent: descriptor?.extent,
    context: normalizeOpenGLContext(descriptor?.context),
    texture: descriptor?.texture,
    target: descriptor?.target,
  };
}

function normalizeOpenGLSurfaceDescriptor(descriptor) {
  return {
    extent: descriptor?.extent,
    context: normalizeOpenGLContext(descriptor?.context),
    surfaceAddress: nativePointerAddress(descriptor?.surface, "surface"),
  };
}

class RenderSessionHandle {
  constructor(token, nativeHandle, map) {
    if (token !== CONSTRUCTION_TOKEN) {
      throw new InvalidArgumentError(
        null,
        "render sessions are created by MapHandle attach methods",
      );
    }
    recordHandleEnvironment(this);
    defineCheckedNative(this, nativeHandle);
    this.map = map;
  }

  static attachMetalOwnedTexture(map, descriptor) {
    return attachRenderSession(map, () =>
      native.createMetalOwnedTextureRenderSession(
        liveNativeOf(map),
        normalizeMetalOwnedTextureDescriptor(descriptor),
      ),
    );
  }

  static attachMetalBorrowedTexture(map, descriptor) {
    return attachRenderSession(map, () =>
      native.createMetalBorrowedTextureRenderSession(
        liveNativeOf(map),
        normalizeMetalBorrowedTextureDescriptor(descriptor),
      ),
    );
  }

  static attachMetalSurface(map, descriptor) {
    return attachRenderSession(map, () =>
      native.createMetalSurfaceRenderSession(
        liveNativeOf(map),
        normalizeMetalSurfaceDescriptor(descriptor),
      ),
    );
  }

  static attachVulkanOwnedTexture(map, descriptor) {
    return attachRenderSession(map, () =>
      native.createVulkanOwnedTextureRenderSession(
        liveNativeOf(map),
        normalizeVulkanOwnedTextureDescriptor(descriptor),
      ),
    );
  }

  static attachVulkanBorrowedTexture(map, descriptor) {
    return attachRenderSession(map, () =>
      native.createVulkanBorrowedTextureRenderSession(
        liveNativeOf(map),
        normalizeVulkanBorrowedTextureDescriptor(descriptor),
      ),
    );
  }

  static attachVulkanSurface(map, descriptor) {
    return attachRenderSession(map, () =>
      native.createVulkanSurfaceRenderSession(
        liveNativeOf(map),
        normalizeVulkanSurfaceDescriptor(descriptor),
      ),
    );
  }

  static attachOpenGLOwnedTexture(map, descriptor) {
    return attachRenderSession(map, () =>
      native.createOpenGLOwnedTextureRenderSession(
        liveNativeOf(map),
        normalizeOpenGLOwnedTextureDescriptor(descriptor),
      ),
    );
  }

  static attachOpenGLBorrowedTexture(map, descriptor) {
    return attachRenderSession(map, () =>
      native.createOpenGLBorrowedTextureRenderSession(
        liveNativeOf(map),
        normalizeOpenGLBorrowedTextureDescriptor(descriptor),
      ),
    );
  }

  static attachOpenGLSurface(map, descriptor) {
    return attachRenderSession(map, () =>
      native.createOpenGLSurfaceRenderSession(
        liveNativeOf(map),
        normalizeOpenGLSurfaceDescriptor(descriptor),
      ),
    );
  }

  close() {
    return translateNativeErrors(() => nativeOf(this).close());
  }

  get closed() {
    return nativeOf(this).closed;
  }

  resize(width, height, scaleFactor) {
    return translateNativeErrors(() =>
      liveNativeOf(this).resize(width, height, scaleFactor),
    );
  }

  renderUpdate() {
    return translateNativeErrors(() => liveNativeOf(this).renderUpdate());
  }

  detach() {
    return translateNativeErrors(() => liveNativeOf(this).detach());
  }

  reduceMemoryUse() {
    return translateNativeErrors(() => liveNativeOf(this).reduceMemoryUse());
  }

  clearData() {
    return translateNativeErrors(() => liveNativeOf(this).clearData());
  }

  dumpDebugLogs() {
    return translateNativeErrors(() => liveNativeOf(this).dumpDebugLogs());
  }

  setFeatureState(selector, state) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setFeatureState(selector, stringifyJson(state)),
    );
  }

  getFeatureState(selector) {
    return translateNativeErrors(() =>
      JSON.parse(liveNativeOf(this).getFeatureState(selector)),
    );
  }

  removeFeatureState(selector) {
    return translateNativeErrors(() =>
      liveNativeOf(this).removeFeatureState(selector),
    );
  }

  queryRenderedFeatures(geometry, options = null) {
    const nativeOptions =
      options == null
        ? null
        : {
            ...options,
            filter:
              options.filter == null ? null : stringifyJson(options.filter),
          };
    return translateNativeErrors(() =>
      JSON.parse(
        liveNativeOf(this).queryRenderedFeatures(geometry, nativeOptions),
      ),
    );
  }

  querySourceFeatures(sourceId, options = null) {
    const nativeOptions =
      options == null
        ? null
        : {
            ...options,
            filter:
              options.filter == null ? null : stringifyJson(options.filter),
          };
    return translateNativeErrors(() =>
      JSON.parse(
        liveNativeOf(this).querySourceFeatures(sourceId, nativeOptions),
      ),
    );
  }

  queryFeatureExtension(
    sourceId,
    feature,
    extension,
    extensionField,
    args = null,
  ) {
    return translateNativeErrors(() =>
      JSON.parse(
        liveNativeOf(this).queryFeatureExtension(
          sourceId,
          stringifyJson(feature),
          extension,
          extensionField,
          args == null ? null : stringifyJson(args),
        ),
      ),
    );
  }

  withMetalOwnedTextureFrame(callback) {
    if (typeof callback !== "function") {
      throw new InvalidArgumentError(
        null,
        "metal texture frame callback must be a function",
      );
    }
    const frame = new MetalOwnedTextureFrame(
      CONSTRUCTION_TOKEN,
      translateNativeErrors(() =>
        liveNativeOf(this).acquireMetalOwnedTextureFrame(),
      ),
    );
    try {
      return callback(frame);
    } finally {
      try {
        translateNativeErrors(() =>
          liveNativeOf(this).releaseMetalOwnedTextureFrame(
            frame[TEXTURE_FRAME_RAW](),
          ),
        );
      } finally {
        frame[TEXTURE_FRAME_DEACTIVATE]();
      }
    }
  }

  withVulkanOwnedTextureFrame(callback) {
    if (typeof callback !== "function") {
      throw new InvalidArgumentError(
        null,
        "vulkan texture frame callback must be a function",
      );
    }
    const frame = new VulkanOwnedTextureFrame(
      CONSTRUCTION_TOKEN,
      translateNativeErrors(() =>
        liveNativeOf(this).acquireVulkanOwnedTextureFrame(),
      ),
    );
    try {
      return callback(frame);
    } finally {
      try {
        translateNativeErrors(() =>
          liveNativeOf(this).releaseVulkanOwnedTextureFrame(
            frame[TEXTURE_FRAME_RAW](),
          ),
        );
      } finally {
        frame[TEXTURE_FRAME_DEACTIVATE]();
      }
    }
  }

  withOpenGLOwnedTextureFrame(callback) {
    if (typeof callback !== "function") {
      throw new InvalidArgumentError(
        null,
        "OpenGL texture frame callback must be a function",
      );
    }
    const frame = new OpenGLOwnedTextureFrame(
      CONSTRUCTION_TOKEN,
      translateNativeErrors(() =>
        liveNativeOf(this).acquireOpenGLOwnedTextureFrame(),
      ),
    );
    try {
      return callback(frame);
    } finally {
      try {
        translateNativeErrors(() =>
          liveNativeOf(this).releaseOpenGLOwnedTextureFrame(
            frame[TEXTURE_FRAME_RAW](),
          ),
        );
      } finally {
        frame[TEXTURE_FRAME_DEACTIVATE]();
      }
    }
  }

  readPremultipliedRgba8() {
    return translateNativeErrors(() =>
      liveNativeOf(this).readPremultipliedRgba8(),
    );
  }

  [Symbol.dispose]() {
    this.close();
  }
}

function attachRenderSession(map, attach) {
  if (!(map instanceof MapHandle)) {
    throw new InvalidArgumentError(null, "map must be a MapHandle");
  }
  return new RenderSessionHandle(
    CONSTRUCTION_TOKEN,
    translateNativeErrors(attach),
    map,
  );
}

class MapHandle {
  constructor(runtime, options) {
    recordHandleEnvironment(this);
    if (!(runtime instanceof RuntimeHandle)) {
      throw new InvalidArgumentError(null, "runtime must be a RuntimeHandle");
    }
    this.runtime = runtime;
    defineCheckedNative(
      this,
      translateNativeErrors(() =>
        native.createNativeMapHandle(liveNativeOf(runtime), options ?? {}),
      ),
    );
  }

  close() {
    return translateNativeErrors(() => nativeOf(this).close());
  }

  get closed() {
    return nativeOf(this).closed;
  }

  createProjection() {
    return new MapProjectionHandle(this);
  }

  attachMetalOwnedTexture(descriptor) {
    return RenderSessionHandle.attachMetalOwnedTexture(this, descriptor);
  }

  attachMetalBorrowedTexture(descriptor) {
    return RenderSessionHandle.attachMetalBorrowedTexture(this, descriptor);
  }

  attachMetalSurface(descriptor) {
    return RenderSessionHandle.attachMetalSurface(this, descriptor);
  }

  attachVulkanOwnedTexture(descriptor) {
    return RenderSessionHandle.attachVulkanOwnedTexture(this, descriptor);
  }

  attachVulkanBorrowedTexture(descriptor) {
    return RenderSessionHandle.attachVulkanBorrowedTexture(this, descriptor);
  }

  attachVulkanSurface(descriptor) {
    return RenderSessionHandle.attachVulkanSurface(this, descriptor);
  }

  attachOpenGLOwnedTexture(descriptor) {
    return RenderSessionHandle.attachOpenGLOwnedTexture(this, descriptor);
  }

  attachOpenGLBorrowedTexture(descriptor) {
    return RenderSessionHandle.attachOpenGLBorrowedTexture(this, descriptor);
  }

  attachOpenGLSurface(descriptor) {
    return RenderSessionHandle.attachOpenGLSurface(this, descriptor);
  }

  requestRepaint() {
    return translateNativeErrors(() => liveNativeOf(this).requestRepaint());
  }

  requestStillImage() {
    return translateNativeErrors(() => liveNativeOf(this).requestStillImage());
  }

  isFullyLoaded() {
    return translateNativeErrors(() => liveNativeOf(this).isFullyLoaded());
  }

  dumpDebugLogs() {
    return translateNativeErrors(() => liveNativeOf(this).dumpDebugLogs());
  }

  getDebugOptions() {
    const mask = translateNativeErrors(() =>
      liveNativeOf(this).getDebugOptionsRaw(),
    );
    return MAP_DEBUG_OPTIONS.filter((option) =>
      Boolean(mask & mapDebugOptionMaskBit(option)),
    );
  }

  setDebugOptions(options) {
    let mask = 0;
    for (const option of options) {
      mask |= mapDebugOptionMaskBit(option);
    }
    return translateNativeErrors(() =>
      liveNativeOf(this).setDebugOptionsRaw(mask),
    );
  }

  moveBy(deltaX, deltaY) {
    return translateNativeErrors(() =>
      liveNativeOf(this).moveBy(deltaX, deltaY),
    );
  }

  scaleBy(scale, anchor = null) {
    return translateNativeErrors(() =>
      liveNativeOf(this).scaleBy(scale, anchor),
    );
  }

  rotateBy(first, second) {
    return translateNativeErrors(() =>
      liveNativeOf(this).rotateBy(first, second),
    );
  }

  pitchBy(pitch) {
    return translateNativeErrors(() => liveNativeOf(this).pitchBy(pitch));
  }

  moveByAnimated(deltaX, deltaY, animation = null) {
    return translateNativeErrors(() =>
      liveNativeOf(this).moveByAnimated(deltaX, deltaY, animation),
    );
  }

  scaleByAnimated(scale, anchor = null, animation = null) {
    return translateNativeErrors(() =>
      liveNativeOf(this).scaleByAnimated(scale, anchor, animation),
    );
  }

  rotateByAnimated(first, second, animation = null) {
    return translateNativeErrors(() =>
      liveNativeOf(this).rotateByAnimated(first, second, animation),
    );
  }

  pitchByAnimated(pitch, animation = null) {
    return translateNativeErrors(() =>
      liveNativeOf(this).pitchByAnimated(pitch, animation),
    );
  }

  cancelTransitions() {
    return translateNativeErrors(() => liveNativeOf(this).cancelTransitions());
  }

  getViewportOptions() {
    return translateNativeErrors(() => liveNativeOf(this).getViewportOptions());
  }

  setViewportOptions(options) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setViewportOptions(options),
    );
  }

  getTileOptions() {
    return translateNativeErrors(() => liveNativeOf(this).getTileOptions());
  }

  setTileOptions(options) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setTileOptions(options),
    );
  }

  getBounds() {
    return translateNativeErrors(() => liveNativeOf(this).getBounds());
  }

  setBounds(options) {
    return translateNativeErrors(() => liveNativeOf(this).setBounds(options));
  }

  getFreeCameraOptions() {
    return translateNativeErrors(() =>
      liveNativeOf(this).getFreeCameraOptions(),
    );
  }

  setFreeCameraOptions(options) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setFreeCameraOptions(options),
    );
  }

  getProjectionMode() {
    return translateNativeErrors(() => liveNativeOf(this).getProjectionMode());
  }

  setProjectionMode(mode) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setProjectionMode(mode),
    );
  }

  getCamera() {
    return translateNativeErrors(() => liveNativeOf(this).getCamera());
  }

  jumpTo(camera) {
    return translateNativeErrors(() => liveNativeOf(this).jumpTo(camera));
  }

  easeTo(camera, animation = null) {
    return translateNativeErrors(() =>
      liveNativeOf(this).easeTo(camera, animation),
    );
  }

  flyTo(camera, animation = null) {
    return translateNativeErrors(() =>
      liveNativeOf(this).flyTo(camera, animation),
    );
  }

  cameraForLatLngBounds(bounds) {
    return translateNativeErrors(() =>
      liveNativeOf(this).cameraForLatLngBounds(bounds),
    );
  }

  cameraForLatLngs(coordinates) {
    return translateNativeErrors(() =>
      liveNativeOf(this).cameraForLatLngs(coordinates),
    );
  }

  cameraForGeometry(geometry) {
    return translateNativeErrors(() =>
      liveNativeOf(this).cameraForGeometry(stringifyJson(geometry)),
    );
  }

  latLngBoundsForCamera(camera) {
    return translateNativeErrors(() =>
      liveNativeOf(this).latLngBoundsForCamera(camera),
    );
  }

  latLngBoundsForCameraUnwrapped(camera) {
    return translateNativeErrors(() =>
      liveNativeOf(this).latLngBoundsForCameraUnwrapped(camera),
    );
  }

  pixelForLatLng(coordinate) {
    return translateNativeErrors(() =>
      liveNativeOf(this).pixelForLatLng(coordinate),
    );
  }

  latLngForPixel(point) {
    return translateNativeErrors(() =>
      liveNativeOf(this).latLngForPixel(point),
    );
  }

  pixelsForLatLngs(coordinates) {
    return translateNativeErrors(() =>
      liveNativeOf(this).pixelsForLatLngs(coordinates),
    );
  }

  latLngsForPixels(points) {
    return translateNativeErrors(() =>
      liveNativeOf(this).latLngsForPixels(points),
    );
  }

  get renderingStatsViewEnabled() {
    return translateNativeErrors(
      () => liveNativeOf(this).renderingStatsViewEnabled,
    );
  }

  set renderingStatsViewEnabled(enabled) {
    translateNativeErrors(() => {
      liveNativeOf(this).renderingStatsViewEnabled = enabled;
    });
  }

  addStyleSourceJson(sourceId, source) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addStyleSourceJson(sourceId, stringifyJson(source)),
    );
  }

  styleSourceExists(sourceId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).styleSourceExists(sourceId),
    );
  }

  removeStyleSource(sourceId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).removeStyleSource(sourceId),
    );
  }

  listStyleSourceIds() {
    return translateNativeErrors(() => liveNativeOf(this).listStyleSourceIds());
  }

  getStyleSourceType(sourceId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).getStyleSourceType(sourceId),
    );
  }

  getStyleSourceInfo(sourceId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).getStyleSourceInfo(sourceId),
    );
  }

  addGeoJsonSourceUrl(sourceId, url) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addGeoJsonSourceUrl(sourceId, url),
    );
  }

  addGeoJsonSourceData(sourceId, data) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addGeoJsonSourceData(sourceId, stringifyJson(data)),
    );
  }

  setGeoJsonSourceUrl(sourceId, url) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setGeoJsonSourceUrl(sourceId, url),
    );
  }

  setGeoJsonSourceData(sourceId, data) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setGeoJsonSourceData(sourceId, stringifyJson(data)),
    );
  }

  addVectorSourceUrl(sourceId, url) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addVectorSourceUrl(sourceId, url),
    );
  }

  addRasterSourceUrl(sourceId, url) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addRasterSourceUrl(sourceId, url),
    );
  }

  addRasterDemSourceUrl(sourceId, url) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addRasterDemSourceUrl(sourceId, url),
    );
  }

  addVectorSourceTiles(sourceId, tiles) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addVectorSourceTiles(sourceId, Array.from(tiles)),
    );
  }

  addRasterSourceTiles(sourceId, tiles) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addRasterSourceTiles(sourceId, Array.from(tiles)),
    );
  }

  addRasterDemSourceTiles(sourceId, tiles) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addRasterDemSourceTiles(sourceId, Array.from(tiles)),
    );
  }

  addCustomGeometrySource(sourceId, options = null) {
    const { fetchTile, cancelTile, ...nativeOptions } = options ?? {};
    if (fetchTile != null && typeof fetchTile !== "function") {
      throw new InvalidArgumentError(
        null,
        "custom geometry fetchTile callback must be a function",
      );
    }
    if (cancelTile != null && typeof cancelTile !== "function") {
      throw new InvalidArgumentError(
        null,
        "custom geometry cancelTile callback must be a function",
      );
    }
    return translateNativeErrors(() =>
      liveNativeOf(this).addCustomGeometrySource(
        sourceId,
        nativeOptions,
        fetchTile ?? null,
        cancelTile ?? null,
      ),
    );
  }

  setCustomGeometrySourceTileData(sourceId, tileId, data) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setCustomGeometrySourceTileData(
        sourceId,
        tileId,
        stringifyJson(data),
      ),
    );
  }

  invalidateCustomGeometrySourceTile(sourceId, tileId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).invalidateCustomGeometrySourceTile(sourceId, tileId),
    );
  }

  invalidateCustomGeometrySourceRegion(sourceId, bounds) {
    return translateNativeErrors(() =>
      liveNativeOf(this).invalidateCustomGeometrySourceRegion(sourceId, bounds),
    );
  }

  setStyleImage(imageId, image) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setStyleImage(imageId, image),
    );
  }

  styleImageExists(imageId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).styleImageExists(imageId),
    );
  }

  removeStyleImage(imageId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).removeStyleImage(imageId),
    );
  }

  getStyleImageInfo(imageId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).getStyleImageInfo(imageId),
    );
  }

  copyStyleImagePremultipliedRgba8(imageId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).copyStyleImagePremultipliedRgba8(imageId),
    );
  }

  addImageSourceUrl(sourceId, coordinates, url) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addImageSourceUrl(sourceId, coordinates, url),
    );
  }

  addImageSourceImage(sourceId, coordinates, image) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addImageSourceImage(sourceId, coordinates, image),
    );
  }

  setImageSourceUrl(sourceId, url) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setImageSourceUrl(sourceId, url),
    );
  }

  setImageSourceImage(sourceId, image) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setImageSourceImage(sourceId, image),
    );
  }

  setImageSourceCoordinates(sourceId, coordinates) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setImageSourceCoordinates(sourceId, coordinates),
    );
  }

  getImageSourceCoordinates(sourceId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).getImageSourceCoordinates(sourceId),
    );
  }

  addHillshadeLayer(layerId, sourceId, beforeLayerId = null) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addHillshadeLayer(layerId, sourceId, beforeLayerId),
    );
  }

  addColorReliefLayer(layerId, sourceId, beforeLayerId = null) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addColorReliefLayer(layerId, sourceId, beforeLayerId),
    );
  }

  addLocationIndicatorLayer(layerId, beforeLayerId = null) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addLocationIndicatorLayer(layerId, beforeLayerId),
    );
  }

  setLocationIndicatorLocation(layerId, coordinate, altitude = 0) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setLocationIndicatorLocation(
        layerId,
        coordinate,
        altitude,
      ),
    );
  }

  setLocationIndicatorBearing(layerId, bearing) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setLocationIndicatorBearing(layerId, bearing),
    );
  }

  setLocationIndicatorAccuracyRadius(layerId, radius) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setLocationIndicatorAccuracyRadius(layerId, radius),
    );
  }

  setLocationIndicatorImageName(layerId, imageKind, imageId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setLocationIndicatorImageName(
        layerId,
        imageKind,
        imageId,
      ),
    );
  }

  addStyleLayerJson(layer, beforeLayerId = null) {
    return translateNativeErrors(() =>
      liveNativeOf(this).addStyleLayerJson(stringifyJson(layer), beforeLayerId),
    );
  }

  styleLayerExists(layerId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).styleLayerExists(layerId),
    );
  }

  removeStyleLayer(layerId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).removeStyleLayer(layerId),
    );
  }

  listStyleLayerIds() {
    return translateNativeErrors(() => liveNativeOf(this).listStyleLayerIds());
  }

  getStyleLayerType(layerId) {
    return translateNativeErrors(() =>
      liveNativeOf(this).getStyleLayerType(layerId),
    );
  }

  getStyleLayerJson(layerId) {
    const json = translateNativeErrors(() =>
      liveNativeOf(this).getStyleLayerJson(layerId),
    );
    return json === null ? null : JSON.parse(json);
  }

  moveStyleLayer(layerId, beforeLayerId = null) {
    return translateNativeErrors(() =>
      liveNativeOf(this).moveStyleLayer(layerId, beforeLayerId),
    );
  }

  setLayerProperty(layerId, propertyName, value) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setLayerPropertyJson(
        layerId,
        propertyName,
        stringifyJson(value),
      ),
    );
  }

  getLayerProperty(layerId, propertyName) {
    const json = translateNativeErrors(() =>
      liveNativeOf(this).getLayerPropertyJson(layerId, propertyName),
    );
    return json === null ? null : JSON.parse(json);
  }

  setLayerFilter(layerId, filter) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setLayerFilterJson(
        layerId,
        filter === null ? null : stringifyJson(filter),
      ),
    );
  }

  getLayerFilter(layerId) {
    const json = translateNativeErrors(() =>
      liveNativeOf(this).getLayerFilterJson(layerId),
    );
    return json === null ? null : JSON.parse(json);
  }

  setStyleLight(light) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setStyleLightJson(stringifyJson(light)),
    );
  }

  setStyleLightProperty(propertyName, value) {
    return translateNativeErrors(() =>
      liveNativeOf(this).setStyleLightPropertyJson(
        propertyName,
        stringifyJson(value),
      ),
    );
  }

  getStyleLightProperty(propertyName) {
    const json = translateNativeErrors(() =>
      liveNativeOf(this).getStyleLightPropertyJson(propertyName),
    );
    return json === null ? null : JSON.parse(json);
  }

  setStyleJson(json) {
    return translateNativeErrors(() => liveNativeOf(this).setStyleJson(json));
  }

  setStyleUrl(url) {
    return translateNativeErrors(() => liveNativeOf(this).setStyleUrl(url));
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
  RenderSessionHandle,
  MetalOwnedTextureFrame,
  VulkanOwnedTextureFrame,
  OpenGLOwnedTextureFrame,
  NativePointer,
  NativeBuffer,
  cVersion,
  supportedRenderBackends,
  supportedOpenGLContextProviders,
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

module.exports.MaplibreError = MaplibreError;
module.exports.InvalidArgumentError = InvalidArgumentError;
module.exports.InvalidStateError = InvalidStateError;
module.exports.WrongThreadError = WrongThreadError;
module.exports.UnsupportedFeatureError = UnsupportedFeatureError;
module.exports.NativeError = NativeError;
module.exports.MaplibreStatus = MaplibreStatus;
module.exports.RuntimeHandle = RuntimeHandle;
module.exports.ResourceRequestHandle = ResourceRequestHandle;
module.exports.OfflineOperationHandle = OfflineOperationHandle;
module.exports.MapHandle = MapHandle;
module.exports.MapProjectionHandle = MapProjectionHandle;
module.exports.RenderSessionHandle = RenderSessionHandle;
module.exports.MetalOwnedTextureFrame = MetalOwnedTextureFrame;
module.exports.VulkanOwnedTextureFrame = VulkanOwnedTextureFrame;
module.exports.OpenGLOwnedTextureFrame = OpenGLOwnedTextureFrame;
module.exports.NativePointer = NativePointer;
module.exports.NativeBuffer = NativeBuffer;
module.exports.cVersion = cVersion;
module.exports.supportedRenderBackends = supportedRenderBackends;
module.exports.supportedOpenGLContextProviders =
  supportedOpenGLContextProviders;
module.exports.threadLastErrorMessage = threadLastErrorMessage;
module.exports.networkStatus = networkStatus;
module.exports.setNetworkStatus = setNetworkStatus;
module.exports.projectedMetersForLatLng = projectedMetersForLatLng;
module.exports.latLngForProjectedMeters = latLngForProjectedMeters;
module.exports.setLogCallback = setLogCallback;
module.exports.clearLogCallback = clearLogCallback;
module.exports.setAsyncLogSeverities = setAsyncLogSeverities;
module.exports.restoreDefaultAsyncLogSeverities =
  restoreDefaultAsyncLogSeverities;

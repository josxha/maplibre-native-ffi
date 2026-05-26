import { type CameraOptions } from "@maplibre/native-ffi-node/camera";
import { InvalidArgumentError } from "@maplibre/native-ffi-node/error";
import {
  projectedMetersForLatLng,
  type LatLng,
} from "@maplibre/native-ffi-node/geo";
import { type JsonValue } from "@maplibre/native-ffi-node/json";
import { setLogCallback, type LogRecord } from "@maplibre/native-ffi-node/log";
import {
  MapHandle,
  type MapTileOptionsInput,
  type MapViewportOptionsInput,
} from "@maplibre/native-ffi-node/map";
import { OfflineOperationHandle } from "@maplibre/native-ffi-node/offline";
import { type RenderedQueryGeometry } from "@maplibre/native-ffi-node/query";
import {
  NativeBuffer,
  NativePointer,
  RenderSessionHandle,
  type MetalBorrowedTextureDescriptor,
} from "@maplibre/native-ffi-node/render";
import {
  ResourceRequestHandle,
  type ResourceProviderCallback,
  type ResourceResponseInput,
  type ResourceRoute,
  type ResourceTransformRule,
} from "@maplibre/native-ffi-node/resource";
import {
  RuntimeHandle,
  networkStatus,
} from "@maplibre/native-ffi-node/runtime";
import { type StyleImageInput } from "@maplibre/native-ffi-node/style";

const camera: CameraOptions = { center: { latitude: 1, longitude: 2 } };
const coordinate: LatLng = { latitude: 1, longitude: 2 };
projectedMetersForLatLng(coordinate);
setLogCallback((record: LogRecord) => {
  void record.message;
});
networkStatus();
void InvalidArgumentError;
void MapHandle;
void OfflineOperationHandle;
void RenderSessionHandle;
void ResourceRequestHandle;
void RuntimeHandle;
void NativeBuffer;
const descriptor: MetalBorrowedTextureDescriptor = {
  extent: { width: 1, height: 1, scaleFactor: 1 },
  texture: NativePointer.null,
};
const geometry: RenderedQueryGeometry = {
  kind: "point",
  point: { x: 0, y: 0 },
};
const response: ResourceResponseInput = { status: "ok", modifiedUnixMs: 1n };
const route: ResourceRoute = { urlPrefix: "custom://", kind: "source" };
const providerCallback: ResourceProviderCallback = (request) => {
  request.handle.complete(response);
};
// @ts-expect-error provider callbacks complete through request.handle, not return values.
const invalidProviderCallback: ResourceProviderCallback = () => response;
RuntimeHandle.prototype.setResourceProviderRoutes.call(
  {} as RuntimeHandle,
  [route],
  providerCallback,
);
const viewportOptions: MapViewportOptionsInput = { northOrientation: "up" };
const tileOptions: MapTileOptionsInput = { lodMode: "distance" };
const invalidViewportOptions: MapViewportOptionsInput = {
  // @ts-expect-error input descriptors reject unknown enum sentinels.
  viewportMode: "unknown",
};
// @ts-expect-error input descriptors reject unknown enum sentinels.
const invalidTileOptions: MapTileOptionsInput = { lodMode: "unknown" };
const runtimeEvent = RuntimeHandle.prototype.pollEvent.call(
  {} as RuntimeHandle,
);
if (runtimeEvent?.payload.kind === "offline-operation-completed") {
  const operationId: bigint =
    runtimeEvent.payload.offlineOperationCompleted.operationId;
  void operationId;
}
const transformRule: ResourceTransformRule = {
  urlPrefix: "http://example.test/",
  replacementUrlPrefix: "https://example.test/",
};
// @ts-expect-error prefix replacements require a matched URL prefix.
const invalidTransformRuleMissingPrefix: ResourceTransformRule = {
  replacementUrlPrefix: "https://example.test/",
};
// @ts-expect-error transform rules set exactly one replacement form.
const invalidTransformRuleBothReplacements: ResourceTransformRule = {
  urlPrefix: "http://example.test/",
  replacementUrl: "https://example.test/a",
  replacementUrlPrefix: "https://example.test/",
};
const image: StyleImageInput = {
  width: 1,
  height: 1,
  pixels: new Uint8Array(4),
};
const json: JsonValue = { ok: true };
void camera;
void descriptor;
void geometry;
void response;
void route;
void providerCallback;
void viewportOptions;
void tileOptions;
void invalidViewportOptions;
void invalidTileOptions;
void invalidProviderCallback;
void transformRule;
void invalidTransformRuleMissingPrefix;
void invalidTransformRuleBothReplacements;
void image;
void json;

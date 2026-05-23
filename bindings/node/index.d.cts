export declare const MaplibreStatus: Readonly<{
  invalidArgument: "invalid-argument";
  invalidState: "invalid-state";
  wrongThread: "wrong-thread";
  unsupported: "unsupported";
  nativeError: "native-error";
  abiVersionMismatch: "abi-version-mismatch";
  unknownStatus: "unknown-status";
}>;

export type MaplibreStatusKind =
  (typeof MaplibreStatus)[keyof typeof MaplibreStatus];

export declare class MaplibreError extends Error {
  readonly status: MaplibreStatusKind;
  readonly nativeStatusCode: number | null;
  readonly diagnostic: string;
  constructor(
    status: MaplibreStatusKind,
    nativeStatusCode: number | null,
    diagnostic: string,
    options?: ErrorOptions,
  );
}

export declare class InvalidArgumentError extends MaplibreError {
  constructor(
    nativeStatusCode: number | null,
    diagnostic: string,
    options?: ErrorOptions,
  );
}

export declare class InvalidStateError extends MaplibreError {
  constructor(
    nativeStatusCode: number | null,
    diagnostic: string,
    options?: ErrorOptions,
  );
}

export declare class WrongThreadError extends MaplibreError {
  constructor(
    nativeStatusCode: number | null,
    diagnostic: string,
    options?: ErrorOptions,
  );
}

export declare class UnsupportedFeatureError extends MaplibreError {
  constructor(
    nativeStatusCode: number | null,
    diagnostic: string,
    options?: ErrorOptions,
  );
}

export declare class NativeError extends MaplibreError {
  constructor(
    nativeStatusCode: number | null,
    diagnostic: string,
    options?: ErrorOptions,
  );
}

export interface RenderBackends {
  rawMask: number;
  metal: boolean;
  vulkan: boolean;
}

export interface NetworkStatusValue {
  kind: "online" | "offline" | "unknown";
  raw: number;
}

export interface RuntimeOptions {
  assetPath?: string | null;
  cachePath?: string | null;
  maximumCacheSize?: bigint | null;
}

export interface RuntimeEvent {
  eventType: string;
  rawEventType: number;
  sourceType: string;
  rawSourceType: number;
  sourceAddress: string;
  code: number;
  message?: string | null;
  payloadKind: string;
}

export interface MapOptions {
  width?: number | null;
  height?: number | null;
  scaleFactor?: number | null;
  mapMode?: "continuous" | "static" | "tile" | null;
}

export declare class RuntimeHandle {
  constructor(options?: RuntimeOptions | null);
  readonly closed: boolean;
  createMap(options?: MapOptions | null): MapHandle;
  close(): void;
  runOnce(): void;
  pollEvent(): RuntimeEvent | null;
  [Symbol.dispose](): void;
}

export declare class MapHandle {
  constructor(runtime: RuntimeHandle, options?: MapOptions | null);
  readonly closed: boolean;
  close(): void;
  requestRepaint(): void;
  setStyleJson(json: string): void;
  setStyleUrl(url: string): void;
  [Symbol.dispose](): void;
}

export interface LatLng {
  latitude: number;
  longitude: number;
}

export interface ProjectedMeters {
  northing: number;
  easting: number;
}

export declare function cVersion(): number;
export declare function supportedRenderBackends(): RenderBackends;
export declare function networkStatus(): NetworkStatusValue;
export declare function setNetworkStatus(status: "online" | "offline"): void;
export declare function projectedMetersForLatLng(
  coordinate: LatLng,
): ProjectedMeters;
export declare function latLngForProjectedMeters(
  meters: ProjectedMeters,
): LatLng;

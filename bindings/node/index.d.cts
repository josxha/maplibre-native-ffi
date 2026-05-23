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

export type MapDebugOption =
  | "tileBorders"
  | "parseStatus"
  | "timestamps"
  | "collision"
  | "overdraw"
  | "stencilClip"
  | "depthBuffer";

export interface CameraOptions {
  center?: LatLng | null;
  zoom?: number | null;
  bearing?: number | null;
  pitch?: number | null;
}

export interface MapOptions {
  width?: number | null;
  height?: number | null;
  scaleFactor?: number | null;
  mapMode?: "continuous" | "static" | "tile" | null;
}

export declare class NativePointer {
  static readonly null: NativePointer;
  static unsafeFromAddress(address: bigint): NativePointer;
  private constructor(address: bigint);
  readonly address: bigint;
  readonly isNull: boolean;
  equals(other: unknown): boolean;
  toString(): string;
}

export declare class NativeBuffer {
  static allocate(byteLength: number): NativeBuffer;
  static from(data: NativeBuffer | ArrayBuffer | ArrayBufferView): NativeBuffer;
  constructor(data: number | ArrayBuffer | ArrayBufferView);
  readonly byteLength: number;
  asArrayBuffer(): ArrayBuffer;
  asUint8Array(): Uint8Array;
  readonly [Symbol.toStringTag]: "NativeBuffer";
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
  renderingStatsViewEnabled: boolean;
  close(): void;
  requestRepaint(): void;
  requestStillImage(): void;
  isFullyLoaded(): boolean;
  dumpDebugLogs(): void;
  getDebugOptions(): MapDebugOption[];
  setDebugOptions(options: Iterable<MapDebugOption>): void;
  getCamera(): CameraOptions;
  jumpTo(camera: CameraOptions): void;
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
export type LogSeverity = "info" | "warning" | "error";

export declare function setNetworkStatus(status: "online" | "offline"): void;
export declare function setAsyncLogSeverities(
  severities: Iterable<LogSeverity>,
): void;
export declare function restoreDefaultAsyncLogSeverities(): void;
export declare function projectedMetersForLatLng(
  coordinate: LatLng,
): ProjectedMeters;
export declare function latLngForProjectedMeters(
  meters: ProjectedMeters,
): LatLng;

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
  centerAltitude?: number | null;
  padding?: EdgeInsets | null;
  anchor?: ScreenPoint | null;
  roll?: number | null;
  fieldOfView?: number | null;
}

export interface UnitBezier {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

export interface AnimationOptions {
  durationMs?: number | null;
  velocity?: number | null;
  minZoom?: number | null;
  easing?: UnitBezier | null;
}

export interface Vec3 {
  x: number;
  y: number;
  z: number;
}

export interface Quaternion {
  x: number;
  y: number;
  z: number;
  w: number;
}

export interface FreeCameraOptions {
  position?: Vec3 | null;
  orientation?: Quaternion | null;
}

export interface EdgeInsets {
  top: number;
  left: number;
  bottom: number;
  right: number;
}

export interface MapViewportOptions {
  northOrientation?: "up" | "right" | "down" | "left" | "unknown" | null;
  constrainMode?:
    | "none"
    | "heightOnly"
    | "widthAndHeight"
    | "screen"
    | "unknown"
    | null;
  viewportMode?: "default" | "flippedY" | "unknown" | null;
  frustumOffset?: EdgeInsets | null;
}

export interface MapTileOptions {
  prefetchZoomDelta?: number | null;
  lodMinRadius?: number | null;
  lodScale?: number | null;
  lodPitchThreshold?: number | null;
  lodZoomShift?: number | null;
  lodMode?: "default" | "distance" | "unknown" | null;
}

export interface BoundOptions {
  bounds?: LatLngBounds | null;
  minZoom?: number | null;
  maxZoom?: number | null;
  minPitch?: number | null;
  maxPitch?: number | null;
}

export interface ProjectionMode {
  axonometric?: boolean | null;
  xSkew?: number | null;
  ySkew?: number | null;
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

export type AmbientCacheOperation =
  | "resetDatabase"
  | "packDatabase"
  | "invalidate"
  | "clear";

export declare class RuntimeHandle {
  constructor(options?: RuntimeOptions | null);
  readonly closed: boolean;
  createMap(options?: MapOptions | null): MapHandle;
  close(): void;
  runOnce(): void;
  runAmbientCacheOperation(
    operation: AmbientCacheOperation,
  ): OfflineOperationHandle;
  pollEvent(): RuntimeEvent | null;
  [Symbol.dispose](): void;
}

export declare class OfflineOperationHandle {
  constructor(runtime: RuntimeHandle, operationId: bigint);
  readonly operationId: bigint;
  readonly closed: boolean;
  close(): void;
  [Symbol.dispose](): void;
}

export declare class MapProjectionHandle {
  constructor(map: MapHandle);
  readonly closed: boolean;
  close(): void;
  getCamera(): CameraOptions;
  setCamera(camera: CameraOptions): void;
  setVisibleCoordinates(coordinates: LatLng[], padding: EdgeInsets): void;
  pixelForLatLng(coordinate: LatLng): ScreenPoint;
  latLngForPixel(point: ScreenPoint): LatLng;
  [Symbol.dispose](): void;
}

export declare class MapHandle {
  constructor(runtime: RuntimeHandle, options?: MapOptions | null);
  readonly closed: boolean;
  renderingStatsViewEnabled: boolean;
  close(): void;
  createProjection(): MapProjectionHandle;
  requestRepaint(): void;
  requestStillImage(): void;
  isFullyLoaded(): boolean;
  dumpDebugLogs(): void;
  getDebugOptions(): MapDebugOption[];
  setDebugOptions(options: Iterable<MapDebugOption>): void;
  getViewportOptions(): MapViewportOptions;
  setViewportOptions(options: MapViewportOptions): void;
  getTileOptions(): MapTileOptions;
  setTileOptions(options: MapTileOptions): void;
  getBounds(): BoundOptions;
  setBounds(options: BoundOptions): void;
  getFreeCameraOptions(): FreeCameraOptions;
  setFreeCameraOptions(options: FreeCameraOptions): void;
  getProjectionMode(): ProjectionMode;
  setProjectionMode(mode: ProjectionMode): void;
  moveBy(deltaX: number, deltaY: number): void;
  scaleBy(scale: number, anchor?: ScreenPoint | null): void;
  rotateBy(first: ScreenPoint, second: ScreenPoint): void;
  pitchBy(pitch: number): void;
  moveByAnimated(
    deltaX: number,
    deltaY: number,
    animation?: AnimationOptions | null,
  ): void;
  scaleByAnimated(
    scale: number,
    anchor?: ScreenPoint | null,
    animation?: AnimationOptions | null,
  ): void;
  rotateByAnimated(
    first: ScreenPoint,
    second: ScreenPoint,
    animation?: AnimationOptions | null,
  ): void;
  pitchByAnimated(pitch: number, animation?: AnimationOptions | null): void;
  cancelTransitions(): void;
  getCamera(): CameraOptions;
  jumpTo(camera: CameraOptions): void;
  easeTo(camera: CameraOptions, animation?: AnimationOptions | null): void;
  flyTo(camera: CameraOptions, animation?: AnimationOptions | null): void;
  cameraForLatLngBounds(bounds: LatLngBounds): CameraOptions;
  cameraForLatLngs(coordinates: LatLng[]): CameraOptions;
  latLngBoundsForCamera(camera: CameraOptions): LatLngBounds;
  latLngBoundsForCameraUnwrapped(camera: CameraOptions): LatLngBounds;
  pixelForLatLng(coordinate: LatLng): ScreenPoint;
  latLngForPixel(point: ScreenPoint): LatLng;
  pixelsForLatLngs(coordinates: LatLng[]): ScreenPoint[];
  latLngsForPixels(points: ScreenPoint[]): LatLng[];
  addStyleSourceJson(sourceId: string, source: JsonValue): void;
  styleSourceExists(sourceId: string): boolean;
  removeStyleSource(sourceId: string): boolean;
  listStyleSourceIds(): string[];
  getStyleSourceType(sourceId: string): StyleSourceType | null;
  getStyleSourceInfo(sourceId: string): StyleSourceInfo | null;
  addGeoJsonSourceUrl(sourceId: string, url: string): void;
  addGeoJsonSourceData(sourceId: string, data: JsonValue): void;
  setGeoJsonSourceUrl(sourceId: string, url: string): void;
  setGeoJsonSourceData(sourceId: string, data: JsonValue): void;
  addVectorSourceUrl(sourceId: string, url: string): void;
  addRasterSourceUrl(sourceId: string, url: string): void;
  addRasterDemSourceUrl(sourceId: string, url: string): void;
  addVectorSourceTiles(sourceId: string, tiles: Iterable<string>): void;
  addRasterSourceTiles(sourceId: string, tiles: Iterable<string>): void;
  addRasterDemSourceTiles(sourceId: string, tiles: Iterable<string>): void;
  setStyleImage(imageId: string, image: StyleImageInput): void;
  styleImageExists(imageId: string): boolean;
  removeStyleImage(imageId: string): boolean;
  getStyleImageInfo(imageId: string): StyleImageInfo | null;
  copyStyleImagePremultipliedRgba8(imageId: string): StyleImage | null;
  addImageSourceUrl(sourceId: string, coordinates: LatLng[], url: string): void;
  addImageSourceImage(
    sourceId: string,
    coordinates: LatLng[],
    image: PremultipliedRgba8ImageInput,
  ): void;
  setImageSourceUrl(sourceId: string, url: string): void;
  setImageSourceImage(
    sourceId: string,
    image: PremultipliedRgba8ImageInput,
  ): void;
  setImageSourceCoordinates(sourceId: string, coordinates: LatLng[]): void;
  getImageSourceCoordinates(sourceId: string): LatLng[] | null;
  addHillshadeLayer(
    layerId: string,
    sourceId: string,
    beforeLayerId?: string | null,
  ): void;
  addColorReliefLayer(
    layerId: string,
    sourceId: string,
    beforeLayerId?: string | null,
  ): void;
  addLocationIndicatorLayer(
    layerId: string,
    beforeLayerId?: string | null,
  ): void;
  setLocationIndicatorLocation(
    layerId: string,
    coordinate: LatLng,
    altitude?: number,
  ): void;
  setLocationIndicatorBearing(layerId: string, bearing: number): void;
  setLocationIndicatorAccuracyRadius(layerId: string, radius: number): void;
  setLocationIndicatorImageName(
    layerId: string,
    imageKind: LocationIndicatorImageKind,
    imageId: string,
  ): void;
  addStyleLayerJson(layer: JsonValue, beforeLayerId?: string | null): void;
  styleLayerExists(layerId: string): boolean;
  removeStyleLayer(layerId: string): boolean;
  listStyleLayerIds(): string[];
  getStyleLayerType(layerId: string): string | null;
  getStyleLayerJson(layerId: string): JsonValue | null;
  moveStyleLayer(layerId: string, beforeLayerId?: string | null): void;
  setLayerProperty(
    layerId: string,
    propertyName: string,
    value: JsonValue,
  ): void;
  getLayerProperty(layerId: string, propertyName: string): JsonValue | null;
  setLayerFilter(layerId: string, filter: JsonValue | null): void;
  getLayerFilter(layerId: string): JsonValue | null;
  setStyleLight(light: JsonValue): void;
  setStyleLightProperty(propertyName: string, value: JsonValue): void;
  getStyleLightProperty(propertyName: string): JsonValue | null;
  setStyleJson(json: string): void;
  setStyleUrl(url: string): void;
  [Symbol.dispose](): void;
}

export interface LatLng {
  latitude: number;
  longitude: number;
}

export interface LatLngBounds {
  southwest: LatLng;
  northeast: LatLng;
}

export interface ProjectedMeters {
  northing: number;
  easting: number;
}

export interface ScreenPoint {
  x: number;
  y: number;
}

export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

export type StyleSourceType =
  | "unknown"
  | "vector"
  | "raster"
  | "raster-dem"
  | "geojson"
  | "image"
  | "video"
  | "annotations"
  | "custom-vector";

export interface StyleSourceInfo {
  sourceType: StyleSourceType;
  rawType: number;
  idSize: number;
  isVolatile: boolean;
  hasAttribution: boolean;
  attributionSize: number;
  attribution?: string | null;
}

export interface PremultipliedRgba8ImageInput {
  width: number;
  height: number;
  stride?: number | null;
  pixels: Uint8Array;
}

export interface StyleImageInput extends PremultipliedRgba8ImageInput {
  pixelRatio?: number | null;
  sdf?: boolean | null;
}

export interface StyleImageInfo {
  width: number;
  height: number;
  stride: number;
  byteLength: number;
  pixelRatio: number;
  sdf: boolean;
}

export interface StyleImage extends StyleImageInfo {
  pixels: Uint8Array;
}

export type LocationIndicatorImageKind = "top" | "bearing" | "shadow";

export declare function cVersion(): number;
export declare function supportedRenderBackends(): RenderBackends;
export declare function networkStatus(): NetworkStatusValue;
export type LogSeverity = "info" | "warning" | "error";

export interface LogRecord {
  severity: LogSeverity | "unknown";
  rawSeverity: number;
  event: string;
  rawEvent: number;
  code: number;
  message: string;
}

export type LogCallback = (record: LogRecord) => void;

export declare function setNetworkStatus(status: "online" | "offline"): void;
export declare function setLogCallback(callback: LogCallback): void;
export declare function clearLogCallback(): void;
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

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

export type ResourceKind =
  | "unknown"
  | "style"
  | "source"
  | "tile"
  | "glyphs"
  | "sprite-image"
  | "sprite-json"
  | "image";

export interface ResourceTransformRequest {
  kind: ResourceKind;
  rawKind: number;
  url: string;
}

export type ResourceTransformCallback = (
  request: ResourceTransformRequest,
) => string | null | undefined;

export interface ResourceByteRange {
  start: string;
  end: string;
}

export interface ResourceProviderRequest {
  url: string;
  kind: ResourceKind;
  rawKind: number;
  loadingMethod: "all" | "cacheOnly" | "networkOnly" | "unknown";
  rawLoadingMethod: number;
  priority: "regular" | "low" | "unknown";
  rawPriority: number;
  usage: "online" | "offline" | "unknown";
  rawUsage: number;
  storagePolicy: "permanent" | "volatile" | "unknown";
  rawStoragePolicy: number;
  range?: ResourceByteRange | null;
  priorModifiedUnixMs?: number | null;
  priorExpiresUnixMs?: number | null;
  priorEtag?: string | null;
  priorData: Uint8Array;
  handle: ResourceRequestHandle;
}

export interface ResourceResponseInput {
  status?: "ok" | "error" | "noContent" | "notModified" | null;
  errorReason?:
    | "none"
    | "notFound"
    | "server"
    | "connection"
    | "rateLimit"
    | "other"
    | null;
  bytes?: Uint8Array | null;
  errorMessage?: string | null;
  mustRevalidate?: boolean | null;
  modifiedUnixMs?: number | null;
  expiresUnixMs?: number | null;
  etag?: string | null;
  retryAfterUnixMs?: number | null;
}

export type ResourceProviderDecision = "passThrough" | "handle";

export type ResourceProviderCallback = (
  request: ResourceProviderRequest,
) => ResourceProviderDecision | null | undefined;

export declare class ResourceRequestHandle {
  private constructor(nativeHandle: unknown);
  readonly closed: boolean;
  complete(response?: ResourceResponseInput): void;
  cancelled(): boolean;
  close(): void;
  [Symbol.dispose](): void;
}

export declare class RuntimeHandle {
  constructor(options?: RuntimeOptions | null);
  readonly closed: boolean;
  createMap(options?: MapOptions | null): MapHandle;
  close(): void;
  runOnce(): void;
  setResourceTransform(callback: ResourceTransformCallback): void;
  clearResourceTransform(): void;
  setResourceProvider(callback: ResourceProviderCallback): void;
  runAmbientCacheOperation(
    operation: AmbientCacheOperation,
  ): OfflineOperationHandle;
  offlineRegionsList(): OfflineOperationHandle;
  offlineRegionGet(regionId: bigint): OfflineOperationHandle;
  offlineRegionsMergeDatabase(path: string): OfflineOperationHandle;
  offlineRegionUpdateMetadata(
    regionId: bigint,
    metadata?: Uint8Array | null,
  ): OfflineOperationHandle;
  offlineRegionGetStatus(regionId: bigint): OfflineOperationHandle;
  offlineRegionSetObserved(
    regionId: bigint,
    observed: boolean,
  ): OfflineOperationHandle;
  offlineRegionSetDownloadState(
    regionId: bigint,
    state: OfflineRegionDownloadState,
  ): OfflineOperationHandle;
  offlineRegionInvalidate(regionId: bigint): OfflineOperationHandle;
  offlineRegionDelete(regionId: bigint): OfflineOperationHandle;
  offlineRegionCreate(
    definition: OfflineRegionDefinition,
    metadata?: Uint8Array | null,
  ): OfflineOperationHandle;
  offlineRegionCreateTakeResult(
    operation: OfflineOperationRef,
  ): OfflineRegionInfo;
  offlineRegionGetTakeResult(
    operation: OfflineOperationRef,
  ): OfflineRegionInfo | null;
  offlineRegionsListTakeResult(
    operation: OfflineOperationRef,
  ): OfflineRegionInfo[];
  offlineRegionsMergeDatabaseTakeResult(
    operation: OfflineOperationRef,
  ): OfflineRegionInfo[];
  offlineRegionUpdateMetadataTakeResult(
    operation: OfflineOperationRef,
  ): OfflineRegionInfo;
  offlineRegionGetStatusTakeResult(
    operation: OfflineOperationRef,
  ): OfflineRegionStatus;
  pollEvent(): RuntimeEvent | null;
  [Symbol.dispose](): void;
}

export interface OfflineTilePyramidRegionDefinition {
  kind: "tilePyramid";
  styleUrl: string;
  bounds: LatLngBounds;
  minZoom: number;
  maxZoom: number;
  pixelRatio: number;
  includeIdeographs?: boolean | null;
}

export interface OfflineGeometryRegionDefinition {
  kind: "geometry";
  styleUrl: string;
  geometry: JsonValue;
  minZoom: number;
  maxZoom: number;
  pixelRatio: number;
  includeIdeographs?: boolean | null;
}

export type OfflineRegionDefinition =
  | OfflineTilePyramidRegionDefinition
  | OfflineGeometryRegionDefinition;

export interface OfflineRegionDefinitionValue {
  kind: "tilePyramid" | "geometry";
  styleUrl: string;
  bounds?: LatLngBounds | null;
  geometry?: string | null;
  minZoom: number;
  maxZoom: number;
  pixelRatio: number;
  includeIdeographs: boolean;
}

export interface OfflineRegionInfo {
  id: string;
  definition: OfflineRegionDefinitionValue;
  metadata: Uint8Array;
}

export interface OfflineRegionStatus {
  downloadState: OfflineRegionDownloadState | "unknown";
  rawDownloadState: number;
  completedResourceCount: string;
  completedResourceSize: string;
  completedTileCount: string;
  completedTileSize: string;
  requiredResourceCount: string;
  requiredResourceCountIsPrecise: boolean;
}

export type OfflineOperationRef = OfflineOperationHandle | bigint;

export type OfflineRegionDownloadState = "inactive" | "active";

export declare class OfflineOperationHandle {
  constructor(runtime: RuntimeHandle, operationId: bigint);
  readonly operationId: bigint;
  readonly closed: boolean;
  close(): void;
  [Symbol.dispose](): void;
}

export interface RenderTargetExtent {
  width: number;
  height: number;
  scaleFactor: number;
}

export interface MetalContextDescriptor {
  deviceAddress?: bigint | null;
}

export interface MetalOwnedTextureDescriptor {
  extent: RenderTargetExtent;
  context: MetalContextDescriptor;
}

export interface MetalBorrowedTextureDescriptor {
  extent: RenderTargetExtent;
  textureAddress: bigint;
}

export interface MetalSurfaceDescriptor {
  extent: RenderTargetExtent;
  context: MetalContextDescriptor;
  layerAddress: bigint;
}

export interface VulkanContextDescriptor {
  instanceAddress: bigint;
  physicalDeviceAddress: bigint;
  deviceAddress: bigint;
  graphicsQueueAddress: bigint;
  graphicsQueueFamilyIndex: number;
}

export interface VulkanOwnedTextureDescriptor {
  extent: RenderTargetExtent;
  context: VulkanContextDescriptor;
}

export interface VulkanBorrowedTextureDescriptor {
  extent: RenderTargetExtent;
  context: VulkanContextDescriptor;
  imageAddress: bigint;
  imageViewAddress: bigint;
  format: number;
  initialLayout: number;
  finalLayout: number;
}

export interface VulkanSurfaceDescriptor {
  extent: RenderTargetExtent;
  context: VulkanContextDescriptor;
  surfaceAddress: bigint;
}

export interface TextureImageInfo {
  width: number;
  height: number;
  stride: number;
  byteLength: number;
}

export interface TextureReadback {
  info: TextureImageInfo;
  pixels: Uint8Array;
}

export declare class MetalOwnedTextureFrame {
  private constructor(nativeFrame: unknown);
  readonly generation: bigint;
  readonly width: number;
  readonly height: number;
  readonly scaleFactor: number;
  readonly frameId: bigint;
  readonly texture: NativePointer;
  readonly device: NativePointer;
  readonly pixelFormat: bigint;
}

export declare class VulkanOwnedTextureFrame {
  private constructor(nativeFrame: unknown);
  readonly generation: bigint;
  readonly width: number;
  readonly height: number;
  readonly scaleFactor: number;
  readonly frameId: bigint;
  readonly image: NativePointer;
  readonly imageView: NativePointer;
  readonly device: NativePointer;
  readonly format: number;
  readonly layout: number;
}

export interface FeatureStateSelector {
  sourceId: string;
  sourceLayerId?: string | null;
  featureId?: string | null;
  stateKey?: string | null;
}

export interface ScreenBox {
  min: ScreenPoint;
  max: ScreenPoint;
}

export type RenderedQueryGeometry =
  | { kind: "point"; point: ScreenPoint }
  | { kind: "box"; box: ScreenBox }
  | { kind: "lineString"; points: ScreenPoint[] };

export interface RenderedFeatureQueryOptions {
  layerIds?: string[] | null;
  filter?: JsonValue | null;
}

export interface SourceFeatureQueryOptions {
  sourceLayerIds?: string[] | null;
  filter?: JsonValue | null;
}

export interface QueriedFeature {
  feature: JsonValue;
  sourceId?: string | null;
  sourceLayerId?: string | null;
  state?: JsonValue | null;
}

export declare class RenderSessionHandle {
  private constructor(nativeHandle: unknown, map: MapHandle);
  readonly map: MapHandle;
  static attachMetalOwnedTexture(
    map: MapHandle,
    descriptor: MetalOwnedTextureDescriptor,
  ): RenderSessionHandle;
  static attachMetalBorrowedTexture(
    map: MapHandle,
    descriptor: MetalBorrowedTextureDescriptor,
  ): RenderSessionHandle;
  static attachMetalSurface(
    map: MapHandle,
    descriptor: MetalSurfaceDescriptor,
  ): RenderSessionHandle;
  static attachVulkanOwnedTexture(
    map: MapHandle,
    descriptor: VulkanOwnedTextureDescriptor,
  ): RenderSessionHandle;
  static attachVulkanBorrowedTexture(
    map: MapHandle,
    descriptor: VulkanBorrowedTextureDescriptor,
  ): RenderSessionHandle;
  static attachVulkanSurface(
    map: MapHandle,
    descriptor: VulkanSurfaceDescriptor,
  ): RenderSessionHandle;
  readonly closed: boolean;
  close(): void;
  resize(width: number, height: number, scaleFactor: number): void;
  renderUpdate(): void;
  detach(): void;
  reduceMemoryUse(): void;
  clearData(): void;
  dumpDebugLogs(): void;
  setFeatureState(selector: FeatureStateSelector, state: JsonValue): void;
  getFeatureState(selector: FeatureStateSelector): JsonValue;
  removeFeatureState(selector: FeatureStateSelector): void;
  queryRenderedFeatures(
    geometry: RenderedQueryGeometry,
    options?: RenderedFeatureQueryOptions | null,
  ): QueriedFeature[];
  querySourceFeatures(
    sourceId: string,
    options?: SourceFeatureQueryOptions | null,
  ): QueriedFeature[];
  queryFeatureExtension(
    sourceId: string,
    feature: JsonValue,
    extension: string,
    extensionField: string,
    args?: JsonValue | null,
  ): JsonValue;
  withMetalOwnedTextureFrame<T>(
    callback: (frame: MetalOwnedTextureFrame) => T,
  ): T;
  withVulkanOwnedTextureFrame<T>(
    callback: (frame: VulkanOwnedTextureFrame) => T,
  ): T;
  readPremultipliedRgba8(): TextureReadback;
  [Symbol.dispose](): void;
}

export declare class MapProjectionHandle {
  constructor(map: MapHandle);
  readonly closed: boolean;
  close(): void;
  getCamera(): CameraOptions;
  setCamera(camera: CameraOptions): void;
  setVisibleCoordinates(coordinates: LatLng[], padding: EdgeInsets): void;
  setVisibleGeometry(geometry: JsonValue, padding: EdgeInsets): void;
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
  attachMetalOwnedTexture(
    descriptor: MetalOwnedTextureDescriptor,
  ): RenderSessionHandle;
  attachMetalBorrowedTexture(
    descriptor: MetalBorrowedTextureDescriptor,
  ): RenderSessionHandle;
  attachMetalSurface(descriptor: MetalSurfaceDescriptor): RenderSessionHandle;
  attachVulkanOwnedTexture(
    descriptor: VulkanOwnedTextureDescriptor,
  ): RenderSessionHandle;
  attachVulkanBorrowedTexture(
    descriptor: VulkanBorrowedTextureDescriptor,
  ): RenderSessionHandle;
  attachVulkanSurface(descriptor: VulkanSurfaceDescriptor): RenderSessionHandle;
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
  cameraForGeometry(geometry: JsonValue): CameraOptions;
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
  addCustomGeometrySource(
    sourceId: string,
    options?: CustomGeometrySourceOptions | null,
  ): void;
  setCustomGeometrySourceTileData(
    sourceId: string,
    tileId: CanonicalTileId,
    data: JsonValue,
  ): void;
  invalidateCustomGeometrySourceTile(
    sourceId: string,
    tileId: CanonicalTileId,
  ): void;
  invalidateCustomGeometrySourceRegion(
    sourceId: string,
    bounds: LatLngBounds,
  ): void;
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

export interface CanonicalTileId {
  z: number;
  x: number;
  y: number;
}

export type CustomGeometrySourceCallback = (tileId: CanonicalTileId) => void;

export interface CustomGeometrySourceOptions {
  fetchTile?: CustomGeometrySourceCallback | null;
  cancelTile?: CustomGeometrySourceCallback | null;
  minZoom?: number | null;
  maxZoom?: number | null;
  tolerance?: number | null;
  tileSize?: number | null;
  buffer?: number | null;
  clip?: boolean | null;
  wrap?: boolean | null;
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
export declare function threadLastErrorMessage(): string;
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

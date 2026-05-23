#pragma once

#include <glib-object.h>
#include <stdbool.h>
#include <stdint.h>

G_BEGIN_DECLS

#define MLN_VALA_ERROR (mln_vala_error_quark())

typedef enum {
  MLN_VALA_ERROR_INVALID_ARGUMENT,
  MLN_VALA_ERROR_INVALID_STATE,
  MLN_VALA_ERROR_WRONG_THREAD,
  MLN_VALA_ERROR_UNSUPPORTED,
  MLN_VALA_ERROR_NATIVE_ERROR,
  MLN_VALA_ERROR_UNKNOWN_STATUS,
} MlnValaError;

typedef enum {
  MLN_VALA_RENDER_BACKEND_FLAGS_METAL = 1u << 0u,
  MLN_VALA_RENDER_BACKEND_FLAGS_VULKAN = 1u << 1u,
} MlnValaRenderBackendFlags;

typedef enum {
  MLN_VALA_NETWORK_STATUS_ONLINE = 1,
  MLN_VALA_NETWORK_STATUS_OFFLINE = 2,
} MlnValaNetworkStatus;

typedef enum {
  MLN_VALA_RUNTIME_OPTION_FLAGS_MAXIMUM_CACHE_SIZE = 1u << 0u,
} MlnValaRuntimeOptionFlags;

typedef enum {
  MLN_VALA_AMBIENT_CACHE_OPERATION_RESET_DATABASE = 1,
  MLN_VALA_AMBIENT_CACHE_OPERATION_PACK_DATABASE = 2,
  MLN_VALA_AMBIENT_CACHE_OPERATION_INVALIDATE = 3,
  MLN_VALA_AMBIENT_CACHE_OPERATION_CLEAR = 4,
} MlnValaAmbientCacheOperation;

typedef enum {
  MLN_VALA_MAP_MODE_CONTINUOUS = 0,
  MLN_VALA_MAP_MODE_STATIC = 1,
  MLN_VALA_MAP_MODE_TILE = 2,
} MlnValaMapMode;

typedef enum {
  MLN_VALA_RESOURCE_KIND_UNKNOWN = 0,
  MLN_VALA_RESOURCE_KIND_STYLE = 1,
  MLN_VALA_RESOURCE_KIND_SOURCE = 2,
  MLN_VALA_RESOURCE_KIND_TILE = 3,
  MLN_VALA_RESOURCE_KIND_GLYPHS = 4,
  MLN_VALA_RESOURCE_KIND_SPRITE_IMAGE = 5,
  MLN_VALA_RESOURCE_KIND_SPRITE_JSON = 6,
  MLN_VALA_RESOURCE_KIND_IMAGE = 7,
} MlnValaResourceKind;

typedef enum {
  MLN_VALA_RESOURCE_LOADING_METHOD_ALL = 0,
  MLN_VALA_RESOURCE_LOADING_METHOD_CACHE_ONLY = 1,
  MLN_VALA_RESOURCE_LOADING_METHOD_NETWORK_ONLY = 2,
} MlnValaResourceLoadingMethod;

typedef enum {
  MLN_VALA_RESOURCE_PRIORITY_REGULAR = 0,
  MLN_VALA_RESOURCE_PRIORITY_LOW = 1,
} MlnValaResourcePriority;

typedef enum {
  MLN_VALA_RESOURCE_USAGE_ONLINE = 0,
  MLN_VALA_RESOURCE_USAGE_OFFLINE = 1,
} MlnValaResourceUsage;

typedef enum {
  MLN_VALA_RESOURCE_STORAGE_POLICY_PERMANENT = 0,
  MLN_VALA_RESOURCE_STORAGE_POLICY_VOLATILE = 1,
} MlnValaResourceStoragePolicy;

typedef enum {
  MLN_VALA_RESOURCE_RESPONSE_STATUS_OK = 0,
  MLN_VALA_RESOURCE_RESPONSE_STATUS_ERROR = 1,
  MLN_VALA_RESOURCE_RESPONSE_STATUS_NO_CONTENT = 2,
  MLN_VALA_RESOURCE_RESPONSE_STATUS_NOT_MODIFIED = 3,
} MlnValaResourceResponseStatus;

typedef enum {
  MLN_VALA_RESOURCE_ERROR_REASON_NONE = 0,
  MLN_VALA_RESOURCE_ERROR_REASON_NOT_FOUND = 1,
  MLN_VALA_RESOURCE_ERROR_REASON_SERVER = 2,
  MLN_VALA_RESOURCE_ERROR_REASON_CONNECTION = 3,
  MLN_VALA_RESOURCE_ERROR_REASON_RATE_LIMIT = 4,
  MLN_VALA_RESOURCE_ERROR_REASON_OTHER = 5,
} MlnValaResourceErrorReason;

typedef enum {
  MLN_VALA_RESOURCE_PROVIDER_DECISION_PASS_THROUGH = 0,
  MLN_VALA_RESOURCE_PROVIDER_DECISION_HANDLE = 1,
} MlnValaResourceProviderDecision;

typedef enum {
  MLN_VALA_STYLE_SOURCE_TYPE_UNKNOWN = 0,
  MLN_VALA_STYLE_SOURCE_TYPE_VECTOR = 1,
  MLN_VALA_STYLE_SOURCE_TYPE_RASTER = 2,
  MLN_VALA_STYLE_SOURCE_TYPE_RASTER_DEM = 3,
  MLN_VALA_STYLE_SOURCE_TYPE_GEOJSON = 4,
  MLN_VALA_STYLE_SOURCE_TYPE_IMAGE = 5,
  MLN_VALA_STYLE_SOURCE_TYPE_VIDEO = 6,
  MLN_VALA_STYLE_SOURCE_TYPE_ANNOTATIONS = 7,
  MLN_VALA_STYLE_SOURCE_TYPE_CUSTOM_VECTOR = 8,
} MlnValaStyleSourceType;

typedef enum {
  MLN_VALA_STYLE_TILE_SOURCE_OPTION_FIELDS_MIN_ZOOM = 1u << 0u,
  MLN_VALA_STYLE_TILE_SOURCE_OPTION_FIELDS_MAX_ZOOM = 1u << 1u,
  MLN_VALA_STYLE_TILE_SOURCE_OPTION_FIELDS_ATTRIBUTION = 1u << 2u,
  MLN_VALA_STYLE_TILE_SOURCE_OPTION_FIELDS_SCHEME = 1u << 3u,
  MLN_VALA_STYLE_TILE_SOURCE_OPTION_FIELDS_BOUNDS = 1u << 4u,
  MLN_VALA_STYLE_TILE_SOURCE_OPTION_FIELDS_TILE_SIZE = 1u << 5u,
  MLN_VALA_STYLE_TILE_SOURCE_OPTION_FIELDS_VECTOR_ENCODING = 1u << 6u,
  MLN_VALA_STYLE_TILE_SOURCE_OPTION_FIELDS_RASTER_ENCODING = 1u << 7u,
} MlnValaStyleTileSourceOptionFields;

typedef enum {
  MLN_VALA_STYLE_TILE_SCHEME_XYZ = 0,
  MLN_VALA_STYLE_TILE_SCHEME_TMS = 1,
} MlnValaStyleTileScheme;

typedef enum {
  MLN_VALA_STYLE_VECTOR_TILE_ENCODING_MVT = 0,
  MLN_VALA_STYLE_VECTOR_TILE_ENCODING_MLT = 1,
} MlnValaStyleVectorTileEncoding;

typedef enum {
  MLN_VALA_STYLE_RASTER_DEM_ENCODING_MAPBOX = 0,
  MLN_VALA_STYLE_RASTER_DEM_ENCODING_TERRARIUM = 1,
} MlnValaStyleRasterDemEncoding;

/**
 * MlnValaResourceTransformCallback:
 * @kind: resource kind.
 * @url: (not nullable): original network URL.
 *
 * Returns: (transfer full) (nullable): replacement URL, or `NULL` to keep the
 * original URL.
 */
typedef char* (*MlnValaResourceTransformCallback)(
  MlnValaResourceKind kind, const char* url, gpointer user_data
);

typedef enum {
  MLN_VALA_LOG_SEVERITY_INFO = 1,
  MLN_VALA_LOG_SEVERITY_WARNING = 2,
  MLN_VALA_LOG_SEVERITY_ERROR = 3,
} MlnValaLogSeverity;

typedef enum {
  MLN_VALA_LOG_SEVERITY_FLAGS_INFO = 1u << 1u,
  MLN_VALA_LOG_SEVERITY_FLAGS_WARNING = 1u << 2u,
  MLN_VALA_LOG_SEVERITY_FLAGS_ERROR = 1u << 3u,
  MLN_VALA_LOG_SEVERITY_FLAGS_DEFAULT =
    MLN_VALA_LOG_SEVERITY_FLAGS_INFO | MLN_VALA_LOG_SEVERITY_FLAGS_WARNING,
  MLN_VALA_LOG_SEVERITY_FLAGS_ALL = MLN_VALA_LOG_SEVERITY_FLAGS_INFO |
                                    MLN_VALA_LOG_SEVERITY_FLAGS_WARNING |
                                    MLN_VALA_LOG_SEVERITY_FLAGS_ERROR,
} MlnValaLogSeverityFlags;

typedef enum {
  MLN_VALA_LOG_EVENT_GENERAL = 0,
  MLN_VALA_LOG_EVENT_SETUP = 1,
  MLN_VALA_LOG_EVENT_SHADER = 2,
  MLN_VALA_LOG_EVENT_PARSE_STYLE = 3,
  MLN_VALA_LOG_EVENT_PARSE_TILE = 4,
  MLN_VALA_LOG_EVENT_RENDER = 5,
  MLN_VALA_LOG_EVENT_STYLE = 6,
  MLN_VALA_LOG_EVENT_DATABASE = 7,
  MLN_VALA_LOG_EVENT_HTTP_REQUEST = 8,
  MLN_VALA_LOG_EVENT_SPRITE = 9,
  MLN_VALA_LOG_EVENT_IMAGE = 10,
  MLN_VALA_LOG_EVENT_OPENGL = 11,
  MLN_VALA_LOG_EVENT_JNI = 12,
  MLN_VALA_LOG_EVENT_ANDROID = 13,
  MLN_VALA_LOG_EVENT_CRASH = 14,
  MLN_VALA_LOG_EVENT_GLYPH = 15,
  MLN_VALA_LOG_EVENT_TIMING = 16,
} MlnValaLogEvent;

/**
 * MlnValaLogCallback:
 * @severity: log severity.
 * @event: log event category.
 * @code: native log code.
 * @message: (nullable): borrowed log message for the callback duration.
 *
 * Returns: `TRUE` to consume the record, `FALSE` to let native logging handle
 * it.
 */
typedef gboolean (*MlnValaLogCallback)(
  MlnValaLogSeverity severity, MlnValaLogEvent event, int64_t code,
  const char* message, gpointer user_data
);

typedef enum {
  MLN_VALA_MAP_DEBUG_OPTIONS_TILE_BORDERS = 1u << 1u,
  MLN_VALA_MAP_DEBUG_OPTIONS_PARSE_STATUS = 1u << 2u,
  MLN_VALA_MAP_DEBUG_OPTIONS_TIMESTAMPS = 1u << 3u,
  MLN_VALA_MAP_DEBUG_OPTIONS_COLLISION = 1u << 4u,
  MLN_VALA_MAP_DEBUG_OPTIONS_OVERDRAW = 1u << 5u,
  MLN_VALA_MAP_DEBUG_OPTIONS_STENCIL_CLIP = 1u << 6u,
  MLN_VALA_MAP_DEBUG_OPTIONS_DEPTH_BUFFER = 1u << 7u,
} MlnValaMapDebugOptions;

typedef enum {
  MLN_VALA_CAMERA_OPTION_FIELDS_CENTER = 1u << 0u,
  MLN_VALA_CAMERA_OPTION_FIELDS_ZOOM = 1u << 1u,
  MLN_VALA_CAMERA_OPTION_FIELDS_BEARING = 1u << 2u,
  MLN_VALA_CAMERA_OPTION_FIELDS_PITCH = 1u << 3u,
  MLN_VALA_CAMERA_OPTION_FIELDS_CENTER_ALTITUDE = 1u << 4u,
  MLN_VALA_CAMERA_OPTION_FIELDS_PADDING = 1u << 5u,
  MLN_VALA_CAMERA_OPTION_FIELDS_ANCHOR = 1u << 6u,
  MLN_VALA_CAMERA_OPTION_FIELDS_ROLL = 1u << 7u,
  MLN_VALA_CAMERA_OPTION_FIELDS_FOV = 1u << 8u,
} MlnValaCameraOptionFields;

typedef enum {
  MLN_VALA_ANIMATION_OPTION_FIELDS_DURATION = 1u << 0u,
  MLN_VALA_ANIMATION_OPTION_FIELDS_VELOCITY = 1u << 1u,
  MLN_VALA_ANIMATION_OPTION_FIELDS_MIN_ZOOM = 1u << 2u,
  MLN_VALA_ANIMATION_OPTION_FIELDS_EASING = 1u << 3u,
} MlnValaAnimationOptionFields;

typedef enum {
  MLN_VALA_BOUND_OPTION_FIELDS_BOUNDS = 1u << 0u,
  MLN_VALA_BOUND_OPTION_FIELDS_MIN_ZOOM = 1u << 1u,
  MLN_VALA_BOUND_OPTION_FIELDS_MAX_ZOOM = 1u << 2u,
  MLN_VALA_BOUND_OPTION_FIELDS_MIN_PITCH = 1u << 3u,
  MLN_VALA_BOUND_OPTION_FIELDS_MAX_PITCH = 1u << 4u,
} MlnValaBoundOptionFields;

typedef enum {
  MLN_VALA_FREE_CAMERA_OPTION_FIELDS_POSITION = 1u << 0u,
  MLN_VALA_FREE_CAMERA_OPTION_FIELDS_ORIENTATION = 1u << 1u,
} MlnValaFreeCameraOptionFields;

typedef enum {
  MLN_VALA_PROJECTION_MODE_FIELDS_AXONOMETRIC = 1u << 0u,
  MLN_VALA_PROJECTION_MODE_FIELDS_X_SKEW = 1u << 1u,
  MLN_VALA_PROJECTION_MODE_FIELDS_Y_SKEW = 1u << 2u,
} MlnValaProjectionModeFields;

typedef enum {
  MLN_VALA_MAP_VIEWPORT_OPTION_FIELDS_NORTH_ORIENTATION = 1u << 0u,
  MLN_VALA_MAP_VIEWPORT_OPTION_FIELDS_CONSTRAIN_MODE = 1u << 1u,
  MLN_VALA_MAP_VIEWPORT_OPTION_FIELDS_VIEWPORT_MODE = 1u << 2u,
  MLN_VALA_MAP_VIEWPORT_OPTION_FIELDS_FRUSTUM_OFFSET = 1u << 3u,
} MlnValaMapViewportOptionFields;

typedef enum {
  MLN_VALA_MAP_TILE_OPTION_FIELDS_PREFETCH_ZOOM_DELTA = 1u << 0u,
  MLN_VALA_MAP_TILE_OPTION_FIELDS_LOD_MIN_RADIUS = 1u << 1u,
  MLN_VALA_MAP_TILE_OPTION_FIELDS_LOD_SCALE = 1u << 2u,
  MLN_VALA_MAP_TILE_OPTION_FIELDS_LOD_PITCH_THRESHOLD = 1u << 3u,
  MLN_VALA_MAP_TILE_OPTION_FIELDS_LOD_ZOOM_SHIFT = 1u << 4u,
  MLN_VALA_MAP_TILE_OPTION_FIELDS_LOD_MODE = 1u << 5u,
} MlnValaMapTileOptionFields;

typedef enum {
  MLN_VALA_NORTH_ORIENTATION_UP = 0,
  MLN_VALA_NORTH_ORIENTATION_RIGHT = 1,
  MLN_VALA_NORTH_ORIENTATION_DOWN = 2,
  MLN_VALA_NORTH_ORIENTATION_LEFT = 3,
} MlnValaNorthOrientation;

typedef enum {
  MLN_VALA_CONSTRAIN_MODE_NONE = 0,
  MLN_VALA_CONSTRAIN_MODE_HEIGHT_ONLY = 1,
  MLN_VALA_CONSTRAIN_MODE_WIDTH_AND_HEIGHT = 2,
  MLN_VALA_CONSTRAIN_MODE_SCREEN = 3,
} MlnValaConstrainMode;

typedef enum {
  MLN_VALA_VIEWPORT_MODE_DEFAULT = 0,
  MLN_VALA_VIEWPORT_MODE_FLIPPED_Y = 1,
} MlnValaViewportMode;

typedef enum {
  MLN_VALA_TILE_LOD_MODE_DEFAULT = 0,
  MLN_VALA_TILE_LOD_MODE_DISTANCE = 1,
} MlnValaTileLodMode;

GQuark mln_vala_error_quark(void);

typedef struct {
  double latitude;
  double longitude;
} MlnValaLatLng;

typedef struct {
  double northing;
  double easting;
} MlnValaProjectedMeters;

typedef struct {
  double x;
  double y;
} MlnValaScreenPoint;

typedef struct {
  uint32_t size;
  MlnValaRuntimeOptionFlags flags;
  const char* asset_path;
  const char* cache_path;
  uint64_t maximum_cache_size;
} MlnValaRuntimeOptions;

typedef struct {
  uint32_t size;
  uint32_t width;
  uint32_t height;
  double scale_factor;
  MlnValaMapMode map_mode;
} MlnValaMapOptions;

typedef struct {
  uint32_t size;
  MlnValaResourceResponseStatus status;
  MlnValaResourceErrorReason error_reason;
  const uint8_t* bytes;
  size_t byte_count;
  const char* error_message;
  bool must_revalidate;
  bool has_modified;
  int64_t modified_unix_ms;
  bool has_expires;
  int64_t expires_unix_ms;
  const char* etag;
  bool has_retry_after;
  int64_t retry_after_unix_ms;
} MlnValaResourceResponse;

/**
 * mln_vala_runtime_options_default:
 * @out_options: (out): return location for initialized runtime options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_runtime_options_default(
  MlnValaRuntimeOptions* out_options, GError** error
);

/**
 * mln_vala_map_options_default:
 * @out_options: (out): return location for initialized map options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_options_default(
  MlnValaMapOptions* out_options, GError** error
);

typedef struct {
  double top;
  double left;
  double bottom;
  double right;
} MlnValaEdgeInsets;

typedef struct {
  double x1;
  double y1;
  double x2;
  double y2;
} MlnValaUnitBezier;

typedef struct {
  uint32_t size;
  MlnValaCameraOptionFields fields;
  double latitude;
  double longitude;
  double center_altitude;
  MlnValaEdgeInsets padding;
  MlnValaScreenPoint anchor;
  double zoom;
  double bearing;
  double pitch;
  double roll;
  double field_of_view;
} MlnValaCameraOptions;

typedef struct {
  uint32_t size;
  MlnValaAnimationOptionFields fields;
  double duration_ms;
  double velocity;
  double min_zoom;
  MlnValaUnitBezier easing;
} MlnValaAnimationOptions;

typedef struct {
  MlnValaLatLng southwest;
  MlnValaLatLng northeast;
} MlnValaLatLngBounds;

typedef struct {
  uint32_t size;
  MlnValaBoundOptionFields fields;
  MlnValaLatLngBounds bounds;
  double min_zoom;
  double max_zoom;
  double min_pitch;
  double max_pitch;
} MlnValaBoundOptions;

typedef struct {
  double x;
  double y;
  double z;
} MlnValaVec3;

typedef struct {
  double x;
  double y;
  double z;
  double w;
} MlnValaQuaternion;

typedef struct {
  uint32_t size;
  MlnValaFreeCameraOptionFields fields;
  MlnValaVec3 position;
  MlnValaQuaternion orientation;
} MlnValaFreeCameraOptions;

typedef struct {
  uint32_t size;
  MlnValaProjectionModeFields fields;
  bool axonometric;
  double x_skew;
  double y_skew;
} MlnValaProjectionMode;

typedef struct {
  uint32_t size;
  MlnValaMapViewportOptionFields fields;
  MlnValaNorthOrientation north_orientation;
  MlnValaConstrainMode constrain_mode;
  MlnValaViewportMode viewport_mode;
  MlnValaEdgeInsets frustum_offset;
} MlnValaMapViewportOptions;

typedef struct {
  uint32_t size;
  MlnValaMapTileOptionFields fields;
  uint32_t prefetch_zoom_delta;
  double lod_min_radius;
  double lod_scale;
  double lod_pitch_threshold;
  double lod_zoom_shift;
  MlnValaTileLodMode lod_mode;
} MlnValaMapTileOptions;

typedef struct {
  uint32_t size;
  MlnValaStyleSourceType type;
  size_t id_size;
  bool is_volatile;
  bool has_attribution;
  size_t attribution_size;
} MlnValaStyleSourceInfo;

typedef struct {
  uint32_t size;
  MlnValaStyleTileSourceOptionFields fields;
  double min_zoom;
  double max_zoom;
  const char* attribution;
  size_t attribution_size;
  MlnValaStyleTileScheme scheme;
  MlnValaLatLngBounds bounds;
  uint32_t tile_size;
  MlnValaStyleVectorTileEncoding vector_encoding;
  MlnValaStyleRasterDemEncoding raster_encoding;
} MlnValaStyleTileSourceOptions;

typedef struct {
  uint32_t size;
  uint32_t width;
  uint32_t height;
  double scale_factor;
} MlnValaRenderTargetExtent;

typedef struct {
  uint32_t size;
  gpointer device;
} MlnValaMetalContextDescriptor;

typedef struct {
  uint32_t size;
  gpointer instance;
  gpointer physical_device;
  gpointer device;
  gpointer graphics_queue;
  uint32_t graphics_queue_family_index;
} MlnValaVulkanContextDescriptor;

typedef struct {
  uint32_t size;
  MlnValaRenderTargetExtent extent;
  MlnValaMetalContextDescriptor context;
  gpointer layer;
} MlnValaMetalSurfaceDescriptor;

typedef struct {
  uint32_t size;
  MlnValaRenderTargetExtent extent;
  MlnValaVulkanContextDescriptor context;
  gpointer surface;
} MlnValaVulkanSurfaceDescriptor;

typedef struct {
  uint32_t size;
  MlnValaRenderTargetExtent extent;
  MlnValaMetalContextDescriptor context;
} MlnValaMetalOwnedTextureDescriptor;

typedef struct {
  uint32_t size;
  MlnValaRenderTargetExtent extent;
  gpointer texture;
} MlnValaMetalBorrowedTextureDescriptor;

typedef struct {
  uint32_t size;
  MlnValaRenderTargetExtent extent;
  MlnValaVulkanContextDescriptor context;
} MlnValaVulkanOwnedTextureDescriptor;

typedef struct {
  uint32_t size;
  MlnValaRenderTargetExtent extent;
  MlnValaVulkanContextDescriptor context;
  gpointer image;
  gpointer image_view;
  uint32_t format;
  uint32_t initial_layout;
  uint32_t final_layout;
} MlnValaVulkanBorrowedTextureDescriptor;

typedef struct {
  uint32_t size;
  uint32_t width;
  uint32_t height;
  uint32_t stride;
  size_t byte_length;
} MlnValaTextureImageInfo;

/**
 * mln_vala_camera_options_default:
 * @out_options: (out): return location for initialized camera options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_camera_options_default(
  MlnValaCameraOptions* out_options, GError** error
);

/**
 * mln_vala_animation_options_default:
 * @out_options: (out): return location for initialized animation options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_animation_options_default(
  MlnValaAnimationOptions* out_options, GError** error
);

gboolean mln_vala_bound_options_default(
  MlnValaBoundOptions* out_options, GError** error
);
gboolean mln_vala_free_camera_options_default(
  MlnValaFreeCameraOptions* out_options, GError** error
);
gboolean mln_vala_projection_mode_default(
  MlnValaProjectionMode* out_mode, GError** error
);
gboolean mln_vala_map_viewport_options_default(
  MlnValaMapViewportOptions* out_options, GError** error
);
gboolean mln_vala_map_tile_options_default(
  MlnValaMapTileOptions* out_options, GError** error
);
gboolean mln_vala_style_tile_source_options_default(
  MlnValaStyleTileSourceOptions* out_options, GError** error
);
gboolean mln_vala_metal_surface_descriptor_default(
  MlnValaMetalSurfaceDescriptor* out_descriptor, GError** error
);
gboolean mln_vala_vulkan_surface_descriptor_default(
  MlnValaVulkanSurfaceDescriptor* out_descriptor, GError** error
);
gboolean mln_vala_metal_owned_texture_descriptor_default(
  MlnValaMetalOwnedTextureDescriptor* out_descriptor, GError** error
);
gboolean mln_vala_metal_borrowed_texture_descriptor_default(
  MlnValaMetalBorrowedTextureDescriptor* out_descriptor, GError** error
);
gboolean mln_vala_vulkan_owned_texture_descriptor_default(
  MlnValaVulkanOwnedTextureDescriptor* out_descriptor, GError** error
);
gboolean mln_vala_vulkan_borrowed_texture_descriptor_default(
  MlnValaVulkanBorrowedTextureDescriptor* out_descriptor, GError** error
);
gboolean mln_vala_texture_image_info_default(
  MlnValaTextureImageInfo* out_info, GError** error
);

#define MLN_VALA_TYPE_NATIVE_POINTER (mln_vala_native_pointer_get_type())
typedef struct _MlnValaNativePointer MlnValaNativePointer;

GType mln_vala_native_pointer_get_type(void);

/**
 * mln_vala_native_pointer_new:
 * @bits: non-zero opaque native address bits.
 * @out_pointer: (out) (transfer full): return location for a boxed native
 * pointer.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_native_pointer_new(
  size_t bits, MlnValaNativePointer** out_pointer, GError** error
);

/**
 * mln_vala_native_pointer_get_bits:
 * @pointer: a native pointer value.
 *
 * Returns: opaque native address bits, or zero when @pointer is `NULL`.
 */
size_t mln_vala_native_pointer_get_bits(const MlnValaNativePointer* pointer);

MlnValaNativePointer* mln_vala_native_pointer_copy(
  const MlnValaNativePointer* pointer
);
void mln_vala_native_pointer_free(MlnValaNativePointer* pointer);

#define MLN_VALA_TYPE_RUNTIME_EVENT (mln_vala_runtime_event_get_type())
typedef struct _MlnValaRuntimeEvent MlnValaRuntimeEvent;

struct _MlnValaRuntimeEvent {
  uint32_t type;
  uint32_t source_type;
  int32_t code;
  uint32_t payload_type;
  char* message;
  size_t message_size;
};

GType mln_vala_runtime_event_get_type(void);
MlnValaRuntimeEvent* mln_vala_runtime_event_copy(
  const MlnValaRuntimeEvent* event
);
void mln_vala_runtime_event_free(MlnValaRuntimeEvent* event);

#define MLN_VALA_TYPE_RUNTIME_HANDLE (mln_vala_runtime_handle_get_type())
G_DECLARE_FINAL_TYPE(
  MlnValaRuntimeHandle, mln_vala_runtime_handle, MLN_VALA, RUNTIME_HANDLE,
  GObject
)

#define MLN_VALA_TYPE_MAP_HANDLE (mln_vala_map_handle_get_type())
G_DECLARE_FINAL_TYPE(
  MlnValaMapHandle, mln_vala_map_handle, MLN_VALA, MAP_HANDLE, GObject
)

#define MLN_VALA_TYPE_MAP_PROJECTION_HANDLE \
  (mln_vala_map_projection_handle_get_type())
G_DECLARE_FINAL_TYPE(
  MlnValaMapProjectionHandle, mln_vala_map_projection_handle, MLN_VALA,
  MAP_PROJECTION_HANDLE, GObject
)

#define MLN_VALA_TYPE_RENDER_SESSION_HANDLE \
  (mln_vala_render_session_handle_get_type())
G_DECLARE_FINAL_TYPE(
  MlnValaRenderSessionHandle, mln_vala_render_session_handle, MLN_VALA,
  RENDER_SESSION_HANDLE, GObject
)

#define MLN_VALA_TYPE_METAL_OWNED_TEXTURE_FRAME_HANDLE \
  (mln_vala_metal_owned_texture_frame_handle_get_type())
G_DECLARE_FINAL_TYPE(
  MlnValaMetalOwnedTextureFrameHandle,
  mln_vala_metal_owned_texture_frame_handle, MLN_VALA,
  METAL_OWNED_TEXTURE_FRAME_HANDLE, GObject
)

#define MLN_VALA_TYPE_VULKAN_OWNED_TEXTURE_FRAME_HANDLE \
  (mln_vala_vulkan_owned_texture_frame_handle_get_type())
G_DECLARE_FINAL_TYPE(
  MlnValaVulkanOwnedTextureFrameHandle,
  mln_vala_vulkan_owned_texture_frame_handle, MLN_VALA,
  VULKAN_OWNED_TEXTURE_FRAME_HANDLE, GObject
)

#define MLN_VALA_TYPE_RESOURCE_REQUEST_HANDLE \
  (mln_vala_resource_request_handle_get_type())
G_DECLARE_FINAL_TYPE(
  MlnValaResourceRequestHandle, mln_vala_resource_request_handle, MLN_VALA,
  RESOURCE_REQUEST_HANDLE, GObject
)

/**
 * mln_vala_resource_response_default:
 * @out_response: (out): return location for initialized resource response.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_resource_response_default(
  MlnValaResourceResponse* out_response, GError** error
);

/**
 * mln_vala_c_version:
 *
 * Returns: the MapLibre Native C ABI version.
 */
uint32_t mln_vala_c_version(void);

/**
 * mln_vala_supported_render_backends:
 *
 * Returns: a bit mask of supported `MlnValaRenderBackendFlags` values.
 */
MlnValaRenderBackendFlags mln_vala_supported_render_backends(void);

/**
 * mln_vala_network_status_get:
 * @out_status: (out): return location for the network status value.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_network_status_get(
  MlnValaNetworkStatus* out_status, GError** error
);

/**
 * mln_vala_network_status_set:
 * @status: network status value.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_network_status_set(
  MlnValaNetworkStatus status, GError** error
);

/**
 * mln_vala_log_set_callback:
 * @callback: (scope async) (closure user_data) (destroy destroy_notify): log
 * callback.
 * @user_data: closure data for @callback.
 * @destroy_notify: destroy notify for @user_data.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_log_set_callback(
  MlnValaLogCallback callback, gpointer user_data,
  GDestroyNotify destroy_notify, GError** error
);

/**
 * mln_vala_log_clear_callback:
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_log_clear_callback(GError** error);

/**
 * mln_vala_log_set_async_severity_mask:
 * @mask: log severity flags.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_log_set_async_severity_mask(
  MlnValaLogSeverityFlags mask, GError** error
);

/**
 * mln_vala_projected_meters_for_lat_lng:
 * @coordinate: (not nullable): geographic coordinate in degrees.
 * @out_meters: (out): return location for projected meters.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_projected_meters_for_lat_lng(
  const MlnValaLatLng* coordinate, MlnValaProjectedMeters* out_meters,
  GError** error
);

/**
 * mln_vala_lat_lng_for_projected_meters:
 * @meters: (not nullable): spherical Mercator projected meters.
 * @out_coordinate: (out): return location for geographic coordinates.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_lat_lng_for_projected_meters(
  const MlnValaProjectedMeters* meters, MlnValaLatLng* out_coordinate,
  GError** error
);

/**
 * mln_vala_runtime_handle_new:
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): a new runtime handle, or `NULL` on failure.
 * Throws: MlnValaError
 */
MlnValaRuntimeHandle* mln_vala_runtime_handle_new(GError** error);

/**
 * mln_vala_runtime_handle_new_with_options:
 * @options: (not nullable): runtime options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): a new runtime handle, or `NULL` on failure.
 * Throws: MlnValaError
 */
MlnValaRuntimeHandle* mln_vala_runtime_handle_new_with_options(
  const MlnValaRuntimeOptions* options, GError** error
);

/**
 * mln_vala_runtime_handle_close:
 * @self: a runtime handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_runtime_handle_close(
  MlnValaRuntimeHandle* self, GError** error
);

/**
 * mln_vala_runtime_handle_run_once:
 * @self: a runtime handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_runtime_handle_run_once(
  MlnValaRuntimeHandle* self, GError** error
);

/**
 * mln_vala_runtime_handle_poll_event:
 * @self: a runtime handle.
 * @out_event: (out) (transfer full) (optional): return location for a copied
 * event, or `NULL` when no event is queued.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` when polling succeeds; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_runtime_handle_poll_event(
  MlnValaRuntimeHandle* self, MlnValaRuntimeEvent** out_event, GError** error
);

/**
 * mln_vala_runtime_handle_set_resource_transform:
 * @self: a runtime handle.
 * @callback: (scope async) (closure user_data) (destroy destroy_notify): URL
 * transform callback.
 * @user_data: closure data for @callback.
 * @destroy_notify: destroy notify for @user_data.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_runtime_handle_set_resource_transform(
  MlnValaRuntimeHandle* self, MlnValaResourceTransformCallback callback,
  gpointer user_data, GDestroyNotify destroy_notify, GError** error
);

/**
 * mln_vala_runtime_handle_clear_resource_transform:
 * @self: a runtime handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_runtime_handle_clear_resource_transform(
  MlnValaRuntimeHandle* self, GError** error
);

/**
 * mln_vala_runtime_handle_run_ambient_cache_operation_start:
 * @self: a runtime handle.
 * @operation: ambient cache operation.
 * @out_operation_id: (out): return location for operation ID.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_runtime_handle_run_ambient_cache_operation_start(
  MlnValaRuntimeHandle* self, MlnValaAmbientCacheOperation operation,
  uint64_t* out_operation_id, GError** error
);

/**
 * mln_vala_runtime_handle_offline_operation_discard:
 * @self: a runtime handle.
 * @operation_id: offline operation ID.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_runtime_handle_offline_operation_discard(
  MlnValaRuntimeHandle* self, uint64_t operation_id, GError** error
);

/**
 * mln_vala_resource_request_handle_complete:
 * @self: a resource request handle.
 * @response: (not nullable): response descriptor copied by native code.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_resource_request_handle_complete(
  MlnValaResourceRequestHandle* self, const MlnValaResourceResponse* response,
  GError** error
);

/**
 * mln_vala_resource_request_handle_is_cancelled:
 * @self: a resource request handle.
 * @out_cancelled: (out): return location for cancellation state.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_resource_request_handle_is_cancelled(
  MlnValaResourceRequestHandle* self, gboolean* out_cancelled, GError** error
);

/**
 * mln_vala_resource_request_handle_release:
 * @self: (nullable): a resource request handle.
 *
 * Releases the provider-owned request reference exactly once. Null and repeated
 * release are no-ops.
 */
void mln_vala_resource_request_handle_release(
  MlnValaResourceRequestHandle* self
);

/**
 * mln_vala_resource_request_handle_close:
 * @self: (nullable): a resource request handle.
 *
 * Alias for release().
 */
void mln_vala_resource_request_handle_close(MlnValaResourceRequestHandle* self);

/**
 * mln_vala_map_handle_new:
 * @runtime: a runtime handle.
 * @width: map width in logical pixels.
 * @height: map height in logical pixels.
 * @scale_factor: map scale factor.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): a new map handle, or `NULL` on failure.
 * Throws: MlnValaError
 */
MlnValaMapHandle* mln_vala_map_handle_new(
  MlnValaRuntimeHandle* runtime, uint32_t width, uint32_t height,
  double scale_factor, GError** error
);

/**
 * mln_vala_map_handle_new_with_options:
 * @runtime: a runtime handle.
 * @options: (not nullable): map options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): a new map handle, or `NULL` on failure.
 * Throws: MlnValaError
 */
MlnValaMapHandle* mln_vala_map_handle_new_with_options(
  MlnValaRuntimeHandle* runtime, const MlnValaMapOptions* options,
  GError** error
);

/**
 * mln_vala_map_handle_close:
 * @self: a map handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_close(MlnValaMapHandle* self, GError** error);

/**
 * mln_vala_map_handle_request_repaint:
 * @self: a map handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_request_repaint(
  MlnValaMapHandle* self, GError** error
);

/**
 * mln_vala_map_handle_request_still_image:
 * @self: a map handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_request_still_image(
  MlnValaMapHandle* self, GError** error
);

/**
 * mln_vala_map_handle_set_debug_options:
 * @self: a map handle.
 * @options: debug option flags.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_set_debug_options(
  MlnValaMapHandle* self, MlnValaMapDebugOptions options, GError** error
);

/**
 * mln_vala_map_handle_get_debug_options:
 * @self: a map handle.
 * @out_options: (out): return location for debug option flags.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_get_debug_options(
  MlnValaMapHandle* self, MlnValaMapDebugOptions* out_options, GError** error
);

/**
 * mln_vala_map_handle_set_rendering_stats_view_enabled:
 * @self: a map handle.
 * @enabled: whether to show the rendering stats overlay.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_set_rendering_stats_view_enabled(
  MlnValaMapHandle* self, gboolean enabled, GError** error
);

/**
 * mln_vala_map_handle_get_rendering_stats_view_enabled:
 * @self: a map handle.
 * @out_enabled: (out): return location for overlay state.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_get_rendering_stats_view_enabled(
  MlnValaMapHandle* self, gboolean* out_enabled, GError** error
);

/**
 * mln_vala_map_handle_get_camera:
 * @self: a map handle.
 * @out_camera: (out): return location for a camera snapshot.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_get_camera(
  MlnValaMapHandle* self, MlnValaCameraOptions* out_camera, GError** error
);

/**
 * mln_vala_map_handle_get_viewport_options:
 * @self: a map handle.
 * @out_options: (out): return location for viewport options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_get_viewport_options(
  MlnValaMapHandle* self, MlnValaMapViewportOptions* out_options, GError** error
);

/**
 * mln_vala_map_handle_set_viewport_options:
 * @self: a map handle.
 * @options: (not nullable): viewport options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_set_viewport_options(
  MlnValaMapHandle* self, const MlnValaMapViewportOptions* options,
  GError** error
);

/**
 * mln_vala_map_handle_get_tile_options:
 * @self: a map handle.
 * @out_options: (out): return location for tile options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_get_tile_options(
  MlnValaMapHandle* self, MlnValaMapTileOptions* out_options, GError** error
);

/**
 * mln_vala_map_handle_set_tile_options:
 * @self: a map handle.
 * @options: (not nullable): tile options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_set_tile_options(
  MlnValaMapHandle* self, const MlnValaMapTileOptions* options, GError** error
);

/**
 * mln_vala_map_handle_get_bounds:
 * @self: a map handle.
 * @out_options: (out): return location for bound options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_get_bounds(
  MlnValaMapHandle* self, MlnValaBoundOptions* out_options, GError** error
);

/**
 * mln_vala_map_handle_set_bounds:
 * @self: a map handle.
 * @options: (not nullable): bound options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_set_bounds(
  MlnValaMapHandle* self, const MlnValaBoundOptions* options, GError** error
);

/**
 * mln_vala_map_handle_get_free_camera_options:
 * @self: a map handle.
 * @out_options: (out): return location for free camera options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_get_free_camera_options(
  MlnValaMapHandle* self, MlnValaFreeCameraOptions* out_options, GError** error
);

/**
 * mln_vala_map_handle_set_free_camera_options:
 * @self: a map handle.
 * @options: (not nullable): free camera options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_set_free_camera_options(
  MlnValaMapHandle* self, const MlnValaFreeCameraOptions* options,
  GError** error
);

/**
 * mln_vala_map_handle_get_projection_mode:
 * @self: a map handle.
 * @out_mode: (out): return location for projection mode.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_get_projection_mode(
  MlnValaMapHandle* self, MlnValaProjectionMode* out_mode, GError** error
);

/**
 * mln_vala_map_handle_set_projection_mode:
 * @self: a map handle.
 * @mode: (not nullable): projection mode.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_set_projection_mode(
  MlnValaMapHandle* self, const MlnValaProjectionMode* mode, GError** error
);

/**
 * mln_vala_map_handle_jump_to:
 * @self: a map handle.
 * @camera: (not nullable): camera options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_jump_to(
  MlnValaMapHandle* self, const MlnValaCameraOptions* camera, GError** error
);

/**
 * mln_vala_map_handle_ease_to:
 * @self: a map handle.
 * @camera: (not nullable): camera options.
 * @animation: (nullable): animation options, or `NULL` for native defaults.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_ease_to(
  MlnValaMapHandle* self, const MlnValaCameraOptions* camera,
  const MlnValaAnimationOptions* animation, GError** error
);

/**
 * mln_vala_map_handle_fly_to:
 * @self: a map handle.
 * @camera: (not nullable): camera options.
 * @animation: (nullable): animation options, or `NULL` for native defaults.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_fly_to(
  MlnValaMapHandle* self, const MlnValaCameraOptions* camera,
  const MlnValaAnimationOptions* animation, GError** error
);

/**
 * mln_vala_map_handle_move_by:
 * @self: a map handle.
 * @delta_x: horizontal delta in logical pixels.
 * @delta_y: vertical delta in logical pixels.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_move_by(
  MlnValaMapHandle* self, double delta_x, double delta_y, GError** error
);

/**
 * mln_vala_map_handle_move_by_animated:
 * @self: a map handle.
 * @delta_x: horizontal delta in logical pixels.
 * @delta_y: vertical delta in logical pixels.
 * @animation: (nullable): animation options, or `NULL` for native defaults.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_move_by_animated(
  MlnValaMapHandle* self, double delta_x, double delta_y,
  const MlnValaAnimationOptions* animation, GError** error
);

/**
 * mln_vala_map_handle_scale_by:
 * @self: a map handle.
 * @scale: scale factor.
 * @anchor: (nullable): optional screen anchor.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_scale_by(
  MlnValaMapHandle* self, double scale, const MlnValaScreenPoint* anchor,
  GError** error
);

/**
 * mln_vala_map_handle_scale_by_animated:
 * @self: a map handle.
 * @scale: scale factor.
 * @anchor: (nullable): optional screen anchor.
 * @animation: (nullable): animation options, or `NULL` for native defaults.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_scale_by_animated(
  MlnValaMapHandle* self, double scale, const MlnValaScreenPoint* anchor,
  const MlnValaAnimationOptions* animation, GError** error
);

/**
 * mln_vala_map_handle_rotate_by:
 * @self: a map handle.
 * @first: first screen point.
 * @second: second screen point.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_rotate_by(
  MlnValaMapHandle* self, const MlnValaScreenPoint* first,
  const MlnValaScreenPoint* second, GError** error
);

/**
 * mln_vala_map_handle_rotate_by_animated:
 * @self: a map handle.
 * @first: first screen point.
 * @second: second screen point.
 * @animation: (nullable): animation options, or `NULL` for native defaults.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_rotate_by_animated(
  MlnValaMapHandle* self, const MlnValaScreenPoint* first,
  const MlnValaScreenPoint* second, const MlnValaAnimationOptions* animation,
  GError** error
);

/**
 * mln_vala_map_handle_pitch_by:
 * @self: a map handle.
 * @pitch: pitch delta.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_pitch_by(
  MlnValaMapHandle* self, double pitch, GError** error
);

/**
 * mln_vala_map_handle_pitch_by_animated:
 * @self: a map handle.
 * @pitch: pitch delta.
 * @animation: (nullable): animation options, or `NULL` for native defaults.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_pitch_by_animated(
  MlnValaMapHandle* self, double pitch,
  const MlnValaAnimationOptions* animation, GError** error
);

/**
 * mln_vala_map_handle_cancel_transitions:
 * @self: a map handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_cancel_transitions(
  MlnValaMapHandle* self, GError** error
);

/**
 * mln_vala_map_handle_is_fully_loaded:
 * @self: a map handle.
 * @out_loaded: (out): return location for fully-loaded state.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_is_fully_loaded(
  MlnValaMapHandle* self, gboolean* out_loaded, GError** error
);

/**
 * mln_vala_map_handle_dump_debug_logs:
 * @self: a map handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_dump_debug_logs(
  MlnValaMapHandle* self, GError** error
);

/**
 * mln_vala_map_handle_set_style_url:
 * @self: a map handle.
 * @url: (not nullable): NUL-terminated style URL.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_set_style_url(
  MlnValaMapHandle* self, const char* url, GError** error
);

/**
 * mln_vala_map_handle_set_style_json:
 * @self: a map handle.
 * @json: (not nullable): NUL-terminated style JSON.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_set_style_json(
  MlnValaMapHandle* self, const char* json, GError** error
);

/**
 * mln_vala_map_handle_add_geojson_source_url:
 * @self: a map handle.
 * @source_id: (not nullable): source identifier.
 * @url: (not nullable): GeoJSON URL.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_add_geojson_source_url(
  MlnValaMapHandle* self, const char* source_id, const char* url, GError** error
);

/**
 * mln_vala_map_handle_set_geojson_source_url:
 * @self: a map handle.
 * @source_id: (not nullable): source identifier.
 * @url: (not nullable): GeoJSON URL.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_set_geojson_source_url(
  MlnValaMapHandle* self, const char* source_id, const char* url, GError** error
);

/**
 * mln_vala_map_handle_add_vector_source_url:
 * @self: a map handle.
 * @source_id: (not nullable): source identifier.
 * @url: (not nullable): tile source URL.
 * @options: (not nullable): tile source options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_add_vector_source_url(
  MlnValaMapHandle* self, const char* source_id, const char* url,
  const MlnValaStyleTileSourceOptions* options, GError** error
);

/**
 * mln_vala_map_handle_add_raster_source_url:
 * @self: a map handle.
 * @source_id: (not nullable): source identifier.
 * @url: (not nullable): tile source URL.
 * @options: (not nullable): tile source options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_add_raster_source_url(
  MlnValaMapHandle* self, const char* source_id, const char* url,
  const MlnValaStyleTileSourceOptions* options, GError** error
);

/**
 * mln_vala_map_handle_add_raster_dem_source_url:
 * @self: a map handle.
 * @source_id: (not nullable): source identifier.
 * @url: (not nullable): tile source URL.
 * @options: (not nullable): tile source options.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_add_raster_dem_source_url(
  MlnValaMapHandle* self, const char* source_id, const char* url,
  const MlnValaStyleTileSourceOptions* options, GError** error
);

/**
 * mln_vala_map_handle_style_source_exists:
 * @self: a map handle.
 * @source_id: (not nullable): source identifier.
 * @out_exists: (out): return location for source existence.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_style_source_exists(
  MlnValaMapHandle* self, const char* source_id, gboolean* out_exists,
  GError** error
);

/**
 * mln_vala_map_handle_get_style_source_type:
 * @self: a map handle.
 * @source_id: (not nullable): source identifier.
 * @out_source_type: (out): return location for source type.
 * @out_found: (out): return location for found state.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_get_style_source_type(
  MlnValaMapHandle* self, const char* source_id,
  MlnValaStyleSourceType* out_source_type, gboolean* out_found, GError** error
);

/**
 * mln_vala_map_handle_get_style_source_info:
 * @self: a map handle.
 * @source_id: (not nullable): source identifier.
 * @out_info: (out): return location for source info.
 * @out_found: (out): return location for found state.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_get_style_source_info(
  MlnValaMapHandle* self, const char* source_id,
  MlnValaStyleSourceInfo* out_info, gboolean* out_found, GError** error
);

/**
 * mln_vala_map_handle_remove_style_source:
 * @self: a map handle.
 * @source_id: (not nullable): source identifier.
 * @out_removed: (out): return location for removal state.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_handle_remove_style_source(
  MlnValaMapHandle* self, const char* source_id, gboolean* out_removed,
  GError** error
);

/**
 * mln_vala_map_handle_attach_metal_surface:
 * @self: a map handle.
 * @descriptor: (not nullable): Metal surface descriptor.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): a render session handle, or `NULL` on failure.
 * Throws: MlnValaError
 */
MlnValaRenderSessionHandle* mln_vala_map_handle_attach_metal_surface(
  MlnValaMapHandle* self, const MlnValaMetalSurfaceDescriptor* descriptor,
  GError** error
);

/**
 * mln_vala_map_handle_attach_vulkan_surface:
 * @self: a map handle.
 * @descriptor: (not nullable): Vulkan surface descriptor.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): a render session handle, or `NULL` on failure.
 * Throws: MlnValaError
 */
MlnValaRenderSessionHandle* mln_vala_map_handle_attach_vulkan_surface(
  MlnValaMapHandle* self, const MlnValaVulkanSurfaceDescriptor* descriptor,
  GError** error
);

/**
 * mln_vala_map_handle_attach_metal_owned_texture:
 * @self: a map handle.
 * @descriptor: (not nullable): Metal owned-texture descriptor.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): a render session handle, or `NULL` on failure.
 * Throws: MlnValaError
 */
MlnValaRenderSessionHandle* mln_vala_map_handle_attach_metal_owned_texture(
  MlnValaMapHandle* self, const MlnValaMetalOwnedTextureDescriptor* descriptor,
  GError** error
);

/**
 * mln_vala_map_handle_attach_metal_borrowed_texture:
 * @self: a map handle.
 * @descriptor: (not nullable): Metal borrowed-texture descriptor.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): a render session handle, or `NULL` on failure.
 * Throws: MlnValaError
 */
MlnValaRenderSessionHandle* mln_vala_map_handle_attach_metal_borrowed_texture(
  MlnValaMapHandle* self,
  const MlnValaMetalBorrowedTextureDescriptor* descriptor, GError** error
);

/**
 * mln_vala_map_handle_attach_vulkan_owned_texture:
 * @self: a map handle.
 * @descriptor: (not nullable): Vulkan owned-texture descriptor.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): a render session handle, or `NULL` on failure.
 * Throws: MlnValaError
 */
MlnValaRenderSessionHandle* mln_vala_map_handle_attach_vulkan_owned_texture(
  MlnValaMapHandle* self, const MlnValaVulkanOwnedTextureDescriptor* descriptor,
  GError** error
);

/**
 * mln_vala_map_handle_attach_vulkan_borrowed_texture:
 * @self: a map handle.
 * @descriptor: (not nullable): Vulkan borrowed-texture descriptor.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): a render session handle, or `NULL` on failure.
 * Throws: MlnValaError
 */
MlnValaRenderSessionHandle* mln_vala_map_handle_attach_vulkan_borrowed_texture(
  MlnValaMapHandle* self,
  const MlnValaVulkanBorrowedTextureDescriptor* descriptor, GError** error
);

gboolean mln_vala_render_session_handle_resize(
  MlnValaRenderSessionHandle* self, uint32_t width, uint32_t height,
  double scale_factor, GError** error
);
gboolean mln_vala_render_session_handle_render_update(
  MlnValaRenderSessionHandle* self, GError** error
);
gboolean mln_vala_render_session_handle_detach(
  MlnValaRenderSessionHandle* self, GError** error
);
gboolean mln_vala_render_session_handle_close(
  MlnValaRenderSessionHandle* self, GError** error
);
gboolean mln_vala_render_session_handle_reduce_memory_use(
  MlnValaRenderSessionHandle* self, GError** error
);
gboolean mln_vala_render_session_handle_clear_data(
  MlnValaRenderSessionHandle* self, GError** error
);
gboolean mln_vala_render_session_handle_dump_debug_logs(
  MlnValaRenderSessionHandle* self, GError** error
);
/**
 * mln_vala_render_session_handle_read_premultiplied_rgba8:
 * @self: a render session handle.
 * @out_data: (out caller-allocates) (array length=out_data_capacity): output
 * buffer.
 * @out_data_capacity: byte length of @out_data.
 * @out_info: (out): return location for image metadata.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_render_session_handle_read_premultiplied_rgba8(
  MlnValaRenderSessionHandle* self, uint8_t* out_data, size_t out_data_capacity,
  MlnValaTextureImageInfo* out_info, GError** error
);
/**
 * mln_vala_render_session_handle_acquire_metal_owned_texture_frame:
 * @self: a render session handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): an acquired frame handle, or `NULL` on failure.
 * Throws: MlnValaError
 */
MlnValaMetalOwnedTextureFrameHandle*
mln_vala_render_session_handle_acquire_metal_owned_texture_frame(
  MlnValaRenderSessionHandle* self, GError** error
);

/**
 * mln_vala_render_session_handle_acquire_vulkan_owned_texture_frame:
 * @self: a render session handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): an acquired frame handle, or `NULL` on failure.
 * Throws: MlnValaError
 */
MlnValaVulkanOwnedTextureFrameHandle*
mln_vala_render_session_handle_acquire_vulkan_owned_texture_frame(
  MlnValaRenderSessionHandle* self, GError** error
);
gboolean mln_vala_metal_owned_texture_frame_handle_close(
  MlnValaMetalOwnedTextureFrameHandle* self, GError** error
);
gboolean mln_vala_vulkan_owned_texture_frame_handle_close(
  MlnValaVulkanOwnedTextureFrameHandle* self, GError** error
);

/**
 * mln_vala_metal_owned_texture_frame_handle_get_texture:
 * @self: a Metal texture frame handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): borrowed native texture pointer boxed for Vala.
 * Throws: MlnValaError
 */
MlnValaNativePointer* mln_vala_metal_owned_texture_frame_handle_get_texture(
  MlnValaMetalOwnedTextureFrameHandle* self, GError** error
);

/**
 * mln_vala_metal_owned_texture_frame_handle_get_device:
 * @self: a Metal texture frame handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): borrowed native device pointer boxed for Vala.
 * Throws: MlnValaError
 */
MlnValaNativePointer* mln_vala_metal_owned_texture_frame_handle_get_device(
  MlnValaMetalOwnedTextureFrameHandle* self, GError** error
);

/**
 * mln_vala_metal_owned_texture_frame_handle_get_generation:
 * @self: a Metal texture frame handle.
 * @out_generation: (out): return location for the frame generation.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_metal_owned_texture_frame_handle_get_generation(
  MlnValaMetalOwnedTextureFrameHandle* self, uint64_t* out_generation,
  GError** error
);

/**
 * mln_vala_metal_owned_texture_frame_handle_get_width:
 * @self: a Metal texture frame handle.
 * @out_width: (out): return location for physical width.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_metal_owned_texture_frame_handle_get_width(
  MlnValaMetalOwnedTextureFrameHandle* self, uint32_t* out_width, GError** error
);

/**
 * mln_vala_metal_owned_texture_frame_handle_get_height:
 * @self: a Metal texture frame handle.
 * @out_height: (out): return location for physical height.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_metal_owned_texture_frame_handle_get_height(
  MlnValaMetalOwnedTextureFrameHandle* self, uint32_t* out_height,
  GError** error
);

/**
 * mln_vala_metal_owned_texture_frame_handle_get_scale_factor:
 * @self: a Metal texture frame handle.
 * @out_scale_factor: (out): return location for scale factor.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_metal_owned_texture_frame_handle_get_scale_factor(
  MlnValaMetalOwnedTextureFrameHandle* self, double* out_scale_factor,
  GError** error
);

/**
 * mln_vala_metal_owned_texture_frame_handle_get_frame_id:
 * @self: a Metal texture frame handle.
 * @out_frame_id: (out): return location for opaque frame identity.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_metal_owned_texture_frame_handle_get_frame_id(
  MlnValaMetalOwnedTextureFrameHandle* self, uint64_t* out_frame_id,
  GError** error
);

/**
 * mln_vala_metal_owned_texture_frame_handle_get_pixel_format:
 * @self: a Metal texture frame handle.
 * @out_pixel_format: (out): return location for native pixel format.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_metal_owned_texture_frame_handle_get_pixel_format(
  MlnValaMetalOwnedTextureFrameHandle* self, uint64_t* out_pixel_format,
  GError** error
);

/**
 * mln_vala_vulkan_owned_texture_frame_handle_get_image:
 * @self: a Vulkan texture frame handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): borrowed native image pointer boxed for Vala.
 * Throws: MlnValaError
 */
MlnValaNativePointer* mln_vala_vulkan_owned_texture_frame_handle_get_image(
  MlnValaVulkanOwnedTextureFrameHandle* self, GError** error
);

/**
 * mln_vala_vulkan_owned_texture_frame_handle_get_image_view:
 * @self: a Vulkan texture frame handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): borrowed native image-view pointer boxed for Vala.
 * Throws: MlnValaError
 */
MlnValaNativePointer* mln_vala_vulkan_owned_texture_frame_handle_get_image_view(
  MlnValaVulkanOwnedTextureFrameHandle* self, GError** error
);

/**
 * mln_vala_vulkan_owned_texture_frame_handle_get_device:
 * @self: a Vulkan texture frame handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): borrowed native device pointer boxed for Vala.
 * Throws: MlnValaError
 */
MlnValaNativePointer* mln_vala_vulkan_owned_texture_frame_handle_get_device(
  MlnValaVulkanOwnedTextureFrameHandle* self, GError** error
);

/**
 * mln_vala_vulkan_owned_texture_frame_handle_get_generation:
 * @self: a Vulkan texture frame handle.
 * @out_generation: (out): return location for the frame generation.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_vulkan_owned_texture_frame_handle_get_generation(
  MlnValaVulkanOwnedTextureFrameHandle* self, uint64_t* out_generation,
  GError** error
);

/**
 * mln_vala_vulkan_owned_texture_frame_handle_get_width:
 * @self: a Vulkan texture frame handle.
 * @out_width: (out): return location for physical width.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_vulkan_owned_texture_frame_handle_get_width(
  MlnValaVulkanOwnedTextureFrameHandle* self, uint32_t* out_width,
  GError** error
);

/**
 * mln_vala_vulkan_owned_texture_frame_handle_get_height:
 * @self: a Vulkan texture frame handle.
 * @out_height: (out): return location for physical height.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_vulkan_owned_texture_frame_handle_get_height(
  MlnValaVulkanOwnedTextureFrameHandle* self, uint32_t* out_height,
  GError** error
);

/**
 * mln_vala_vulkan_owned_texture_frame_handle_get_scale_factor:
 * @self: a Vulkan texture frame handle.
 * @out_scale_factor: (out): return location for scale factor.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_vulkan_owned_texture_frame_handle_get_scale_factor(
  MlnValaVulkanOwnedTextureFrameHandle* self, double* out_scale_factor,
  GError** error
);

/**
 * mln_vala_vulkan_owned_texture_frame_handle_get_frame_id:
 * @self: a Vulkan texture frame handle.
 * @out_frame_id: (out): return location for opaque frame identity.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_vulkan_owned_texture_frame_handle_get_frame_id(
  MlnValaVulkanOwnedTextureFrameHandle* self, uint64_t* out_frame_id,
  GError** error
);

/**
 * mln_vala_vulkan_owned_texture_frame_handle_get_format:
 * @self: a Vulkan texture frame handle.
 * @out_format: (out): return location for native image format.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_vulkan_owned_texture_frame_handle_get_format(
  MlnValaVulkanOwnedTextureFrameHandle* self, uint32_t* out_format,
  GError** error
);

/**
 * mln_vala_vulkan_owned_texture_frame_handle_get_layout:
 * @self: a Vulkan texture frame handle.
 * @out_layout: (out): return location for native image layout.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_vulkan_owned_texture_frame_handle_get_layout(
  MlnValaVulkanOwnedTextureFrameHandle* self, uint32_t* out_layout,
  GError** error
);

/**
 * mln_vala_map_projection_handle_new:
 * @map: a map handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: (transfer full): a new standalone projection handle, or `NULL` on
 * failure. Throws: MlnValaError
 */
MlnValaMapProjectionHandle* mln_vala_map_projection_handle_new(
  MlnValaMapHandle* map, GError** error
);

/**
 * mln_vala_map_projection_handle_close:
 * @self: a projection handle.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_projection_handle_close(
  MlnValaMapProjectionHandle* self, GError** error
);

/**
 * mln_vala_map_projection_handle_pixel_for_lat_lng:
 * @self: a projection handle.
 * @coordinate: (not nullable): geographic coordinate in degrees.
 * @out_point: (out): return location for the projected screen point.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_projection_handle_pixel_for_lat_lng(
  MlnValaMapProjectionHandle* self, const MlnValaLatLng* coordinate,
  MlnValaScreenPoint* out_point, GError** error
);

/**
 * mln_vala_map_projection_handle_lat_lng_for_pixel:
 * @self: a projection handle.
 * @point: (not nullable): screen point in logical pixels.
 * @out_coordinate: (out): return location for the geographic coordinate.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_map_projection_handle_lat_lng_for_pixel(
  MlnValaMapProjectionHandle* self, const MlnValaScreenPoint* point,
  MlnValaLatLng* out_coordinate, GError** error
);

G_END_DECLS

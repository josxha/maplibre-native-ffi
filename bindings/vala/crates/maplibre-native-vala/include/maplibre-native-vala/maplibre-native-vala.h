#pragma once

#include <glib-object.h>
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

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

GQuark mln_vala_error_quark(void);

#define MLN_VALA_TYPE_RUNTIME_HANDLE (mln_vala_runtime_handle_get_type())
G_DECLARE_FINAL_TYPE(
  MlnValaRuntimeHandle, mln_vala_runtime_handle, MLN_VALA, RUNTIME_HANDLE,
  GObject
)

#define MLN_VALA_TYPE_MAP_HANDLE (mln_vala_map_handle_get_type())
G_DECLARE_FINAL_TYPE(
  MlnValaMapHandle, mln_vala_map_handle, MLN_VALA, MAP_HANDLE, GObject
)

/**
 * mln_vala_c_version:
 *
 * Returns: the MapLibre Native C ABI version.
 */
uint32_t mln_vala_c_version(void);

/**
 * mln_vala_supported_render_backend_mask:
 *
 * Returns: a bit mask of supported `MlnValaRenderBackendFlags` values.
 */
uint32_t mln_vala_supported_render_backend_mask(void);

/**
 * mln_vala_network_status_get:
 * @out_status: (out): return location for the raw network status value.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_network_status_get(uint32_t* out_status, GError** error);

/**
 * mln_vala_network_status_set:
 * @raw_status: raw network status value.
 * @error: return location for a `GError`, or `NULL`.
 *
 * Returns: `TRUE` on success; `FALSE` with @error set on failure.
 * Throws: MlnValaError
 */
gboolean mln_vala_network_status_set(uint32_t raw_status, GError** error);

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

G_END_DECLS

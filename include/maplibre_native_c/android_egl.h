#ifndef MAPLIBRE_NATIVE_C_ANDROID_EGL_H
#define MAPLIBRE_NATIVE_C_ANDROID_EGL_H

#include "base.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Holds the EGL objects required for MapLibre OpenGL ES rendering.
 *
 * All handles are borrowed by the caller. The caller must release them using
 * mln_android_destroy_egl_context().
 */
typedef struct {
  void* display;
  void* config;
  void* context;
} mln_android_egl_context;

/**
 * Creates an EGL context suitable for MapLibre rendering.
 *
 * Returns an empty context (all fields null) on failure.
 */
MLN_API
mln_android_egl_context
mln_android_create_egl_context(void) MLN_NOEXCEPT;

/**
 * Destroys an EGL context created by mln_android_create_egl_context().
 *
 * Safe to call with an empty context.
 */
MLN_API
void
mln_android_destroy_egl_context(
    mln_android_egl_context context
) MLN_NOEXCEPT;

#ifdef __cplusplus
}
#endif

#endif  // MAPLIBRE_NATIVE_C_ANDROID_EGL_H
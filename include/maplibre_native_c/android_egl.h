#ifndef MAPLIBRE_NATIVE_C_ANDROID_EGL_H
#define MAPLIBRE_NATIVE_C_ANDROID_EGL_H

#include "base.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
  void* display;
  void* config;
  void* context;
} mln_android_egl_context;

/**
 * Creates an EGL context suitable for MapLibre rendering.
 */
MLN_API
mln_android_egl_context
mln_android_create_egl_context(void) MLN_NOEXCEPT;

#ifdef __cplusplus
}
#endif

#endif
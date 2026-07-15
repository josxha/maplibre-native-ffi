#define MLN_BUILDING_C

#include "maplibre_native_c/android_egl.h"

#ifdef __ANDROID__

#include <EGL/egl.h>
#include <GLES2/gl2.h>

#endif

mln_android_egl_context
mln_android_create_egl_context(void) noexcept {
#ifdef __ANDROID__

    // implementation...

#else

    return {};

#endif
}
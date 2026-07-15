#define MLN_BUILDING_C

#include "maplibre_native_c/android_egl.h"

#ifdef __ANDROID__

#include <EGL/egl.h>
#include <GLES2/gl2.h>

extern "C"
mln_android_egl_context
mln_android_create_egl_context(void) noexcept {

    EGLDisplay display = eglGetDisplay(EGL_DEFAULT_DISPLAY);

    if (display == EGL_NO_DISPLAY) {
        return {};
    }

    EGLint major = 0;
    EGLint minor = 0;

    if (!eglInitialize(display, &major, &minor)) {
        return {};
    }

    const EGLint configAttribs[] = {
        EGL_RENDERABLE_TYPE,
        EGL_OPENGL_ES2_BIT,

        EGL_SURFACE_TYPE,
        EGL_PBUFFER_BIT,

        EGL_NONE
    };

    EGLConfig config = nullptr;
    EGLint numConfigs = 0;

    if (!eglChooseConfig(
            display,
            configAttribs,
            &config,
            1,
            &numConfigs) ||
        numConfigs == 0) {
        eglTerminate(display);
        return {};
    }

    const EGLint contextAttribs[] = {
        EGL_CONTEXT_CLIENT_VERSION,
        2,
        EGL_NONE
    };

    EGLContext context = eglCreateContext(
        display,
        config,
        EGL_NO_CONTEXT,
        contextAttribs
    );

    if (context == EGL_NO_CONTEXT) {
        eglTerminate(display);
        return {};
    }

    return {
        .display = display,
        .config = config,
        .context = context,
    };
}


extern "C"
void
mln_android_destroy_egl_context(
    mln_android_egl_context value
) noexcept {

    EGLDisplay display =
        reinterpret_cast<EGLDisplay>(value.display);

    EGLContext context =
        reinterpret_cast<EGLContext>(value.context);

    if (display == EGL_NO_DISPLAY) {
        return;
    }

    if (context != EGL_NO_CONTEXT) {
        eglDestroyContext(
            display,
            context
        );
    }

    eglTerminate(display);
}


#else

extern "C"
mln_android_egl_context
mln_android_create_egl_context(void) noexcept {
    return {};
}


extern "C"
void
mln_android_destroy_egl_context(
    mln_android_egl_context
) noexcept {
}

#endif
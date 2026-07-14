#define MLN_BUILDING_C

#include "c_api/boundary.hpp"
#include "diagnostics/diagnostics.hpp"
#include "maplibre_native_c.h"

#ifdef __ANDROID__

#include <EGL/egl.h>
#include <GLES2/gl2.h>

extern "C" auto mlnffi_rust_android_init_tls_verifier(
  void* jni_env, void* context
) -> char*;
extern "C" auto mlnffi_rust_android_error_free(char* error) -> void;
#endif

auto mln_android_init(void* jni_env, void* context) noexcept -> mln_status {
  return mln::c_api::status_boundary([&]() -> mln_status {
    if (jni_env == nullptr || context == nullptr) {
      mln::core::set_thread_error("jni_env and context must not be null");
      return MLN_STATUS_INVALID_ARGUMENT;
    }

#ifndef __ANDROID__
    mln::core::set_thread_error("Android initialization is not supported");
    return MLN_STATUS_UNSUPPORTED;
#else
    auto* error = mlnffi_rust_android_init_tls_verifier(jni_env, context);
    if (error == nullptr) {
      return MLN_STATUS_OK;
    }

    mln::core::set_thread_error(error);
    mlnffi_rust_android_error_free(error);
    return MLN_STATUS_NATIVE_ERROR;
#endif
  });
}

struct mln_android_egl_context {
    void* display;
    void* config;
    void* context;
};


auto mln_android_create_egl_context() noexcept
    -> mln_android_egl_context {

#ifdef __ANDROID__

    EGLDisplay display = eglGetDisplay(EGL_DEFAULT_DISPLAY);

    if (display == EGL_NO_DISPLAY) {
        return {};
    }

    EGLint major;
    EGLint minor;

    if (!eglInitialize(display, &major, &minor)) {
        return {};
    }


    const EGLint attributes[] = {
        EGL_RENDERABLE_TYPE,
        EGL_OPENGL_ES2_BIT,

        EGL_SURFACE_TYPE,
        EGL_PBUFFER_BIT,

        EGL_NONE
    };


    EGLConfig config;
    EGLint numConfigs;

    if (!eglChooseConfig(
            display,
            attributes,
            &config,
            1,
            &numConfigs)) {
        return {};
    }


    const EGLint contextAttributes[] = {
        EGL_CONTEXT_CLIENT_VERSION,
        2,
        EGL_NONE
    };


    EGLContext context =
        eglCreateContext(
            display,
            config,
            EGL_NO_CONTEXT,
            contextAttributes
        );


    if (context == EGL_NO_CONTEXT) {
        return {};
    }


    return {
        .display = display,
        .config = config,
        .context = context,
    };

#else

    return {};

#endif
}
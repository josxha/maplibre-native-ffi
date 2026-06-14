function(mln_configure_android_platform target)
  include(${MLN_SOURCE_DIR}/vendor/icu.cmake)

  set(MLN_FFI_VENDOR_ANDROID_SOURCES
      ${MLN_SOURCE_DIR}/platform/android/src/async_task.cpp
      ${MLN_SOURCE_DIR}/platform/android/src/attach_env.cpp
      ${MLN_SOURCE_DIR}/platform/android/src/jni.cpp
      ${MLN_SOURCE_DIR}/platform/android/src/run_loop.cpp
      ${MLN_SOURCE_DIR}/platform/android/src/string_util.cpp
      ${MLN_SOURCE_DIR}/platform/android/src/thread.cpp
      ${MLN_SOURCE_DIR}/platform/android/src/timer.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/util/thread_local.cpp
      ${MLN_SOURCE_DIR}/platform/android/src/test/collator_test_stub.cpp
      ${MLN_SOURCE_DIR}/platform/android/src/test/number_format_test_stub.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/text/bidi.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/text/local_glyph_rasterizer.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/storage/http_file_source.cpp
      ${PROJECT_SOURCE_DIR}/src/platform/android/image.cpp
      ${PROJECT_SOURCE_DIR}/src/platform/android/jvm.cpp)

  mln_target_vendor_sources(${target} ${MLN_FFI_VENDOR_ANDROID_SOURCES})

  target_include_directories(
    ${target}
    SYSTEM
    PRIVATE
      ${MLN_SOURCE_DIR}/platform/default/include
      ${MLN_SOURCE_DIR}/platform/android/src
      ${MLN_SOURCE_DIR}/vendor/icu/include)

  find_package(curl CONFIG REQUIRED)
  target_link_libraries(${target} PRIVATE curl::curl_static)

  target_link_libraries(
    ${target}
    PRIVATE
      android
      atomic
      jnigraphics
      log
      mbgl-vendor-icu
      z
      MapLibreNative::Base::jni.hpp)
endfunction()

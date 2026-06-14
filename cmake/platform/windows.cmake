# http_file_source.cpp includes <dlfcn.h>; MSVC has no native dlopen shim.
if(MSVC AND NOT TARGET dlfcn-win32::dl)
  include(FetchContent)

  fetchcontent_declare(
    dlfcn_win32
    GIT_REPOSITORY
    https://github.com/dlfcn-win32/dlfcn-win32.git
    GIT_TAG
    v1.4.2
    GIT_SHALLOW
    TRUE)

  fetchcontent_makeavailable(dlfcn_win32)
endif()

function(mln_configure_windows_platform target)
  find_package(CURL REQUIRED)
  find_package(JPEG REQUIRED)
  find_package(libuv REQUIRED)
  find_package(PNG REQUIRED)
  find_package(WebP REQUIRED)

  set(MLN_FFI_VENDOR_WINDOWS_SOURCES
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/i18n/collator.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/i18n/number_format.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/storage/http_file_source.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/text/bidi.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/text/local_glyph_rasterizer.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/util/async_task.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/util/image.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/util/jpeg_reader.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/util/png_reader.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/util/png_writer.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/util/run_loop.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/util/string_stdlib.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/util/timer.cpp
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/util/webp_reader.cpp
      ${MLN_SOURCE_DIR}/platform/windows/src/thread.cpp
      ${MLN_SOURCE_DIR}/platform/windows/src/thread_local.cpp)

  mln_target_vendor_sources(${target} ${MLN_FFI_VENDOR_WINDOWS_SOURCES})

  set_source_files_properties(
    ${MLN_SOURCE_DIR}/platform/default/src/mbgl/i18n/number_format.cpp
    PROPERTIES COMPILE_DEFINITIONS MBGL_USE_BUILTIN_ICU)

  target_include_directories(
    ${target}
    SYSTEM
    PRIVATE
      ${MLN_SOURCE_DIR}/platform/windows/include
      ${MLN_SOURCE_DIR}/vendor/icu/include)

  target_compile_definitions(
    ${target}
    PRIVATE CURL_STATICLIB NOMINMAX USE_STD_FILESYSTEM _USE_MATH_DEFINES)

  target_link_libraries(
    ${target}
    PRIVATE
      mbgl-vendor-icu
      CURL::libcurl
      dlfcn-win32::dl
      JPEG::JPEG
      PNG::PNG
      WebP::webp
      $<IF:$<TARGET_EXISTS:libuv::uv_a>,libuv::uv_a,libuv::uv>)
endfunction()

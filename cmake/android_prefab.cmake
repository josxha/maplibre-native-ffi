function(mln_android_configure_prefab target)
  if(NOT ANDROID_ABI)
    message(
      FATAL_ERROR "ANDROID_ABI is required for Android Prefab dependency lookup")
  endif()

  find_package(curl CONFIG)
  if(curl_FOUND)
    target_link_libraries(${target} PRIVATE curl::curl_static)
    return()
  endif()

  if(NOT MLN_FFI_ANDROID_PREFAB_ROOT)
    set(MLN_FFI_ANDROID_PREFAB_ROOT "${PROJECT_SOURCE_DIR}/build/android-prefab")
  endif()

  set(_mln_android_prefab_dir "${MLN_FFI_ANDROID_PREFAB_ROOT}/${ANDROID_ABI}")
  if(NOT EXISTS "${_mln_android_prefab_dir}")
    message(
      FATAL_ERROR
        "Android Prefab directory not found: ${_mln_android_prefab_dir}. "
        "Run mise run //:fetch-android-prefab before configuring Android builds.")
  endif()

  # Prefab CLI writes CMake package configs under lib/<ndk-triple>/cmake/,
  # matching
  # AGP externalNativeBuild. Android cross-compiles use FIND_ROOT_PATH for
  # packages.
  if(ANDROID_ABI STREQUAL "arm64-v8a")
    set(_mln_android_prefab_triple "aarch64-linux-android")
  elseif(ANDROID_ABI STREQUAL "armeabi-v7a")
    set(_mln_android_prefab_triple "arm-linux-androideabi")
  elseif(ANDROID_ABI STREQUAL "x86")
    set(_mln_android_prefab_triple "i686-linux-android")
  elseif(ANDROID_ABI STREQUAL "x86_64")
    set(_mln_android_prefab_triple "x86_64-linux-android")
  else()
    message(
      FATAL_ERROR "Unsupported ANDROID_ABI for Prefab lookup: ${ANDROID_ABI}")
  endif()

  set(_mln_android_prefab_prefix
      "${_mln_android_prefab_dir}/lib/${_mln_android_prefab_triple}")
  if(NOT EXISTS "${_mln_android_prefab_prefix}/cmake/curl/curlConfig.cmake")
    message(
      FATAL_ERROR
        "Android Prefab curl package not found under ${_mln_android_prefab_prefix}. "
        "Run mise run //:fetch-android-prefab before configuring Android builds.")
  endif()

  set(CMAKE_FIND_ROOT_PATH
      "${CMAKE_FIND_ROOT_PATH};${_mln_android_prefab_prefix}")
  set(boringssl_DIR "${_mln_android_prefab_prefix}/cmake/boringssl")
  set(curl_DIR "${_mln_android_prefab_prefix}/cmake/curl")
  find_package(curl REQUIRED CONFIG)

  target_link_libraries(${target} PRIVATE curl::curl_static)
endfunction()

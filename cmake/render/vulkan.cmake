function(mln_configure_vulkan_backend target)
  set(_vulkan_search_paths)
  if(DEFINED ENV{MLN_FFI_GRAPHICS_LIBRARY_DIR}
     AND NOT "$ENV{MLN_FFI_GRAPHICS_LIBRARY_DIR}" STREQUAL "")
    list(APPEND _vulkan_search_paths "$ENV{MLN_FFI_GRAPHICS_LIBRARY_DIR}")
  endif()

  if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    if(CMAKE_SYSTEM_PROCESSOR MATCHES "aarch64|arm64")
      list(APPEND _vulkan_search_paths /usr/lib/aarch64-linux-gnu)
    else()
      list(APPEND _vulkan_search_paths /usr/lib/x86_64-linux-gnu)
    endif()
  endif()

  find_library(
    MLN_VULKAN_LOADER_LIBRARY
    NAMES vulkan vulkan-1
    HINTS ${_vulkan_search_paths}
    REQUIRED)

  set(MLN_FFI_VENDOR_VULKAN_SOURCES
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/vulkan/headless_backend.cpp)
  set(MLN_FFI_VULKAN_SOURCES
      ${PROJECT_SOURCE_DIR}/src/render/vulkan/vulkan_texture_session.cpp
      ${PROJECT_SOURCE_DIR}/src/render/vulkan/vulkan_texture_backend.cpp
      ${PROJECT_SOURCE_DIR}/src/render/vulkan/vulkan_surface_session.cpp)

  mln_target_vendor_sources(${target} ${MLN_FFI_VENDOR_VULKAN_SOURCES})
  mln_target_project_sources(${target} ${MLN_FFI_VULKAN_SOURCES})

  target_link_libraries(${target} PRIVATE ${MLN_VULKAN_LOADER_LIBRARY})
endfunction()

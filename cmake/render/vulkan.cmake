function(mln_configure_vulkan_backend target)
  if(DEFINED ENV{MLN_FFI_GRAPHICS_LIBRARY_DIR}
     AND NOT "$ENV{MLN_FFI_GRAPHICS_LIBRARY_DIR}" STREQUAL "")
    list(APPEND CMAKE_LIBRARY_PATH "$ENV{MLN_FFI_GRAPHICS_LIBRARY_DIR}")
  endif()

  find_package(Vulkan MODULE REQUIRED)

  set(MLN_FFI_VENDOR_VULKAN_SOURCES
      ${MLN_SOURCE_DIR}/platform/default/src/mbgl/vulkan/headless_backend.cpp)
  set(MLN_FFI_VULKAN_SOURCES
      ${PROJECT_SOURCE_DIR}/src/render/vulkan/vulkan_texture_session.cpp
      ${PROJECT_SOURCE_DIR}/src/render/vulkan/vulkan_texture_backend.cpp
      ${PROJECT_SOURCE_DIR}/src/render/vulkan/vulkan_surface_session.cpp)

  mln_target_vendor_sources(${target} ${MLN_FFI_VENDOR_VULKAN_SOURCES})
  mln_target_project_sources(${target} ${MLN_FFI_VULKAN_SOURCES})

  target_link_libraries(${target} PRIVATE "${Vulkan_LIBRARY}")
endfunction()

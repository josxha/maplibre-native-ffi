if(WIN32 AND MSVC)
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

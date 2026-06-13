# Android native dependencies

Android cross-builds consume a small set of native libraries that the NDK does
not ship. This directory pins those versions and fetches them the same way the
MapLibre Native Android test runner does: **Maven Prefab packages**.

## curl

[`io.github.vvb2060.ndk:curl`](https://github.com/vvb2060/curl-android) provides
`libcurl` as a Prefab package. MapLibre Native's
`test/android/app/build.gradle.kts` already depends on the same artifact.

Standalone `mise run build` for Android variants does not use Gradle. Instead,
`fetch-prefab.sh`:

1. Downloads the curl AAR from Maven Central.
2. Runs Google's Prefab CLI to generate CMake configs for the active ABI, min
   SDK, NDK major version, and `c++_static` STL.
3. Writes the result to `build/android-prefab/<abi>/`.

CMake then uses `find_package(curl CONFIG)` with the Prefab output added to
`CMAKE_FIND_ROOT_PATH` at `build/android-prefab/<abi>/lib/<ndk-triple>/`,
matching the Gradle + `externalNativeBuild` workflow used elsewhere in the
MapLibre ecosystem.

When an Android Gradle module is added for JNI bindings, it can depend on the
same Maven coordinate and drop the standalone fetch step.

# Android native dependencies

Android cross-builds consume a small set of native libraries that the NDK does
not ship. This directory pins those versions for optional standalone CMake
builds.

## curl

[`io.github.vvb2060.ndk:curl`](https://github.com/vvb2060/curl-android) provides
`libcurl` as a Prefab package. MapLibre Native's
`test/android/app/build.gradle.kts` already depends on the same artifact.

The primary Android native build path is `bindings/android-native`, an Android
Gradle library that uses `externalNativeBuild` and resolves curl from Maven
Prefab. JNI bindings and future Android consumers depend on that module.

Standalone `mise -E android-* run build` still uses `fetch-prefab.sh` to
generate CMake package configs under `build/android-prefab/<abi>/` because those
invocations do not run Gradle.

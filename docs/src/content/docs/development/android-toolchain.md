---
title: Android Toolchain
description: Technical decisions for Android cross-compilation in this repository.
sidebar:
  order: 20
---

## Scope

This note records Android toolchain decisions for the maplibre-native-ffi C API
build. It covers mise environments, SDK/NDK provisioning, Pixi usage, Prefab
native dependencies, and the headless platform layer.

## Mise environments

Four Android variants follow the existing `<platform>-<arch>-<backend>` naming:

| Variant                | Render backend    | NDK ABI     | Notes                                     |
| ---------------------- | ----------------- | ----------- | ----------------------------------------- |
| `android-arm64-vulkan` | Vulkan            | `arm64-v8a` | Primary backend on modern Android devices |
| `android-arm64-egl`    | OpenGL ES via EGL | `arm64-v8a` | GLES3 headless backend                    |
| `android-x64-vulkan`   | Vulkan            | `x86_64`    | Emulator and x86_64 Android targets       |
| `android-x64-egl`      | OpenGL ES via EGL | `x86_64`    | JNI tests on x86_64 emulators             |

All use `android-30` as the minimum platform level. API 30 is required for
`AImageDecoder` image decoding.

### Rejected alternatives

- **`android-arm64-opengl` without a context provider suffix** — rejected
  because desktop variants distinguish EGL and WGL explicitly (`linux-x64-egl`,
  `windows-x64-wgl`). Android only supports EGL today, but keeping the provider
  in the name preserves parity with desktop OpenGL variants.
- **`android-egl` envs without arch in the name** — rejected. Desktop Linux uses
  separate `linux-x64-*` and `linux-arm64-*` envs; Android follows the same
  model instead of building multiple ABIs from one misleadingly named env.
- **Per-ABI task arguments on a single `android-arm64-*` env** — rejected for
  the same reason: arch belongs in the variant name.
- **Gradle/AGP as the primary native build driver** — rejected for the C API
  library. MapLibre Native's Android SDK uses AGP `externalNativeBuild`; this
  repository keeps CMake+Ninja as the native entrypoint so Android variants stay
  consistent with desktop mise workflows. The Java JNI binding uses Gradle only
  for Java compilation, JavaCPP JNI bridge packaging, and Android
  instrumentation tests; CMake still produces `libmaplibre-native-c.so`.

## Android SDK and NDK

The Android SDK is treated as an **external platform SDK**, like Xcode or Visual
Studio:

- Contributors install the Android SDK command-line tools or Android Studio and
  export `ANDROID_SDK_ROOT` or `ANDROID_HOME`.
- `.mise/bin/android-sdk-env.sh` resolves `ANDROID_NDK_ROOT` from the pinned NDK
  version under that SDK.
- `mise run //:ensure-android-sdk` is an optional helper: when `sdkmanager` is
  on `PATH`, it installs `platform-tools`, `platforms;android-34`, and
  `ndk;28.1.13356709`.
- NDK **28.1.13356709** matches the MapLibre Native Android tree.

### Rejected alternatives

- **mise-managed Android SDK in committed config** — rejected. The SDK is large,
  license-gated, and already installed by most Android contributors via Studio.
- **System-wide `apt install android-sdk`** — rejected. It conflicts with the
  repository policy of locally managed, pinned toolchains and is harder to
  reproduce across hosts.
- **Checking the SDK into the repository** — rejected for size and license
  churn.
- **Letting contributors install NDK versions manually without pinning** —
  rejected because MapLibre Native and the NDK libc++ ABI are version-sensitive.

## Pixi and native dependencies

Pixi remains the provider of **host build tools** for Android variants: CMake,
Ninja, glslang, and formatting dependencies. Pixi does **not** supply Android
target libraries.

Android cross-compiles set `MLN_FFI_DEPENDENCY_INCLUDE_DIR` and
`MLN_FFI_DEPENDENCY_LIBRARY_DIR` to empty values. Target dependencies come from:

- the NDK (`z`, `log`, `android`, `EGL`, `GLESv3`, `vulkan`, `jnigraphics`),
- MapLibre Native vendored libraries (`mbgl-vendor-icu`, `mbgl-vendor-sqlite`,
  …),
- MapLibre Native platform sources under
  `third_party/maplibre-native/platform/android/`,
- Maven Prefab packages fetched into `build/android-prefab/` (see below).

### Rejected alternatives

- **Adding an `android-arm64` Pixi platform** — rejected. conda-forge does not
  provide a maintained Android cross sysroot/package set comparable to its Linux
  and macOS offerings; fighting that model would duplicate the NDK.
- **Dropping Pixi for Android envs entirely** — rejected. Host tasks still need
  pinned CMake/Ninja/glslang; skipping Pixi would fork the contributor workflow.
- **Reusing desktop Pixi libraries with `-DCMAKE_FIND_ROOT_PATH`** — rejected.
  Linux `.so` artifacts are not link-compatible with Android targets.
- **vcpkg for Android target deps** — deferred to a separate change. vcpkg may
  replace Pixi on all targets later; for now Maven Prefab matches MapLibre
  Native's existing Android test app.

## Maven Prefab dependencies (`platform/android-deps/`)

Standalone `mise run build` for Android does not use Gradle. Instead,
`fetch-prefab.sh` downloads Prefab AARs from Maven Central and runs Google's
Prefab CLI to generate CMake package configs:

| Package   | Maven coordinate                      | Purpose                         |
| --------- | ------------------------------------- | ------------------------------- |
| curl      | `io.github.vvb2060.ndk:curl:8.8.0`    | HTTP via `http_file_source.cpp` |
| boringssl | `io.github.vvb2060.ndk:boringssl:4.0` | curl transitive dependency      |

Versions are pinned in `platform/android-deps/dep-versions.toml`, aligned with
`third_party/maplibre-native/test/android/app/build.gradle.kts`.

The configure task runs `mise run //:fetch-android-prefab` automatically. Output
lands in `build/android-prefab/<abi>/lib/<ndk-triple>/cmake/`. CMake adds that
prefix to `CMAKE_FIND_ROOT_PATH` and uses `find_package(curl CONFIG)` with
`curl::curl_static`, matching the AGP `externalNativeBuild` workflow.

When an Android Gradle module is added for JNI bindings, it can depend on the
same Maven coordinates and drop the standalone fetch step. The
`bindings/java-jni` module is that Gradle surface today: it consumes the
CMake-built `libmaplibre-native-c.so`, cross-compiles `libjniMaplibreNativeC.so`
with JavaCPP, and runs instrumentation tests on device or emulator.

### Rejected alternatives

- **JNI OkHttp HTTP bridge** — rejected for the headless C API layer. The
  default `http_file_source.cpp` plus static curl keeps behavior aligned with
  other platforms.
- **HTTP stub returning errors** — rejected once Prefab curl was viable.
- **Pixi linux-aarch64 curl** — rejected. Host libc artifacts are not
  link-compatible with Android targets.

## CMake and platform code

Android builds pass the NDK's `android.toolchain.cmake` from the `configure`
task. `MLN_WITH_CORE_ONLY` stays enabled for MapLibre Native; the C API wrapper
continues to own platform sources through `cmake/platform/android.cmake`.

Android platform choices:

- Android event-loop sources (`run_loop`, `timer`, `thread`, `async_task`) from
  MapLibre Native's Android tree.
- Collator/number-format test stubs instead of the JNI i18n stack used by the
  MapLibre Android SDK.
- Default `http_file_source.cpp` linked against Prefab `curl::curl_static`.
- `decodeImage` in `src/platform/android/image.cpp` using `AImageDecoder` and
  `libjnigraphics` (requires minSdk 30).

### Rejected alternatives

- **Disabling `MLN_WITH_CORE_ONLY` and reusing MapLibre Native `android.cmake`
  wholesale** — rejected for now. That file also defines SDK/test/benchmark
  targets and JNI-centric HTTP/image paths that are out of scope for the
  headless C API library.
- **JNI `BitmapFactory` image decode** — rejected. `AImageDecoder` is available
  from API 30 without JNI and fits the headless C API model.
- **Using desktop Linux platform sources with Pixi sysroots** — rejected.
  Android is not Linux for dependency discovery even though the NDK uses a Linux
  host toolchain triple.

## Commands

```bash
# Install host tools and hooks
mise install

# Point at your Android SDK (Studio default shown)
export ANDROID_SDK_ROOT="$HOME/Android/Sdk"

# Optional: install the pinned NDK when sdkmanager is available
mise run //:ensure-android-sdk

# Configure and build the Vulkan Android variant
mise -E android-arm64-vulkan run build

# Configure and build the OpenGL ES/EGL Android variant
mise -E android-arm64-egl run build
mise -E android-x64-egl run build

# Build the Android JNI binding (packages arm64-v8a and x86_64)
mise run //bindings/java-jni:build
```

Zig binding tests are not wired into `mise run test` for Android yet. JNI
instrumentation tests run with `mise run //bindings/java-jni:test` on a
connected device or emulator. Use an `x86_64` emulator for local software-mode
testing, or `arm64-v8a` on a KVM-capable host.

## Follow-up work

- Additional ABIs (`armeabi-v7a`, `x86`).
- Android AAR publishing for JNI consumers.
- Vulkan JNI render tests (EGL GLES tests run today).
- vcpkg migration for cross-target native dependencies (separate change).

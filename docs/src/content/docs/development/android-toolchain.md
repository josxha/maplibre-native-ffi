---
title: Android Toolchain
description: Technical decisions for Android cross-compilation in this repository.
sidebar:
  order: 20
---

## Scope

This note records Android toolchain decisions for the maplibre-native-ffi C API
build. It covers mise environments, SDK/NDK provisioning, Pixi usage, and the
initial headless platform layer.

## Mise environments

Two Android variants follow the existing `<platform>-<arch>-<backend>` naming:

| Variant                | Render backend    | Notes                                     |
| ---------------------- | ----------------- | ----------------------------------------- |
| `android-arm64-vulkan` | Vulkan            | Primary backend on modern Android devices |
| `android-arm64-egl`    | OpenGL ES via EGL | GLES3 headless backend                    |

Both target `arm64-v8a` with `android-24` as the minimum platform level,
matching MapLibre Native's current Android baseline.

### Rejected alternatives

- **`android-arm64-opengl` without a context provider suffix** — rejected
  because desktop variants distinguish EGL and WGL explicitly (`linux-x64-egl`,
  `windows-x64-wgl`). Android only supports EGL today, but keeping the provider
  in the env name preserves parity with desktop OpenGL variants.
- **Per-ABI mise envs (`android-armv7-vulkan`, etc.)** — deferred. MapLibre
  Native still builds multiple ABIs, but this repository starts with `arm64-v8a`
  only to keep the first cross-compile path small. Additional ABIs can share the
  same env with a task argument later.
- **Gradle/AGP as the primary native build driver** — rejected for the C API
  library. MapLibre Native's Android SDK uses AGP `externalNativeBuild`; this
  repository keeps CMake+Ninja as the native entrypoint so Android variants stay
  consistent with desktop mise workflows. Gradle may return when we add Android
  packaging or JNI bindings.

## Android SDK and NDK

The Android SDK is treated as an **external platform SDK**, like Xcode or Visual
Studio:

- mise installs the command-line tools via
  [`vfox-android-sdk`](https://github.com/mise-plugins/vfox-android-sdk).
- `mise run //:ensure-android-sdk` accepts licenses and installs pinned
  packages: `platform-tools`, `platforms;android-34`, and `ndk;28.1.13356709`.
- NDK **28.1.13356709** matches the MapLibre Native Android tree.

### Rejected alternatives

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

- the NDK (`z`, `log`, `android`, `EGL`, `GLESv3`, `vulkan`),
- MapLibre Native vendored libraries (`mbgl-vendor-icu`, `mbgl-vendor-sqlite`,
  …),
- MapLibre Native platform sources under
  `third_party/maplibre-native/platform/android/`.

### Rejected alternatives

- **Adding an `android-arm64` Pixi platform** — rejected. conda-forge does not
  provide a maintained Android cross sysroot/package set comparable to its Linux
  and macOS offerings; fighting that model would duplicate the NDK.
- **Dropping Pixi for Android envs entirely** — rejected. Host tasks still need
  pinned CMake/Ninja/glslang; skipping Pixi would fork the contributor workflow.
- **Reusing desktop Pixi libraries with `-DCMAKE_FIND_ROOT_PATH`** — rejected.
  Linux `.so` artifacts are not link-compatible with Android targets.

## CMake and platform code

Android builds pass the NDK's `android.toolchain.cmake` from the `configure`
task. `MLN_WITH_CORE_ONLY` stays enabled for MapLibre Native; the C API wrapper
continues to own platform sources through `cmake/platform/android.cmake`.

Initial Android platform choices:

- Android event-loop sources (`run_loop`, `timer`, `thread`, `async_task`) from
  MapLibre Native's Android tree.
- Collator/number-format test stubs instead of the JNI i18n stack used by the
  MapLibre Android SDK.
- No `http_file_source` / `online_file_source` until we add a static networking
  stack or JNI HTTP bridge.
- `decodeImage` stub in `src/platform/android/image_stub.cpp` until we either
  vendor static libpng/libjpeg/libwebp or adopt the JNI bitmap decoder path.

### Rejected alternatives

- **Disabling `MLN_WITH_CORE_ONLY` and reusing MapLibre Native `android.cmake`
  wholesale** — rejected for now. That file also defines SDK/test/benchmark
  targets and JNI-centric HTTP/image paths that are out of scope for the
  headless C API library.
- **Using desktop Linux platform sources with Pixi sysroots** — rejected.
  Android is not Linux for dependency discovery even though the NDK uses a Linux
  host toolchain triple.

## Commands

```bash
# Install host tools, hooks, and the Android command-line SDK package
mise install

# Configure and build the Vulkan Android variant
mise -E android-arm64-vulkan run build

# Configure and build the OpenGL ES/EGL Android variant
mise -E android-arm64-egl run build
```

Zig binding tests are not wired into `mise run test` for Android yet. The first
milestone is a successful `libmaplibre-native-c.so` link for each Android
variant.

## Follow-up work

- Static or JNI-backed HTTP and image decoding.
- Additional ABIs (`armeabi-v7a`, `x86_64`).
- Android packaging (AAR/Gradle) and JNI bindings.
- CI job on an Ubuntu host with the mise-managed SDK.

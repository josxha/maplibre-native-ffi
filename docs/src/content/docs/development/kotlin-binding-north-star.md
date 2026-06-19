---
title: Kotlin Binding North Star
description: Target architecture for consolidating JVM, Android, and Kotlin Native bindings.
sidebar:
  order: 6
---

## Scope

This document describes the target shape for consolidating the current Java FFM,
Java JNI, and Kotlin Native bindings into one Kotlin Multiplatform binding.

The binding remains a low-level layer over the public C API. It exposes MapLibre
Native concepts directly, follows the
[binding specification](/maplibre-native-ffi/development/binding-specification/),
and leaves UI widgets, gestures, lifecycle adapters, coroutine dispatchers, and
application framework policy to downstream adapters.

## Target Shape

The north-star module is a single Kotlin Multiplatform subproject:

```text
bindings/kotlin/
  src/commonMain/kotlin/
  src/jvmMain/kotlin/
  src/androidMain/kotlin/
  src/nativeMain/kotlin/
```

`commonMain` owns the public Kotlin API and pure Kotlin support code. Platform
source sets own only the raw C bridge and host-specific loading, allocation,
callback, and packaging mechanics.

| Source set    | Bridge                   | Responsibility                                                                  |
| ------------- | ------------------------ | ------------------------------------------------------------------------------- |
| `commonMain`  | none                     | Public values, handles, errors, callbacks, validation, and `expect` facades.    |
| `jvmMain`     | Java FFM / `jextract`    | Desktop and server JVM bridge for JDKs that support FFM.                        |
| `androidMain` | JNI / JavaCPP            | Android bridge, AAR packaging, Android platform initialization, emulator tests. |
| `nativeMain`  | Kotlin/Native `cinterop` | Native bridge for macOS, Linux, iOS, and other Kotlin/Native targets.           |

The public package root should stay `org.maplibre.nativeffi` unless a concrete
publishing reason emerges to rename it. The current `org.maplibre.nativejni`
package should disappear with the separate Android JNI binding.

## Public API Direction

Use the current Kotlin Native API as the starting public shape. It is already
closer to idiomatic Kotlin than the Java bindings: data classes for copied
values, value classes for raw-value concepts, properties where they fit,
nullable values instead of `Optional`, and Kotlin exception types.

Java FFM and Java JNI are implementation inputs, not public APIs to preserve.
The project is still prerelease, so consolidation should prefer one clean Kotlin
API over compatibility shims for old Java packages. Java-callable polish is not
a goal for the consolidation PR. Keep future Java interop in mind when it costs
little, but do not add Java-specific overloads, annotations, builders, or API
shapes just to improve Java ergonomics in this pass.

Public APIs keep the same concepts across all targets:

- `RuntimeHandle`, `MapHandle`, `MapProjectionHandle`, `RenderSessionHandle`,
  `OfflineOperationHandle`, and `ResourceRequestHandle` own or control native
  state.
- Input descriptors and copied results stay as Kotlin-owned values.
- Raw C declarations, JavaCPP pointers, FFM `MemorySegment`, cinterop pointers,
  JNI handles, and raw addresses stay internal.
- `NativePointer` remains the only public carrier for borrowed backend-native
  addresses.
- Unknown-preserving value wrappers remain available for C enum domains where
  future native values can appear in outputs.

## Common Code

Move code to `commonMain` when it is pure Kotlin and expresses public binding
semantics rather than a foreign-memory mechanism.

Good `commonMain` candidates:

- copied geometry, camera, style, query, resource, runtime, offline, and render
  value types;
- public error classes, `MaplibreStatus`, and status-to-exception mapping;
- public callback interfaces and copied callback input values;
- binding-owned validation such as embedded-NUL checks, non-negative dimensions,
  one-shot completion checks, and unknown-input rejection;
- handle lifecycle state, parent-child retention, close/release state, leak
  report policy, and callback reentry guards;
- public handle method bodies after native operations are hidden behind internal
  facades.

Bridge-specific code stays outside `commonMain`:

- raw C declarations and generated sources;
- native library loading and ABI probing;
- native memory scopes, arenas, pinned arrays, and struct layout reads/writes;
- thread-local diagnostic reads;
- callback trampolines and exception containment at the C boundary;
- Android `Context`, `JNIEnv`, `JavaVM`, AAR packaging, and instrumentation test
  setup.

## Internal Bridge Facades

The consolidation should push platform details behind small internal
`expect`/`actual` APIs. The common public classes should not know whether a call
crosses through FFM, JNI, or cinterop.

The expected layer should be grouped by C API domain, not by individual C
function. A useful shape is:

```kotlin
internal expect object NativeLibraryBridge {
  fun load()
  fun cVersion(): Long
  fun lastErrorMessage(): String
}

internal expect object NativeMapBridge {
  fun create(runtime: NativeRuntimeHandle, options: MapOptions): NativeMapHandle
  fun destroy(map: NativeMapHandle): Int
  fun setStyleUrl(map: NativeMapHandle, url: String): Int
  fun getStyleSourceInfo(map: NativeMapHandle, sourceId: String): SourceInfo?
}
```

The exact API should grow from migrations, but the rule is stable: common code
owns binding semantics, platform actuals own byte layout and call mechanics.

Handle identity should be an internal expect/actual value rather than a public
address. FFM can wrap `MemorySegment`, JNI can wrap a `Long` or table-owned
handle, and cinterop can wrap `CPointer<T>`. The common handle state tracks
logical liveness and parent retention while actual bridge code performs the
destroy call.

## Android Source Set

Android is not a JVM FFM target. It gets a dedicated `androidMain` actual bridge
and an Android library artifact.

The native Android core build already exists on main through the
`android-arm64-egl` and `android-arm64-vulkan` mise environments. Those
environments currently target `android-24` and `arm64-v8a`. The Kotlin
consolidation should build on that baseline instead of treating Android native
support as new work.

Android-only responsibilities:

- consume or package the mise-built `libmaplibre-native-c.so` for the active
  Android variant and package the JNI bridge beside it;
- call the C API Android initializer from an Android-aware public entry point,
  for example `MaplibreAndroid.initialize(context)`;
- keep Android `Context` and `JNIEnv` out of `commonMain`;
- run instrumentation tests on an emulator for JNI, library loading, callbacks,
  resource loading, render sessions, and OpenGL ES/EGL behavior;
- compile with Android-compatible Java/Kotlin bytecode instead of desktop JDK 25
  assumptions.

`mln_android_init(void* jni_env, void* context)` is platform integration, not a
general binding concept. Keep it in `androidMain` so desktop JVM and
Kotlin/Native targets do not carry Android types or initialization rules.

## Lessons From PR 214

[PR #214](https://github.com/maplibre/maplibre-native-ffi/pull/214) is useful as
a prior Android JNI spike. It should inform this design without becoming the
architecture to copy directly.

What the spike proved:

- Java JNI can be made Android-specific while Java FFM remains the desktop JVM
  bridge.
- AGP can build an Android AAR around the C API and JavaCPP-generated JNI
  declarations.
- Android JNI requires packaging and test wiring beyond the existing native core
  build: JNI `OnLoad`, image decoding, Android-aware library loading, AAR/native
  library packaging, and emulator tests all matter.
- EGL should be modeled as a backend capability such as `MLN_OPENGL_USE_EGL`,
  not inferred from `__linux__`.
- Android tests need emulator execution for behavior that desktop JVM tests
  cannot represent.

What the spike exposed as design debt:

- The build hardcoded OpenGL/EGL and debug-variant paths inside the JNI Gradle
  module. The north-star binding should let the repo variant matrix drive native
  backend selection, ABI selection, and build type.
- JNI bridge packaging was tied to AGP internal output paths and debug CMake
  outputs. The production design needs variant-aware artifact wiring.
- Host-specific library-name helpers such as `System.mapLibraryName()` are a bad
  fit for Android packaging when the host OS differs from the target OS.
- The Java source had to downgrade newer Java constructs for Android. Moving the
  public surface to Kotlin common code avoids keeping separate Java APIs in
  sync, but Android actuals still need Android-compatible bytecode and
  dependencies.
- Manual Android native resources need RAII-style ownership. The image decoder
  leak found in review is a concrete example: every Android native handle should
  be owned by a scope or wrapper that releases on all failure paths.
- Removed or weakened exhaustiveness checks can silently marshal partial native
  structs. Kotlin sealed classes in common code should keep terminal rejection
  paths explicit when actual code materializes native layouts.

PR #214 was still failing its `android-jni` check when this note was written.
Treat it as evidence for the work areas and failure modes, not as a merge-ready
baseline.

## Build and Packaging Direction

Keep the repository's native variant model as the source of truth. Android
already uses the same variant vocabulary as the rest of the repo:
`android-arm64-egl` and `android-arm64-vulkan`. The current baseline is
`android-24` and `arm64-v8a`.

Prefer consuming mise-built native artifacts first because the repo has not yet
focused on external packaging and the native Android core already builds through
mise. Gradle should package artifacts produced for the requested target variant.
Avoid duplicating backend policy inside the binding module. If AGP
`externalNativeBuild` turns out to be materially simpler for local Android
testing, it must still use the same CMake options, API level, ABI, and
dependency versions as the mise environment.

Desired task shape:

```bash
mise -E android-arm64-egl run //bindings/kotlin:androidBuild
mise -E android-arm64-egl run //bindings/kotlin:androidTest
mise run //bindings/kotlin:jvmTest
mise run //bindings/kotlin:nativeTest
```

The exact task names can follow local conventions during implementation. The
important invariant is that one binding module owns all Kotlin-family tests and
artifacts.

## Testing Direction

Use common tests for common semantics:

- value equality, formatting, and unknown-value preservation;
- status mapping and binding-owned diagnostics;
- handle release, parent retention, child-count failures, and leak reporting;
- callback state transitions and one-shot request completion;
- public API validation that does not need a real C call.

Use target tests for bridge behavior:

- JVM tests for FFM loading, `jextract` layout assumptions, desktop render
  sessions, and LWJGL-backed examples;
- Android instrumentation tests for JNI loading, Android initialization,
  emulator OpenGL ES/EGL render paths, Android image/resource behavior, and
  packaged native libraries;
- Kotlin/Native tests for cinterop layouts, native linker behavior, native
  callbacks, and Apple/Linux render targets.

Test skips require strict platform justification. Rendering tests should drive
environment fixes when possible.

## Migration Plan

1. Rename or replace `:bindings:kotlin-native` with `:bindings:kotlin` while
   keeping the existing native target green.
2. Move pure Kotlin public values, errors, and tests from `nativeMain` to
   `commonMain`.
3. Introduce internal native-handle wrappers and `expect` bridge objects for
   process-global `Maplibre`, status, diagnostics, and core value structs.
4. Move one domain at a time behind the bridge facades: runtime, map, render,
   style, query, resources, offline.
5. Add `jvmMain` actuals from Java FFM and retire the separate
   `:bindings:java-ffm` public module after the JVM tests pass against the
   Kotlin API.
6. Add `androidMain` actuals from the Android JNI spike, with variant-aware
   packaging and emulator tests.
7. Remove `:bindings:java-jni` and update Android docs/tasks to point at the
   Kotlin module.
8. Convert JVM examples that consume the Java FFM API, including `lwjgl-map`, to
   Kotlin when needed so examples can use `:bindings:kotlin` without requiring a
   Java-polished public API.
9. Update docs so Android examples start from the Android source set when they
   arrive.

## Open Decisions

- Whether Android packaging can stay purely mise-built for the native core, or
  whether AGP `externalNativeBuild` is simpler for instrumentation tests and AAR
  assembly.
- Whether JavaCPP causes any concrete problems with Kotlin `androidMain`. Keep
  JavaCPP by default unless Kotlin/AGP integration makes it meaningfully worse
  than a smaller purpose-built JNI shim.

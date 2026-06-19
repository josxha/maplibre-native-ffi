---
title: Kotlin Binding Implementation Log
description: Running notes from implementing the Kotlin binding consolidation plan.
sidebar:
  order: 7
---

This log records discoveries, problems, and implementation reflections while the
[Kotlin binding north star](/maplibre-native-ffi/development/kotlin-binding-north-star/)
is being implemented.

## 2026-06-19

### Module Rename Milestone

- Renamed the Kotlin binding subproject from `:bindings:kotlin-native` to
  `:bindings:kotlin` while preserving the existing Kotlin/Native source-set
  layout and native build behavior.
- Discovery: GitHub Actions CI is generated from `ci/subprojects/*.toml`, so the
  subproject manifest needs to move with the Gradle module rename and the
  workflow needs regeneration.
- Discovery: `ci/variants.toml` already includes `android-arm64-egl` and
  `android-arm64-vulkan` build variants, but the overview platform status table
  does not yet distinguish build-only Android coverage from tested platform
  coverage.
- Problem: `mise run //bindings/kotlin:build` succeeded after the rename, but
  Gradle printed a `SerializableTestResultStore$Writer.output` null-pointer
  stack trace while processing Kotlin/Native test output. The test task still
  completed successfully, so this is recorded as existing tooling noise to watch
  during later test refactors.
- Reflection: this is intentionally a small milestone. It makes the repository
  vocabulary match the destination module before moving code between source
  sets, which keeps later diffs easier to review.

### Common Source Set Milestone

- Moved the platform-neutral public model layer from `nativeMain` to
  `commonMain`, including camera, geometry, JSON, logging, error, map option,
  offline, query, render descriptor, resource payload, runtime payload, and
  style payload types.
- Discovery: most Kotlin/Native files without direct `kotlinx.cinterop` imports
  are pure Kotlin model types and can move as dependency-closed groups without
  changing package names or call sites.
- Discovery: `NativePointer` looks like a native-only type by name, but its
  public representation is an opaque address bit pattern used by common render
  descriptors. Its small `FrameScope` lifetime guard is also pure Kotlin and can
  live in `commonMain` as an internal helper.
- Problem: `ResourceProviderCallback` could not move yet because it exposes the
  native `ResourceRequestHandle`. That callback needs an `expect` facade or a
  common handle abstraction before it can become common.
- Problem: `RuntimeEvent` could not move yet because it currently exposes
  concrete `RuntimeHandle` and `MapHandle` references. The payload hierarchy is
  common, but the event envelope still needs an expect facade or a platform-free
  source representation.
- Problem: `mise run //bindings/kotlin:build` continues to succeed while Gradle
  prints Kotlin/Native test-output processing exceptions. This milestone saw
  `LifecycleTrackingTestEventReporter` report that `close()` had already been
  called after native test output. The build still completed successfully.

### JVM Target Milestone

- Added a JVM target to the unified Kotlin Multiplatform module, so the
  extracted `commonMain` model layer now compiles for JVM as well as the host
  Kotlin/Native target.
- Discovery: using the Java 25 toolchain without an explicit Kotlin bytecode
  target makes Kotlin warn and fall back from JVM 25 to JVM 24. The module now
  keeps the Java 25 toolchain for future FFM work while explicitly targeting JVM
  24 bytecode.
- Reflection: this is a build-shape milestone rather than an FFM migration. It
  creates the `jvmMain` destination and verifies the common API surface before
  moving Java FFM internals into Kotlin.

### JVM Loader Milestone

- Moved the first self-contained piece of the Java FFM bridge into Kotlin
  `jvmMain`: the native library lookup and one-time load helper used by future
  JVM FFM actuals.
- Added a JVM test for explicit missing-path failures, which verifies the loader
  reports the configured source and path without requiring a generated FFM
  binding or a successful native load.
- Discovery: the loader can migrate independently from jextract because it only
  depends on JDK APIs. That makes it a useful first `jvmMain` bridge component
  before moving symbol lookup, ABI checks, and generated/native downcalls.

### JVM Native Access Milestone

- Added the Kotlin/JVM native access gate that loads the C ABI library once,
  checks the C ABI version, and converts FFM native-access and missing-symbol
  failures into binding-owned exceptions.
- Discovery: the ABI check does not need jextract-generated declarations. A
  direct Java FFM downcall to `mln_c_version` is enough for the loader/access
  boundary, which keeps this migration step independent from the larger
  generated FFM surface.
- Discovery: JVM tests for this layer need `--enable-native-access=ALL-UNNAMED`
  and the mise-built native library path wired through the Kotlin module's
  `jvmTest` task before direct FFM integration tests can be added safely.
- Reflection: direct FFM is useful for the bootstrap path, but the broader
  wrapper migration still needs a clear decision about whether to keep jextract
  generated declarations in `jvmMain` or replace selected calls with
  hand-written downcall helpers.

### JVM Maplibre Facade Milestone

- Added the first Kotlin/JVM public `Maplibre` facade, backed by the new
  `jvmMain` native access layer, with native library loading and C ABI version
  entry points.
- Discovery: JVM and Kotlin/Native can each provide a platform-specific
  `org.maplibre.nativeffi.Maplibre` object before a common `expect` declaration
  exists. This keeps the first JVM facade small while preserving room to promote
  the shared shape into `commonMain` once the JVM implementation covers the same
  member set as `nativeMain`.
- Reflection: this is still a transitional platform facade. The destination is
  an `expect object Maplibre` in `commonMain` with JVM FFM and Kotlin/Native
  actuals once the JVM side has enough behavior to satisfy the common contract.

### Common Maplibre Expect Milestone

- Promoted the shared bootstrap subset of `Maplibre` into `commonMain` as the
  first `expect` facade, with JVM FFM and Kotlin/Native actual objects for the
  expected C ABI version, native library loading, and C ABI version query.
- Discovery: `expect val` declarations cannot be `const` because they do not
  have common initializers. The common API now exposes `EXPECTED_C_ABI_VERSION`
  as a normal `val`, while platform actuals can still initialize it from
  platform constants.
- Discovery: Kotlin 2.2 still reports expect/actual classes as beta unless the
  module opts in with `-Xexpect-actual-classes`. The Kotlin module now sets that
  compiler flag explicitly.
- Reflection: the first expect facade is intentionally narrow. It proves the
  common facade pattern without forcing the JVM bridge to implement every
  process-global native operation in the same commit.

### Android Source Set Milestone

- Added the Android KMP library plugin to the unified Kotlin module and created
  the first `androidMain` actual for the shared `Maplibre` bootstrap facade.
- Added Android target configuration with namespace `org.maplibre.nativeffi`,
  compile SDK 36, min SDK 24, and JVM 17 bytecode for Android compilation.
- Discovery: AGP 9.2.0 requires Gradle 9.4.1, while the repository currently
  uses Gradle 9.3.1. AGP 9.1.1 matches the current wrapper, so this milestone
  uses 9.1.1 rather than forcing a Gradle wrapper upgrade.
- Discovery: the Android KMP plugin needs Google Maven in the module
  repositories for Android lint artifacts; plugin management repositories alone
  are not enough.
- Discovery: local desktop builds did not export `ANDROID_HOME` even though the
  SDK exists at the standard macOS location. The Kotlin mise task now exports
  that location when present, while CI and Android-specific runs should provide
  `ANDROID_HOME` or `ANDROID_SDK_ROOT` explicitly.
- Problem: the Android `Maplibre` actual currently defines the JNI boundary as a
  private external `nativeCVersion()` bootstrap call, but the JavaCPP-generated
  Android JNI library has not moved into `androidMain` yet and no native library
  is packaged in the AAR. This is a source-set and build-shape milestone, not a
  complete Android runtime milestone.
- Reflection: the official Android KMP plugin fits the mise-built-native-first
  direction because it avoids `externalNativeBuild` inside the KMP module. JNI
  packaging can stay as a later, explicit step once the JavaCPP bridge has moved
  under `androidMain`.

### Android JavaCPP Seed Milestone

- Enabled Java compilation for the Android KMP target and added the JavaCPP
  runtime dependency to `androidMain`.
- Added the JavaCPP C ABI configuration class under the unified
  `org.maplibre.nativeffi.internal.javacpp` package in `androidMain`, adapted
  from the existing Java JNI bridge configuration.
- Discovery: the Android KMP plugin keeps Java compilation disabled by default;
  `withJava()` is required before Android-only JavaCPP configuration sources can
  compile inside the KMP module.
- Reflection: this starts moving the JNI bridge into `androidMain` without
  touching the desktop Java JNI module yet. The next JNI milestone needs to add
  JavaCPP generation/build tasks for the Android target and connect the
  generated `MaplibreNativeC` declarations to the Kotlin Android actuals.

### Android JavaCPP Generation Milestone

- Added Gradle tasks in the Kotlin module to compile the Android JavaCPP config
  class and generate `MaplibreNativeC` declarations into
  `build/generated/sources/javacpp/androidMain/java`.
- Wired the generated JavaCPP source directory into the Android KMP variant and
  made Android Kotlin compilation, annotation extraction, and Java compilation
  depend on generation explicitly.
- Discovery: adding a generated Java source directory through
  `androidComponents` makes both Kotlin compilation and annotation extraction
  observe the directory, so Gradle validation requires explicit dependencies on
  the generator task for those consumers, not just for `javac`.
- Discovery: JavaCPP generation succeeds inside the KMP module using the same C
  header list as the Java JNI bridge, with the generated declarations now using
  the unified `org.maplibre.nativeffi.internal.javacpp` package.
- Problem: this still does not build or package the JavaCPP native JNI bridge
  library for Android. It only moves generated declarations into `androidMain`;
  native JavaCPP builder output and AAR `jniLibs` packaging remain future work.

### Android JavaCPP Facade Milestone

- Replaced the temporary Android `nativeCVersion()` external declaration with a
  call to the generated JavaCPP `MaplibreNativeC.mln_c_version()` declaration.
- Discovery: once the generated JavaCPP source directory is registered with the
  Android KMP variant and generation dependencies are explicit, Kotlin
  `androidMain` sources can compile against generated Android Java declarations
  in the same module.
- Reflection: the Android `Maplibre` actual is now shaped like the existing Java
  JNI bootstrap path: JavaCPP owns the native declaration surface, while Kotlin
  owns the public facade and ABI compatibility behavior.

### Common Maplibre Process Globals Milestone

- Expanded the common `Maplibre` expect facade to include process-global render
  backend support, OpenGL context provider support, and network status access.
- Added JVM FFM downcalls for the same process-global C functions used by the
  existing Java FFM binding, without pulling in the generated jextract surface.
- Added Android JavaCPP calls for the same process-global C functions, keeping
  JavaCPP as the Android declaration bridge while Kotlin owns validation and
  status-to-exception mapping.
- Discovery: this is a good boundary for common facade growth because all three
  platform bridges can implement it without handle identity, callbacks, struct
  layout materialization, or Android platform initialization.
- Reflection: direct FFM remains productive for small process-global functions,
  but broader JVM migration still needs a deliberate generated-vs-handwritten
  bridge boundary before moving map/runtime/resource handles into common code.

### Kotlin Task Shape Milestone

- Added explicit `jvmTest`, `nativeTest`, and `androidBuild` mise tasks for the
  unified Kotlin binding module, matching the task vocabulary proposed in the
  north-star build direction.
- Added Gradle aliases for `:bindings:kotlin:nativeTest` and
  `:bindings:kotlin:androidBuild` so the module owns the target-specific entry
  points instead of requiring contributors to remember generated task names.
- Discovery: the Kotlin module still needs Android SDK discovery even when
  running non-Android Gradle tasks because the Android KMP target participates
  in project configuration. The mise task fallback remains shared across these
  target-specific entries.
- Discovery: the native-library prerequisite is not safe to run in parallel for
  the same build directory. Parallel `jvmTest` and `nativeTest` validation can
  race during CMake regeneration and trigger a Ninja recompaction failure, while
  the same tasks pass when run sequentially.
- Problem: there is no `androidTest` task yet because the module does not have
  instrumentation test sources, packaged JNI bridge libraries, or emulator
  wiring. That remains tied to the Android packaging milestone.

### Common Test Source Set Milestone

- Moved pure common model tests for error mapping, geometry value snapshotting,
  and JSON value semantics from `nativeTest` to `commonTest` so JVM and
  Kotlin/Native both exercise those common APIs.
- Discovery: the first common-test candidates are the tests whose production
  types already moved to `commonMain` and whose assertions do not touch
  cinterop, native handles, render frame implementation classes, or real C
  calls.
- Reflection: moving tests in small dependency-closed groups is safer than
  sweeping whole directories, because some apparently model-oriented native
  tests still validate native struct materialization or live MapLibre behavior.

### Common Status Facade Milestone

- Moved internal C status-to-exception mapping from `nativeMain` to `commonMain`
  and hid thread-local diagnostic copying behind a small `NativeDiagnostics`
  expect/actual bridge.
- Replaced duplicated JVM FFM and Android JavaCPP status checks with the shared
  common `Status.check` path, preserving native diagnostic capture on all three
  targets.
- Discovery: `mln_thread_last_error_message()` is already present in the
  generated Android JavaCPP declarations, so Android can keep JavaCPP as the
  diagnostic bridge without adding a separate JNI helper.
- Reflection: this is the first internal bridge facade below the public
  `Maplibre` expect object. It keeps binding-owned status semantics common while
  leaving raw pointer/string conversion in platform source sets.

### Common Status Test Milestone

- Moved pure status mapping tests into `commonTest`, so JVM and Kotlin/Native
  both exercise the shared `Status` facade.
- Kept tests that depend on deterministic C status producers, copied native
  diagnostics, and cinterop memory utilities in `nativeTest`.
- Discovery: common status mapping tests can synthesize non-OK status codes
  before a native library is loaded. Platform diagnostic bridges therefore need
  to treat missing native diagnostic symbols as an empty diagnostic while still
  copying diagnostics after real C calls.
- Reflection: after a bridge helper moves to `commonMain`, its tests should move
  with it unless they are specifically testing the platform actual or native C
  behavior.

### Common Handle Lifecycle Core Milestone

- Extracted the platform-neutral handle release-state machine into `commonMain`
  as `HandleStateCore`, leaving Kotlin/Native `CPointer` storage and cleaner
  registration in the native `HandleState` wrapper.
- Added common lifecycle tests for failed destroy retry, reentrant release
  rejection, released idempotence, and leak report text.
- Discovery: the existing Kotlin/Native handle wrapper could move lifecycle
  transitions without changing handle call sites because every native handle
  already funnels `requireLive`, `isReleased`, `address`, and `closeOnce`
  through one internal class.
- Reflection: this creates the lifecycle base needed by future common public
  handle classes without prematurely deciding whether JVM handles are
  `MemorySegment`, Android handles are JavaCPP pointer objects, or both use
  address carriers internally.

### Common Handle Child Retention Milestone

- Added parent-child retention bookkeeping to the common lifecycle core, with a
  common `Status.liveChildren` diagnostic for attempted parent close while child
  handles are still live.
- Added common tests for blocking parent close, retrying after child release,
  and idempotent child retention release.
- Reflection: this aligns the Kotlin lifecycle core with the Java bindings'
  deterministic destruction model while keeping actual handle ownership and
  platform native handle carriers out of `commonMain`.

### Native Handle Retention Wiring Milestone

- Wired the common lifecycle child-retention core into the Kotlin/Native handle
  wrapper and the first real parent relationships: runtime-to-map and
  map-to-render-session.
- Kept map projections independent because they are standalone snapshots and the
  existing Kotlin API intentionally allows them to outlive the map that created
  them.
- Discovery: the native runtime close test already expected live maps to block
  runtime destruction, so wiring the common retention core tightened the
  diagnostic without changing that public behavior.

### Offline Operation Retention Milestone

- Wired `OfflineOperationHandle` into the common lifecycle child-retention model
  so live offline operations retain their `RuntimeHandle` until they are taken,
  discarded, or explicitly consumed by tests.
- Updated the native runtime test to expect runtime close to fail while an
  offline operation is live, matching the Java bindings' deterministic
  destruction behavior.
- Reflection: this is a small public behavior tightening, but it aligns with the
  north-star rule that handle lifecycle state and parent retention are binding
  semantics rather than platform bridge mechanics.

### Common Callback Gate Milestone

- Extracted the close-during-callback state machine into a common
  `CallbackGate`, with common tests for immediate close, deferred native
  cleanup, and multiple active callbacks.
- Rewired the Kotlin/Native resource transform, resource provider, and custom
  geometry source states to use the common gate while keeping native
  descriptors, stable references, and C trampolines in `nativeMain`.
- Discovery: three native callback owners had already converged on the same
  active-callback bitfield semantics, which made the extraction a direct move of
  binding-owned lifecycle behavior rather than a new abstraction.

### Common Active Frame State Milestone

- Moved `ActiveFrameState` and its tests to common source sets because active
  session-owned frame tracking is render-handle lifecycle policy, not a
  Kotlin/Native bridge concern.
- Kept the render session and owned texture frame handles in `nativeMain`; they
  still own native frame pointers, native heap cleanup, and C release calls.
- Reflection: this is a small move, but it keeps the render-session API on the
  same trajectory as handle retention and callback gating: common code owns the
  public invalid-state behavior while actual source sets own native calls.

### Common Resource Request Handle Core Milestone

- Extracted provider-owned resource request handle state into
  `ResourceRequestHandleCore`, covering one-shot completion, live-operation
  retention, provider-decision finalization, pass-through release accounting,
  and provider-owned native release.
- Added common tests for provider-owned release, pass-through ownership,
  completion-before-decision behavior, retry after pre-native completion
  failure, duplicate completion rejection, and close-during-operation release
  deferral.
- Kept `ResourceRequestHandle` in `nativeMain` for now because it still owns the
  native request pointer, response materialization, cancellation calls, and
  Kotlin/Native cleaner registration.
- Discovery: the existing native tests already described most of this state
  machine through public behavior; extracting the core makes those rules
  reusable for FFM and Android actuals without standardizing their handle
  carriers yet.

### Common Borrowed Resource Core Milestone

- Added a common `BorrowedResourceCore` for closeable resources that permit
  temporary active borrows and defer native cleanup until the last borrow exits.
- Rewired `NativeBuffer` through the common borrowed-resource core while keeping
  native heap allocation, pointer reads, and native heap free in `nativeMain`.
- Added common tests for immediate close, close during a borrow, post-close
  borrow rejection, and cleaner-style release idempotence.
- Reflection: this prepares another small lifecycle primitive for future JVM and
  Android buffer implementations without deciding the eventual public buffer
  bridge API.

### Common Log Callback Registry Milestone

- Extracted process-global log callback install, replacement, clear, and failed
  install cleanup behavior into a common `LogCallbackRegistry`.
- Rewired the Kotlin/Native log callback state to use the common registry while
  leaving raw C callback installation, message copying, and trampoline dispatch
  in `nativeMain`.
- Added common tests for first install, replacement cleanup, failed initial
  install cleanup, and native clear accounting.
- Discovery: the native log callback owner already had a clean split between
  global replacement policy and raw upcall materialization, so the registry can
  serve future FFM and Android actuals without changing the public log callback
  API.

### Common Owned Texture Frame Core Milestone

- Extracted session-owned texture frame closed-state, retryable release policy
  integration, and leak reporting into `OwnedTextureFrameHandleCore`.
- Rewired Metal, Vulkan, and OpenGL owned texture frame handles to use the
  common core while keeping backend-specific native release calls, native heap
  frame frees, frame scope cleanup, and render-session borrow completion in
  `nativeMain`.
- Added common tests for successful close idempotence, retry after live-owner
  release failure, closed-owner local cleanup, configured closed messages, and
  leak report text.
- Reflection: this removes another set of duplicated native-only lifecycle code
  and leaves the platform source set focused on the actual frame pointer
  mechanics.

### Common Resource Provider Callback Milestone

- Moved `ResourceProviderCallback` to `commonMain`, matching the north-star rule
  that public callback interfaces belong to common code.
- Added a common `expect` facade for `ResourceRequestHandle` so the public
  callback contract no longer depends on a Kotlin/Native pointer-backed handle.
- Converted the existing Kotlin/Native `ResourceRequestHandle` into the native
  actual and added explicit JVM and Android actual placeholders until their FFM
  and JNI resource provider bridges are implemented.
- Reflection: this is the first public handle facade moved to expect/actual form
  instead of only extracting internal state, which is a step toward domain-level
  bridge facades.

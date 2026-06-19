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

### Common Native Buffer Facade Milestone

- Added a common `expect` facade for `NativeBuffer`, moving its public
  allocation, byte length, byte copy, and close contract into `commonMain`.
- Converted the existing Kotlin/Native off-heap buffer into the native actual
  while leaving native heap allocation, pointer borrowing, and C interop helpers
  in `nativeMain`.
- Added explicit JVM and Android actual placeholders until their render readback
  bridges exist.
- Reflection: this mirrors the `ResourceRequestHandle` facade pattern for a
  render-domain public type, making more public API visible from common code
  without pretending the non-native bridges are finished.

### Common Owned Texture Frame Values Milestone

- Moved `MetalOwnedTextureFrame`, `VulkanOwnedTextureFrame`, and
  `OpenGLOwnedTextureFrame` to `commonMain` because they are Kotlin-owned view
  objects over copied frame metadata and common `NativePointer` values.
- Moved `FrameScopeTest` to `commonTest`, keeping stale-frame access behavior
  covered across JVM, Android, and Native targets.
- Kept the frame handle classes in `nativeMain` because they still own native
  frame pointers and backend release calls.
- Discovery: the previous extraction of `FrameScope`, `NativePointer`, and
  `OwnedTextureFrameHandleCore` made these value objects platform-neutral
  without further bridge work.

### Common Runtime Event Facade Milestone

- Added initial common `expect` facades for `RuntimeHandle` and `MapHandle`
  covering the shared close/isClosed surface and the map-to-runtime reference.
- Moved `RuntimeEvent` to `commonMain`; it is copied event data plus nullable
  handle references and no longer needs to be Kotlin/Native-only.
- Added explicit JVM and Android handle placeholders until their FFM and JNI
  runtime/map domains are migrated.
- Reflection: this starts the domain-level handle facade migration described in
  the north star, while keeping the actual runtime and map method bodies in
  `nativeMain` until native operations are behind bridge facades.

### Common Offline Operation Facade Milestone

- Added a common `expect` facade for `OfflineOperationHandle`, covering the
  public operation id, kind, result kind, close state, and close contract.
- Converted the existing Kotlin/Native offline operation handle into the native
  actual while keeping native operation ids, runtime validation, leak reporting,
  and discard mechanics in `nativeMain`.
- Added explicit JVM and Android actual placeholders until their offline bridges
  are implemented.
- Reflection: this keeps the offline public API visible from `commonMain` and
  builds on the initial `RuntimeHandle` facade without moving native operation
  transport prematurely.

### Common Render Session Facade Milestone

- Added a common `expect` facade for `RenderSessionHandle`, covering the shared
  close/isClosed surface and the render-session-to-map reference.
- Converted the existing Kotlin/Native render session handle into the native
  actual and added explicit JVM and Android actual placeholders until their
  render bridges are implemented.
- Reflection: this extends the handle facade pattern into the render domain
  while leaving resize, render, query, texture readback, and frame acquisition
  calls in `nativeMain` until they can sit behind bridge facades.

### Common Owned Texture Frame Handle Facade Milestone

- Added common `expect` facades for the Metal, Vulkan, and OpenGL owned texture
  frame handles, covering frame access, close, and closed-state inspection.
- Converted the existing Kotlin/Native owned texture frame handles into native
  actuals and added JVM and Android actual placeholders until their render
  bridge implementations are available.
- Reflection: this completes the common surface for owned texture frame values
  and handles, leaving the platform-specific frame acquisition and release
  machinery inside platform source sets.

### Common Map Projection Facade Milestone

- Added a common `expect` facade for `MapProjectionHandle`, covering camera
  access, camera updates, visible-coordinate and visible-geometry fitting,
  pixel/coordinate conversion, close, and closed-state inspection.
- Promoted `MapHandle.createProjection()` into the common map handle facade so
  the projection handle has a common entrypoint instead of existing only as a
  native-only return type.
- Converted the existing Kotlin/Native projection handle and projection factory
  into native actuals and added JVM and Android placeholders until the map
  bridge implementations are migrated.
- Discovery: projection is the first handle facade that forced an entrypoint
  method onto an existing common owner facade, so future domain facades should
  move return types and creation methods together.

### Common Map Camera And Control Facade Milestone

- Expanded the common `MapHandle` facade with repaint requests, debug toggles,
  viewport and tile options, camera reads and transitions, camera fitting,
  bounds/free-camera/projection-mode properties, and pixel/coordinate
  conversions.
- Converted the existing Kotlin/Native map camera and control methods into
  native actuals and added JVM and Android actual placeholders until their map
  bridge implementations are migrated.
- Reflection: `MapHandle` is now the main remaining consolidation surface. The
  next useful slices are style/source mutation, style layer/image mutation,
  render-session attachment, and runtime/offline operation entrypoints.

### Common Render Session Operations Facade Milestone

- Expanded the common `RenderSessionHandle` facade with resize, render update,
  detach, memory/data/debug operations, feature state, rendered/source/extension
  queries, texture readback, and owned texture frame acquisition.
- Converted the existing Kotlin/Native render-session operation methods into
  native actuals and added JVM and Android actual placeholders until their
  render bridges are implemented.
- Reflection: render-session attachment still belongs with the platform map
  bridge for now because each attach path materializes backend-specific native
  descriptors, but an already-created render session now has a shared common
  control surface.

### Common Runtime Operations Facade Milestone

- Expanded the common `RuntimeHandle` facade with runtime pumping, offline
  operation start/take methods, resource provider and transform callbacks, event
  polling, and runtime creation.
- Converted the existing Kotlin/Native runtime operations and companion factory
  into native actuals and added JVM and Android actual placeholders until their
  runtime bridges are migrated.
- Discovery: runtime is the first facade where the common API includes both
  operation handles and companion construction. The operation result typing can
  stay common because `OfflineOperationHandle` already carries the binding-owned
  lifecycle and result-kind metadata.

### Common Map Style Source Facade Milestone

- Expanded the common `MapHandle` facade with style URL/JSON loading, generic
  style source JSON mutation, style source inspection, GeoJSON source mutation,
  custom geometry source mutation, and vector/raster/raster-dem source creation.
- Converted the existing Kotlin/Native source-domain map methods into native
  actuals and added JVM and Android actual placeholders until their map bridges
  are migrated.
- Reflection: custom geometry source ownership remains platform-local because
  the native callback descriptor is bridge-specific, but the public source
  lifecycle and tile invalidation API can already be common.

### Common Map Style Image Facade Milestone

- Expanded the common `MapHandle` facade with style image mutation and
  inspection plus image source creation, mutation, and coordinate reads.
- Converted the existing Kotlin/Native style image and image source methods into
  native actuals and added JVM and Android actual placeholders until their map
  bridges are migrated.
- Reflection: image byte ownership is already represented by common copied
  values, so this slice did not need a new handle facade. Platform source sets
  still own the native copy/readback mechanics.

### Common Map Style Layer Facade Milestone

- Expanded the common `MapHandle` facade with style layer creation, location
  indicator mutation, layer inspection/reordering, light mutation, layer
  property access, and layer filter access.
- Converted the existing Kotlin/Native style layer, light, property, and filter
  methods into native actuals and added JVM and Android actual placeholders
  until their map bridges are migrated.
- Discovery: the location indicator image method uses `imageKind` and `imageId`
  in the existing Kotlin/Native API; the common facade preserves those names to
  avoid accidental API drift during expect/actual extraction.

### Common Map Render Attachment Facade Milestone

- Expanded the common `MapHandle` facade with Metal, Vulkan, and OpenGL texture
  and surface attachment methods, plus the `MapHandle.create` factory.
- Converted the existing Kotlin/Native render attachment methods and map factory
  into native actuals and added JVM and Android actual placeholders until their
  map/render bridges are migrated.
- Reflection: the public attachment descriptors are already common copied
  values, so the common facade can expose the full render-session creation
  surface while each platform actual remains responsible for native descriptor
  materialization.

### JVM Native Buffer Actual Milestone

- Replaced the JVM `NativeBuffer` placeholder with an FFM-backed implementation
  using `Arena` and `MemorySegment`, matching the existing Java FFM buffer
  behavior.
- Added JVM tests for byte length, copied byte-array reads, capacity checks,
  idempotent close, close-during-borrow behavior, and zero-length buffers.
- Reflection: this is the first non-placeholder JVM handle actual below
  `Maplibre`. It confirms small bridge-support objects can migrate before the
  larger runtime/map/render handle transports are ready.

### Android Native Buffer Actual Milestone

- Replaced the Android `NativeBuffer` placeholder with a direct `ByteBuffer`
  implementation, matching the existing Java JNI buffer behavior.
- Kept the same copied byte-array reads, capacity checks, idempotent close, and
  close-during-borrow semantics as the JVM and Kotlin/Native buffers.
- Reflection: this gives Android JNI render readback a real platform carrier
  without requiring the larger render-session bridge to be implemented in the
  same milestone.

### Common Projection Helper Facade Milestone

- Promoted `Maplibre.projectedMetersForLatLng` and
  `Maplibre.latLngForProjectedMeters` into the common process-global facade.
- Converted the Kotlin/Native implementations into native actuals, added JVM FFM
  actuals using explicit `mln_lat_lng` and `mln_projected_meters` struct
  layouts, and added Android JNI actuals using JavaCPP-generated structs.
- Discovery: small by-value C structs can be bridged in Kotlin/JVM without
  importing the Java `jextract` classes, but each layout must stay local and
  explicit until a broader internal bridge facade owns generated layouts.
- Discovery: a JVM projection smoke test loaded the process-global native
  library before `NativeLibraryTest`, making the existing missing-path assertion
  order-dependent. JVM tests that load the native library need isolation before
  they can be added freely.

### Common Logging Facade Milestone

- Promoted process-global log callback and async log severity controls into the
  common `Maplibre` facade.
- Converted the Kotlin/Native log callback and severity implementations into
  native actuals, added JVM FFM and Android JNI actuals for async severity
  masks, and left explicit JVM/Android placeholders for log callback
  registration until callback trampolines are migrated.
- Discovery: callback registration should move as a dedicated bridge milestone
  because JVM FFM and Android JavaCPP need platform-specific native callback
  lifetime management, while async severity masks are plain status-returning C
  calls.

### JVM Runtime Lifecycle Bridge Milestone

- Replaced the JVM `RuntimeHandle` placeholder for runtime creation, `runOnce`,
  `close`, and `isClosed` with a real FFM-backed handle.
- Added explicit JVM `mln_runtime_options` materialization in `NativeAccess`,
  including C string validation and the maximum-cache-size flag, while leaving
  offline operations, event polling, maps, and runtime callbacks as dedicated
  follow-up bridge migrations.
- Added a JVM smoke test that creates a runtime, pumps it once, closes it
  idempotently, and verifies post-close lifecycle rejection.
- Discovery: JVM tests that load the process-global native library must not
  depend on running before loader error-path tests. The missing-path loader test
  now treats an already-loaded library as the valid no-op state for
  `NativeLibrary.load(path)`.

### Android Runtime Lifecycle Bridge Milestone

- Replaced the Android `RuntimeHandle` placeholder for runtime creation,
  `runOnce`, `close`, and `isClosed` with a JavaCPP-backed JNI handle.
- Shared Android ABI loading by making the Android `NativeAccess` object
  internal instead of private to `Maplibre.kt`.
- Added Android runtime option materialization from common `RuntimeOptions`,
  including C string validation and maximum-cache-size flag handling, while
  leaving offline operations, event polling, maps, and callbacks as dedicated
  follow-up bridge migrations.
- Discovery: the Kotlin module currently has no Android test source set, so
  Android runtime lifecycle is compile-validated through
  `//bindings/kotlin:build` until instrumentation packaging and emulator tests
  are introduced.

### JVM and Android Offline Operation Handle Milestone

- Replaced JVM and Android `OfflineOperationHandle` placeholders with real
  lifecycle wrappers that expose id, kind, result kind, close state, runtime
  ownership checks, child retention, and discard-on-close behavior.
- Implemented `RuntimeHandle.startAmbientCacheOperation` on JVM through FFM and
  on Android through JavaCPP so both platform runtime handles can create and
  discard a native offline operation without waiting for full offline result
  marshalling.
- Added JVM coverage for ambient cache operation child retention: a runtime with
  a live operation rejects close until the operation is discarded.
- Discovery: this is the first migrated operation handle where common lifecycle
  state is shared but the actual discard transport remains bridge-specific,
  reinforcing the north-star split between common handle semantics and platform
  call mechanics.

### JVM and Android Offline Operation Start Milestone

- Implemented JVM FFM and Android JavaCPP actuals for start-only offline
  operations that do not require struct materialization or result marshalling:
  region get, region list, status get, observe toggle, download-state mutation,
  invalidation, and deletion.
- Preserved the existing take-result placeholders so copied offline region
  snapshots, lists, and status structs can migrate in a focused follow-up.
- Discovery: JavaCPP's generated overloads for these calls expose `long[]`
  output parameters cleanly, while the JVM FFM bridge benefits from small
  grouped helpers for runtime plus out-operation-id function shapes.

### JVM and Android Offline Metadata Start Milestone

- Implemented JVM FFM and Android JavaCPP actuals for starting offline database
  merge operations and offline region metadata updates.
- Added JVM bridge support for temporary UTF-8 C strings and byte-array
  materialization, including embedded-NUL rejection and null metadata pointers
  for zero-length payloads.
- Discovery: merge/update are a useful boundary before offline result
  marshalling because they exercise borrowed path and byte storage without
  needing copied offline region structs yet.

### JVM and Android Runtime Event Poll Milestone

- Implemented `RuntimeHandle.pollEvent` on JVM and Android for runtime events
  copied from native event storage into common Kotlin values.
- Decoded `None` and `OfflineOperationCompleted` payloads and preserved all
  other payloads as `RuntimeEventPayload.Unknown` with copied payload bytes.
- Added JVM coverage for the empty event queue path on a fresh runtime.
- Discovery: event polling can migrate incrementally before map handles because
  runtime-originated events already copy cleanly. Map-originated event source
  lookup and richer typed payload decoding should remain a separate map/event
  bridge milestone.

### JVM and Android Offline Status Result Milestone

- Implemented `RuntimeHandle.takeOfflineRegionStatusResult` on JVM and Android,
  including fixed-size `mln_offline_region_status` materialization and copied
  common `OfflineRegionStatus` values.
- Added expected operation-kind/result-kind checks to JVM and Android
  `OfflineOperationHandle` before consuming typed results.
- Discovery: status result marshalling is a useful first result-taking slice
  because it exercises typed operation consumption without introducing snapshot
  handle or list ownership yet.

### JVM and Android Runtime Event Payload Milestone

- Expanded JVM and Android `RuntimeHandle.pollEvent` payload materialization
  from offline-completed-only decoding to every current C runtime event payload
  variant.
- Moved JVM FFM payload materialization into `NativeAccess` so borrowed string
  fields inside payload structs are copied while their native pointers are still
  readable.
- Kept Android payload decoding on JavaCPP struct wrappers and runtime `sizeof`
  checks instead of duplicating struct offsets in Kotlin.
- Discovery: payload decoding can reach parity before map handle migration, but
  map-originated events still cannot attach a live `MapHandle` source until the
  JVM and Android map handle registries exist.

### JVM and Android Offline Snapshot/List Result Milestone

- Implemented JVM and Android `startCreateOfflineRegion` for tile-pyramid
  definitions and implemented the create/get/list/merge/update metadata
  take-result methods.
- Added JVM FFM snapshot and list readers that copy `OfflineRegionInfo` and
  destroy returned `mln_offline_region_snapshot`/`mln_offline_region_list`
  owners on every path.
- Added Android JavaCPP snapshot and list readers with matching owner cleanup
  and copied tile-pyramid definition materialization.
- Discovery: offline result ownership is the first runtime area where the JVM
  bridge needs reusable value-struct materialization. Tile-pyramid definitions
  can migrate directly, while geometry offline definitions remain an explicit
  follow-up because the JVM/Android geometry bridge is not consolidated yet.

### JVM and Android Resource Transform Milestone

- Implemented `RuntimeHandle.setResourceTransform` and `clearResourceTransform`
  for JVM and Android.
- Added JVM FFM and Android JavaCPP `ResourceTransformState` owners that keep
  callback descriptors live while installed, copy callback inputs into common
  `ResourceTransformRequest` values, and use the C response helper for
  replacement URLs.
- Preserved the common `CallbackGate` close-after-in-flight-callback behavior
  for both bridges and kept callback exceptions contained as native status
  values.
- Discovery: resource transform is the smallest runtime callback slice because
  it has no request handle lifetime. Resource provider should remain separate so
  `ResourceRequestHandle` completion/cancellation semantics can migrate as one
  focused milestone.

### JVM and Android Log Callback Milestone

- Implemented process-global `Maplibre.setLogCallback` and `clearLogCallback` on
  JVM and Android.
- Added JVM FFM and Android JavaCPP callback registrations that copy log records
  into common `LogRecord` values and contain callback exceptions.
- Added JVM coverage for direct callback state invocation.
- Discovery: log callbacks can migrate independently from runtime/resource
  provider callbacks because they have no per-runtime or request-handle
  lifetime, but they still need process-global replacement discipline.

### JVM and Android Resource Provider Milestone

- Implemented `RuntimeHandle.setResourceProvider` for JVM and Android.
- Added JVM FFM and Android JavaCPP resource provider callback owners that copy
  borrowed native requests into common `ResourceRequest` values and contain
  callback exceptions.
- Replaced JVM and Android `ResourceRequestHandle` placeholders with real
  completion, cancellation, provider-decision, and release behavior backed by
  the common handle core.
- Added JVM coverage for direct provider callback invocation and request
  copying.
- Discovery: the resource provider bridge completes the callback family that can
  migrate before map handles. It also exposes a cleanup target: response
  materialization is still platform-local and should eventually collapse behind
  a smaller expected resource bridge.

### JVM and Android Map Lifecycle Milestone

- Implemented JVM FFM and Android JavaCPP `MapHandle.create`, `close`,
  `isClosed`, `runtime`, `setStyleUrl`, and `setStyleJson`.
- Added platform map option materialization and runtime child retention so a
  live map prevents its runtime from closing.
- Added JVM coverage for map creation, style command acceptance, close
  idempotency, and runtime retention.
- Discovery: map creation is the first bridge slice where runtime events need a
  live map registry to attach `mapSource`. This milestone keeps event source
  lookup unchanged and leaves map registration for the broader map/event bridge.

### JVM and Android Style Source JSON Milestone

- Implemented JVM FFM and Android JavaCPP `MapHandle` style source JSON mutation
  and source inspection: add, remove, exists, type, info, and ID list.
- Added JVM FFM materialization for common `JsonValue` input descriptors, source
  info copying, attribution copying, and owned style ID list destruction.
- Added Android JavaCPP descriptor scopes for common `JsonValue` trees plus
  copied source info and owned style ID list cleanup.
- Added JVM coverage for adding a GeoJSON source from common JSON, inspecting
  its type/info/list entry, and removing it.
- Discovery: FFM `Arena.allocate(byteSize, byteAlignment)` is easy to mistake
  for array allocation. Descriptor arrays in the hand-written JVM bridge must
  allocate `elementSize * count` bytes explicitly.

### JVM and Android Style Layer JSON Milestone

- Implemented JVM FFM and Android JavaCPP `MapHandle` style layer JSON mutation
  and basic inspection: add, remove, exists, type, and ID list.
- Reused the common `JsonValue` descriptor materialization added for style
  sources, which kept this milestone focused on C call shape and owned layer ID
  list cleanup.
- Added JVM coverage for adding a background layer from common JSON, inspecting
  its type/list entry, and removing it.
- Discovery: source and layer JSON bridges share enough mechanics that a future
  internal map/style bridge facade should expose domain operations while
  centralizing string-view, JSON descriptor, and style ID list ownership.

### JVM and Android Style Layer Command Milestone

- Implemented JVM FFM and Android JavaCPP commands for typed layer insertion,
  location indicator layer updates, and style layer movement.
- Added FFM call-shape helpers for two- and three-string-view calls, string-view
  plus coordinate/double calls, and string-view plus enum/string-view calls.
- Added Android JavaCPP `LatLng` materialization for map style commands.
- Extended JVM coverage to add and update a location indicator layer, move it
  relative to a JSON layer, and remove both layers.
- Discovery: simple map style commands are a good bridge migration slice because
  they reuse string-view ownership and status handling without requiring JSON
  snapshot/result decoding.

### JVM and Android Style JSON Snapshot Milestone

- Implemented JVM FFM and Android JavaCPP style JSON snapshot operations for
  layer JSON, style light JSON/properties, layer properties, and layer filters.
- Added JVM FFM snapshot decoding that borrows the root `mln_json_value`, copies
  it into common `JsonValue`, and destroys the owned `mln_json_snapshot` on all
  paths.
- Added Android JavaCPP snapshot decoding beside the existing Android JSON
  descriptor writer, using raw `PointerPointer<Pointer>` outputs where the
  generated overloads require them.
- Extended JVM coverage to round-trip layer JSON, layer property, and style
  light property values through the common `JsonValue` API.
- Discovery: JavaCPP generates different overload shapes for similar owned
  pointer outputs. JSON snapshot getters prefer raw pointer-pointer outputs,
  while some style ID list calls accept typed pointer-pointers.

### JVM and Android Style Image Milestone

- Implemented JVM FFM and Android JavaCPP runtime style image operations: set,
  remove, exists, metadata lookup, and premultiplied RGBA8 copy.
- Added JVM FFM struct materialization for `mln_premultiplied_rgba8_image`,
  `mln_style_image_options`, and `mln_style_image_info`, including option field
  masks and two-phase pixel copying.
- Added Android JavaCPP scopes for copied pixel buffers and style image option
  descriptors.
- Extended JVM coverage to set a style image, inspect copied metadata, copy the
  image back into common Kotlin values, and remove it.
- Discovery: the common `PremultipliedRgba8Image` defensively copies its byte
  array, so bridge calls should read `pixels` once and pass that stable snapshot
  through the native call.

### JVM and Android Map Debug Controls Milestone

- Implemented JVM FFM and Android JavaCPP map debug/status controls: repaint
  requests, still-image requests, debug option masks, rendering stats view
  toggles, loaded-state reads, and debug-log dumps.
- Added shared-style mask conversion in each bridge actual so common Kotlin
  exposes `Set<DebugOption>` instead of raw bitfields.
- Extended JVM coverage to round-trip debug option sets, toggle the rendering
  stats view, and call repaint/load/debug-log operations through the Kotlin map
  handle.
- Discovery: this slice is a useful boundary between style state and broader
  camera/render work because it removes map control placeholders without adding
  descriptor structs or render-session ownership.

### JVM and Android Tile Source Milestone

- Implemented JVM FFM and Android JavaCPP vector, raster, and raster DEM source
  insertion for TileJSON URLs and inline tile URL arrays.
- Added platform materialization for common `TileSourceOptions`, including field
  masks, attribution string views, bounds, tile size, tile scheme, vector
  encoding, and raster DEM encoding.
- Added JVM coverage that inserts vector, raster, and raster DEM sources and
  inspects their copied source metadata through the common Kotlin map API.
- Discovery: tile sources are a clean bridge slice before GeoJSON/custom sources
  because they exercise option structs and string-view arrays without requiring
  the geometry and feature descriptor graph.

### JVM and Android Image Source Milestone

- Implemented JVM FFM and Android JavaCPP image source operations for URL-backed
  and inline-image sources, image updates, coordinate updates, and coordinate
  lookup.
- Added platform coordinate-array materialization for the fixed four-corner
  image source contract and reused the existing premultiplied RGBA8 image
  materialization.
- Added JVM coverage that adds URL and inline image sources, updates URL, image,
  and coordinates, reads coordinates back, and checks missing-source lookup.
- Discovery: image sources are the first migrated source family that benefits
  from small array-scope helpers on both bridges. Keeping those helpers
  primitive-specific avoids pulling in the broader geometry/GeoJSON descriptor
  graph too early.

### JVM and Android Map Option Properties Milestone

- Implemented JVM FFM and Android JavaCPP `viewportOptions`, `tileOptions`, and
  `projectionMode` map properties.
- Added platform materialization for viewport, tile, projection, and edge-inset
  descriptors, including field masks and unknown-input rejection for wrapped
  enum domains.
- Added JVM coverage that round-trips viewport orientation/constrain/frustum
  options, tile LOD options, and projection mode options through the Kotlin map
  handle.
- Discovery: exact FFM offsets matter for these descriptor structs because
  `mln_map_viewport_options` and `mln_map_tile_options` both contain padding
  before double-aligned fields. A tiny `offsetof` probe is a better source of
  truth than hand-counting from the header.

### JVM and Android Coordinate Projection Milestone

- Implemented JVM FFM and Android JavaCPP map coordinate conversion APIs:
  `pixelForLatLng`, `latLngForPixel`, `pixelsForLatLngs`, and
  `latLngsForPixels`.
- Added owned JVM and Android `MapProjectionHandle` actuals for standalone
  projection creation, coordinate conversion, close, and closed-state handling.
- Kept projection camera access and visible-coordinate fitting unsupported in
  this slice because they depend on the broader camera and geometry descriptor
  migration.
- Added JVM coverage for map coordinate round-trips, empty batch conversion
  calls, projection round-trips, idempotent projection close, and projection use
  after the source map closes.
- Discovery: JavaCPP top-level private helper names still collide at generated
  bytecode level within the same package. Android bridge helpers that need
  per-file pointer wrappers should use distinct class names, even when they are
  source-private.
- Discovery: the existing `mln_map_projection` C API snapshots transform state,
  so a projection handle can remain usable after the source map closes without
  retaining the source map.

### JVM and Android Camera Control Milestone

- Implemented JVM FFM and Android JavaCPP camera snapshot and command APIs:
  `camera`, `jumpTo`, `easeTo`, `flyTo`, pan/zoom/rotate/pitch commands, and
  transition cancellation.
- Added platform materialization for `CameraOptions`, `AnimationOptions`, and
  `CameraFitOptions`, including field masks, nested edge-inset/screen-point
  values, and cubic easing descriptors.
- Implemented camera fitting for bounds and coordinate lists, plus wrapped and
  unwrapped bounds-for-camera helpers.
- Completed standalone projection camera support for camera read/write and
  visible-coordinate fitting on JVM and Android.
- Kept geometry fitting unsupported in this slice because it depends on the
  broader `Geometry`/GeoJSON descriptor graph migration.
- Added JVM coverage for camera read/write, animated and non-animated camera
  commands, camera fitting, bounds calculation, and projection camera fitting.
- Discovery: camera-family FFM structs have enough nested layout and padding
  that offset probing is worthwhile. `mln_camera_options` is 120 bytes,
  `mln_animation_options` is 64 bytes, and `mln_camera_fit_options` is 56 bytes
  on the current ABI.

### JVM and Android Bounds and Free Camera Milestone

- Implemented JVM FFM and Android JavaCPP `bounds` and `freeCameraOptions` map
  properties.
- Added platform materialization for `BoundOptions`, `FreeCameraOptions`,
  `Vec3`, and `Quaternion`, including field masks and nested value structs.
- Added JVM coverage that round-trips bound constraints and free-camera options
  through the common Kotlin map handle.
- Discovery: bounds and free-camera updates mutate camera transform state, so
  tests that assert projection camera snapshots need to run before applying
  those stateful constraints.
- Discovery: JVM FFM helpers that read and write the same descriptor can collide
  by signature inside `NativeAccess`; naming the reader `readFreeCameraOptions`
  keeps the public bridge method name available.

### JVM and Android Geometry and GeoJSON Source Milestone

- Implemented JVM FFM and Android JavaCPP descriptor materialization for
  `Geometry`, `Feature`, `FeatureIdentifier`, and `GeoJson` inputs.
- Implemented GeoJSON source URL/data add and update APIs on JVM and Android.
- Implemented `cameraForGeometry` on JVM and Android using the same descriptor
  graph as GeoJSON source data.
- Added JVM coverage for URL-backed GeoJSON sources, inline GeoJSON feature
  collections, source data updates, and geometry camera fitting.
- Discovery: the geometry descriptor graph can be write-only for this migration
  slice because current Kotlin map APIs only pass geometry and GeoJSON into the
  C API. Query result and offline-region snapshots can add readback later when
  they migrate.
- Discovery: the C layout probe confirmed 16-byte view structs, 24-byte
  `mln_geometry`/`mln_geojson`, and a 56-byte `mln_feature`; using those offsets
  kept JVM FFM materialization aligned with the generated Java FFM bridge.

### JVM and Android Projection Geometry Milestone

- Implemented `MapProjectionHandle.setVisibleGeometry` on JVM FFM and Android
  JavaCPP.
- Reused the geometry descriptor graph from the GeoJSON source milestone for
  projection fitting instead of adding a second geometry materializer.
- Added JVM coverage that fits a standalone projection to a geometry and reads
  the computed camera back through the common Kotlin projection API.
- Discovery: making the Android geometry scope package-internal is enough to
  share descriptor ownership across map and projection actuals while keeping
  JavaCPP pointer details out of common code.

### JVM and Android Custom Geometry Source Milestone

- Implemented JVM FFM and Android JavaCPP custom geometry source add, tile data,
  tile invalidation, and region invalidation APIs.
- Added JVM and Android callback state owners for custom geometry source
  fetch/cancel callbacks, including callback-gate protection and native
  descriptor cleanup.
- Registered JVM and Android maps with their runtime so map runtime events can
  resolve the Kotlin `MapHandle` and sweep custom source states after
  style-loaded events.
- Added JVM coverage for adding/removing custom geometry sources and
  updating/invalidating custom tile data.
- Discovery: Kotlin/JVM FFM custom-source callbacks can keep `user_data` null
  when upcall stubs are bound to the state object; Android JavaCPP callbacks
  instead carry state through callback subclass captures.
- Discovery: custom source lifecycle needs map/runtime coordination, not just
  direct map methods, because style URL loads drop previous style sources
  asynchronously.

### JVM Render Session Core Milestone

- Implemented JVM FFM render target attachment for Metal, Vulkan, and OpenGL
  texture and surface descriptors.
- Replaced the JVM render session placeholder with a real handle for lifecycle,
  resize, render update, detach, renderer maintenance commands, feature state,
  and premultiplied RGBA8 readback metadata/data.
- Kept query result APIs and owned texture frame acquisition unsupported in this
  slice because they need additional result-graph and active-frame ownership.
- Added JVM coverage for unsupported-backend attach diagnostics through the new
  descriptor bridge.
- Discovery: render target descriptors have nested inline structs with enough
  padding that probing offsets is less error-prone than hand-counting from
  headers. The current ABI uses 24-byte extents, 48-byte OpenGL contexts,
  72-byte feature-state selectors, and 24-byte texture image info.
- Discovery: the JVM render session should retain its parent map through common
  `HandleStateCore` child retention, matching the Kotlin Native ownership rule
  without introducing a JVM-only lifecycle path.

### Android Render Session Core Milestone

- Implemented Android JavaCPP render target attachment for Metal, Vulkan, and
  OpenGL texture and surface descriptors.
- Replaced the Android render session placeholder with a JavaCPP-backed handle
  for lifecycle, resize, render update, detach, renderer maintenance commands,
  and premultiplied RGBA8 readback metadata/data.
- Kept Android feature state, query result APIs, and owned texture frame
  acquisition unsupported in this slice so the descriptor and lifecycle bridge
  can land separately from the larger query/value materialization work.
- Discovery: Android can reuse JavaCPP default descriptor constructors for
  render targets, which avoids duplicating the manual FFM offset table needed by
  the JVM bridge.
- Discovery: Kotlin actual classes cannot add an `actual companion object`
  unless the expect declaration has one; Android-only construction helpers need
  a plain companion or top-level internal functions.

### Android Render Feature State Milestone

- Implemented Android JavaCPP render-session feature state set, get, and remove
  APIs.
- Added Android render-local materializers for `FeatureStateSelector`,
  `JsonValue` inputs, and JSON snapshot readback so Android render sessions
  match the JVM feature-state surface.
- Kept Android feature query result APIs unsupported in this slice because they
  need additional query geometry/options materializers and native result
  ownership.
- Discovery: Android JSON descriptor helpers are still duplicated between map
  and render code. They should be consolidated after query migration clarifies
  the final shared descriptor surface.

### JVM Owned Texture Frame Milestone

- Implemented JVM FFM owned texture frame acquisition and release for Metal,
  Vulkan, and OpenGL render sessions.
- Replaced the JVM owned texture frame handle placeholders with real handle
  implementations that expose scoped frame values and release native frames on
  close.
- Added active-frame guards to JVM render-session commands so resize, render,
  detach, readback, feature state, and destroy reject while a frame is borrowed.
- Discovery: JVM FFM `MemorySegment.scope()` is not the object to close in this
  JDK API shape. The frame owner needs to retain the allocating `Arena`
  explicitly and close that arena when the frame handle closes.
- Discovery: current frame ABI sizes are 64 bytes for Metal, 72 bytes for
  Vulkan, and 64 bytes for OpenGL; all three share generation/extent/frame-id
  offsets before backend-specific handles.

### Android Owned Texture Frame Milestone

- Implemented Android JavaCPP owned texture frame acquisition and release for
  Metal, Vulkan, and OpenGL render sessions.
- Replaced the Android owned texture frame handle placeholders with real handle
  implementations that expose scoped frame values and release native frames on
  close.
- Added active-frame guards to Android render-session commands so resize,
  render, detach, readback, feature state, and destroy reject while a frame is
  borrowed.
- Discovery: Android JavaCPP frame handling can mirror the old JNI binding
  closely because generated frame structs own their local native allocations and
  close directly after native release.

### JVM Render Query Milestone

- Implemented JVM FFM rendered-feature, source-feature, and feature-extension
  query APIs on `RenderSessionHandle`.
- Added JVM query geometry and option materializers for screen points, boxes,
  line strings, layer filters, source-layer filters, and JSON filters.
- Added JVM query result copy-out for queried features, feature state JSON,
  feature extension values, and feature extension feature collections.
- Removed the JVM render-session unsupported placeholder now that query APIs are
  bridged.
- Discovery: queried feature results require read-side `Feature` and `Geometry`
  materialization on JVM. The previous JVM bridge only needed write-side
  descriptors for GeoJSON and style inputs.
- Discovery: the current query ABI uses a 40-byte rendered query geometry,
  32-byte rendered/source query options, 104-byte queried feature, 24-byte
  feature extension result info, and 16-byte feature collection.
- Discovery: an initial `mise run //bindings/kotlin:build` attempt stopped in
  the root dependency prerequisite before printing CMake output, but a rerun
  completed the native configure/build and full Kotlin build successfully.

### Android Render Query Milestone

- Implemented Android JavaCPP rendered-feature, source-feature, and
  feature-extension query APIs on `RenderSessionHandle`.
- Added Android query geometry and option scopes for rendered point, box, and
  line-string queries plus layer/source-layer and JSON filters.
- Added Android query result copy-out for queried features, optional source
  metadata, feature state JSON, feature extension values, and feature extension
  feature collections.
- Added render-local Android feature and geometry descriptor scopes for
  feature-extension inputs.
- Removed the Android render-session unsupported placeholder now that render
  query APIs are bridged.
- Removed stale private JVM and Android map unsupported helpers that no longer
  had callers after earlier map bridge milestones.
- Discovery: JavaCPP generated direct wrappers for the query result handles,
  `SizeTPointer`, and the query geometry constructor functions, so Android can
  avoid JVM-style manual offset tables for this slice.
- Discovery: Android render query support duplicates feature/geometry descriptor
  logic that already exists in the Android map binding. That is now the clearest
  candidate for the next Android/common consolidation cleanup.

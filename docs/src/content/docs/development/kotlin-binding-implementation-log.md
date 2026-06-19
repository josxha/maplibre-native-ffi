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

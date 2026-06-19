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

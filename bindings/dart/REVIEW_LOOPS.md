# Dart binding review loop outcomes

This file records the iterative review loop run on the `dart` branch for the
Dart binding work. Each round lists applied findings, rejected or non-actionable
findings, and findings that required user input.

## Round 1

Review inputs:

- Base commit: `96d8cbd31c4ac54ec038ef2f2c57758e44f69955`
- Review artifacts: three fresh reviewer passes covering
  lifecycle/callbacks/resources/custom geometry, public API/tests/docs, and
  native shim/build/FFI exposure.

### Applied findings

1. `MapHandle.setStyleUrl` retired custom-geometry callback state too early for
   URL style replacement.
   - Fix: keep custom-geometry callback states after URL style load requests;
     inline `setStyleJson`, explicit source removal, and map close still retire
     them.
   - Validation: Dart analyzer/tests.
2. Long-lived null-terminated resource/provider strings bypassed embedded-NUL
   validation.
   - Fix: validate rewrite/provider URLs before native-owned storage allocation
     and allocate with the same NUL-rejecting C-string helper used elsewhere.
   - Validation: Dart tests for rewrite rule URL/replacement URL, provider rule
     URL, and queued provider route URL embedded-NUL rejection.
3. `ResourceRequestHandle` did not enforce owner-isolate use.
   - Fix: store creating isolate identity and reject use from another isolate
     before native calls.
   - Validation: covered by owner-isolate lifecycle tests and existing provider
     handle tests.
4. Negative public `int` values could wrap into unsigned C fields before
   validation.
   - Fix: validate map dimensions, render target dimensions, and canonical tile
     IDs before assigning to native `uint32_t` fields.
   - Validation: Dart tests for negative map width, render target width, and
     tile ID component rejection.
5. Unsigned 64-bit JSON/feature values had an implicit high-bit mismatch with
   Dart FFI `@Uint64` access.
   - Fix: explicitly define and enforce a supported unsigned 63-bit subset for
     `JsonUInt` and `UIntFeatureIdentifier`, including native-copy rejection for
     high-bit values.
   - Validation: Dart tests for negative, high-bit, and synthetic native
     high-bit unsigned values.
6. `dart_shim.cpp` used `MLN_API` definitions without `MLN_BUILDING_C`, risking
   Windows `dllimport` definitions.
   - Fix: define `MLN_BUILDING_C` at the top of `src/c_api/dart_shim.cpp`.
   - Validation: native build through Dart binding build.
7. Dart binding CI was absent from the generated binding matrix.
   - Fix: add `[bindings.dart]` to `.github/config/variants.toml` using
     `//bindings/dart:ci` for Linux/macOS variants.
   - Validation: `mise run -q ci:matrix bindings --pretty` shows Dart entries.

### Rejected / non-actionable findings

- Missing `plan.md` / `progress.md` was not actionable; those files are not
  required branch artifacts.
- Large generated `maplibre_native_c.g.dart` is expected and private to the
  internal FFI layer.
- Flutter/platform UI, package publishing, standalone examples, and generated
  API reference publishing remain explicitly out of scope.
- Direct `NativeCallable.listener` use for custom-geometry tile notifications
  remains acceptable for void notifications with copied tile identifiers.
- Resource transform state replacement remains acceptable because the C API
  unregistration contract prevents old transform invocations after the native
  setter/clear call returns.

### Findings requiring user input

None.

## Round 2

Review inputs:

- Base commit: `96d8cbd31c4ac54ec038ef2f2c57758e44f69955`
- Head reviewed: `54544273ec63c9df626b4bdde8a842f154d59d60`
- Review artifacts: two fresh reviewer passes covering round-1 fix correctness
  and tests/docs/API consistency.

### Applied findings

1. URL style reloads retained stale custom-geometry callback states
   indefinitely.
   - Fix: `RuntimeHandle` now registers live maps and processes copied runtime
     events. When a map-originated `MLN_RUNTIME_EVENT_MAP_STYLE_LOADED` event
     arrives after a URL style load, the map clears old custom-geometry callback
     states. `setStyleUrl` still avoids early cleanup while URL replacement is
     pending.
   - Validation: `mise run //bindings/dart:ci`.
2. Log callback exceptions were not contained at the Dart listener boundary.
   - Fix: `_LogCallbackState` catches and contains user `LogCallback` exceptions
     while still destroying copied native log records.
   - Validation: `maplibre_native_ffi_test.dart` installs a throwing log
     callback, triggers native debug logging, verifies the callback ran without
     uncaught test failure, then clears it; `mise run //bindings/dart:ci`
     passed.

### Rejected / non-actionable findings

- Embedded-NUL resource string validation from round 1 was verified by reviewers
  and kept as-is.
- `ResourceRequestHandle` owner-isolate guard from round 1 was verified by
  reviewers and kept as-is.
- Unsigned/range validation from round 1 was verified by reviewers and kept
  as-is.
- Dart CI matrix entry from round 1 was verified by reviewers and kept as-is.

### Findings requiring user input

None.

## Round 3

Review inputs:

- Base commit: `96d8cbd31c4ac54ec038ef2f2c57758e44f69955`
- Head reviewed: post-round-2 branch (`93339df` plus lifecycle helper commit
  `3633876`)
- Review artifacts: two fresh reviewer passes covering final correctness and
  final tests/docs/API consistency.

### Applied findings

1. Tile-source and custom-geometry integer options could wrap before native
   validators.
   - Fix: validate custom-geometry `tileSize` in `1..65535`, custom-geometry
     `buffer` in `0..65535`, and tile-source `tileSize` in `1..65535` before
     assigning to native `uint32_t` fields.
   - Validation: Dart tests for oversized tile-source tile size, custom-geometry
     tile size, and custom-geometry buffer values; `mise run //bindings/dart:ci`
     passed.
2. Public `ResourceKind.unknown` was indistinguishable from wildcard/null in
   Dart shim rules.
   - Fix: reserve private wildcard sentinel `0xffffffff` for null filters, pass
     that sentinel from Dart materializers, and make the C++ shim wildcard
     matching compare against the sentinel rather than
     `MLN_RESOURCE_KIND_UNKNOWN` / `0`.
   - Validation: existing provider/rewrite tests still pass;
     `mise run //bindings/dart:ci` passed.

### Rejected / non-actionable findings

- Runtime event source routing and URL-style custom-geometry cleanup were
  verified by reviewers and kept as-is.
- Log callback exception containment was verified by reviewers and kept as-is.
- Resource string NUL validation, `ResourceRequestHandle` owner-isolate guards,
  unsigned JSON/feature range validation, and Dart CI matrix entries were
  verified by reviewers and kept as-is.
- Windows Dart binding CI remains excluded in the matrix because the current
  binding entry intentionally targets Linux/macOS variants where Dart support
  and native artifacts are available.

### Findings requiring user input

None.

## Round 4

Review inputs:

- Base commit: `96d8cbd31c4ac54ec038ef2f2c57758e44f69955`
- Head reviewed: `a5afd3b263139107cca31f2e7abcdc8b66b20eb1`
- Review artifacts: two fresh reviewer passes covering final
  runtime/resource/custom-geometry/log/lifecycle/CI correctness and
  tests/docs/API consistency.

### Applied findings

1. Retired callback states could drop already-delivered native payloads without
   cleanup.
   - Fix: `RetainedCallbackState.runUpcall` now returns whether the callback
     body ran. Log callbacks destroy copied log records on the false/drop path.
     Queued resource provider callbacks complete/release a retired copied
     request with an error response and always destroy the copied request
     record.
   - Validation: callback-state test now asserts false return after close;
     `mise run //bindings/dart:ci` passed.
2. Provider-rule response strings were not prevalidated before long-lived native
   allocation.
   - Fix: `_ResourceProviderRulesState` prevalidates
     `ResourceResponse.errorMessage` and `etag` for embedded NUL before
     allocating retained native rule storage.
   - Validation: Dart tests cover embedded-NUL provider-rule `errorMessage` and
     `etag`; `mise run //bindings/dart:ci` passed.

### Rejected / non-actionable findings

- Resource-kind wildcard handling, queued provider handoff, custom-geometry
  lifecycle, log exception containment, lifecycle cleanup, and Dart CI matrix
  entries were otherwise verified by reviewers and kept as-is.
- Missing `plan.md`/`progress.md` remains non-actionable; the review used branch
  diff, commit history, docs, and tests directly.
- Pub outdated notices and the expected provider-exception native error log
  during tests do not require fixes now.

### Findings requiring user input

None.

## Round 5

Review inputs:

- Base commit: `96d8cbd31c4ac54ec038ef2f2c57758e44f69955`
- Head reviewed: `b2c439ac042d743bea02fc85df7f6cc6bdec5ea8`
- Review artifact: one fresh reviewer pass covering the full branch diff and
  current working tree.

### Applied findings

1. `ResourceRequestHandle.complete()` released the native request before
   Dart-side response string validation succeeded.
   - Fix: validate `ResourceResponse.errorMessage` and `etag` for embedded NUL
     at the start of `complete()` before `_takePointer()` releases the one-shot
     handle.
   - Validation: queued provider callback test now attempts an invalid
     embedded-NUL response, verifies `InvalidArgumentException`, verifies the
     handle remains unreleased, then successfully completes the same handle;
     `mise run //bindings/dart:ci` passed.
2. Runtime map registry strongly retained `MapHandle` objects, defeating leak
   reporting/recovery for dropped maps.
   - Fix: runtime registry now stores `WeakReference<MapHandle>` entries,
     removes entries whose targets have been collected, and still unregisters
     explicit map closes by address.
   - Validation: `mise run //bindings/dart:ci` passed; this change is
     intentionally low-level and avoids nondeterministic GC tests.

### Rejected / non-actionable findings

- Provider-rule string prevalidation from round 4 was verified and kept as-is.
- Callback-drop cleanup from round 4 was verified and kept as-is.
- Missing `plan.md` / `progress.md` remains non-actionable; the review used
  branch diff, commit history, docs, and tests directly.

### Findings requiring user input

None.

## Round 6

Review inputs:

- Base commit: `96d8cbd31c4ac54ec038ef2f2c57758e44f69955`
- Head reviewed: `de6370a01ff3f2205153ccb480975221271da3fa`
- Review artifacts: two fresh reviewer passes covering final
  runtime/resource/custom-geometry/log/lifecycle/CI correctness and
  tests/docs/API consistency.

### Applied findings

None. Both reviewers reported no remaining actionable findings.

### Rejected / non-actionable findings

- Missing `plan.md` / `progress.md` remained non-actionable; reviewers used the
  branch diff, source, docs, commit history, and tests directly.
- Dart package resolution may create ignored `.dart_tool/` and `pubspec.lock`
  artifacts during validation; these are not source changes, and `pubspec.lock`
  remains intentionally untracked/removed for this binding-development package.
- Dart-only exported shim symbols were reviewed as a possible API-boundary risk
  and rejected as non-actionable: they are binding-private shim entry points,
  not public curated Dart APIs, and the Dart docs/spec explicitly describe the
  native shim handoff model.
- Windows Dart binding CI remains excluded while Dart CI is selected for
  supported Linux/macOS binding variants.

### Findings requiring user input

None.

### Validation cited by reviewers

- `mise run //bindings/dart:ci` passed: Dart analyze reported no issues and Dart
  tests passed (`29` tests).
- `mise run test` passed: native/Zig suites succeeded (`129/131` passed, `2`
  skipped; C API suite `40/43` passed, `3` skipped).
- `git diff --check 96d8cbd31c4ac54ec038ef2f2c57758e44f69955...HEAD` passed.
- CI matrix checks confirmed Dart binding jobs for supported Linux/macOS
  variants and exclusion on Windows.

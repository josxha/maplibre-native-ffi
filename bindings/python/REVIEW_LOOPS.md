# Python binding review loops

## Round 1

Review fanout: API correctness/regressions; tests and validation;
lifecycle/callback/threading/native ownership; SPEC/docs and maintainability.

### Applied findings

- Preserve unknown feature-extension query result types in the Python public
  value layer.
  - Evidence: reviewers noted the native bridge can forward unknown
    feature-extension result types and the Python value layer must preserve
    growable output domains.
  - Resolution: verified `FeatureExtensionResult.from_native()` branches on the
    raw result type before constructing known value shapes, and added/kept
    focused test coverage for unknown raw types.
- Reject unknown `OfflineRegionDownloadState` values before setter calls cross
  into native code.
  - Evidence: Python preserved unknown download-state output values but
    `RuntimeHandle.set_offline_region_download_state()` forwarded them to
    `_native`.
  - Resolution: added `OfflineRegionDownloadState.native_code_for_set()` and
    routed the setter through it, with tests using a fake native runtime to
    prove unknown states are rejected before native calls.
- Enforce exactly-once public `ResourceRequestHandle.complete()` behavior.
  - Evidence: the public wrapper had no closed-state guard around `complete()`
    even though request completion is one-shot.
  - Resolution: `complete()` now raises `InvalidStateError` after completion or
    close, and focused tests assert duplicate completion does not call native
    twice.
- Retain custom geometry source callback memory safely after `set_style_url()`.
  - Evidence: URL style replacement completes asynchronously, so dropping custom
    geometry callback state immediately after the URL request is accepted can
    leave native with stale `user_data` while the previous style is still live.
  - Resolution: `set_style_url()` now retires active custom geometry states by
    closing their public queues while retaining callback boxes until map
    teardown; tests assert the public handle is closed and queues no longer
    accept events.
- Release the GIL around resource-transform teardown/replacement and runtime
  destruction paths that can wait for in-flight native callbacks.
  - Evidence: native clear/replace/destroy calls can wait for callbacks that may
    need to attach to Python.
  - Resolution: `_native` releases the GIL with `py.detach(...)` around
    `mln_runtime_destroy()`, `mln_runtime_set_resource_transform()`, and
    `mln_runtime_clear_resource_transform()`.
- Contain panics in native callback trampolines.
  - Evidence: callback contracts prohibit unwinding through native frames.
  - Resolution: logging, resource-provider, resource-transform, and
    custom-geometry trampolines now catch unwinds and return the callback
    contract's safe fallback.
- Drop custom geometry state when a custom source is explicitly removed.
  - Evidence: explicit source removal is a known-safe teardown point for that
    source.
  - Resolution: successful `remove_style_source()` removes retained custom
    geometry state for the source ID.
- Harden JSON convenience values accepted by style JSON/property/filter APIs.
  - Evidence: public style APIs are easier and less error-prone when ordinary
    Python JSON-like values are accepted, while explicit JSON wrappers remain
    available for duplicate-key and numeric-shape preservation.
  - Resolution: `_to_native_wire()` accepts finite `int`/`float`,
    `list`/`tuple`, and `dict` values; tests cover plain Python style values and
    non-finite float rejection.
- Strengthen local Python binding CI.
  - Evidence: the review loop relies on local validation evidence, and a wheel
    build exercises packaging metadata and extension import without entering
    release/publishing scope.
  - Resolution: `//bindings/python:ci` now runs `maturin build` before the
    installed package metadata/import check.
- Align `bindings/python/SPEC.md` with the implementation map by listing
  `_lifecycle.py` and current root event-payload exports.

### Rejected or deferred findings

- Additional native resource callback bridge test helpers for overflow/exception
  paths: deferred for now because existing Rust state-machine tests and public
  Python adapter tests cover the most brittle lifecycle cases, while adding
  broad private callback simulators would expand test-only bridge surface.
  Revisit if later reviewers find a concrete regression or low-cost trigger
  path.
- Render-session readback/frame invariant tests: deferred as optional validation
  hardening. The existing public render/session tests and Rust bridge checks
  cover current behavior; deeper backend-owned texture tests are
  host/backend-specific and not necessary for this round's actionable
  correctness fixes.
- Packaging/distribution, example applications, and CI-matrix integration:
  rejected as out of scope for the review-loop goal unless a Python-binding
  finding directly requires them.
- Broad enum deduplication/refactoring: rejected as needless churn; the specific
  unknown-preservation/setter-validation issues were addressed directly.

### Findings requiring user input

- None in this round.

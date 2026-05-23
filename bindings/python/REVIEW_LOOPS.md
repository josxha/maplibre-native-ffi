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

## Round 2

Review fanout: final public API/SPEC check, Rust/PyO3 bridge check, and branch
state/validation audit after Round 1 commits.

### Applied findings

- Make handle finalizers quiet during interpreter shutdown.
  - Evidence: one reviewer reproduced noisy unraisable `ImportError` from
    `__del__` methods that imported `_lifecycle` after `sys.meta_path` had been
    cleared during shutdown, even for already-closed handles.
  - Resolution: modules bind `_warn_unclosed` at import time, finalizers use the
    bound helper instead of importing during `__del__`, and finalizers suppress
    shutdown-time exceptions. `_lifecycle.warn_unclosed()` also captures
    `warnings.warn` at definition time and treats leak reporting as best-effort.
  - Coverage: added a subprocess regression test that closes runtime, map, and
    custom-geometry handles, exits the interpreter, and asserts stderr does not
    contain shutdown finalizer noise.
- Cover explicit custom-geometry source removal.
  - Evidence: reviewers noted the Round 1 record claimed successful
    `remove_style_source()` releases retained custom geometry state, but there
    was no focused test for the handle behavior.
  - Resolution: added a test that removes a custom geometry source, verifies the
    returned handle is closed, and verifies test callback events are ignored.

### Rejected or deferred findings

- No new rejected or deferred findings in this round. Previously deferred items
  remain unchanged: broad private callback simulators, backend-specific render
  readback/frame hardening, packaging/distribution/CI-matrix expansion, and
  broad enum deduplication.

### Findings requiring user input

- None in this round.

### Validation

- Focused finalizer regression:
  `mise run //bindings/python:test --
  tests/test_package.py::test_closed_handle_finalizers_are_quiet_at_interpreter_shutdown`
- Focused custom-source removal regression:
  `mise run //bindings/python:test --
  tests/test_package.py::test_remove_style_source_releases_custom_geometry_handle`
- Round validation: `uv run ruff check bindings/python`,
  `uv run ruff format --check bindings/python`,
  `cargo check -p maplibre-native-python`, and `mise run //bindings/python:ci`
  (58 Python tests, wheel build, and metadata/`_native` import check) passed.

## Round 3

Review fanout: final-final public Python API/tests/SPEC review after Round 2.

### Applied findings

- Align public JSON input type hints with runtime JSON-like input behavior.
  - Evidence: final review found `_to_native_wire()` accepts plain finite
    numbers, lists/tuples, and dicts, but some public `py.typed` annotations
    still named only explicit `JsonValue`/`JsonObject` wrappers.
  - Resolution: added public `JsonLike` and widened JSON-input annotations for
    style source JSON, rendered/source query filters, feature-extension
    arguments, and feature state setters while preserving `JsonValue` for copied
    outputs.
  - Coverage: updated public API smoke tests to pass plain dict/list values
    through style source JSON, query filters, feature-extension arguments, and
    feature state setters.

### Rejected or deferred findings

- No new rejected or deferred findings in this round. Previously deferred items
  remain unchanged: broad private callback simulators, backend-specific render
  readback/frame hardening, packaging/distribution/CI-matrix expansion, and
  broad enum deduplication.

### Findings requiring user input

- None in this round.

### Validation

- Focused JSON-like input regression:
  `mise run //bindings/python:test --
  tests/test_package.py::test_style_source_url_metadata_and_removal_public_api
  tests/test_package.py::test_render_session_query_public_api_uses_query_and_geojson_wire_values
  tests/test_package.py::test_render_session_feature_state_public_api_uses_json_wire_values`
- Repository fix/check task: `mise run fix`
- Full Python binding CI: `mise run //bindings/python:ci` (58 Python tests,
  wheel build, and metadata/`_native` import check)

## Round 4

Review fanout: final completion review after Round 3 JsonLike annotation fixes.

### Applied findings

- Align remaining style JSON/property/filter type hints with `JsonLike` and
  `JsonValue`.
  - Evidence: final review found style layer, light, property, and filter APIs
    still used `object`/`object | None` annotations despite accepting JSON-like
    inputs and returning copied JSON values through `_to_native_wire()` and
    `_from_native_wire()`.
  - Resolution: widened style JSON/property/filter inputs to `JsonLike` and
    narrowed copied JSON outputs to `JsonValue | None`.
  - Coverage: existing style JSON/property/filter tests already pass plain dict,
    list, string, and number values through these APIs.

### Rejected or deferred findings

- No new rejected or deferred findings in this round. Previously deferred items
  remain unchanged: broad private callback simulators, backend-specific render
  readback/frame hardening, packaging/distribution/CI-matrix expansion, and
  broad enum deduplication.

### Findings requiring user input

- None in this round.

### Validation

- Focused style JSON/property/filter regression:
  `mise run //bindings/python:test --
  tests/test_package.py::test_style_json_light_layer_property_and_filter_public_api`
- Repository fix/check task: `mise run fix`
- Full Python binding CI: `mise run //bindings/python:ci`

## Round 5

Review fanout: final API correctness, tests/validation accuracy, and public
type-surface polish after Round 4.

### Applied findings

- Restrict top-level JSON object parameters to object-shaped inputs in public
  annotations.
  - Evidence: final review found source JSON, layer JSON, light JSON, and
    feature-extension argument APIs used broad `JsonLike` annotations even
    though their C contracts require a top-level JSON object.
  - Resolution: added public `JsonObjectLike` and used it for top-level
    source/layer/light JSON and feature-extension argument inputs, while keeping
    property/filter/state inputs as `JsonLike`.
- Tighten remaining public typed-surface annotations.
  - Evidence: final review found several `py.typed` public methods still
    returned or accepted `object` where concrete public types were known.
  - Resolution: narrowed runtime offline/map/resource callback annotations,
    `MapHandle.set_style_image()` image input, and added explicit `__all__`
    lists to `errors`, `map`, `render`, and `runtime` modules.

### Rejected or deferred findings

- `RenderSessionHandle.read_premultiplied_rgba8_into(buffer: object)` remains
  intentionally broad because Python typing cannot precisely express arbitrary
  writable contiguous buffer-protocol objects without over-promising.
- Existing deferred items remain unchanged: broad private callback simulators,
  backend-specific render readback/frame hardening,
  packaging/distribution/CI-matrix expansion, and broad enum deduplication.

### Findings requiring user input

- None in this round.

### Validation

- Focused JSON object/type-surface regression:
  `mise run //bindings/python:test --
  tests/test_package.py::test_style_source_url_metadata_and_removal_public_api
  tests/test_package.py::test_style_json_light_layer_property_and_filter_public_api
  tests/test_package.py::test_render_session_query_public_api_uses_query_and_geojson_wire_values
  tests/test_package.py::test_render_session_feature_state_public_api_uses_json_wire_values`
- Round validation: `uv run ruff check bindings/python`,
  `uv run ruff format --check bindings/python`,
  `cargo check -p maplibre-native-python`, and `mise run //bindings/python:ci`
  (58 Python tests, wheel build, and metadata/`_native` import check) passed.

## Round 6

Review fanout: final API/type-surface and lifecycle/callback review after Round
5 typed-surface polish.

### Applied findings

- Avoid holding the runtime handle-state mutex while detached native calls wait
  for resource-transform callbacks.
  - Evidence: final lifecycle review found
    `RuntimeHandle.set_resource_transform()`, `clear_resource_transform()`, and
    `close()` released the GIL for native calls that can wait for callbacks, but
    still held the Rust handle-state mutex. A Python callback or another Python
    thread touching the same runtime could then block on that mutex while
    holding or needing the GIL.
  - Resolution: capture the native runtime address, release the handle-state
    mutex before `py.detach(...)`, and reacquire only for close failure
    recovery/bookkeeping. Runtime close marks the handle closed before detached
    destruction and restores the address if native destruction fails so
    owner-thread retry semantics are preserved.

### Rejected or deferred findings

- Broader public top-level export expansion remains deferred; reviewers found no
  evidence it is required for this PR.
- Existing deferred items remain unchanged: broad private callback simulators,
  backend-specific render readback/frame hardening,
  packaging/distribution/CI-matrix expansion, and broad enum deduplication.

### Findings requiring user input

- None in this round.

### Validation

- Focused resource-transform/runtime regression:
  `mise run //bindings/python:test --
  tests/test_package.py::test_resource_transform_registers_and_clears
  tests/test_package.py::test_runtime_handle_context_manager_closes_once
  tests/test_package.py::test_duplicate_runtime_reports_invalid_state
  tests/test_package.py::test_runtime_rejects_close_while_map_is_live`
- Round validation: `uv run ruff check bindings/python`,
  `uv run ruff format --check bindings/python`,
  `cargo check -p maplibre-native-python`, and `mise run //bindings/python:ci`
  (58 Python tests, wheel build, and metadata/`_native` import check) passed.

## Round 7

Review fanout: final lifecycle/callback, API typing, and review-record check
after Round 6 runtime wait fix.

### Applied findings

- Preserve callback `user_data` boxes when Python finalizers intentionally leak
  native runtime/map handles.
  - Evidence: final lifecycle review found Python finalizers warn without
    closing thread-affine native owners, but dropping the PyO3 owner could still
    drop boxed callback state used as native `user_data`.
  - Resolution: added `Drop` handling for `_RuntimeHandle` and `_MapHandle` that
    intentionally leaks retained callback boxes when the native owner is still
    live. Explicit close still releases retained callback state normally.
- Serialize detached runtime wait operations without holding the GIL or
  handle-state mutex.
  - Evidence: after Round 6, detached native waits no longer held the
    handle-state mutex, but a concurrent close/transform operation could race on
    the same raw runtime pointer.
  - Resolution: added a runtime operation gate that rejects concurrent detached
    runtime operations and close while another detached operation is active,
    without waiting while holding the GIL.
- Make public annotations resolvable through `typing.get_type_hints()`.
  - Evidence: final API review found several public annotations used names
    imported only under `TYPE_CHECKING`, causing `typing.get_type_hints()` to
    raise `NameError` for public `py.typed` APIs.
  - Resolution: added runtime fallback aliases for annotation-only imports and a
    focused smoke test covering the affected map/render/runtime/offline public
    methods.

### Rejected or deferred findings

- Annotation-only fallback aliases resolve to `Any` at runtime for cyclic type
  dependencies while preserving static `TYPE_CHECKING` imports for type
  checkers. This avoids runtime import cycles without weakening static
  `py.typed` consumers.
- Existing deferred items remain unchanged: broad private callback simulators,
  backend-specific render readback/frame hardening,
  packaging/distribution/CI-matrix expansion, broad enum deduplication, and
  broader root export expansion.

### Findings requiring user input

- None in this round.

### Validation

- Focused lifecycle/type-hints regression:
  `mise run //bindings/python:test --
  tests/test_package.py::test_public_type_hints_are_resolvable
  tests/test_package.py::test_resource_transform_registers_and_clears
  tests/test_package.py::test_runtime_handle_context_manager_closes_once
  tests/test_package.py::test_runtime_rejects_close_while_map_is_live`
- Round validation: `uv run ruff check bindings/python`,
  `uv run ruff format --check bindings/python`,
  `cargo check -p maplibre-native-python`,
  `uv run ty check --error-on-warning .mise`, and
  `mise run //bindings/python:ci` (59 Python tests, wheel build, and
  metadata/`_native` import check) passed.

## Round 8

Review fanout: final post-Round-7 check for lifecycle/threading, public
typing/API correctness, and review-record accuracy.

### Applied findings

- Prevent concurrent runtime close from reporting false success while teardown
  is in flight.
  - Evidence: lifecycle review found `RuntimeHandle.close()` checked the native
    address before entering the operation gate. A concurrent close could observe
    the first close's temporary `None` address while native destroy was detached
    and return success, even if the first close later failed and restored the
    address.
  - Resolution: `RuntimeOperationGate` now tracks terminal closed state.
    `close()` enters the gate before any closed fast path, reports
    `runtime is closing` to concurrent close callers, marks terminal closed only
    after successful native destroy, and resets the gate after failed destroy so
    retry semantics remain intact.
- Preserve precise runtime JSON type hints where imports are acyclic.
  - Evidence: public API review found `typing.get_type_hints()` resolved JSON
    aliases in `map.py` and `render.py` to `Any` because the aliases were
    imported only under `TYPE_CHECKING` and replaced by runtime fallbacks.
  - Resolution: import `JsonLike`, `JsonObjectLike`, and `JsonValue`
    unconditionally in `map.py` and `render.py`, keep `Any` fallbacks only for
    cyclic annotation dependencies, and strengthen the smoke test to assert
    representative JSON hints stay non-`Any` and JSON-shaped.

### Rejected or deferred findings

- Cyclic public annotation fallbacks remain `Any` at runtime where importing the
  real type would create module cycles. Static `TYPE_CHECKING` imports keep the
  source/type-checker surface precise.
- Existing deferred items remain unchanged: broad private callback simulators,
  backend-specific render readback/frame hardening,
  packaging/distribution/CI-matrix expansion, broad enum deduplication, and
  broader root export expansion.

### Findings requiring user input

- None in this round.

### Validation

- Focused regression:
  `mise run //bindings/python:test --
  tests/test_package.py::test_public_type_hints_are_resolvable
  tests/test_package.py::test_runtime_close_from_wrong_thread_reports_wrong_thread
  tests/test_package.py::test_runtime_handle_context_manager_closes_once
  tests/test_package.py::test_resource_transform_registers_and_clears`
- Round validation: `uv run ruff check bindings/python`,
  `uv run ruff format --check bindings/python`,
  `cargo check -p maplibre-native-python`,
  `uv run ty check --error-on-warning .mise`, and
  `mise run //bindings/python:ci` (59 Python tests, wheel build, and
  metadata/`_native` import check) passed.

## Round 9

Review fanout: post-Round-8 lifecycle/threading, public typing/API correctness,
and review-record accuracy.

### Applied findings

- Report `RuntimeHandle.closed` from the terminal close state, not the transient
  native pointer state.
  - Evidence: lifecycle review found `close()` temporarily marks the native
    pointer closed while native teardown runs detached, then restores it if
    teardown fails. A concurrent reader of `runtime.closed` could observe `True`
    while close was still in flight and might later fail.
  - Resolution: added `RuntimeOperationGate::is_closed()` and made the Python
    `closed` getter use the gate's terminal `closed` flag, which is set only
    after successful native destroy.
- Preserve precise runtime hints for additional acyclic public annotation
  dependencies.
  - Evidence: public API review found several public annotations still resolved
    to `Any` even though their dependency graph is acyclic, including resource
    callbacks in `runtime.py`, query/feature types in `render.py`, and
    render/style inputs in `map.py`.
  - Resolution: import resource callback aliases, offline operation/definition
    types, render/query/geo types, and map-facing render/style/camera/geo types
    unconditionally where they do not create import cycles. Strengthened the
    type-hint smoke test with representative non-`Any` assertions for resource
    callbacks, feature extension inputs/results, rendered query options, and
    style image inputs.

### Rejected or deferred findings

- Runtime `create_map()` annotations still use `Any` fallbacks for
  `MapHandle`/`MapOptions` because importing `map.py` from `runtime.py` at
  module load time would create the real module cycle. Static `TYPE_CHECKING`
  imports keep type-checker behavior precise.
- Existing deferred items remain unchanged: broad private callback simulators,
  backend-specific render readback/frame hardening,
  packaging/distribution/CI-matrix expansion, broad enum deduplication, and
  broader root export expansion.

### Findings requiring user input

- None in this round.

### Validation

- Focused regression:
  `mise run //bindings/python:test --
  tests/test_package.py::test_public_type_hints_are_resolvable
  tests/test_package.py::test_runtime_close_from_wrong_thread_reports_wrong_thread
  tests/test_package.py::test_runtime_handle_context_manager_closes_once
  tests/test_package.py::test_resource_transform_registers_and_clears`
- Round validation: `uv run ruff check bindings/python`,
  `uv run ruff format --check bindings/python`,
  `cargo check -p maplibre-native-python`,
  `uv run ty check --error-on-warning .mise`, and
  `mise run //bindings/python:ci` (59 Python tests, wheel build, and
  metadata/`_native` import check) passed.

## Round 10

Review fanout: post-Round-9 final lifecycle, typing/API, and review-record
audit.

### Applied findings

- Preserve the offline response-error reason annotation at runtime.
  - Evidence: typing review found `OfflineRegionResponseError.reason` resolved
    to `Any` because `ResourceErrorReason` was imported only under
    `TYPE_CHECKING`, even though `resource.py` does not create a module cycle
    for this dependency.
  - Resolution: import `ResourceErrorReason` unconditionally in `offline.py`,
    keep only the cyclic `RuntimeHandle` fallback, and add a
    `typing.get_type_hints()` assertion for `OfflineRegionResponseError.reason`.

### Rejected or deferred findings

- Remaining `Any` fallbacks are tied to real import cycles: `render.MapHandle`,
  `runtime.MapHandle`/`MapOptions`, and `offline.RuntimeHandle`. Static
  `TYPE_CHECKING` imports keep type-checker behavior precise.
- Existing deferred items remain unchanged: broad private callback simulators,
  backend-specific render readback/frame hardening,
  packaging/distribution/CI-matrix expansion, broad enum deduplication, and
  broader root export expansion.

### Findings requiring user input

- None in this round.

### Validation

- Focused type-hint regression:
  `mise run //bindings/python:test --
  tests/test_package.py::test_public_type_hints_are_resolvable`
- Round validation: `uv run ruff check bindings/python`,
  `uv run ruff format --check bindings/python`,
  `cargo check -p maplibre-native-python`,
  `uv run ty check --error-on-warning .mise`, and
  `mise run //bindings/python:ci` (59 Python tests, wheel build, and
  metadata/`_native` import check) passed.

## Round 11

Review fanout: final post-Round-10 lifecycle, typing/API, and review-record
audit.

### Applied findings

- Reject offline operation result takes after the operation handle is closed.
  - Evidence: lifecycle review found `OfflineOperationHandle.take_region()`,
    `take_optional_region()`, `take_region_list()`, `take_updated_region()`, and
    `take_status()` called native even after `close()` or a prior successful
    take, bypassing Python-side closed-handle validation.
  - Resolution: added a shared `_ensure_open()` guard that raises
    `InvalidStateError` before native calls on closed offline operation handles.
    Added regression coverage for double-take and take-after-close paths,
    including a fake native backend that would fail if called after closure.

### Rejected or deferred findings

- Broader runtime-close coordination with still-live offline-operation wrappers
  remains deferred as ownership-policy work outside this narrow closed-handle
  guard.
- Remaining `Any` fallbacks are tied to real import cycles: `render.MapHandle`,
  `runtime.MapHandle`/`MapOptions`, and `offline.RuntimeHandle`. Static
  `TYPE_CHECKING` imports keep type-checker behavior precise.
- Existing deferred items remain unchanged: broad private callback simulators,
  backend-specific render readback/frame hardening,
  packaging/distribution/CI-matrix expansion, broad enum deduplication, and
  broader root export expansion.

### Findings requiring user input

- None in this round.

### Validation

- Focused offline lifecycle regression:
  `mise run //bindings/python:test --
  tests/test_package.py::test_offline_operation_take_rejects_closed_handles
  tests/test_package.py::test_offline_operation_take_results_convert_public_values`
- Round validation: `uv run ruff check bindings/python`,
  `uv run ruff format --check bindings/python`,
  `cargo check -p maplibre-native-python`,
  `uv run ty check --error-on-warning .mise`, and
  `mise run //bindings/python:ci` (60 Python tests, wheel build, and
  metadata/`_native` import check) passed.

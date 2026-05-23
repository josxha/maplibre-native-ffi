# Python Binding Implementation Map

## Audience and documentation role

Audience: contributors implementing and reviewing the Python binding. Category:
reference with a short explanatory map. This document names the concrete files,
modules, tasks, and coverage targets for the binding. The convention documents
remain the source of design rules.

## Normative references

The implementation follows these documents. This spec links to them instead of
restating their rules.

- [Concepts](../../docs/src/content/docs/concepts.md): runtime, map, render
  session, events, and ownership boundaries.
- [C API conventions](../../docs/src/content/docs/development/c-conventions.md):
  status, diagnostics, callbacks, ABI ownership, and thread-affinity contract.
- [Binding conventions](../../docs/src/content/docs/development/bindings.md):
  shared handle, type, callback, rendering, and testing rules.
- [Python binding conventions](../../docs/src/content/docs/development/bindings-python.md):
  PyO3 and maturin architecture, Python ownership, exceptions, buffers, GIL
  handling, callbacks, and tests.
- [Rust binding conventions](../../docs/src/content/docs/development/bindings-rust.md):
  `maplibre-native-sys`, `maplibre-native-core`, status conversion, copied
  native results, and bridge binding boundaries.
- [Rust bridge binding plan](../rust/PLAN.md): bridge crate ownership and the
  first shared-core proof slice.
- [Java FFM binding conventions](../../docs/src/content/docs/development/bindings-java-ffm.md)
  and
  [Java JNI binding conventions](../../docs/src/content/docs/development/bindings-java-jni.md):
  parity targets for low-level public concepts, exception taxonomy, handle
  names, and JNI bridge responsibilities.

The scaffold style follows the Kotlin/Native binding scaffold in PR #179 and the
Swift binding scaffold in PR #180: one proof slice, package/module markers,
shared error and pointer types, binding-local tasks, and an implementation map
that records future coverage.

When this spec and a convention document appear to overlap, the convention
contains the rule and this spec names the concrete Python implementation points.
The public C headers are the ABI source. The shared Rust crates are the bridge
source for reusable C ABI adaptation.

## Scope

`bindings/python` is the low-level Python binding over the public MapLibre
Native C API. It targets CPython 3.14+ and exposes the `maplibre_native` Python
package.

The private extension module is `maplibre_native._native`. It is built with PyO3
and maturin from the `maplibre-native-python` Rust crate. `_native` depends on
`maplibre-native-core` and `maplibre-native-sys`; it keeps raw C layouts, PyO3
types, pointer-sized handles, and native callback trampolines private.

The public Python package wraps `_native` with typed classes, enums,
dataclasses, exceptions, and concept modules. It preserves the C API's runtime,
map, render session, event, resource, query, style, and rendering model while
adapting ownership, diagnostics, buffers, callbacks, and native backend handles
to Python conventions. Higher-level `asyncio`, GUI-loop, executor, view
lifecycle, and SDK adapters belong above this package.

## Python differences and omissions

Record Python-only differences here.

| Item               | Difference or omission                                                       | Reason                                                                                         | User-visible behavior                                                                                       | Tests/docs impact                                                                                               |
| ------------------ | ---------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Callback execution | Low-level callbacks may use bounded queues before Python callable execution. | Native callbacks may arrive on threads where arbitrary Python execution is unsafe or too slow. | Queue overflow follows each callback contract: fail, drop best-effort notification, or report cancellation. | Callback tests will cover queue capacity, exception conversion, and non-blocking behavior.                      |
| Finalizers         | Finalizers report leaked thread-affine handles instead of destroying them.   | Python finalizers can run on arbitrary threads or during interpreter finalization.             | Users close handles explicitly or with context managers. Leak reports identify unreleased handles.          | Handle tests will assert close-once behavior; leak reporting tests will avoid native destruction from GC paths. |

## Current implementation map

```text
bindings/python/
  SPEC.md
  Cargo.toml
  pyproject.toml
  uv.lock
  mise.toml
  src/lib.rs
  python/maplibre_native/
    __init__.py
    _global.py
    camera.py
    errors.py
    geo.py
    json.py
    log.py
    map.py
    offline.py
    query.py
    render.py
    resource.py
    runtime.py
    style.py
    py.typed
  tests/test_package.py
```

The current implementation includes these completed slices:

- maturin builds the private `maplibre_native._native` PyO3 extension.
- `_native` links through the Rust `maplibre-native-sys` crate to the public C
  ABI.
- `maplibre_native.c_version()` calls `mln_c_version()` through `_native`.
- `maplibre_native.supported_render_backends()` calls
  `mln_supported_render_backend_mask()` and returns a Python `IntFlag` value.
- `maplibre_native.network_status()` and `maplibre_native.set_network_status()`
  cross native status-returning APIs and raise Python `MaplibreError` subclasses
  for failures.
- `RuntimeHandle` creates, runs, closes, and supports context-manager cleanup
  for a native runtime handle.
- `MapHandle` creates and closes maps with parent runtime retention and basic
  map options.
- `MapHandle` exposes map debug overlay options, rendering-stats view state,
  loaded-state queries, viewport options, and tile options.
- `MapHandle` exposes GeoJSON source URL insertion plus vector, raster, and
  raster DEM style source URL/inline-tile insertion, source removal, existence
  checks, type/info lookup, attribution copying, and source ID listing.
- `MapHandle` exposes built-in hillshade, color-relief, and location-indicator
  layer insertion, location-indicator property setters, style layer removal,
  existence checks, type lookup, ID listing, and layer reordering.
- `MapHandle` exposes runtime style image set, removal, existence checks,
  metadata lookup, and premultiplied RGBA8 copying.
- `MapHandle` exposes image source URL/inline-image insertion, URL/image
  updates, coordinate updates, and coordinate copying.
- `RuntimeHandle.poll_event()` returns runtime events copied into Python-owned
  values.
- `maplibre_native.camera` provides camera descriptors, and `MapHandle` exposes
  camera snapshot, jump, ease, fly, pan, scale, rotate, pitch, and
  transition-cancel operations.
- `maplibre_native.log` provides process-global logging callback configuration,
  bounded copied-record queues, async severity masks, and copied log records.
- `maplibre_native.query` provides rendered/source query descriptors, feature
  state selectors, and copied query result value shapes.
- `maplibre_native.offline` provides offline operation, region definition,
  region status, and runtime event payload value shapes.
- `RuntimeHandle.run_ambient_cache_operation()` starts native ambient cache
  maintenance and returns an `OfflineOperationHandle` that discards
  runtime-owned operation state on close.
- `maplibre_native.map` provides `MapProjectionHandle` and standalone Mercator
  projection helpers for copied camera, coordinate, and screen-point values.
- Public error classes, `MaplibreStatus`, `NetworkStatus`, `RenderBackend`, and
  `NativePointer` establish shared naming and value semantics.
- `maplibre_native.json` provides JSON value trees that preserve numeric shape,
  ordering, and duplicate object members.
- `maplibre_native.geo` provides geographic, geometry, and GeoJSON value trees
  for later native source/query APIs.
- Concept modules exist so future changes land in stable package locations.

## Build artifacts and tasks

| Artifact                 | Path                         | Contents                                                                                                                         |
| ------------------------ | ---------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Python project           | `bindings/python`            | Public Python package, PyO3 crate, tests, maturin metadata.                                                                      |
| PyO3 extension crate     | `bindings/python/Cargo.toml` | `maplibre-native-python` cdylib compiled as `maplibre_native._native`.                                                           |
| Public Python package    | `python/maplibre_native`     | Typed public facade, exceptions, values, handles, and concept modules.                                                           |
| Private extension module | `maplibre_native._native`    | Proof-slice bridge functions, native status conversion, runtime/map handle state, copied events, and later callback trampolines. |
| Test suite               | `tests`                      | Python tests against the real native C library.                                                                                  |

Implemented tasks:

| Task                                          | Required behavior                                              |
| --------------------------------------------- | -------------------------------------------------------------- |
| `mise run //bindings/python:build`            | Build the wheel after the native C library exists.             |
| `mise run //bindings/python:develop`          | Install the extension into the local uv environment for tests. |
| `mise run //bindings/python:test`             | Run Python tests against the real C library.                   |
| `mise run //bindings/python:ci`               | Run tests and validate installed package metadata.             |
| `uv run --locked --group dev maturin build`   | Build the wheel from `bindings/python`.                        |
| `uv run --locked --group dev maturin develop` | Build and install the extension in editable development mode.  |
| `uv run --locked --group dev pytest`          | Run package tests from `bindings/python`.                      |

The extension links `maplibre-native-c` through `maplibre-native-sys`, which
reads `MLN_FFI_BUILD_DIR`. Local tasks depend on `//:ensure-native-library` so
the native library exists before maturin invokes Cargo.

## Module responsibilities

### `maplibre_native._native`

`_native` is private. The current scaffold exposes proof-slice bridge functions,
status-converting network status entry points, runtime/map/render-session handle
state, copied event adapters, render target descriptor materialization, texture
readback, texture frame guards, resource callback trampolines, one-shot resource
request handles, and queued custom geometry source callbacks needed by the
Python package. As coverage grows, `_native` owns PyO3-specific conversion, GIL
handling, Python exception construction, buffer guards, callback queues, and
free-threaded synchronization.

`_native` may call `maplibre-native-sys` directly only for the initial proof
slice or host-runtime trampoline code. Repeated C ABI adaptation moves into
`maplibre-native-core`.

### `maplibre_native`

The package root exports stable public names:

```python
c_version()
network_status()
set_network_status()
supported_render_backends()
EXPECTED_C_ABI_VERSION
InvalidArgumentError
InvalidStateError
MapHandle
MapMode
MapOptions
MaplibreError
MaplibreStatus
NativeError
NativePointer
NetworkStatus
RenderBackend
UnknownStatusError
UnsupportedFeatureError
RuntimeEvent
RuntimeEventSource
RuntimeEventSourceType
RuntimeEventType
RuntimeHandle
RuntimeOptions
WrongThreadError
```

Future root-level exports stay limited to process-global entry points and common
values. Concept-specific names live in their concept modules and may be
re-exported from the root only when they are part of the common low-level entry
surface.

### Concept modules

Create and keep these public modules:

```text
maplibre_native.camera
maplibre_native.errors
maplibre_native.geo
maplibre_native.json
maplibre_native.log
maplibre_native.map
maplibre_native.offline
maplibre_native.query
maplibre_native.render
maplibre_native.resource
maplibre_native.runtime
maplibre_native.style
```

The modules group Python API by C concept. Internal support modules use a
leading underscore. Raw pointers, PyO3 classes, generated C declarations,
callback trampolines, and handle internals stay out of public modules.

## Public API map

| C concept                            | Python module                                  | Public shape                                                                                                    |
| ------------------------------------ | ---------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Process-global ABI and library state | `maplibre_native`                              | Functions such as `c_version()`, `supported_render_backends()`, `network_status()`, and `set_network_status()`. |
| Status and diagnostics               | `maplibre_native.errors`                       | `MaplibreError` subclasses carrying `MaplibreStatus`, raw status code, and diagnostic.                          |
| Runtime and events                   | `maplibre_native.runtime`                      | `RuntimeHandle`, `RuntimeOptions`, event values, offline operation handles, and resource callback owners.       |
| Map and style                        | `maplibre_native.map`, `maplibre_native.style` | `MapHandle`, map options, style source/layer/image values, and custom geometry source state.                    |
| Projection                           | `maplibre_native.map`                          | `MapProjectionHandle` and standalone projection helpers.                                                        |
| Render targets                       | `maplibre_native.render`                       | `RenderSessionHandle`, render target descriptors, `NativePointer`, backend masks, buffers, and frame handles.   |
| Resources                            | `maplibre_native.resource`                     | Resource requests, responses, provider decisions, transform requests, and one-shot request handles.             |
| Camera                               | `maplibre_native.camera`                       | Camera, bounds, fit, animation, and free-camera descriptors.                                                    |
| Geometry and JSON                    | `maplibre_native.geo`, `maplibre_native.json`  | Copied value trees that preserve numeric shape, order, and duplicate JSON object members.                       |
| Queries                              | `maplibre_native.query`                        | Rendered/source query descriptors and copied result values.                                                     |
| Logging                              | `maplibre_native.log`                          | Process-global logging callback configuration and copied log records.                                           |
| Offline                              | `maplibre_native.offline`                      | Offline operation values and event payloads not owned by `runtime`.                                             |

## Naming rules

Use `Maplibre` in code identifiers and `MapLibre` in prose. Public owned native
objects use the `Handle` suffix:

```text
RuntimeHandle
MapHandle
MapProjectionHandle
RenderSessionHandle
ResourceRequestHandle
OfflineOperationHandle
```

Descriptors use Python dataclasses or small classes. Mutable descriptors expose
explicit setters or ordinary mutable attributes consistently within a type.
Field masks, `size` fields, C string views, and nested native arrays stay
internal to `_native` and `maplibre-native-core` materializers.

Closed C enum domains map to Python `Enum` classes with explicit native values.
C bit masks map to `IntFlag` classes when callers combine bits. Output domains
that may grow preserve unknown raw values in the returned value or in the
exception/report object. `NetworkStatus` preserves unknown raw values for native
outputs and future setters reject unknown values.

## Handle lifecycle

Each public handle stores synchronized extension-owned state:

- native pointer;
- live or closed flag;
- parent references needed for native validity;
- active scoped-borrow flags;
- optional leak context.

Each handle exposes `close()` and implements the context-manager protocol.
Successful `close()` releases the native object exactly once; later closes
no-op. Failed native destruction leaves the handle live so callers can retry on
the owner thread.

`__del__`, weakref finalizers, and GC callbacks report leaks. They destroy
native resources only when the native release function is thread-independent and
infallible. Thread-affine handles close through explicit user calls on the owner
thread.

Child handles keep parent wrappers alive while native validity depends on the
parent. `MapProjectionHandle` is the shared exception: after creation, it owns a
standalone projection snapshot and does not keep the source `MapHandle` alive
for native validity.

## Threads, the GIL, and free-threaded CPython

The Python GIL is not the MapLibre owner-thread identity. Handle methods execute
on the native thread that invoked them. The binding does not dispatch to another
thread or event loop. Native `MLN_STATUS_WRONG_THREAD` results become
`WrongThreadError` with the copied native diagnostic.

Free-threaded CPython uses the same invariants as GIL builds. Handle state,
callback state, queues, and one-shot request completion use explicit Rust
synchronization and do not rely on a process-wide GIL for correctness.

Release the GIL around long native calls only after every Python object used by
the call has been converted to Rust-owned storage or is pinned through a valid
PyO3 buffer guard.

## Exceptions and diagnostics

Status-returning `_native` operations raise `MaplibreError` subclasses. Each
exception stores:

- the stable `MaplibreStatus` category;
- the raw C status value;
- the native diagnostic copied immediately on the same native thread.

Unknown future status values raise `UnknownStatusError` and preserve the raw
status. `_native` catches Rust panics and Python `BaseException` values inside
callback adapters, then converts them to the C callback's documented behavior.
No Python exception or Rust panic unwinds through native frames.

The Python layer validates Python-owned state before crossing into native code:
closed handles, active scoped borrows, one-shot request completion, embedded NUL
in null-terminated strings, writable-buffer shape, descriptor depth, and
callback lifetime. The C API validates native state, thread affinity, ranges,
and MapLibre-specific rules.

## Memory, buffers, and copied data

Materialize native inputs at the call boundary with Rust-owned temporary
storage. Copy borrowed native strings, event payloads, list entries, snapshots,
and result views into Python-owned values before releasing native handles.
Runtime polling returns independent Python event objects; later polls do not
mutate earlier events.

Use `bytes` for immutable copied byte output. Writable storage APIs accept
objects with a writable contiguous buffer, such as `bytearray` or a suitable
`memoryview`. Hold a buffer borrow only for the native call or documented scoped
operation.

`NativePointer` is a borrowed opaque backend-native address value. It grants no
memory access and transfers no ownership. Public APIs accept or return it only
where the C API already accepts or returns opaque backend-native handles.

## Callbacks

Callback state is owned for the native scope that can invoke it. Native
callbacks may arrive on MapLibre worker, network, logging, or render-related
threads, so callback state is thread-safe. `_native` acquires the GIL only while
creating Python objects or invoking Python callables.

Resource provider callbacks copy borrowed request fields before Python code can
retain them. `ResourceRequestHandle` enforces one-shot completion and releases
the native request handle exactly once. Completion may occur inside the callback
or later from an allowed thread.

Resource transforms use native-owned rewrite rules when possible. Synchronous
Python callables are exposed only when the binding can honor the callback
without blocking unsafe native work. Custom geometry callbacks enqueue Python
notifications and return tile data, cancellation, and map mutation through
owner-thread map APIs.

Logging callbacks use a bounded best-effort queue unless direct invocation is
safe for the callback contract. Queue overflow behavior is explicit for each
callback API.

## Render targets

Render descriptors are Python values. Surface and texture descriptors use
`NativePointer` for host-owned backend handles; callers keep backend objects
valid and synchronized for the lifetime required by the C API.

`RenderSessionHandle` represents one attached target for one map and keeps the
map wrapper alive. Texture readback supports caller-owned writable buffers and a
convenience path returning copied image bytes plus `TextureImageInfo` metadata.

Session-owned texture targets expose explicit frame handles. Safe metadata
access returns copied Python values. Backend pointer accessors return scoped
`NativePointer` values tied to the live frame. Frame handles reject use after
close and close before resize, another render update, detach, or session
destruction.

## Testing strategy

Python tests exercise the public package against the real native library.
Binding tests focus on adaptation invariants rather than C behavior already
covered by C ABI tests.

Coverage targets:

- wheel build and import of `maplibre_native._native`;
- ABI version and supported backend proof slice;
- exception mapping and copied diagnostics;
- context-manager cleanup and close-once handle release;
- wrong-thread propagation;
- free-threaded synchronization assumptions;
- writable-buffer validation;
- copied events and copied native result lists;
- callback queueing and Python exception conversion;
- exactly-once resource request completion;
- leak reporting from finalizers without thread-affine native destruction.

## Rollout checklist

- [x] Add Python binding project, maturin metadata, and mise tasks.
- [x] Add private PyO3 extension scaffold using the shared Rust crates.
- [x] Add public package modules, error classes, `NativePointer`, and first
      process-global proof slice.
- [x] Move repeated direct `sys` sequences from `_native` into
      `maplibre-native-core` as bridge-neutral adapters for network status.
- [x] Implement native status-to-Python exception conversion in `_native`.
- [x] Add `network_status()` and `set_network_status()` over native status
      conversion.
- [x] Add `RuntimeHandle` with context-manager close and owner-thread error
      propagation.
- [x] Add `MapHandle`, map options, and parent retention.
- [x] Add runtime event polling with copied Python event values.
- [x] Add render session descriptors, writable-buffer readback, and texture
      frame handles.
- [x] Add resource transform and provider callbacks with bounded queue policy.
- [x] Add custom geometry callback scaffolding.
- [ ] Add package publication metadata once artifact policy is chosen.
- [ ] Add the Python binding to the CI binding matrix after the first native
      status-converting vertical slice passes on Linux and macOS.

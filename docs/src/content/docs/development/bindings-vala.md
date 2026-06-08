---
title: Vala Binding Conventions
description: Language-specific implementation conventions for Vala bindings.
---

Resources:

- Tracking issue:
  [#119](https://github.com/maplibre/maplibre-native-ffi/issues/119)
- [Vala manual](https://docs.vala.dev/)
- [Vala bindings documentation](https://docs.vala.dev/developer-guides/bindings.html)
- [Vala CCode attributes](https://docs.vala.dev/developer-guides/bindings/writing-a-vapi-manually.html)
- [`valac`](https://docs.vala.dev/)

## Architecture

The Vala binding is a direct binding over the public MapLibre Native C API. It
uses a private raw VAPI for `maplibre_native_c` declarations and organized
public Vala source files for handle classes, descriptors, copied values, errors,
callbacks, and lifetime policy.

```text
public Vala API
  Low-level Vala classes, structs, enums, errors, and callback types.

private raw C VAPI
  Handwritten or generated declarations for the public C headers.
  Raw pointers, C unions, field masks, borrowed storage, and native status
  values stay here.

public C API
  The shared MapLibre Native FFI ABI in include/maplibre_native_c/.
```

Do not build a separate Rust, GObject, GIR, or scanner-facing adapter layer for
Vala. Vala can call the C ABI directly, and `valac` already owns the language's
C generation model. Small C helpers may be added only when the public C ABI
shape is hard for Vala to express safely; prefer helpers that also simplify
other direct C bindings.

Keep this layer low-level. It exposes runtime, map, render-session, resource,
event, camera, geometry, and render-target concepts directly. GLib main-loop
integration, widgets, async frameworks, and application lifecycle policy belong
above this binding.

Organize public wrapper sources by C API domain under `bindings/vala/src`.
`base.vala` contains the public error domain, shared enums, primitive value
wrappers, `NativePointer`, and copied string-list values. `map.vala` contains
map options, tile options, viewport options, and `MapHandle`. `camera.vala`
contains camera, animation, fit, bounds, and free-camera descriptors.
`projection.vala` contains projection conversion helpers and
`MapProjectionHandle`. `internal.vala` contains status conversion, string-view
helpers, byte copying, list copying, and enum mapping helpers shared by domain
files. `runtime.vala` contains runtime options and lifecycle, process-global
logging/network helpers, copied runtime event payloads, resource request values,
provider response descriptors, provider decision values, one-shot resource
request handles, offline region IDs, offline operation IDs, region definitions,
and offline snapshot/list handles. `query.vala` contains query descriptors,
result handles, and feature-extension results. `render_session.vala` contains
render session lifecycle, maintenance, readback, feature-state, query entry
points, and Metal, Vulkan, and OpenGL frame handles. `geometry.vala` contains
JSON, GeoJSON, geometry, feature, feature-collection, and feature-state selector
descriptor graphs with their call-scoped native materialization helpers.
`style.vala` contains style source metadata values, tile source option
descriptors, custom geometry source callbacks, image source coordinate
materialization, runtime style image values, location indicator property enums,
style layer JSON/property/filter/light wrappers, style layer convenience
wrappers, and style option materialization helpers. `texture.vala` contains
texture render-target descriptors, backend context descriptors, readback
metadata values, and scoped Vulkan frame handles. `surface.vala` contains Metal
and Vulkan surface render-target descriptors. New or refactored API groups move
into focused backend-specific files as needed. Keep descriptor materialization
utilities near the descriptor types they support unless multiple domains use
them. The Vala build compiles all `src/*.vala` files so a new source file
participates in build and tests automatically.

The direct binding preserves practical coverage by wrapping C ABI concepts, not
by recreating the previous generated GObject surface. Keep compact wrappers for
lifecycle, diagnostics, options, rendering, projection, style values, resource
transforms, resource providers, offline operation tokens, copied lists, query
handles, descriptor graphs, and offline snapshots. Do not keep adapter-only
APIs, generated compatibility names, GIR metadata, or scanner helpers as parity
surfaces.

The direct-C surface covers the API groups that the old generated adapter could
not express safely:

- JSON, GeoJSON, geometry, feature, and feature-collection descriptor graphs use
  Vala-owned builders that keep nested storage alive for one C call. Coverage
  includes JSON values, feature and feature-collection descriptors, all geometry
  variants, geometry/feature/feature-collection GeoJSON, GeoJSON URL sources,
  style values, and copied query results.
- Resource provider callbacks use copied request fields, explicit provider
  decisions, pass-through-by-default behavior, one-shot request handles,
  cancellation checks, and explicit release. Runtime resource transforms keep
  replacement URL storage scoped to the runtime callback owner. Custom geometry
  source callbacks keep map-retained callback state, typed tile IDs, scoped
  GeoJSON tile data, and explicit invalidation helpers.
- Offline region definitions, operation starts, take-result helpers, snapshots,
  lists, statuses, and completion/event payloads are direct wrappers. Snapshot
  and list handles copy `OfflineRegionInfo` values before native handle close.
  Offline operations use `OfflineOperationId` value tokens because the C API
  owns operation lifetime until completion, discard, or take-result calls. The
  required Vala test task runs both a baseline smoke pass and an extended smoke
  pass that enables fixture-dependent offline operations.
- Query coverage includes rendered point, box, and line-string geometries;
  rendered/source query options with Vala-owned IDs and JSON filters; copied
  source-feature and rendered-feature result arrays; feature-state commands;
  copied feature-state JSON snapshots; queried-feature payload copying; and
  feature-extension results with copied JSON value or feature-collection
  payloads. The required extended smoke pass exercises feature-extension calls
  against the available source/extension fixture.
- Render-target wrappers mirror the C ABI for Metal, Vulkan, OpenGL, borrowed
  texture, owned texture, and surface sessions. Vulkan descriptors carry
  host-controlled loader procedure addresses. OpenGL descriptors cover EGL and
  WGL context providers. The Vala smoke test exercises the active backend's
  public attach wrappers, resize/detach lifecycle, CPU readback for owned
  textures, scoped frame handles, active-frame rejection, and unsupported
  owned-frame/readback/resize calls on borrowed texture sessions. The macOS
  Vulkan variant creates test-owned Vulkan images and Metal-backed Vulkan
  surfaces so the public Vala wrappers cross the real C ABI.

## Public Types

Long-lived native objects are Vala classes with the shared `Handle` suffix. Each
class stores the private raw native pointer, records live or closed state, and
exposes `close() throws MaplibreNative.Error`. A successful `close()` releases
exactly once; later calls no-op. Failed native destruction leaves the object
live so callers can retry on the owner thread.

Child handles hold strong Vala references to parents while native validity
depends on them: maps retain runtimes, render sessions retain maps, and
style-scoped objects retain their owner. Finalizers may report leaked handles,
but thread-affine native handles close deterministically through `close()` on
the owner thread. `MapProjectionHandle` follows the shared exception: it owns a
standalone projection snapshot and does not retain its source `MapHandle` for
native validity after creation.

Copied descriptors, events, snapshots, camera values, animation options, camera
fit and bound options, free camera options, projection mode options, projection
visible-geometry inputs, geometry values, query results, source attribution
strings, offline region IDs, offline operation IDs, offline region definitions,
offline region info snapshots, map viewport and tile options, debug masks, and
render metadata are Vala-owned values. Public APIs hide ABI `size` fields, field
masks, raw union storage, borrowed pointers, and backend pointer fields.
Constructors and setter methods initialize those ABI details inside the binding.

`NativePointer` is a small borrowed value around an address-sized integer. It
wraps a non-null backend-native address, grants no memory access, and transfers
no ownership. Use it only where the C API already accepts or returns opaque
backend handles.

## Errors, Metadata, and Ownership

Expose one public Vala error domain that maps C status categories directly:

```vala
public errordomain MaplibreNative.Error {
    INVALID_ARGUMENT,
    INVALID_STATE,
    WRONG_THREAD,
    UNSUPPORTED,
    NATIVE_ERROR,
    UNKNOWN_STATUS
}
```

Every status-returning operation captures the native thread-local diagnostic
immediately on the same thread and stores it in the error message. The binding
validates Vala-owned state before crossing into C: closed wrappers, active frame
borrows, nullable arguments, and callback scope. The C API continues to validate
native state, string bytes, enum domains, ranges, and owner-thread rules.

Treat the raw C VAPI as private implementation metadata. It may expose C layout,
borrowed pointers, and nullable raw values because public Vala wrappers copy or
scope those values before callers see them. Keep the public Vala API stable and
idiomatic; change raw declarations when the C API changes.

## Threading, Events, and Callbacks

Owner-thread-affine methods run on the calling thread. The binding does not
dispatch internally. Native `MLN_STATUS_WRONG_THREAD` becomes
`MaplibreNative.Error.WRONG_THREAD` with the copied diagnostic. Higher-level
GLib adapters may add `GMainContext` dispatch or async APIs; this layer keeps
native thread identity visible.

Runtime event polling returns copied Vala values. Event objects expose typed
event, source, payload, render-mode, tile-operation, resource-error, and offline
operation enums plus copied message and typed payload data. Render-frame,
render-map, style-image-missing, tile-action, offline-region-status,
offline-response-error, tile-count-limit, and offline-operation-completed
payloads are copied before the next native poll invalidates the C storage.
Public event objects remain valid after the next native poll.

Callback adapters keep delegates and user data alive for the native owner scope.
Native upcalls may arrive on worker, network, logging, or render-related
threads, so callback state must be safe for those threads. Vala errors stay
inside callback code and must not cross native frames. The process-global
logging wrapper mirrors the C ABI callback and `user_data` shape internally, and
public code controls asynchronous log dispatch with `LogSeverityMask`.

Resource transform callbacks receive copied request URLs and return replacement
URLs that remain scoped to the native callback. Resource provider callbacks
receive copied request fields and a request handle, return an explicit provider
decision, and pass through by default. Handled `ResourceRequestHandle` values
complete once and release explicitly. Custom geometry callbacks run on the
native callback thread; callback code dispatches to the map owner thread before
calling thread-affine map APIs.

## Rendering and Memory

`RenderSessionHandle` represents one attached render target for one map and
keeps the map wrapper alive. Render-target and borrowed-texture descriptors use
`NativePointer` setters for backend-native handles; callers keep those backend
objects valid and synchronized for the C API's documented lifetime.

Session-owned texture targets expose explicit frame handles. A frame handle
keeps the native frame acquired, exposes copied metadata and scoped backend
resource values, and releases on `close()`. Access after close reports a binding
error. The binding prevents nested acquisition and rejects render updates,
resize, detach, and session destruction while a frame is active. Render-session
resize, detach, memory-reduction, data-clear, and debug-log commands remain
direct owner-thread C calls with status-to-error mapping; `detach()` records
detached state while `close()` still releases the handle.

Temporary native storage lives for one adapter call. Native result, snapshot,
and list handles stay private or sit behind deterministic Vala handle classes;
internal guards copy data into Vala-owned values and release native handles on
every exit path. Offline region snapshots and lists expose `get()`/`to_array()`
copying helpers, then close their native handles exactly once. JSON, feature,
geometry, and GeoJSON builders keep nested descriptor storage alive while the
wrapper materializes raw C descriptors for one call. `JsonValue` owns scalar
values, strings, arrays, and object members. `Feature` owns its geometry, JSON
properties, and identifier. `FeatureCollection` owns Vala feature arrays and
materializes native feature spans per call. `FeatureStateSelector` owns source,
source-layer, feature-id, and state-key strings while the wrapper materializes
the raw field mask privately. Feature-state snapshots copy borrowed native JSON
into Vala-owned `JsonValue` objects before destroying the native snapshot.
Geometry builders cover empty, point, line-string, polygon, multi-point,
multi-line, multi-polygon, and geometry collection descriptors with Vala-owned
coordinate lists, polygons, and child geometries. Rendered/source query options
own layer IDs, source-layer IDs, and JSON filters, and materialize private C
views for one query call. Public query methods copy result-owned feature, source
ID, source-layer ID, and feature-state views into Vala-owned arrays before the
native result handle closes. Feature-extension queries copy borrowed JSON value
or feature-collection payloads into Vala-owned descriptors before the native
extension result closes. Public replacements use Vala values such as strings,
arrays, `GLib.Bytes`, descriptor structs, and handle classes with deterministic
release.

## Testing

Treat successful Vala compilation and execution against the real C library as
the bindability gate. Add Vala tests for ownership transfer, nullability, error
mapping, diagnostic capture, closed handles, wrong-thread statuses, callback
replacement, event copying, query handles, style values, and frame lifetime.
When the C ABI already proves native behavior, the Vala test proves that Vala
wrappers preserve that behavior at the language boundary.

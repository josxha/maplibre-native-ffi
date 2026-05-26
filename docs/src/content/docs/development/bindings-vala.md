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
uses a private raw VAPI for `maplibre_native_c` declarations and public Vala
source for handle classes, descriptors, copied values, errors, callbacks, and
lifetime policy.

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

The direct binding preserves practical coverage by wrapping C ABI concepts, not
by recreating the previous generated GObject surface. Keep compact wrappers for
lifecycle, diagnostics, options, rendering, projection, style values, resource
transforms, offline operation tokens, copied lists, and selected query handles.
Add larger descriptor graphs, custom resource providers, custom geometry
sources, and offline snapshots as direct C wrappers when a Vala caller needs
those surfaces and the wrapper can keep ownership clear. Do not keep
adapter-only APIs, generated compatibility names, GIR metadata, or scanner
helpers as parity surfaces.

The old generated adapter exposed some surfaces that remain deliberate direct-C
follow-up work:

- JSON, GeoJSON, geometry, feature, and feature-collection descriptor graphs
  need Vala-owned builders that keep nested storage alive for one C call.
  Current coverage keeps style JSON strings, GeoJSON URL sources, style values,
  and query handles direct.
- Resource provider request handles and custom geometry callbacks need dedicated
  one-shot Vala handles before they are public. Current coverage includes the
  lower-risk runtime resource transform callback.
- Offline region snapshots, lists, and definitions need direct result-handle
  wrappers. Current coverage includes offline operation tokens for ambient cache
  operations.
- Source-feature queries, feature-state details, and feature-extension payloads
  depend on the JSON and GeoJSON descriptor wrappers. Current coverage includes
  rendered query result handles and counts.
- Vulkan render-target wrappers should mirror the C ABI for Vulkan builds.
  Current tests exercise the Metal path available in the macOS development
  environment.

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

Copied descriptors, events, snapshots, camera values, geometry values, query
results, offline region definitions, and render metadata are Vala-owned values.
Public APIs hide ABI `size` fields, field masks, raw union storage, borrowed
pointers, and backend pointer fields. Constructors and setter methods initialize
those ABI details inside the binding.

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
event, source, payload, render-mode, and tile-operation enums plus copied
message and payload data when the binding supports the payload. Public event
objects remain valid after the next native poll.

Callback adapters keep delegates and user data alive for the native owner scope.
Native upcalls may arrive on worker, network, logging, or render-related
threads, so callback state must be safe for those threads. Vala errors stay
inside callback code and must not cross native frames.

Resource transform callbacks receive copied request URLs and return replacement
URLs that remain scoped to the native callback. Custom geometry callbacks run on
the native callback thread; callback code dispatches to the map owner thread
before calling thread-affine map APIs.

## Rendering and Memory

`RenderSessionHandle` represents one attached render target for one map and
keeps the map wrapper alive. Render-target and borrowed-texture descriptors use
`NativePointer` setters for backend-native handles; callers keep those backend
objects valid and synchronized for the C API's documented lifetime.

Session-owned texture targets expose explicit frame handles. A frame handle
keeps the native frame acquired, exposes copied metadata and scoped
`NativePointer` values, and releases on `close()`. Access after close reports a
binding error. The binding prevents nested acquisition and rejects render
updates, resize, detach, and session destruction while a frame is active.

Temporary native storage lives for one adapter call. Native result, snapshot,
and list handles stay private; internal guards copy data into Vala-owned values
and release native handles on every exit path. Public replacements use Vala
values such as strings, arrays, `GLib.Bytes`, descriptor structs, and handle
classes with deterministic release.

## Testing

Treat successful Vala compilation and execution against the real C library as
the bindability gate. Add Vala tests for ownership transfer, nullability, error
mapping, diagnostic capture, closed handles, wrong-thread statuses, callback
replacement, event copying, query handles, style values, and frame lifetime.
When the C ABI already proves native behavior, the Vala test proves that Vala
wrappers preserve that behavior at the language boundary.

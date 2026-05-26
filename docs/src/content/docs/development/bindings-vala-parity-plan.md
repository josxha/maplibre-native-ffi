---
title: Vala API Parity Plan
description: Plan for completing Vala binding parity with the public C API and other low-level bindings.
---

The Vala binding reaches API parity when it exposes every public MapLibre Native
C API concept through the direct C API, private VAPI, and public Vala wrapper
architecture. Parity means equivalent low-level capability, not generated
GObject compatibility. Each public Vala API owns the language-level safety
contract for memory, callbacks, errors, and handle lifetime.

## Parity Definition

A C API surface is complete in Vala when all of these conditions hold:

1. The private raw VAPI declares the C types and functions with ABI-correct
   signatures.
2. The public Vala wrapper exposes the capability with Vala-owned handles,
   descriptors, values, errors, and callbacks.
3. Raw C pointers, unions, `size` fields, field masks, borrowed storage, and
   result handles stay private unless the C API intentionally exposes an opaque
   backend-native handle through `NativePointer`.
4. The wrapper validates Vala-owned state before crossing into C.
5. Every non-OK C status becomes `MaplibreNative.Error` with the thread-local
   diagnostic copied immediately on the same thread.
6. Tests exercise the Vala/C ABI boundary for ownership, copying, errors,
   diagnostics, callbacks, and representative successful behavior.
7. Documentation names the ownership and threading rules for the exposed
   surface.

## Coverage Matrix

Maintain a coverage matrix while implementing parity. Each row maps one public C
API group to raw VAPI declarations, public Vala wrappers, tests, and notes.

| C API group                                                     | Required Vala parity                                                                                                                                                               |
| --------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `base.h`, diagnostics, logging                                  | Status and error mapping, diagnostic capture, process-global logging callback, render backend flags, native pointer values.                                                        |
| `runtime.h`                                                     | Runtime options, runtime handle lifecycle, event polling, network status, resource transforms, resource providers, offline operation APIs, offline event payloads.                 |
| `map.h`                                                         | Map options, map handle lifecycle, debug/rendering options, style URL/JSON, feature-state selectors, JSON/GeoJSON/geometry/feature descriptors.                                    |
| `camera.h`                                                      | Complete camera options, animation options, bounds/fit options, camera operations, geometry-based camera helpers.                                                                  |
| `render_target.h`, `render_session.h`, `texture.h`, `surface.h` | Metal, Vulkan, and surface render target descriptors, render sessions, resize/detach operations, CPU readback, frame handles, scoped backend-native pointers.                      |
| `projection.h`                                                  | Projection handles, camera snapshots, coordinate conversion, visible coordinate fitting, geometry projection helpers.                                                              |
| `style.h`                                                       | Style sources, source metadata, source attribution copying, tile source options, GeoJSON/image/custom sources, style layers, layer JSON, filters, properties, light, style images. |
| `query.h`                                                       | Rendered/source feature queries, query geometries, query options, feature query results, feature-extension results, copied feature payloads.                                       |

The matrix is complete when every public function, enum, struct, opaque handle,
callback type, and result payload in `include/maplibre_native_c/*.h` has a Vala
row marked implemented and tested.

## Memory and Ownership Rules

### Handles

Every long-lived C-owned opaque handle becomes a public Vala `*Handle` class
with deterministic release:

- The handle stores the raw pointer privately.
- The handle records live or closed state.
- `close() throws MaplibreNative.Error` calls the matching C destroy or release
  function.
- A successful close releases exactly once.
- Later close calls no-op.
- A failed close leaves the handle live so the caller can retry on the owner
  thread.
- Finalizers report leaked handles and do not perform thread-affine cleanup.

Parent retention preserves native validity:

- `MapHandle` keeps its `RuntimeHandle` alive.
- `RenderSessionHandle` keeps its `MapHandle` alive.
- Style-scoped handles keep their owning map or style owner alive.
- Query, snapshot, list, and result handles own their native result until copied
  or closed.
- `MapProjectionHandle` owns a standalone projection snapshot after creation and
  follows the shared exception that it does not retain the source `MapHandle`
  for native validity.

### Descriptors

C option structs and input descriptors become Vala-owned descriptors. Public
callers set semantic fields only. The wrapper initializes ABI bookkeeping:

- `size` fields
- field masks
- string views
- temporary arrays
- nested descriptor pointers
- raw union tags

Most descriptors materialize native structs at the call boundary. Temporary
native storage lives until the C call returns. Storage with an address that must
outlive one call belongs to an explicit Vala handle or callback state object.

### Copied Data

Borrowed C output becomes copied Vala data unless a Vala handle owns the native
result lifetime. The wrapper copies these values before the native borrow window
ends:

- C strings and string views
- runtime event messages and payloads
- style ID lists
- attribution strings
- image pixels and image metadata
- query features and feature-state values
- offline region snapshots and statuses

Native list, snapshot, result, and query handles stay private. Public Vala
objects either own copied values or own a deterministic native result handle
with checked accessors.

## JSON, GeoJSON, Geometry, and Feature Descriptors

Vala parity includes public value builders for the descriptor graphs used by
style, query, feature-state, camera, projection, and GeoJSON APIs.

Required public values:

- `JsonValue`
- `JsonMember`
- `Geometry`
- `FeatureIdentifier`
- `Feature`
- `FeatureCollection`
- `GeoJson`
- `FeatureStateSelector`
- bounds and coordinate span helpers where the C API requires them

Implementation rules:

1. Public values store only Vala-owned data.
2. A private native materialization scope owns all temporary native structs,
   arrays, and string storage for one C call.
3. The scope builds the full nested descriptor graph before calling C.
4. The scope stays alive until the C function returns.
5. Raw C unions and pointer graphs stay private.
6. Invalid nested descriptors produce `MaplibreNative.Error.INVALID_ARGUMENT`
   with the native diagnostic.

This descriptor scope unlocks full parity for inline GeoJSON sources, custom
geometry tile data, style layer JSON, layer filters, feature-state APIs, source
queries, feature extensions, projection helpers, and camera geometry helpers.

## Callback Rules

Callbacks are explicit owner-scoped state. The wrapper stores each Vala callback
strongly for the native lifetime that can invoke it. Native trampolines copy or
scope arguments, invoke Vala code, convert the result to C, and contain Vala
errors inside the callback boundary.

Callback code must assume native invocation from worker, network, logging, or
render-related threads. Thread-affine map and runtime APIs run only on their
owner threads. Higher-level GLib dispatch belongs above the low-level binding.

### Logging Callback

The process-global logging callback remains live until replaced or cleared. The
raw VAPI signature matches the C ABI exactly: `void* user_data` is the first
callback argument, and `mln_log_set_callback()` receives the callback and
`user_data` separately. The public callback receives copied or scoped Vala
values and returns whether the record was consumed.

### Resource Transform Callback

A `RuntimeHandle` owns the resource transform callback. The trampoline copies
the resource URL before invoking Vala code. Replacement URL storage remains
valid until native code consumes it during the transform invocation. Clearing
the transform releases the retained Vala callback and response storage after
native in-flight callbacks finish.

### Resource Provider Callbacks

Resource provider parity uses explicit request handles:

- The provider callback copies request fields before returning to native code.
- Non-matching requests return pass-through immediately and retain no native
  handle.
- Matching requests receive a `ResourceRequestHandle`.
- `ResourceRequestHandle` enforces exactly-once completion or release.
- Completion copies response headers and bytes into native-owned response
  storage according to the C API contract.
- Cancellation and completion paths are safe from every C-permitted callback
  thread.

### Custom Geometry Callbacks

Custom geometry source callbacks are map/style-scoped. The callback state
remains live until the source is removed, the style is replaced, the map closes,
and in-flight callbacks have returned. Callback arguments are copied or exposed
through callback-scoped values. Callback code dispatches to the map owner thread
before calling thread-affine map APIs.

## Query Parity

Rendered and source query parity includes:

- point, box, and line-string rendered query geometries
- rendered query options with layer IDs and JSON filters
- source query options with source-layer IDs and JSON filters
- feature query result count and indexed access
- copied queried feature values, geometry, properties, identifiers, source IDs,
  source layer IDs, and feature-state values
- feature-extension result handles and copied value or feature-collection
  payloads

Result handles release native query results deterministically. Copied feature
values remain valid after the result handle closes.

## Offline Parity

Offline parity includes direct wrappers for region definitions, operations,
status snapshots, result handles, and event payloads.

Required public types:

- offline region IDs
- tile-pyramid and geometry region definitions
- offline region metadata values
- offline operation IDs and operation kinds
- offline region status snapshots
- offline region snapshot values
- offline region list values
- offline operation completion payloads

Native offline results and lists stay private. Public Vala objects contain
copied values or deterministic result handles that copy before close.

## Rendering Backend Parity

Metal, Vulkan, and surface render targets follow the same ownership model:

- backend-native handles use `NativePointer`
- render target descriptors store borrowed backend handles
- callers keep backend objects valid for the C-documented lifetime
- `RenderSessionHandle` owns session lifecycle
- session-owned frames use explicit frame handles
- frame accessors check active state
- frame close releases the native borrow exactly once
- render update, resize, detach, and session close reject active frames

Tests gate backend-specific execution on runtime backend availability.

## Testing Requirements

Vala parity tests focus on the language adaptation layer. The C ABI tests prove
native behavior; Vala tests prove that Vala wrappers preserve ownership,
copying, errors, diagnostics, and callback behavior.

Required test groups:

1. Handle lifecycle, parent retention, close idempotence, and closed-handle
   errors.
2. Status category mapping and immediate diagnostic capture for representative
   invalid arguments, invalid state, unsupported behavior, and wrong-thread
   statuses where the environment can exercise them.
3. Descriptor materialization for options, JSON, GeoJSON, geometry, features,
   filters, and feature-state selectors.
4. Borrowed output copying for events, lists, strings, image pixels, query
   results, and offline snapshots.
5. Callback replacement, clearing, error containment, and owner-scope lifetime
   for logging, resource transforms, resource providers, and custom geometry.
6. One-shot request completion and release for handled resource provider
   requests.
7. Render session frame lifetime, active-frame rejection, readback, Metal,
   Vulkan, and surface descriptors.
8. Query result handles, copied features, source queries, and feature-extension
   payloads.
9. Offline operation start, result events, region snapshots, region status, and
   list copying.

## Completion Criteria

Vala API parity is complete when:

- The coverage matrix maps every public C API declaration to raw VAPI, public
  wrapper, and test coverage.
- Every raw VAPI callback and function signature matches the C header ABI.
- Every public Vala wrapper hides raw ABI bookkeeping and borrowed storage.
- Every native handle, result, list, snapshot, frame, callback, and request has
  deterministic ownership rules.
- Every callback stores Vala state for the full native owner scope and contains
  Vala errors inside the callback boundary.
- Every copied output remains valid after the native borrow or result handle
  ends.
- `mise run //bindings/vala:test`, `mise run fix`, and `mise run test` pass.
- The Vala binding remains direct over the public C API with no Rust GObject,
  GIR, scanner, generated compatibility, or adapter-only surface.

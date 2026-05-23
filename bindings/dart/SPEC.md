# Dart Binding Implementation Map

## Audience and documentation role

Audience: contributors implementing and reviewing the Dart binding. Category:
reference with a short explanatory map. This document names the concrete files,
libraries, tasks, and coverage targets for the binding. The convention documents
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
- [Dart binding conventions](../../docs/src/content/docs/development/bindings-dart.md):
  `dart:ffi` architecture, public type policy, ownership, isolates, callbacks,
  rendering, and tests.

When this spec and a convention document appear to overlap, the convention
contains the rule and this spec names the concrete Dart implementation points.
The public C headers are the ABI source. Existing Rust and Java FFM bindings are
coverage references: Rust shows the full safe low-level model, while Java FFM
shows a direct-C public surface close to Dart's intended concept inventory.
[#179](https://github.com/maplibre/maplibre-native-ffi/pull/179) and
[#180](https://github.com/maplibre/maplibre-native-ffi/pull/180) are scaffold
precedents for spec shape, proof-slice scope, and task integration.

## Scope

`bindings/dart` is the low-level Dart package over the public MapLibre Native C
API. The completed binding uses `dart:ffi` and private `ffigen` declarations
generated from `include/maplibre_native_c.h`. Public APIs expose Dart handle
classes, descriptors, copied values, exceptions, callbacks, typed data, and
opaque native pointers.

The package name is `maplibre_native_ffi`. The public entry library is
`package:maplibre_native_ffi/maplibre_native_ffi.dart`. Generated C declarations
and raw `Pointer` types stay below `lib/src/internal`. Flutter widgets, platform
channels, frame scheduling, and view lifecycle integration belong in adapters
above this package.

## Dart differences and omissions

Record Dart-only differences and current implementation omissions here.

| Item             | Difference or omission                                                                                                                                    | Reason                                                                | User-visible behavior                              | Tests/docs impact                                                                                      |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- | -------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Proof slice only | The current scaffold implements process-global ABI/version/backend/network-status calls and `NativePointer`; the remaining C API areas are planned below. | The scaffold establishes package shape before broad wrapper coverage. | Users can exercise only the proof-slice API today. | Current tests cover the proof slice; future rows in the testing map describe planned binding coverage. |

## Current scaffold

```text
bindings/dart/
  SPEC.md
  analysis_options.yaml
  ffigen.yaml
  mise.toml
  pubspec.yaml
  lib/maplibre_native_ffi.dart
  lib/src/maplibre.dart
  lib/src/<concept>/<concept>.dart
  lib/src/error/maplibre_exception.dart
  lib/src/internal/c/maplibre_native_c.dart
  lib/src/internal/<support>/<support>.dart
  lib/src/internal/loader/native_library.dart
  lib/src/render/native_pointer.dart
  test/maplibre_native_ffi_test.dart
```

The scaffold implements one proof slice:

- `pubspec.yaml` defines a Dart 3.10+ package with `ffi`, `ffigen`, `lints`, and
  `test` dependencies.
- `ffigen.yaml` points at the public umbrella header and records the generated
  private C declaration output path. The generated file is committed and used
  through the curated proof-slice facade.
- `Maplibre.cVersion()` calls `mln_c_version()` through a private C facade.
- `Maplibre.supportedRenderBackends()` preserves backend mask bits in a Dart
  value type.
- `Maplibre.networkStatus()` and `Maplibre.setNetworkStatus()` cross the real C
  ABI and translate native status failures into `MaplibreException` subtypes.
- `NativePointer` establishes borrowed opaque address semantics for render
  backend interop.

## Build artifacts and tasks

| Artifact                 | Path                                          | Contents                                                                                    |
| ------------------------ | --------------------------------------------- | ------------------------------------------------------------------------------------------- |
| Dart package             | `bindings/dart`                               | Public Dart binding, private FFI layer, tests, ffigen config.                               |
| Generated C declarations | `lib/src/internal/c/maplibre_native_c.g.dart` | Private committed `ffigen` output for the public C ABI.                                     |
| Internal C facade        | `lib/src/internal/c`                          | Curated proof-slice calls over generated declarations, status capture, and raw conversions. |
| Public library           | `lib/maplibre_native_ffi.dart`                | Exports handles, values, descriptors, exceptions, callbacks, and backend interop values.    |

Implemented tasks:

| Task                               | Required behavior                                                  |
| ---------------------------------- | ------------------------------------------------------------------ |
| `mise run //bindings/dart:pub-get` | Resolve Dart package dependencies.                                 |
| `mise run //bindings/dart:ffigen`  | Regenerate private C declarations from the public umbrella header. |
| `mise run //bindings/dart:format`  | Format Dart sources and tests.                                     |
| `mise run //bindings/dart:analyze` | Run Dart static analysis.                                          |
| `mise run //bindings/dart:build`   | Run Dart analysis for the scaffold.                                |
| `mise run //bindings/dart:test`    | Build or locate the native C library and run Dart tests.           |
| `mise run //bindings/dart:ci`      | Run the binding CI task set.                                       |

Native library resolution is explicit. Tests use `MLN_FFI_NATIVE_LIBRARY` when
set, otherwise `MLN_FFI_BUILD_DIR` plus the platform library name. Without
either environment variable, the loader falls back to the platform library
search name. Package consumers provide the platform C library according to the
future distribution policy.

## Library responsibilities

### Public library

`lib/maplibre_native_ffi.dart` exports the stable public low-level API:

- process-global entry points in `Maplibre`;
- final `*Handle` classes with explicit `close()` methods;
- Dart value types for copied C data;
- descriptors with semantic fields and internal native materialization;
- `MaplibreException` subclasses and status metadata;
- enum-domain values, bit-mask value types, and unknown-value preservation;
- `NativePointer` as a borrowed opaque address value;
- callback typedefs, resource request handles, and typed-data payloads.

### Internal C layer

`lib/src/internal/c` owns raw C interaction:

- private `ffigen` declarations generated from the public umbrella header;
- a curated facade used by public and support code;
- generated raw calls to imported `mln_*` functions;
- immediate thread-local diagnostic capture after non-OK statuses;
- conversion from C enum values and structs into Dart-friendly raw values;
- native handle state, close-once helpers, and leak-reporting hooks;
- descriptor materializers and copied-result readers;
- C callback trampolines and retained Dart callback boxes.

Raw generated declarations do not appear in public signatures.

### Internal support layer

`lib/src/internal` owns binding mechanics:

- native-library loading;
- status checking and exception construction;
- `calloc`, arena, UTF-8, string-view, and array helpers;
- isolate ownership checks for binding-owned handles;
- callback state and active-upcall accounting;
- scoped native-memory and texture-frame borrow state.

## Dart public API inventory

Create or complete these public source areas. File names may split further when
the split improves locality, but concept names stay stable.

| Dart area                    | Public surface                                                                                                                      |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `maplibre.dart`              | Process-global entry points: ABI version, supported backends, network status, logging configuration, coordinate projection helpers. |
| `error/`                     | `MaplibreException`, stable subclasses, `MaplibreStatus`, raw status, copied diagnostic.                                            |
| `render/native_pointer.dart` | Borrowed opaque backend pointer value.                                                                                              |
| `log/`                       | Log callback, log record, severity, event domain, async severity mask, process-global log callback registration.                    |
| `runtime/`                   | `RuntimeHandle`, `RuntimeOptions`, events, resource provider/transform state, offline operation handles.                            |
| `map/`                       | `MapHandle`, `MapOptions`, map lifecycle, style loading, debug, custom geometry state.                                              |
| `camera/`                    | Camera, animation, bounds, viewport, tile, projection-mode descriptors and map camera operations.                                   |
| `projection/`                | `MapProjectionHandle` and coordinate conversion helpers.                                                                            |
| `geo/`                       | Lat/lng, screen, tile, vector, bounds, quaternion, GeoJSON, geometry, and feature value types.                                      |
| `json/`                      | JSON value model used by style, query, feature state, and descriptors.                                                              |
| `query/`                     | Rendered/source query descriptors, query geometries, queried features, extension results.                                           |
| `render/`                    | `RenderSessionHandle`, render modes, render target descriptors, native buffers, images, texture frames.                             |
| `resource/`                  | Resource request, response, transform, provider decision, one-shot request handle.                                                  |
| `style/`                     | Source, layer, image, light, property, filter, and custom geometry source APIs.                                                     |
| `offline/`                   | Offline region definitions, statuses, metadata, operation handles, and events.                                                      |

## Public type map

| C or shared concept                 | Dart type shape                                                                                                                                                                           |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `mln_runtime*`                      | `final class RuntimeHandle` with `close()`; retains callback state and a map registry.                                                                                                    |
| `mln_map*`                          | `final class MapHandle`; retains `RuntimeHandle`.                                                                                                                                         |
| `mln_map_projection*`               | `final class MapProjectionHandle`; standalone snapshot after creation.                                                                                                                    |
| `mln_render_session*`               | `final class RenderSessionHandle`; retains `MapHandle`.                                                                                                                                   |
| `mln_resource_request_handle*`      | `final class ResourceRequestHandle`; synchronized one-shot completion when C permits cross-thread completion.                                                                             |
| Session-owned texture frames        | Scoped Dart acquired-frame objects, such as `MetalOwnedTextureFrame` and `VulkanOwnedTextureFrame`, that wrap C value structs and release the active frame through `RenderSessionHandle`. |
| C option structs                    | Dart descriptor classes; materializers set `size`, masks, pointers, and nested storage.                                                                                                   |
| C field masks                       | Dart nullable fields, explicit clear methods, or presence wrappers; C masks stay internal.                                                                                                |
| Closed enum domains                 | Dart value types or enums with explicit raw conversion helpers.                                                                                                                           |
| Drift-prone output domains          | Dart value types that preserve unknown raw values.                                                                                                                                        |
| C bit masks                         | Dart value types with raw `bits` and `contains` helpers.                                                                                                                                  |
| Native result/list/snapshot handles | Internal guards that copy into Dart values before release.                                                                                                                                |
| Opaque backend `void*` fields       | `NativePointer`; no ownership or memory access.                                                                                                                                           |
| CPU images and resource bytes       | `Uint8List` or `ByteData` for copied payloads; explicit native buffers where a stable address is required.                                                                                |

## Internal implementation inventory

Implement these internal files under `lib/src/internal`:

| File or area                 | Contents                                                                                |
| ---------------------------- | --------------------------------------------------------------------------------------- |
| `c/maplibre_native_c.g.dart` | Private `ffigen` declarations for public C headers.                                     |
| `c/maplibre_native_c.dart`   | Curated C facade over generated declarations and proof-slice calls.                     |
| `loader/native_library.dart` | Native library path resolution and `DynamicLibrary` opening.                            |
| `status/`                    | Status checking, diagnostic capture, exception construction.                            |
| `lifecycle/`                 | Pointer storage, released state, parent retention, close-once behavior, leak reporting. |
| `memory/`                    | Scoped temporary allocation, UTF-8, string views, arrays, bytes, out pointers.          |
| `struct/`                    | Descriptor materializers and copied-result readers for C structs.                       |
| `callback/`                  | Retained callback boxes, native trampolines, active-upcall accounting, teardown rules.  |
| `events/`                    | Runtime event copying and map-source registry lookup.                                   |
| `frames/`                    | Texture-frame active-state checks and scoped backend pointer conversion.                |

## C API coverage map

Coverage follows the public headers under `include/maplibre_native_c/`.
Generated C declarations may cover more symbols than the public Dart layer
exposes in a milestone; public wrappers advance by coherent concept slices.

### Base and diagnostics

- ABI version and supported backend mask.
- `MaplibreStatus`, `MaplibreException`, and stable exception subclasses.
- Immediate `mln_thread_last_error_message()` copying after non-OK statuses.

### Logging

- Log severity and event domains.
- Process-global log callback registration and clearing.
- Async log severity mask configuration.
- Callback state replacement and release.

### Runtime and resources

- `RuntimeHandle` create, run once, poll events, Dart helper draining by polling
  until empty, and close.
- `RuntimeOptions` materialization.
- Network status get/set.
- Resource transform set/clear and provider callbacks.
- Resource request and response copying.
- Resource request cancellation checks, one-shot completion, and release.

### Offline

- Ambient cache operations.
- Offline region definitions, metadata, status, and info copying.
- Operation start, event completion, take result, and discard flows.
- Region list, merge, create, get, status, observe, download state, invalidate,
  update metadata, and delete operations.

### Map lifecycle and style loading

- `MapHandle` create and close with runtime retention.
- Style URL and JSON loading.
- Repaint and still-image requests.
- Debug logging and load-state helpers.

### Camera and map options

- Map options and default values.
- Camera, free camera, animation, bounds, viewport, tile, and projection mode
  descriptors.
- Jump, ease, fly, move, scale, rotate, pitch, cancel transitions, and camera
  fitting operations.
- Coordinate, pixel, bounds, and projected-meter conversions.

### Projection

- `MapProjectionHandle` create and close.
- Camera, visible coordinates, visible geometry, pixel, and lat/lng projection
  operations.

### Query

- Rendered and source query geometry descriptors.
- Rendered/source feature queries.
- Feature extension queries.
- Query result copying and release.

### Render session

- `RenderSessionHandle` attach result, close, detach, resize, render update,
  reduce memory, clear data, and debug logging.
- Feature state get, set, and remove operations.
- CPU texture readback into `Uint8List`, `ByteData`, or explicit native buffers.

### Surface targets

- Metal surface descriptors and attach.
- Vulkan surface descriptors and attach.
- Borrowed backend handles represented as `NativePointer`.

### Texture targets

- Metal and Vulkan owned texture descriptors and attach.
- Metal and Vulkan borrowed texture descriptors and attach.
- Owned texture frame acquire/release with scoped pointer access through Dart
  APIs documented as unsafe and valid only while the acquired frame is open.
- Texture image metadata and copied premultiplied RGBA8 images.

### Style

- Style source, layer, image, light, and property operations.
- Style source/layer ID lists with copied Dart lists.
- GeoJSON, vector, raster, raster DEM, image, and custom geometry sources.
- Custom geometry source callbacks and tile-data materialization.
- Style image data, image info, and premultiplied RGBA8 copying.

## Example migration target

A future Dart example should mirror the low-level shape of the Zig readback and
Rust map examples:

1. create a runtime;
2. create a map;
3. set a small inline style JSON document;
4. attach an owned texture render target for the host backend;
5. run the runtime until style load and render events arrive;
6. render one update;
7. read premultiplied RGBA8 bytes into a Dart `Uint8List`;
8. close frame, session, map, and runtime handles explicitly.

The example stays binding-focused. Flutter UI, platform views, gestures, and
application lifecycle belong in adapters above this package.

## Testing map

This table describes planned binding coverage. The current scaffold test suite
covers only the proof-slice process globals and `NativePointer` value semantics.

| Area                 | Tests                                                                                                                        |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Process globals      | ABI version, supported backend mask, network status get/set, invalid raw network status.                                     |
| Errors               | Native status-to-exception mapping, diagnostic capture, binding-side validation with null raw status.                        |
| Handles              | Idempotent close after success, failed close leaves handle live, closed-handle validation, parent retention.                 |
| Runtime events       | Polling returns copied events independent of the next native poll; map-originated events identify the live Dart map wrapper. |
| Strings and bytes    | Embedded-NUL rejection, string-view byte lengths, copied C strings, copied typed-data payloads.                              |
| Callbacks            | Strong callback retention, replacement release, exception containment, resource request one-shot completion.                 |
| Rendering            | Descriptor materialization, `NativePointer` values, frame acquisition invalidation, readback buffer sizing and copying.      |
| Offline              | Operation start, completion event matching, take/discard semantics, result handle release.                                   |
| Isolates and threads | Detectable isolate-owner misuse, native wrong-thread propagation, callback threads that avoid thread-affine APIs.            |

Prefer small adaptation tests around real C calls. Raw C validation already
lives in the C ABI tests; Dart tests prove the language boundary preserves that
behavior.

## Implementation milestones

1. Expand the generated-FFI-backed internal C layer beyond the proof slice as
   each public API area is implemented.
2. Complete status, diagnostic, UTF-8, scoped allocation, and handle-state
   support helpers.
3. Implement process globals, logging, runtime create/run/events, map
   create/style, and explicit close semantics.
4. Implement values and descriptors for camera, geometry, JSON, GeoJSON, style,
   query, and resource APIs.
5. Implement render session attachment, texture frames, CPU readback, and
   backend-native pointer descriptors.
6. Implement resource transforms/providers, custom geometry callbacks, and
   offline operation handles.
7. Add examples and generated API reference integration after package artifact
   policy is chosen.

## Completion checklist

- Public names follow Dart conventions and keep `maplibre` as one identifier
  word.
- Generated C declarations, raw pointers, masks, and `size` fields stay out of
  public signatures.
- Every long-lived native object has an explicit `close()` path and close-once
  state.
- Failed native destruction leaves the handle live for retry.
- Thread-affine finalizers report leaks unless cleanup is proven safe on any
  thread and in any finalization order.
- Every non-OK native status copies the thread-local diagnostic immediately.
- Descriptors materialize native storage only for the call scope unless the C
  API documents longer ownership.
- Callback state is retained for the native owner scope and released exactly
  once.
- Callback exceptions are contained and converted to documented C callback
  behavior.
- Runtime events, result handles, strings, and lists copy into Dart-owned values
  before native storage expires.
- Backend pointer accessors that expose scoped `NativePointer` values are named
  and documented with Dart's chosen unsafe convention and state the validity
  scope.
- `NativePointer` remains an opaque borrowed address value with no memory
  access.
- Tests cover Dart-owned lifetime, copying, error, callback, thread, and frame
  invariants through real C calls where practical.

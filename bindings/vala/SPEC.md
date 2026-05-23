# Vala Binding Implementation Map

## Audience and documentation role

Audience: contributors implementing and reviewing the Vala binding. Category:
reference with a short explanatory map. This document names the concrete files,
artifacts, generated outputs, and coverage targets for the binding. The
convention documents remain the source of design rules.

## Normative references

The implementation follows these documents. This spec links to them instead of
restating their rules.

- [Concepts](../../docs/src/content/docs/concepts.md): runtime, map, render
  session, events, and ownership boundaries.
- [C API conventions](../../docs/src/content/docs/development/c-conventions.md):
  status, diagnostics, callbacks, ABI ownership, and thread-affinity contract.
- [Binding conventions](../../docs/src/content/docs/development/bindings.md):
  shared handle, type, callback, rendering, and testing rules.
- [Vala binding conventions](../../docs/src/content/docs/development/bindings-vala.md):
  GLib/GObject architecture, GIR/VAPI generation, `GError`, ownership
  annotations, callbacks, and frame-scope policy.
- [Rust binding conventions](../../docs/src/content/docs/development/bindings-rust.md):
  shared `maplibre-native-core` adaptation layer used by bridge bindings.
- [Java FFM binding conventions](../../docs/src/content/docs/development/bindings-java-ffm.md):
  public concept inventory and low-level naming parity source.

When this spec and a convention document overlap, the convention contains the
rule and this spec names the concrete Vala implementation points. The public C
headers are the ABI source. The Rust and Java bindings are coverage references:
Rust provides the bridge-neutral adaptation layer, and Java FFM provides a broad
public type inventory.

## Scope

`bindings/vala` is the low-level Vala binding over the public MapLibre Native C
API. It exposes a GLib/GObject-shaped API that Vala consumes through GObject
Introspection. The binding is implemented as a Rust shared library over
`maplibre-native-core` and `maplibre-native-sys`; generated annotated C headers
describe that Rust library to `g-ir-scanner`.

The public namespace is `MaplibreNative`. Public GObject classes, boxed values,
functions, callbacks, and errors preserve the C model for runtime, map, render
session, event, resource, query, style, camera, geometry, and render-target
concepts. Higher-level GTK widgets, GLib main-loop dispatch, async frameworks,
and application lifecycle policy belong above this package.

Manual `.vapi` files are out of scope. The binding generates scanner-facing C
headers, GIR, typelib, and VAPI artifacts from metadata and Rust adapter source.
Reviewers treat annotations, generated GIR diffs, generated VAPI diffs, and Vala
compile tests as API review artifacts.

## Current implementation

```text
bindings/vala/
  .gitignore
  SPEC.md
  mise.toml
  metadata/api.toml
  metadata/MaplibreNative-0.1.vapigen.metadata
  crates/maplibre-native-vala/
    Cargo.toml
    include/maplibre-native-vala/maplibre-native-vala.h
    build.rs
    src/events.rs
    src/geo.rs
    src/glib.rs
    src/handles.rs
    src/json.rs
    src/lib.rs
    src/logging.rs
    src/native_pointer.rs
    src/projection.rs
    src/query.rs
    src/render.rs
    src/resource.rs
    src/status.rs
  tools/generate.py
  tests/compile/minimal.vala
```

The Rust adapter implements the low-level GLib/GObject surface:

- `mln_vala_c_version()` calls `mln_c_version()`.
- `mln_vala_supported_render_backends()` calls
  `mln_supported_render_backend_mask()` and exposes typed flags.
- `mln_vala_network_status_get()` and `mln_vala_network_status_set()` cross the
  real C ABI and expose failure through GLib `GError`.
- `RuntimeHandle` and `MapHandle` are registered as GObject classes with
  deterministic close methods, runtime/map option descriptors, ambient cache
  operation controls, map lifecycle/style methods, debug option flags,
  rendering-stats controls, camera fit helpers including geometry fitting,
  camera commands, and map state option descriptors.
- Runtime event polling copies event metadata and message bytes into a boxed
  `RuntimeEvent` value.
- `RenderBackendFlags` and `NetworkStatus` expose typed Vala enum surfaces
  instead of raw integer masks/statuses.
- `NativePointer` is a public boxed value that rejects null addresses and
  exposes only opaque address bits.
- Logging severity/event enums, callback registration with destroy-notify
  replacement, and async severity-mask operations are exposed in generated Vala.
- Resource response descriptors, resource transform/provider callbacks, offline
  operation create/start/status-result/snapshot/list info/discard helpers, and
  `ResourceRequestHandle` one-shot completion/cancellation/release methods are
  visible in generated Vala.
- Basic style-source operations for GeoJSON URL and inline data sources,
  vector/raster/raster DEM tile source URL and inline tile sources, custom
  geometry sources, image sources, source metadata and attribution, source
  lifecycle, runtime style images, style ID list handles, JSON snapshots, DEM
  helper layers, style JSON/property/light/filter operations, basic style-layer
  lifecycle operations, and map coordinate conversion helpers are exposed
  through `MapHandle`.
- Geographic value structs and projected-meter conversion helpers are visible in
  generated Vala.
- `MapProjectionHandle` is registered as a standalone GObject class with
  close-once lifecycle, camera, visible-coordinate/geometry fitting, and
  coordinate conversion methods.
- `RenderSessionHandle` is registered as a GObject class with descriptor
  defaults, render-target attach methods, feature-state JSON snapshot helpers,
  readback helpers, texture frame handles, and basic session lifecycle methods.
- Rendered/source feature query option descriptors, rendered query geometry
  constructors, queried feature views, feature query result handles, and feature
  extension result handles are exposed for Vala bindability.
- `NativePointer` records borrowed opaque-address value semantics for the public
  boxed type.
- `metadata/api.toml` is the generator manifest for namespace, error domain,
  scanner header paths, and the complete public C symbol/type inventory. The
  generator validates the Rust adapter header against that metadata before
  emitting `build/include/maplibre-native-vala.h` for `g-ir-scanner`.

The Rust tests retain a binding-internal `StatusResult` helper for direct status
assertions. Public status-returning entry points use GLib's
`gboolean`/`GError**` convention and expose `throws` in generated Vala.

## Build artifacts and tasks

| Artifact            | Path                                          | Contents                                                                                       |
| ------------------- | --------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Rust adapter crate  | `bindings/vala/crates/maplibre-native-vala`   | Rust shared library, GObject entry points, callback trampolines, GObject class implementation. |
| Binding metadata    | `bindings/vala/metadata`                      | Namespace, ownership, type, callback, and annotation records used by generators.               |
| Annotated C headers | generated under `bindings/vala/build/include` | Scanner-facing declarations for the Rust shared library.                                       |
| GIR                 | generated under `bindings/vala/build/gir`     | GObject Introspection XML for `MaplibreNative`.                                                |
| Typelib             | generated under `bindings/vala/build/typelib` | Runtime introspection data.                                                                    |
| VAPI                | generated under `bindings/vala/build/vapi`    | Vala API generated by `vapigen`.                                                               |
| Vala compile tests  | `bindings/vala/tests/compile`                 | Small source files that prove Vala sees the intended ownership, nullability, and errors.       |

Implemented tasks:

| Task                                     | Required behavior                                                                           |
| ---------------------------------------- | ------------------------------------------------------------------------------------------- |
| `mise run //bindings/vala:build`         | Build the Rust adapter crate after the native C library exists.                             |
| `mise run //bindings/vala:generate`      | Generate annotated C headers, GIR, typelib, and VAPI from metadata and Rust adapter source. |
| `mise run //bindings/vala:compile-tests` | Compile and run Vala fixtures against the generated VAPI and linked Rust adapter.           |
| `mise run //bindings/vala:test`          | Run generation, Vala compile tests, and Rust tests against the real C library.              |
| `mise run //bindings/vala:ci`            | Run tests and clippy for the adapter crate.                                                 |

Future tasks:

| Task                                | Required behavior                                                                        |
| ----------------------------------- | ---------------------------------------------------------------------------------------- |
| `mise run //bindings/vala:api-diff` | Compare generated GIR and VAPI against checked-in review baselines when baselines exist. |

## Module responsibilities

### Rust adapter crate

The `maplibre-native-vala` crate owns the compiled ABI that GLib sees:

- GObject class definitions and finalizers for long-lived handles;
- boxed types for copied values, descriptors, events, render metadata, and
  `NativePointer`;
- GLib error-domain registration and status-to-`GError` conversion;
- C ABI calls through `maplibre-native-core` and `maplibre-native-sys`;
- descriptor materializers, result readers, and temporary native storage;
- callback trampolines, active-upcall accounting, and destroy-notify handling;
- idempotent close/release paths that make repeated calls no-ops after a
  successful release;
- frame-scope state for session-owned texture access.

The crate keeps raw C and Rust implementation details out of generated public
Vala signatures. Public Vala signatures expose GLib types, boxed values, GObject
classes, callbacks, `throws`, arrays with lengths, nullable markers, and
transfer annotations.

### Binding metadata

Metadata drives repetitive ABI generation. Records describe:

- public names and concept grouping;
- GObject class, boxed type, enum, flags, callback, and error-domain shapes;
- native C function mapping and Rust adapter entry symbol;
- ownership transfer, nullability, array length, closure, destroy notify,
  `out`/`inout`, and error annotations;
- parent retention and close semantics for handle classes;
- unsupported C APIs with a recorded reason.

Generated files are products of metadata and adapter source. Fix lost ownership
or safety contracts in metadata or Rust source, not in generated VAPI output.

### Annotated C headers

The generated headers target `g-ir-scanner`. They declare the Rust shared
library's GObject ABI and include introspection annotations. They are not a
separate C implementation.

Header comments and annotations state:

- transfer mode for every returned object, boxed value, list, string, and
  callback-owned object;
- nullable and non-null parameters;
- `out` and `inout` ownership;
- array length fields;
- callback closure and destroy-notify relationships;
- `(throws)` behavior for every `GError**` entry point.

### Generated GIR, typelib, and VAPI

`g-ir-scanner` consumes the annotated headers and compiled Rust shared library.
`vapigen` consumes the GIR. Vala compile tests consume the VAPI.

The generated VAPI is a review artifact, not an edited source file. When Vala
sees the wrong type, ownership, nullability, or exception behavior, update the
metadata, generator, or Rust adapter.

## Public API inventory

Create these public Vala-visible types and functions. Names may split into files
or generated fragments, but concept names stay stable.

### Root namespace

- `MaplibreNative.c_version()`
- `MaplibreNative.supported_render_backends()` returning
  `[Flags] RenderBackendFlags`; raw backend masks stay internal
- `MaplibreNative.network_status`
- process-global logging functions
- process-global coordinate projection helpers

### Error domain

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

Each `GError` message contains the copied native diagnostic captured on the same
thread immediately after the failing C call.

### Handle classes

| C concept                   | GObject/Vala shape                                                                                        |
| --------------------------- | --------------------------------------------------------------------------------------------------------- |
| `mln_runtime*`              | `RuntimeHandle : GLib.Object` with `close() throws Error`; owns callback state and map registry.          |
| `mln_map*`                  | `MapHandle : GLib.Object`; retains `RuntimeHandle`.                                                       |
| `mln_map_projection*`       | `MapProjectionHandle : GLib.Object`; standalone snapshot after creation.                                  |
| `mln_render_session*`       | `RenderSessionHandle : GLib.Object`; retains `MapHandle`.                                                 |
| session-owned texture frame | `MetalOwnedTextureFrameHandle` and `VulkanOwnedTextureFrameHandle`; scoped access and close-once release. |

Thread-affine handle classes expose deterministic `close() throws Error`. A
successful close releases once; later close calls no-op. Failed native
destruction leaves the object live so callers can retry on the owner thread.
`dispose` releases managed parent and callback references only after native
unregistration or `close()` makes callbacks unreachable and in-flight callbacks
complete. `finalize` reports leaks; it does not destroy thread-affine native
handles, and it preserves callback state that native code may still reach from a
leaked handle.

`ResourceRequestHandle` is the request-lifetime exception. It is a `GLib.Object`
that owns the provider's native request reference, enforces one-shot completion,
and releases exactly once. `complete()` and `is_cancelled()` report native
statuses through `throws Error`; `close()` or `release()` is non-throwing
because `mln_resource_request_release()` is a void exactly-once release API. The
handle may complete or release from any thread where the C API permits request
completion.

### Boxed values and descriptors

| Shared concept                      | GLib/Vala shape                                                                                                                                                           |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| C option structs                    | Mutable descriptor objects or boxed values with explicit setters.                                                                                                         |
| C field masks                       | Internal presence state updated by setters and clear methods.                                                                                                             |
| copied coordinates and geometry     | Boxed values or compact structs where GIR/Vala preserves copying and ownership.                                                                                           |
| JSON and GeoJSON                    | GLib-friendly value tree preserving number widths, member order, and duplicate keys.                                                                                      |
| runtime events                      | Copied event boxed values independent of the next native poll.                                                                                                            |
| native result/list/snapshot handles | Internal guards copied into GLib-owned values before release.                                                                                                             |
| backend `void*` handles             | `NativePointer` boxed value wrapping a non-null borrowed address; opaque, no memory access. Nullable pointer positions use nullable metadata, not a null `NativePointer`. |
| CPU images and bytes                | `GLib.Bytes`, `uint8[]`, or explicit native-buffer objects according to ownership.                                                                                        |

### Concept groups

- `camera`: camera, animation, bounds, free-camera, viewport, tile, and
  projection-mode descriptors.
- `geo`: lat/lng, bounds, projected meters, screen points, tile IDs, vectors,
  quaternions, geometry, feature, and GeoJSON values.
- `json`: JSON values and snapshot-copy helpers.
- `log`: log severity, log event, log record, callback registration, and async
  severity mask.
- `map`: map lifecycle, style loading, debug options, repaint, still-image, and
  map-level camera operations.
- `offline`: offline region definitions, snapshots, status, lists, operation
  handles, and operation results.
- `query`: rendered/source query descriptors, query geometries, queried
  features, and feature-extension results.
- `render`: render sessions, render modes, target descriptors, readback, native
  buffers, and texture frame handles.
- `resource`: transform callbacks, provider callbacks, requests, responses,
  decisions, and request handles. Callback replacement installs the new native
  descriptor before closing old state; if installation fails, the replacement
  state closes and the previous callback remains active.
- `runtime`: runtime options, event polling, network status, ambient cache, and
  resource provider state.
- `style`: sources, layers, images, custom geometry sources, light properties,
  layer properties, and filters.

## Internal implementation inventory

Implement these Rust modules as the binding grows:

| Module           | Contents                                                                                                       |
| ---------------- | -------------------------------------------------------------------------------------------------------------- |
| `error`          | GLib error-domain quark, status mapping, diagnostic capture, `GError` helpers.                                 |
| `handle`         | Private native pointer storage, close-once state, parent retention, dispose/finalize ordering, leak reporting. |
| `string`         | UTF-8 conversion, string-view storage, embedded-NUL rejection.                                                 |
| `memory`         | Scoped temporary storage for arrays, strings, out-pointers, and descriptor graphs.                             |
| `boxed`          | Boxed value registration and copy/free hooks.                                                                  |
| `native_pointer` | Non-null borrowed opaque pointer value and nullable metadata helpers.                                          |
| `core_values`    | Coordinates, bounds, screen points, tile IDs, image info, rendering stats.                                     |
| `camera`         | Camera, animation, bounds, viewport, tile, and projection materializers.                                       |
| `runtime`        | Runtime options, event copying, offline operation result copying.                                              |
| `map`            | Map options, map lifecycle, source metadata, and map result readers.                                           |
| `query`          | Query descriptors and copied query result readers.                                                             |
| `render`         | Render target descriptors, native buffers, texture frames, readback helpers.                                   |
| `resource`       | Resource request, response, transform, provider, and one-shot request state.                                   |
| `style`          | Style source, image, layer, light, and custom geometry conversion.                                             |
| `callbacks`      | Closure storage, destroy notify, atomic replacement, active-upcall accounting, panic containment.              |
| `generator`      | Metadata reader, public inventory/annotation verifier, and scanner-facing header emission.                     |

## Naming and packaging

Use `MaplibreNative` as the GIR and Vala namespace. Use `Mln` as the C/GObject
identifier prefix unless generator experiments prove a more idiomatic prefix is
needed. Keep `maplibre` one word inside generated symbols.

Examples:

| Vala name             | C/GObject symbol family                                 |
| --------------------- | ------------------------------------------------------- |
| `RuntimeHandle`       | `MlnRuntimeHandle`, `mln_runtime_handle_*`              |
| `MapHandle`           | `MlnMapHandle`, `mln_map_handle_*`                      |
| `RenderSessionHandle` | `MlnRenderSessionHandle`, `mln_render_session_handle_*` |
| `NativePointer`       | `MlnNativePointer`, `mln_native_pointer_*`              |

The package eventually installs:

- the Rust shared library;
- generated annotated headers needed by downstream scanner workflows when
  applicable;
- `MaplibreNative-0.1.gir`;
- `MaplibreNative-0.1.typelib`;
- `maplibre-native.vapi` and package metadata.

## Type coverage inventory

The generator keeps a type inventory next to the function coverage map. Each
public C type maps to one of the Vala-visible shapes below, or to an internal
implementation-only guard when the C type is a native result, list, snapshot, or
field-mask detail.

| C type group                        | Vala-visible or internal shape                                                                         |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------ |
| `mln_status`                        | `MaplibreNative.Error` plus internal raw-status mapping.                                               |
| closed enum domains                 | Vala enums with explicit raw mapping and unknown-output handling where the C domain may grow.          |
| C bit masks and flags               | Vala `[Flags]` enums or typed flag wrappers; raw masks stay internal.                                  |
| opaque long-lived native handles    | GObject `*Handle` classes with the shared lifetime rules.                                              |
| opaque result/list/snapshot handles | Internal guards that copy to GLib-owned values before release.                                         |
| option and descriptor structs       | Mutable descriptor objects or boxed values with internal size and field-mask materialization.          |
| copied value structs                | Boxed values, compact value types, arrays, or GLib containers according to GIR/Vala ownership support. |
| callbacks                           | Vala delegates with closure and destroy-notify metadata.                                               |

### Public C type checklist

| Header            | C types                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `base.h`          | `mln_status`, `mln_render_backend_flag`, `mln_runtime`, `mln_map`, `mln_map_projection`, `mln_offline_region_snapshot`, `mln_offline_region_list`, `mln_json_snapshot`, `mln_resource_request_handle`, `mln_render_session`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `logging.h`       | `mln_log_severity`, `mln_log_severity_mask`, `mln_log_event`, `mln_log_callback`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| `map.h`           | `mln_camera_option_field`, `mln_animation_option_field`, `mln_camera_fit_option_field`, `mln_bound_option_field`, `mln_free_camera_option_field`, `mln_projection_mode_field`, `mln_map_debug_option`, `mln_north_orientation`, `mln_constrain_mode`, `mln_viewport_mode`, `mln_map_viewport_option_field`, `mln_tile_lod_mode`, `mln_map_tile_option_field`, `mln_map_mode`, `mln_geometry_type`, `mln_json_value_type`, `mln_feature_state_selector_field`, `mln_feature_identifier_type`, `mln_geojson_type`, `mln_map_options`, `mln_screen_point`, `mln_edge_insets`, `mln_camera_options`, `mln_unit_bezier`, `mln_animation_options`, `mln_camera_fit_options`, `mln_vec3`, `mln_quaternion`, `mln_free_camera_options`, `mln_lat_lng`, `mln_string_view`, `mln_json_value`, `mln_geometry`, `mln_coordinate_span`, `mln_polygon_geometry`, `mln_multi_line_geometry`, `mln_multi_polygon_geometry`, `mln_geometry_collection`, `mln_json_array`, `mln_json_member`, `mln_json_object`, `mln_feature_state_selector`, `mln_feature`, `mln_feature_collection`, `mln_geojson`, `mln_lat_lng_bounds`, `mln_bound_options`, `mln_offline_tile_pyramid_region_definition`, `mln_offline_geometry_region_definition`, `mln_offline_region_definition`, `mln_offline_region_info`, `mln_projected_meters`, `mln_projection_mode`, `mln_map_viewport_options`, `mln_map_tile_options` |
| `query.h`         | `mln_rendered_query_geometry_type`, `mln_rendered_feature_query_option_field`, `mln_source_feature_query_option_field`, `mln_queried_feature_field`, `mln_feature_extension_result_type`, `mln_feature_query_result`, `mln_feature_extension_result`, `mln_screen_box`, `mln_screen_line_string`, `mln_rendered_query_geometry`, `mln_rendered_feature_query_options`, `mln_source_feature_query_options`, `mln_queried_feature`, `mln_feature_extension_result_info`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `render_target.h` | `mln_render_target_extent`, `mln_metal_context_descriptor`, `mln_vulkan_context_descriptor`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `runtime.h`       | `mln_network_status`, `mln_runtime_option_flag`, `mln_ambient_cache_operation`, `mln_offline_region_definition_type`, `mln_offline_region_download_state`, `mln_offline_operation_kind`, `mln_offline_operation_result_kind`, `mln_runtime_event_type`, `mln_runtime_event_source_type`, `mln_runtime_event_payload_type`, `mln_render_mode`, `mln_tile_operation`, `mln_resource_kind`, `mln_resource_loading_method`, `mln_resource_priority`, `mln_resource_usage`, `mln_resource_storage_policy`, `mln_resource_response_status`, `mln_resource_error_reason`, `mln_resource_provider_decision`, `mln_offline_region_status`, `mln_runtime_options`, `mln_rendering_stats`, `mln_runtime_event_render_frame`, `mln_runtime_event_render_map`, `mln_runtime_event_style_image_missing`, `mln_tile_id`, `mln_runtime_event_tile_action`, `mln_runtime_event_offline_region_status`, `mln_runtime_event_offline_region_response_error`, `mln_runtime_event_offline_region_tile_count_limit`, `mln_runtime_event_offline_operation_completed`, `mln_runtime_event`, `mln_resource_transform_response`, `mln_resource_transform`, `mln_resource_request`, `mln_resource_response`, `mln_resource_provider`, `mln_resource_transform_callback`, `mln_resource_provider_callback`                                                                                                        |
| `style.h`         | `mln_style_source_type`, `mln_style_tile_source_option_field`, `mln_style_tile_scheme`, `mln_style_vector_tile_encoding`, `mln_style_raster_dem_encoding`, `mln_custom_geometry_source_option_field`, `mln_style_image_option_field`, `mln_location_indicator_image_kind`, `mln_style_id_list`, `mln_style_source_info`, `mln_style_tile_source_options`, `mln_canonical_tile_id`, `mln_custom_geometry_source_options`, `mln_premultiplied_rgba8_image`, `mln_style_image_options`, `mln_style_image_info`, `mln_custom_geometry_source_tile_callback`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `surface.h`       | `mln_metal_surface_descriptor`, `mln_vulkan_surface_descriptor`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `texture.h`       | `mln_metal_owned_texture_descriptor`, `mln_metal_borrowed_texture_descriptor`, `mln_metal_owned_texture_frame`, `mln_vulkan_owned_texture_descriptor`, `mln_vulkan_borrowed_texture_descriptor`, `mln_vulkan_owned_texture_frame`, `mln_texture_image_info`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |

## Coverage map

Every public C function listed here needs a Vala implementation or a recorded
unsupported reason before the binding leaves draft status.

### Base and diagnostics

- `mln_c_version`
- `mln_supported_render_backend_mask`
- `mln_thread_last_error_message` through internal status conversion

### Logging

- `mln_log_set_callback`
- `mln_log_clear_callback`
- `mln_log_set_async_severity_mask`

### Runtime and resources

- `mln_network_status_get`
- `mln_network_status_set`
- `mln_runtime_options_default`
- `mln_runtime_create`
- `mln_runtime_set_resource_provider`
- `mln_resource_request_complete`
- `mln_resource_request_cancelled`
- `mln_resource_request_release`
- `mln_runtime_set_resource_transform`
- `mln_runtime_clear_resource_transform`
- `mln_runtime_run_ambient_cache_operation_start`
- `mln_runtime_offline_operation_discard`
- `mln_runtime_destroy`
- `mln_runtime_run_once`
- `mln_runtime_poll_event`

### Map lifecycle and style document

- `mln_map_options_default`
- `mln_map_create`
- `mln_map_request_repaint`
- `mln_map_request_still_image`
- `mln_map_destroy`
- `mln_map_set_style_url`
- `mln_map_set_style_json`

### Camera, projection, and map state

- `mln_camera_options_default`
- `mln_animation_options_default`
- `mln_camera_fit_options_default`
- `mln_bound_options_default`
- `mln_free_camera_options_default`
- `mln_projection_mode_default`
- `mln_map_viewport_options_default`
- `mln_map_tile_options_default`
- `mln_map_set_debug_options`
- `mln_map_get_debug_options`
- `mln_map_set_rendering_stats_view_enabled`
- `mln_map_get_rendering_stats_view_enabled`
- `mln_map_is_fully_loaded`
- `mln_map_dump_debug_logs`
- `mln_map_get_viewport_options`
- `mln_map_set_viewport_options`
- `mln_map_get_tile_options`
- `mln_map_set_tile_options`
- `mln_map_get_camera`
- `mln_map_jump_to`
- `mln_map_ease_to`
- `mln_map_fly_to`
- `mln_map_move_by`
- `mln_map_move_by_animated`
- `mln_map_scale_by`
- `mln_map_scale_by_animated`
- `mln_map_rotate_by`
- `mln_map_rotate_by_animated`
- `mln_map_pitch_by`
- `mln_map_pitch_by_animated`
- `mln_map_cancel_transitions`
- `mln_map_camera_for_lat_lng_bounds`
- `mln_map_camera_for_lat_lngs`
- `mln_map_camera_for_geometry`
- `mln_map_lat_lng_bounds_for_camera`
- `mln_map_lat_lng_bounds_for_camera_unwrapped`
- `mln_map_get_bounds`
- `mln_map_set_bounds`
- `mln_map_get_free_camera_options`
- `mln_map_set_free_camera_options`
- `mln_map_get_projection_mode`
- `mln_map_set_projection_mode`
- `mln_map_pixel_for_lat_lng`
- `mln_map_lat_lng_for_pixel`
- `mln_map_pixels_for_lat_lngs`
- `mln_map_lat_lngs_for_pixels`
- `mln_map_projection_create`
- `mln_map_projection_destroy`
- `mln_map_projection_get_camera`
- `mln_map_projection_set_camera`
- `mln_map_projection_set_visible_coordinates`
- `mln_map_projection_set_visible_geometry`
- `mln_map_projection_pixel_for_lat_lng`
- `mln_map_projection_lat_lng_for_pixel`
- `mln_projected_meters_for_lat_lng`
- `mln_lat_lng_for_projected_meters`

### Render sessions and targets

- `mln_render_session_resize`
- `mln_render_session_render_update`
- `mln_render_session_detach`
- `mln_render_session_destroy`
- `mln_render_session_reduce_memory_use`
- `mln_render_session_clear_data`
- `mln_render_session_dump_debug_logs`
- `mln_render_session_set_feature_state`
- `mln_render_session_get_feature_state`
- `mln_render_session_remove_feature_state`
- `mln_json_snapshot_get`
- `mln_json_snapshot_destroy`
- `mln_metal_surface_descriptor_default`
- `mln_vulkan_surface_descriptor_default`
- `mln_metal_surface_attach`
- `mln_vulkan_surface_attach`
- `mln_metal_owned_texture_descriptor_default`
- `mln_metal_borrowed_texture_descriptor_default`
- `mln_vulkan_owned_texture_descriptor_default`
- `mln_vulkan_borrowed_texture_descriptor_default`
- `mln_texture_image_info_default`
- `mln_metal_owned_texture_attach`
- `mln_metal_borrowed_texture_attach`
- `mln_vulkan_owned_texture_attach`
- `mln_vulkan_borrowed_texture_attach`
- `mln_texture_read_premultiplied_rgba8`
- `mln_metal_owned_texture_acquire_frame`
- `mln_metal_owned_texture_release_frame`
- `mln_vulkan_owned_texture_acquire_frame`
- `mln_vulkan_owned_texture_release_frame`

### Queries

- `mln_rendered_feature_query_options_default`
- `mln_source_feature_query_options_default`
- `mln_rendered_query_geometry_point`
- `mln_rendered_query_geometry_box`
- `mln_rendered_query_geometry_line_string`
- `mln_render_session_query_rendered_features`
- `mln_render_session_query_source_features`
- `mln_render_session_query_feature_extensions`
- `mln_feature_query_result_count`
- `mln_feature_query_result_get`
- `mln_feature_query_result_destroy`
- `mln_feature_extension_result_get`
- `mln_feature_extension_result_destroy`

### Style

- `mln_style_tile_source_options_default`
- `mln_custom_geometry_source_options_default`
- `mln_premultiplied_rgba8_image_default`
- `mln_style_image_options_default`
- `mln_style_image_info_default`
- `mln_style_id_list_count`
- `mln_style_id_list_get`
- `mln_style_id_list_destroy`
- `mln_map_add_style_source_json`
- `mln_map_remove_style_source`
- `mln_map_style_source_exists`
- `mln_map_get_style_source_type`
- `mln_map_get_style_source_info`
- `mln_map_copy_style_source_attribution`
- `mln_map_list_style_source_ids`
- `mln_map_add_geojson_source_url`
- `mln_map_add_geojson_source_data`
- `mln_map_set_geojson_source_url`
- `mln_map_set_geojson_source_data`
- `mln_map_add_vector_source_url`
- `mln_map_add_vector_source_tiles`
- `mln_map_add_raster_source_url`
- `mln_map_add_raster_source_tiles`
- `mln_map_add_raster_dem_source_url`
- `mln_map_add_raster_dem_source_tiles`
- `mln_map_add_custom_geometry_source`
- `mln_map_set_custom_geometry_source_tile_data`
- `mln_map_invalidate_custom_geometry_source_tile`
- `mln_map_invalidate_custom_geometry_source_region`
- `mln_map_set_style_image`
- `mln_map_remove_style_image`
- `mln_map_style_image_exists`
- `mln_map_get_style_image_info`
- `mln_map_copy_style_image_premultiplied_rgba8`
- `mln_map_add_image_source_url`
- `mln_map_add_image_source_image`
- `mln_map_set_image_source_url`
- `mln_map_set_image_source_image`
- `mln_map_set_image_source_coordinates`
- `mln_map_get_image_source_coordinates`
- `mln_map_add_hillshade_layer`
- `mln_map_add_color_relief_layer`
- `mln_map_add_location_indicator_layer`
- `mln_map_set_location_indicator_location`
- `mln_map_set_location_indicator_bearing`
- `mln_map_set_location_indicator_accuracy_radius`
- `mln_map_set_location_indicator_image_name`
- `mln_map_add_style_layer_json`
- `mln_map_remove_style_layer`
- `mln_map_style_layer_exists`
- `mln_map_get_style_layer_type`
- `mln_map_list_style_layer_ids`
- `mln_map_move_style_layer`
- `mln_map_get_style_layer_json`
- `mln_map_set_style_light_json`
- `mln_map_set_style_light_property`
- `mln_map_get_style_light_property`
- `mln_map_set_layer_property`
- `mln_map_get_layer_property`
- `mln_map_set_layer_filter`
- `mln_map_get_layer_filter`

### Offline

- `mln_runtime_offline_region_create_start`
- `mln_runtime_offline_region_get_start`
- `mln_runtime_offline_regions_list_start`
- `mln_runtime_offline_regions_merge_database_start`
- `mln_runtime_offline_region_update_metadata_start`
- `mln_runtime_offline_region_get_status_start`
- `mln_runtime_offline_region_set_observed_start`
- `mln_runtime_offline_region_set_download_state_start`
- `mln_runtime_offline_region_invalidate_start`
- `mln_runtime_offline_region_delete_start`
- `mln_runtime_offline_region_create_take_result`
- `mln_runtime_offline_region_get_take_result`
- `mln_runtime_offline_regions_list_take_result`
- `mln_runtime_offline_regions_merge_database_take_result`
- `mln_runtime_offline_region_update_metadata_take_result`
- `mln_runtime_offline_region_get_status_take_result`
- `mln_offline_region_snapshot_get`
- `mln_offline_region_snapshot_destroy`
- `mln_offline_region_list_count`
- `mln_offline_region_list_get`
- `mln_offline_region_list_destroy`

## Testing matrix

Use the testing pyramid below as coverage grows:

| Test layer                     | Required coverage                                                                                                                                                                                                   |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rust adapter unit tests        | Status mapping, diagnostic capture, handle state, double-close no-ops, GObject finalizers, callback accounting, frame state.                                                                                        |
| Rust adapter integration tests | Small real C calls for version, network status, runtime/map lifecycle, event copying.                                                                                                                               |
| GIR/VAPI generation tests      | Scanner success, typelib generation, vapigen success, generated diff review.                                                                                                                                        |
| Vala compile tests             | Ownership transfer, nullability, `throws`, boxed value copying, callback signatures.                                                                                                                                |
| Vala runtime tests             | Closed handles, wrong-thread errors, callback replacement, resource request one-shot completion through a real custom style request, event copying, and Metal owned-texture frame lifetime when Metal is available. |

A binding test proves Vala adaptation behavior. If a C ABI test already proves
native validation, the Vala test only needs to prove that the binding propagates
the status, diagnostic, copied output, or ownership contract correctly.

## Draft exit criteria

The Vala binding leaves draft status when:

- `g-ir-scanner`, typelib generation, and `vapigen` run from checked-in tasks;
- the generated VAPI exposes the public inventory and generated type appendix
  with accurate ownership, nullability, arrays, callbacks, flags, and `throws`
  behavior;
- compile tests cover the major GIR/VAPI contracts;
- runtime tests cover deterministic close, diagnostic capture, wrong-thread
  propagation, callback replacement, resource request completion, event copying,
  and texture frame lifetime;
- every C function in the coverage map has an implementation or a recorded
  unsupported reason.

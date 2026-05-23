# Node.js Binding Implementation Map

## Audience and documentation role

Audience: contributors implementing and reviewing the Node.js binding. Category:
reference with a short explanatory map. This document names the concrete files,
package boundaries, tasks, and coverage targets for the binding. The convention
documents remain the source of design rules.

## Normative references

The implementation follows these documents. This spec links to them instead of
restating their rules.

- [Concepts](../../docs/src/content/docs/concepts.md): runtime, map, render
  session, events, and ownership boundaries.
- [C API conventions](../../docs/src/content/docs/development/c-conventions.md):
  status, diagnostics, callbacks, ABI ownership, and thread-affinity contract.
- [Binding conventions](../../docs/src/content/docs/development/bindings.md):
  shared handle, type, callback, rendering, and testing rules.
- [Node.js binding conventions](../../docs/src/content/docs/development/bindings-node.md):
  N-API architecture, TypeScript surface, environment ownership, callbacks, and
  tests.
- [Rust binding conventions](../../docs/src/content/docs/development/bindings-rust.md):
  shared bridge-crate layering and C ABI adaptation responsibilities.
- [Rust bridge binding plan](../rust/PLAN.md): bridge boundary for Python,
  Node.js, and Java JNI.
- [Java FFM binding conventions](../../docs/src/content/docs/development/bindings-java-ffm.md):
  comparison material for cross-binding naming and concept coverage.

When this spec and a convention document overlap, the convention contains the
rule and this spec names the concrete Node implementation points. The public C
headers are the ABI source. `maplibre-native-core` is the bridge-neutral
adaptation source. The Node coverage inventory derives from the public C headers
and the shared and Node-specific binding conventions. Existing bindings such as
Java FFM are comparison material, not normative sources for Node API policy.

## Scope

`bindings/node` is the low-level Node.js binding over the public MapLibre Native
C API. It exposes a TypeScript API backed by one Rust N-API add-on built with
`napi-rs`. The binding targets Node.js 22.14 and newer and uses Node-API v10 as
its ABI baseline.

The Node package stays close to the C API's runtime, map, render session, event,
resource, query, style, offline, and rendering model. It adapts ownership,
diagnostics, callback handoff, copied values, environment ownership, and backend
native handles for JavaScript. Promise-based workflows, Electron integration,
React adapters, UI event-loop policy, and framework scheduling belong above this
package.

The Rust crate is `maplibre-native-node`. It depends on `maplibre-native-core`
for shared C ABI adaptation. It may touch `maplibre-native-sys` directly only
for host-runtime trampoline code or proof slices whose repeated raw sequences
have not moved into `core` yet.

## Node-specific differences and current omissions

Record Node-only differences here. Remove rows as the implementation reaches the
convention-defined behavior.

| Item               | Difference or omission                                                                                    | Reason                                                           | User-visible behavior                                               | Tests/docs impact                                          |
| ------------------ | --------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------- | ---------------------------------------------------------- |
| Direct `sys` calls | `cVersion`, `supportedRenderBackends`, and network status call `maplibre-native-sys` from the Node crate. | Matching helpers are not yet in `maplibre-native-core`.          | The public API remains Node-shaped; raw C details stay inside Rust. | Move repeated sequences into `core` before broad coverage. |
| TypeScript modules | The scaffold uses generated root exports only.                                                            | `napi-rs` proves the add-on before curated concept modules land. | Consumers import the proof-slice functions from the package root.   | Add subpath/module tests when concept modules land.        |

## Current scaffold

```text
bindings/node/
  .gitignore
  SPEC.md
  Cargo.toml
  build.rs
  mise.toml
  package.json
  pnpm-lock.yaml
  tsconfig.json
  index.cjs
  index.d.cts
  src/
    lib.rs
    error.rs
    maplibre.rs
  test/
    maplibre.test.cjs
```

The scaffold implements one proof slice:

- `napi-rs` builds a Rust `cdylib` add-on named `maplibre-native-node`.
- `cVersion()` calls `mln_c_version()` through the linked C ABI.
- `supportedRenderBackends()` returns backend mask bits as a JavaScript object
  while preserving the raw mask.
- `networkStatus()` and `setNetworkStatus(status)` cross the real C ABI and use
  Rust status conversion for native failures.
- `MaplibreError` subclasses expose stable status, raw native status when
  available, and copied diagnostics through the checked-in JavaScript wrapper.
- The Node test uses the package root wrapper after `napi build` and verifies
  the process-global proof slice.

## Build artifacts and tasks

| Artifact               | Path                        | Contents                                                                                      |
| ---------------------- | --------------------------- | --------------------------------------------------------------------------------------------- |
| Node package           | `bindings/node`             | Package metadata, generated JavaScript entry point, generated TypeScript declarations, tests. |
| N-API Rust crate       | `bindings/node`             | Rust add-on crate using `napi-rs`, `maplibre-native-core`, and narrow `sys` access.           |
| Native add-on binary   | `bindings/node/*.node`      | Platform-specific Node-API shared library generated by `napi build`.                          |
| Public wrapper         | `bindings/node/index.cjs`   | Checked-in CommonJS API wrapper with error classes and stable package exports.                |
| Public declarations    | `bindings/node/index.d.cts` | Checked-in TypeScript declarations for the public wrapper.                                    |
| Generated declarations | `bindings/node/index.d.ts`  | TypeScript declarations generated from `#[napi]` exports and kept out of git.                 |
| Generated loader       | `bindings/node/index.js`    | JavaScript native loader generated by `napi-rs` and kept out of git.                          |

Implemented tasks:

| Task                             | Required behavior                                                                                            |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `mise run //bindings/node:build` | Ensure the native C library exists, install Node dependencies from the lockfile, and build the N-API add-on. |
| `mise run //bindings/node:test`  | Build the add-on and run Node tests against the real native library.                                         |
| `mise run //bindings/node:ci`    | Run tests and TypeScript checking.                                                                           |
| `pnpm build`                     | Run `napi build --platform --manifest-path Cargo.toml`.                                                      |
| `pnpm test`                      | Run `node --test test/*.test.cjs` after the add-on exists.                                                   |

The add-on links `maplibre-native-c` through `maplibre-native-sys`, which uses
`MLN_FFI_BUILD_DIR` and the repository native-library search policy.

## Planned package and module map

The current package root exports the proof-slice process-global functions and
error classes. As coverage grows, the package root will re-export concept
modules. Planned public TypeScript modules group C API concepts:

```text
@maplibre/native-ffi-node
@maplibre/native-ffi-node/camera
@maplibre/native-ffi-node/error
@maplibre/native-ffi-node/geo
@maplibre/native-ffi-node/json
@maplibre/native-ffi-node/log
@maplibre/native-ffi-node/map
@maplibre/native-ffi-node/offline
@maplibre/native-ffi-node/query
@maplibre/native-ffi-node/render
@maplibre/native-ffi-node/resource
@maplibre/native-ffi-node/runtime
@maplibre/native-ffi-node/style
```

Current internal Rust modules:

```text
index.cjs          public JavaScript wrapper, error classes, NativePointer, and NativeBuffer values
index.d.cts        public TypeScript declarations
src/error.rs       native error payload conversion for the wrapper
src/maplibre.rs   process-global proof slice, async log severity controls, and root exports
src/runtime.rs    runtime handle, runtime option materialization, and event polling proof slice
src/map.rs        map handle, map option materialization, style-loading, camera, repaint, debug-option, and utility proof slices
src/values.rs     copied coordinate values and projection helper proof slice
```

Planned internal Rust modules own implementation roles:

```text
src/render.rs     render sessions, targets, readback, and frame scopes
src/resource.rs   resource request/response conversion and one-shot handles
src/style.rs      style source, layer, image, and custom geometry conversion
src/query.rs      feature query descriptors and copied query results
src/values.rs     JSON, GeoJSON, geometry, camera, and remaining copied value conversion
src/callback.rs   `ThreadsafeFunction` state and callback lifetime helpers
src/env.rs        N-API environment ownership and cleanup hooks
src/handle.rs     close-once native handle state and parent retention
```

Add an internal module only when its name identifies a concrete role.

## Public API inventory

Implement public TypeScript names that cover the public C API concepts while
using JavaScript naming, string unions, object shapes, `bigint`, typed arrays,
`Symbol.dispose`, and synchronous methods where those choices keep the low-level
contract intact.

### Root

- `cVersion`
- `supportedRenderBackends`
- `networkStatus`
- `setNetworkStatus`
- `setLogCallback`
- `clearLogCallback`
- `setAsyncLogSeverities`
- `restoreDefaultAsyncLogSeverities`
- `projectedMetersForLatLng`
- `latLngForProjectedMeters`

### `error`

- `MaplibreError`
- `InvalidArgumentError`
- `InvalidStateError`
- `WrongThreadError`
- `UnsupportedFeatureError`
- `NativeError`
- `MaplibreStatus`

### Handles and render interop

- `RuntimeHandle`
- `MapHandle`
- `MapProjectionHandle`
- `RenderSessionHandle`
- `OfflineOperationHandle`
- `ResourceRequestHandle`
- `NativePointer`
- `NativeBuffer`

### Scoped frame values

- `MetalOwnedTextureFrame`
- `VulkanOwnedTextureFrame`

Session-owned texture frame access uses synchronous callback scopes. The public
frame values are scoped views, not explicitly releasable `*Handle` objects.

### Callbacks

- `LogCallback`
- `ResourceProviderCallback`
- `ResourceTransformCallback`
- `CustomGeometrySourceCallback`

### Values, descriptors, and events

Use the public C API and convention documents as the coverage source for camera,
geo, JSON, log, map, offline, query, render, resource, runtime, and style
values. Existing bindings can help compare names and coverage. Closed C enum
domains become string-literal unions or frozen objects with explicit native
mappings. Drift-prone output domains preserve unknown raw values in object
shapes. C bit masks become arrays or purpose-built mask objects. C field masks
stay internal.

## Internal implementation inventory

Implement these Rust support areas:

| Area       | Purpose                                                                                                             |
| ---------- | ------------------------------------------------------------------------------------------------------------------- |
| `error`    | Convert `maplibre-native-core::Error` into `MaplibreError` JavaScript subclasses with copied diagnostics.           |
| `env`      | Record the creating N-API environment, reject cross-environment handle use, and register environment cleanup hooks. |
| `handle`   | Store native pointer, closed state, parent references, and leak context; implement close-once behavior.             |
| `values`   | Convert copied values, enums, JSON, GeoJSON, camera descriptors, and opaque pointers.                               |
| `memory`   | Materialize temporary strings, arrays, byte views, out-pointers, and descriptor graphs.                             |
| `callback` | Own `ThreadsafeFunction` bridges, queue policy, active-upcall accounting, and exact-once callback state release.    |
| `runtime`  | Runtime creation, pumping, event polling, resource transform/provider state, and offline operations.                |
| `map`      | Map creation, style loading, camera operations, custom geometry source state, and map child handles.                |
| `render`   | Render descriptors, session lifecycle, readback, native buffers, and texture frame scopes.                          |
| `query`    | Query descriptors, rendered/source query results, and feature-extension results.                                    |
| `style`    | Style sources, images, layers, properties, filters, and custom geometry data.                                       |

Keep bridge-neutral C `size` initialization, field masks, descriptor graphs,
result-handle copying, status conversion, diagnostic capture, and repeated raw C
sequences in `maplibre-native-core`. Keep Node-local code focused on N-API
classes, environment ownership, JavaScript errors, generated TypeScript,
callback queues, and other host-runtime policy. Direct `sys` calls in the Node
crate are limited to host-runtime trampoline code and temporary proof slices.

## C API coverage map

Every public C function listed here needs a Node implementation or a recorded
unsupported reason before the binding leaves draft status. Reviewers compare
this list with `include/maplibre_native_c/*.h` during coverage reviews.

### Base and diagnostics

- `mln_c_version`
- `mln_supported_render_backend_mask`
- `mln_thread_last_error_message`

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

### Map lifecycle, style loading, and camera

- `mln_map_options_default`
- `mln_map_create`
- `mln_map_request_repaint`
- `mln_map_request_still_image`
- `mln_map_destroy`
- `mln_map_set_style_url`
- `mln_map_set_style_json`
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

### Projection

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

### Query

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

### Render sessions, surfaces, and textures

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

## Testing map

The full Node test suite should exercise the public TypeScript/JavaScript
surface against the real native add-on. It should focus on Node-owned behavior:

- status-to-`MaplibreError` conversion and immediate diagnostic copying;
- explicit `close()` idempotence, `Symbol.dispose`, and failed-close retry;
- wrong-thread and wrong-environment errors with Node Workers;
- parent retention while child handles are live;
- embedded-NUL rejection and scoped UTF-8 storage;
- callback handoff through `ThreadsafeFunction` queues;
- one-shot resource request completion and exact-once release;
- copied runtime events and query/style result payloads;
- frame-scope active-state invalidation;
- render readback into `Uint8Array`, `ArrayBuffer`, or `NativeBuffer` storage.

Initial tests cover the proof slice. Add focused regression tests with each API
area instead of retesting all native C validation rules.

## Implementation milestones

1. Keep the proof slice green: `napi-rs` build, ABI version, supported render
   backends, network status, and Node test.
2. Add `MaplibreError` subclasses, status conversion, raw status, and copied
   diagnostic properties. _(Initial proof-slice wrapper complete.)_
3. Add environment ownership, cleanup hooks, close-once handle state, leak
   reporting, `close()`, and `Symbol.dispose`. _(Initial `RuntimeHandle`
   close-once proof slice and `NativePointer` value complete.)_
4. Add `RuntimeHandle`, runtime options, runtime pumping, and copied event
   polling. _(Initial event envelope polling proof slice complete.)_
5. Add `MapHandle`, map options, style loading, map-owned callbacks, and parent
   retention. _(Initial lifecycle, options, style-loading, camera descriptor,
   map utility methods, debug-option string mapping, and parent-retention proof
   slice complete.)_
6. Add copied values, descriptors, enum conversions, JSON, geometry, GeoJSON,
   and TypeScript concept modules. _(Initial coordinate value and projection
   helper proof slice complete.)_
7. Add logging, resource transforms, resource providers, and one-shot resource
   request completion through `ThreadsafeFunction` handoff. _(Async log severity
   control proof slice complete; callback handoff remains.)_
8. Add camera, projection, query, style, and offline APIs. _(Initial map camera
   descriptor proof slice complete.)_
9. Add render sessions, Metal/Vulkan descriptors, texture readback,
   `NativeBuffer`, and texture frame scopes. _(`NativeBuffer` value complete.)_
10. Move repeated direct `sys` sequences and bridge-neutral descriptor/result
    adaptation into `maplibre-native-core` as broad coverage replaces the proof
    slice.
11. Decide package publication and native-binary distribution policy.
12. Mark all coverage items complete before changing the binding from draft to
    ready for review.

## Completion checklist

- [ ] The public TypeScript API exposes no raw C symbols, raw C structs, field
      masks, or callback trampolines.
- [ ] Every long-lived native object has a `*Handle` class with `close()`,
      `Symbol.dispose`, and documented owner-thread behavior.
- [ ] Every handle records its creating N-API environment and rejects use from
      another environment.
- [ ] Every C function in the coverage map has a Node implementation or a
      recorded unsupported reason.
- [ ] Native failures throw `MaplibreError` subclasses with stable kind, raw
      status, and copied diagnostic.
- [ ] Bridge-neutral descriptor materializers in `maplibre-native-core` own C
      `size` fields, field masks, temporary strings, arrays, and nested backing
      storage.
- [ ] Callback state releases exactly once after the C owner scope and active
      upcalls finish.
- [ ] Resource provider requests enforce one-shot completion and exact-once
      native request release.
- [ ] Session-owned texture frame values reject use after frame scope close.
- [ ] Node Worker tests cover wrong-environment and wrong-thread behavior.
- [ ] `mise run //bindings/node:build` passes.
- [ ] `mise run //bindings/node:test` passes.
- [ ] `mise run //bindings/node:ci` passes.

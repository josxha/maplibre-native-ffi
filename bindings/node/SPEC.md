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
for shared C ABI adaptation. The Node crate may call `maplibre-native-sys`
directly for the final synchronous C API invocation, for host-runtime
trampolines, and for Node-owned render/platform-handle descriptors. Reusable
bridge-neutral work such as status conversion, string storage, copied result
handles, JSON/GeoJSON conversion, and shared descriptor materialization lives in
`maplibre-native-core`.

## Node-specific differences

Record Node-only differences here. Remove rows as the implementation reaches the
convention-defined behavior.

| Item | Difference or omission                        | Reason                                            | User-visible behavior                               | Tests/docs impact                                   |
| ---- | --------------------------------------------- | ------------------------------------------------- | --------------------------------------------------- | --------------------------------------------------- |
| None | No in-scope Node-only omissions are recorded. | Current coverage matches this implementation map. | Consumers use the root package or concept subpaths. | Focused Node-layer tests cover the wrapper surface. |

## Current files

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
  {camera,error,geo,json,log,map,offline,query,render,resource,runtime,style}.cjs
  {camera,error,geo,json,log,map,offline,query,render,resource,runtime,style}.d.cts
  src/
    lib.rs
    error.rs
    maplibre.rs
    map.rs
    projection.rs
    render.rs
    runtime.rs
    values.rs
  test/
    maplibre.test.cjs
```

The package implements the low-level Node binding surface named in this file:

- `napi-rs` builds a Rust `cdylib` add-on named `maplibre-native-node`.
- Root and concept subpath modules expose JavaScript-friendly names and
  checked-in TypeScript declarations.
- Process-global, runtime, map, style, query, render, resource, projection,
  logging, and offline APIs cross the real C ABI through Node-shaped wrappers.
- `MaplibreError` subclasses expose stable status, raw native status when
  available, and copied diagnostics through the checked-in JavaScript wrapper.
- The Node tests use the package wrapper after `napi build` and focus on
  Node-layer adaptation behavior.

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

## Package and module map

The package root exports the complete low-level wrapper. Concept subpaths expose
curated CommonJS modules with matching TypeScript declarations:

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
index.cjs          public JavaScript wrapper, error classes, environment-token handle guards, NativePointer, and NativeBuffer values
index.d.cts        public TypeScript declarations
src/error.rs       native error payload conversion for the wrapper
src/maplibre.rs   process-global APIs, thread-local diagnostics, log callback bridge, async log severity controls, and root exports
src/runtime.rs    runtime handle, runtime option materialization, event polling, resource provider/transform, ambient cache, and offline region operation start/take-result APIs
src/map.rs        map handle, map/viewport/tile/projection/bounds/free-camera option materialization, style-loading/probes, URL/tile/custom-geometry source helpers and callbacks, style/image source values, style ID lists, style metadata/layer/light/location/terrain JSON/properties, camera/animation commands, repaint, debug-option, and utility APIs
src/projection.rs standalone map projection handle APIs
src/render.rs     render session handle with JavaScript parent retention, Metal/Vulkan descriptor, feature-state, feature-query, and texture frame-scope APIs
src/values.rs     copied coordinate and screen point values plus projection helper APIs
```

Future internal module splits are maintenance refactors, not draft-completion
requirements. Add an internal module only when its name identifies a concrete
role.

## Public API inventory

Implement public TypeScript names that cover the public C API concepts while
using JavaScript naming, string unions, object shapes, `bigint`, typed arrays,
`Symbol.dispose`, and synchronous methods where those choices keep the low-level
contract intact.

### Root

- `cVersion`
- `supportedRenderBackends`
- `threadLastErrorMessage`
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
callback queues, and other host-runtime policy. The Node crate may perform the
final synchronous C API call directly when `core` already owns the reusable
bridge-neutral conversion around that call.

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

Tests cover focused Node-layer adaptation behavior. Add focused regression tests
with each API area instead of retesting all native C validation rules.
Resource-provider tests cover registration and JavaScript handle guard behavior;
the native request lifecycle remains enforced by the one-shot registry in
`src/runtime.rs` and covered at the C ABI layer because the Node suite does not
have a deterministic network-request trigger that avoids broader integration
policy.

## Implementation milestones

1. Keep the add-on build green: `napi-rs` build, ABI version, supported render
   backends, network status, and Node test. _(Complete.)_
2. Add `MaplibreError` subclasses, status conversion, raw status, and copied
   diagnostic properties. _(Complete.)_
3. Add environment ownership, cleanup hooks, close-once handle state, leak
   reporting, `close()`, and `Symbol.dispose`. _(`RuntimeHandle` close-once,
   JavaScript parent-retention and environment-token checks, and `NativePointer`
   value complete.)_
4. Add `RuntimeHandle`, runtime options, runtime pumping, and copied event
   polling. _(Event envelope polling, ambient cache operation, and offline
   region operation start/take-result APIs complete.)_
5. Add `MapHandle`, map options, style loading, map-owned callbacks, and parent
   retention. _(Lifecycle, map/viewport/tile/projection/bounds options,
   style-loading/probes, camera descriptor, map utility methods, debug-option
   string mapping, and parent-retention APIs complete.)_
6. Add copied values, descriptors, enum conversions, JSON, geometry, GeoJSON,
   and TypeScript concept modules. _(Coordinate values, projection helpers,
   JavaScript-to-native JSON conversion, and concept subpath modules complete.)_
7. Add logging, resource transforms, resource providers, and one-shot resource
   request completion through `ThreadsafeFunction` handoff. _(Log callback,
   async log severity control, resource transform, and resource provider
   callback handoff complete.)_
8. Add camera, projection, query, style, and offline APIs. _(Map camera
   descriptor, camera fitting/movement/animation/free-camera commands,
   standalone projection handle, visible-coordinate/geometry and screen
   projection helpers, geometry camera fitting,
   URL/tile/custom-geometry/GeoJSON-data/style-image/inline-image-source values,
   custom-geometry callback handoff, style
   JSON/list/metadata/layer/light/location/terrain/property helpers, and style
   probe APIs complete.)_
9. Add render sessions, Metal/Vulkan descriptors, texture readback,
   `NativeBuffer`, and texture frame scopes. _(`NativeBuffer`, render
   session/Metal and Vulkan descriptors, feature-state, feature-query,
   feature-extension, and texture frame-scope APIs complete.)_
10. Keep repeated descriptor/result adaptation in `maplibre-native-core`; direct
    Node `sys` calls remain acceptable for final C API invocations that use
    those shared materializers.
11. Package publication and native-binary distribution policy remain outside
    this implementation map.
12. Mark all coverage items complete before changing the binding from draft to
    ready for review. _(Complete.)_

## Completion checklist

- [x] The public TypeScript API exposes no raw C symbols, raw C structs, field
      masks, or callback trampolines.
- [x] Every long-lived native object has a `*Handle` class with `close()`,
      `Symbol.dispose`, and documented owner-thread behavior.
- [x] Every handle records its creating N-API environment and rejects use from
      another environment.
- [x] Every C function in the coverage map has a Node implementation or a
      recorded unsupported reason.
- [x] Native failures throw `MaplibreError` subclasses with stable kind, raw
      status, and copied diagnostic.
- [x] Bridge-neutral descriptor materializers in `maplibre-native-core` own C
      `size` fields, field masks, temporary strings, arrays, and nested backing
      storage for shared descriptors; host-native render descriptors stay in the
      Node layer because they adapt JavaScript-owned platform handles.
- [x] Callback state releases exactly once after the C owner scope and active
      upcalls finish.
- [x] Resource provider requests enforce one-shot completion and exact-once
      native request release.
- [x] Session-owned texture frame values reject use after frame scope close.
- [x] Node Worker tests cover wrong-environment and wrong-thread behavior.
      _(Node handles are non-cloneable across Workers; environment-token guards
      reject detached public handle use, and native wrong-thread status remains
      covered at the C ABI layer.)_
- [x] `mise run //bindings/node:build` passes.
- [x] `mise run //bindings/node:test` passes.
- [x] `mise run //bindings/node:ci` passes.

# Vala binding review-loop findings

This file records findings from iterative review loops on the Vala binding PR.
It keeps the review artifacts actionable without committing the ignored
`review-loop/` subagent output directory.

## Round 1

Review artifacts:

- `review-loop/round1-api-spec.md`
- `review-loop/round1-runtime-lifecycle.md`
- `review-loop/round1-build-generation-tests.md`
- `review-loop/round1-maintainability-docs.md`

### Applied findings

- Fixed caller-allocated buffer VAPI generation for style copy, image-source
  coordinate copy, coordinate projection arrays, and render readback buffers by
  marking those parameters as caller-provided arrays in vapigen metadata.
- Added Vala fixture coverage that passes real preallocated buffers for style
  attribution, style-image pixel copy, image-source coordinates, and coordinate
  projection helpers.
- Removed unconditional Metal link flags and direct Metal symbol references from
  the Vala compile fixture. The fixture now probes Metal through `GModule`, so
  non-Darwin CI can build without `-framework Metal`.
- Added the native C build directory to scanner and compile-test library search
  paths/rpaths.
- Protected Vala resource-provider fixture counters with a mutex and checked the
  Metal render-update wait result before rendering/acquiring a frame.
- Made thread-affine GObject finalizers destroy native state only on the
  recorded owner thread; off-owner finalization reports the leak and preserves
  native state instead of discarding wrong-thread cleanup errors.
- Unref resource-provider request wrappers after every callback decision, while
  disarming pass-through decisions so handled requests no longer leak the
  temporary GObject wrapper.
- Investigated resource-transform URL retention. The attempted per-callback
  cleanup was rejected in Round 2 because transform callbacks may overlap across
  native worker/network threads; the adapter retains returned URLs until
  transform clear/replacement/runtime close to preserve callback-returned
  pointers safely.
- Narrowed Vala generator/convention documentation to match current coarse
  annotation validation, and aligned SPEC notes for the current source tree and
  generated `NetworkStatus.get()`/`set()` API shape.

### Rejected or deferred findings

- JSON/geometry builders and full GLib-friendly value-tree construction remain a
  larger API-design slice. The current PR exposes low-level C value shapes and
  bindability; adding ergonomic builders would expand feature scope beyond a
  review-loop fix.
- Replacing every backend-native raw descriptor field with `NativePointer`
  setters/wrappers is deferred as an API-shape redesign. Frame accessors already
  return `NativePointer`; descriptor construction still mirrors the low-level C
  structs until a dedicated descriptor wrapper pass.
- Hiding all descriptor `size` and field-mask bookkeeping behind object setters
  is deferred for the same reason: it is a broad public API redesign rather than
  a focused corrective patch.
- Custom geometry source callback closure ownership is deferred. The current API
  remains a low-level function-pointer surface; a captured Vala delegate API
  would need owner-scoped callback storage and lifecycle tests.
- Typed runtime-event enum/payload wrappers are deferred. The current boxed
  event copies common metadata; exposing all typed payloads is a separate
  event-model expansion.
- Struct-field byte buffer wrappers are deferred. Caller-allocated buffer
  parameters now generate and compile as arrays, but transparent C structs such
  as `ResourceResponse`, `ResourceRequest`, `OfflineRegionInfo`, and
  `PremultipliedRgba8Image` still expose low-level pointer/length fields; a
  GLib.Bytes/native-buffer wrapper pass would be a broader API-shape change.
- Callback panic/exception containment was documented too strongly, so the docs
  now describe the actual fallback/error boundary. Catching language-level Vala
  exceptions across C callback frames is not a safe small patch.
- The `MlnVala`/`mln_vala` prefix is kept for this PR to avoid whole-ABI churn,
  and the SPEC now documents it as the current prefix.

### User-input-needed findings

- None requiring immediate user input for this round. Deferred API-shape items
  above should be revisited only if maintainers want to expand the PR scope
  before merge.

## Round 2

Review artifacts:

- `review-loop/round2-api-spec.md`
- `review-loop/round2-runtime-lifecycle.md`
- `review-loop/round2-build-generation-tests.md`
- `review-loop/round2-maintainability-docs.md`

### Applied findings

- Recorded the remaining struct-field byte-buffer gap as deferred, with affected
  low-level structs and scope rationale.
- Applied the owner-thread finalizer policy to `MapProjectionHandle` and aligned
  SPEC finalizer wording with the implementation.
- Kept transform replacement URLs retained until transform state teardown after
  rejecting an attempted per-callback cleanup that could race overlapping native
  transform callbacks.
- Replaced hardcoded Pixi `default` scanner paths with `$CONDA_PREFIX` paths.
- Added Vala readback compile/runtime coverage for
  `RenderSessionHandle.read_premultiplied_rgba8` in the Metal owned-texture
  fixture.
- Aligned the SPEC with the current `MlnVala` / `mln_vala_` prefix and narrowed
  shared callback-failure wording to language-safe boundaries.

### Rejected or deferred findings

- Per-callback transform URL cleanup is rejected for now because transform
  callbacks may overlap on worker/network threads. Retain-until-clear is a safe
  conservative policy; bounded per-invocation retention can be a future
  optimization.
- Linux CI validation remains deferred to the configured runner; local macOS
  validation covers generation, VAPI shape, compile fixture execution, and Rust
  tests.

### User-input-needed findings

- None.

## Round 3

Review artifacts:

- `review-loop/round3-api-spec.md`
- `review-loop/round3-runtime-lifecycle.md`
- `review-loop/round3-build-generation-tests.md`
- `review-loop/round3-maintainability-docs.md`

### Applied findings

- Moved Metal readback before frame acquisition and assert readback metadata so
  the fixture does not call readback while a texture frame is held.
- Protected Vala logging fixture counters with a mutex, matching provider-state
  synchronization.
- Added a Rust adapter regression test for non-null resource-transform
  replacement URL retention through repeated trampoline calls.
- Added projection owner-thread finalizer helper coverage for owner-thread and
  off-owner-thread checks.
- Updated the Vala conventions page to document the owner-thread finalizer
  exception and preserve/report behavior.
- Added this Round 2/Round 3 provenance so the committed log reflects iterative
  review artifacts and outcomes.

### Rejected or deferred findings

- First-class async Vala provider retention ergonomics remain deferred as a
  broader API-shape follow-up; the current low-level callback parameter is
  callback-duration borrowed.

### User-input-needed findings

- None.

## Round 4

Review artifacts:

- `review-loop/round4-api-spec.md`
- `review-loop/round4-runtime-lifecycle.md`
- `review-loop/round4-build-generation-tests.md`
- `review-loop/round4-maintainability-docs.md`

### Applied findings

- Fixed the `poll_event` GIR annotation so VAPI marks a no-event result as a
  nullable `RuntimeEvent?` rather than a non-null out value.
- Left native resource-provider request ownership with the C provider path when
  the temporary GObject wrapper cannot be allocated, avoiding double release on
  allocation failure.
- Narrowed SPEC and Vala convention wording so deferred descriptor, JSON, native
  pointer, and struct-buffer ergonomics are documented as future wrapper work
  rather than current API behavior.
- Added `mln_offline_region_id` and `mln_offline_operation_id` to the SPEC
  checklist with their current scalar Vala mappings.
- Added direct owner-thread finalizer coverage for an unclosed projection
  handle.

### Rejected or deferred findings

- Linux CI validation remains deferred to the configured runner; local macOS
  validation covers generation, VAPI shape, compile fixture execution, Rust
  tests, and clippy.

### User-input-needed findings

- None.

## Round 5

Review artifacts:

- `review-loop/round5-api-spec.md`
- `review-loop/round5-runtime-lifecycle.md`
- `review-loop/round5-build-generation-tests.md`
- `review-loop/round5-maintainability-docs.md`

### Applied findings

- Added Round 4 provenance to this committed finding log.
- Renamed single-value flag enum members in vapigen metadata to idiomatic Vala
  names and referenced them from the compile fixture.
- Moved the offline get-status result documentation block next to its actual
  declaration.
- Tightened the generator's annotation parser so it associates only immediately
  adjacent documentation blocks with declarations, then corrected stale
  annotation inventory entries exposed by that stricter check.

### Rejected or deferred findings

- No new deferrals beyond the API-shape and Linux CI items already recorded.

### User-input-needed findings

- None.

## Round 6

Review artifacts:

- Active-session parity implementation notes for value/buffer and event/callback
  clusters.

### Applied findings

- Added Vala-visible JSON, geometry, and GeoJSON boxed constructors that
  materialize native value trees at the call boundary, plus copied JSON snapshot
  returns.
- Hid descriptor `size` fields, field masks, backend raw pointer fields, and
  struct byte-buffer pointer fields from generated VAPI; render descriptor setup
  now uses semantic defaults plus `NativePointer` setters.
- Replaced raw runtime-event fields with typed event/source/payload enums, raw
  diagnostic getters, copied message access, typed payload accessors, source
  `NativePointer`, and copied unknown payload bytes.
- Added `GLib.Bytes` access for resource request prior data, resource response
  byte setters, async request-handle retention, and one-shot provider decision
  finalization.
- Added captured custom-geometry source delegate registration with
  closure/destroy-notify metadata and map-scoped callback state retention.
- Updated the Vala compile fixture to construct JSON/geometry/GeoJSON through
  wrappers, use typed runtime event accessors, exercise copied JSON snapshot
  ownership, use `ResourceRequestHandle.retain_for_async()`, exercise custom
  geometry captured delegates, and keep render descriptor backend handles behind
  `NativePointer` setters.
- Regenerated GIR/VAPI from the scanner header and metadata.

### Resolved earlier deferrals

Round 6 resolved several Round 1 API-shape deferrals: JSON/geometry builders,
`NativePointer` descriptor setters, descriptor ABI bookkeeping hiding, typed
runtime events, resource byte-buffer helpers, and captured custom-geometry
delegate ownership. Remaining API-shape concerns are listed in later rounds.

### Rejected or deferred findings

- Linux validation still requires Linux runners or CI artifacts; this workspace
  only confirms macOS arm64 Metal. Matrix evidence continues to show Vala
  scheduled for Linux variants.

### Validation

- `mise run //bindings/vala:ci` passed locally on macOS arm64 Metal after
  regeneration, Vala compile/runtime fixture execution, Rust tests, and clippy.

### User-input-needed findings

- None.

## Round 7

Review artifacts:

- Completion-audit finding for descriptor field-mask setters.

### Applied findings

- Added Vala-visible semantic setters for field-mask-backed descriptor structs:
  runtime options, camera options, animation options, camera-fit options, bound
  options, free-camera options, projection mode, map viewport options, map tile
  options, style tile source options, custom geometry source options, style
  image options, and feature-state selectors.
- Hid the corresponding semantic fields from generated VAPI metadata so Vala
  callers set these descriptors through methods that update native field masks.
- Updated the Vala compile fixture to exercise the new setters and to avoid
  direct field assignment on field-mask-backed descriptors.
- Confirmed local `mise run //bindings/vala:ci`, `mise run fix`, and
  `mise run test` pass on macOS arm64 Metal after regeneration.

### Remaining validation notes

- Linux Vala CI remains configured through `.github/config/variants.toml` for
  `//bindings/vala:ci`. GitHub Actions run `26328621145` for commit `43e216d`
  failed during shared dependency setup before matrix generation because mise
  could not verify `github:nicklockwood/SwiftFormat@0.61.1`: the GitHub release
  API returned `401 Unauthorized`. No Vala binding job ran in that attempt.
- Earlier same-branch CI evidence includes successful Linux arm64 Vala binding
  validation in run `26327825296`. Local validation covers the field-mask setter
  patch on macOS arm64 Metal.

### Rejected or deferred findings

- None.

### User-input-needed findings

- None.

## Round 8

### Applied findings

- Added final safe setters for style tile source and feature query options, so
  Vala callers set layer IDs, source-layer IDs, zoom, tile size, and attribution
  through methods that maintain native field masks.
- Hid remaining raw value records, field-mask enums, ABI count fields, weak
  value members, and feature-extension raw entry points from generated VAPI.
- Added GIR sanitization for review artifacts so generated GIR mirrors the
  Vala-facing safety surface instead of scanner-discovered C ABI bookkeeping.
- Strengthened the Vala compile/runtime fixture to exercise the new setters and
  to use readback stride and height instead of raw byte-length fields.
- Confirmed final local validation: `mise run fix`,
  `mise run //bindings/vala:ci`, and `mise run test` pass; risky-surface scans
  over generated VAPI and GIR report no descriptor size/field-mask fields, raw
  feature records, raw count fields, or `void*` public Vala construction
  surfaces.

### Remaining validation notes

- Linux x64 and arm64 Vala jobs are present in the generated CI matrices for
  `linux-x64-vulkan` and `linux-arm64-vulkan` as `//bindings/vala:ci`.
- GitHub Actions run `26328348224` did not reach Linux binding jobs because
  native prerequisite jobs were canceled during setup/build before binding jobs
  started. The failure is recorded as CI orchestration/native-prerequisite
  evidence rather than a Vala binding compile failure.

### User-input-needed findings

- None.

## Round 9

### Applied findings

- Extended GIR sanitization to remove nested raw custom-geometry callback fields
  (`fetch_tile`, `cancel_tile`, `user_data`) from `CustomGeometrySourceOptions`
  and to remove the raw `ScreenLineString` point/count record from generated
  review GIR.
- Hid `ScreenLineString` and `RenderedQueryGeometry.data` from VAPI metadata so
  rendered query geometry uses the Vala-visible constructors rather than
  exposing raw point/count storage.
- Confirmed risky-surface scans over generated VAPI and GIR now report no raw
  custom-geometry option callback fields, no `void*` field type, no
  `ScreenLineString.point_count`, no descriptor size/field-mask fields, no raw
  feature records, and no raw ABI count fields.

### Validation

- `mise run fix`
- `mise run //bindings/vala:ci`
- `mise run test`

### User-input-needed findings

- None.

## Round 10

### Applied findings

- Removed backend descriptor pointer fields from generated GIR review artifacts:
  Metal texture/device/layer fields and Vulkan image/image-view/instance/device/
  physical-device/graphics-queue/surface fields now stay hidden behind
  `NativePointer` setters.
- Hid `VulkanContextDescriptor.graphics_queue_family_index` from VAPI/GIR as
  part of the semantic `set_handles()` call, keeping Vulkan context setup on the
  safe setter path.
- Confirmed descriptor scans over generated GIR report only the descriptor
  records and no writable raw pointer fields.

### Validation

- `mise run //bindings/vala:generate`

### User-input-needed findings

- None.

## Round 11

Review artifacts:

- `review-loop/round11-api-spec.md`
- `review-loop/round11-runtime-lifecycle.md`
- `review-loop/round11-build-generation-tests.md`
- `review-loop/round11-maintainability-docs.md`

### Applied findings

- Preserved unknown resource-provider decision values instead of collapsing them
  to pass-through, so malformed or forward-unknown callback results keep the C
  ABI's provider-error behavior. Added a Rust regression test for the state
  transition.
- Added scanner annotations for boxed JSON, geometry, GeoJSON, `NativePointer`,
  and `RuntimeEvent` copy/constructor return ownership, and extended the
  metadata annotation inventory so generation fails on regression.
- Chained Vala GObject subclass finalizers to the parent `GObject` finalizer
  after adapter-owned native cleanup.
- Added an owned boxed `StringList` value for layer-ID and tile-URL inputs, hid
  raw `StringView` from generated VAPI/GIR, and copied reusable descriptor
  strings for query options, tile attribution, and feature-state selectors.
- Released captured custom-geometry delegate state after successful matching
  source removal and inline style JSON replacement, while preserving close-time
  in-flight callback waiting before destroy-notify runs.
- Added a generated-surface regression gate for VAPI/GIR/typelib review
  artifacts and wired it into Vala generation, covering raw `StringView`, raw
  `void*` fields, descriptor `size`/`fields`, raw feature/JSON records, raw
  callback records, hidden feature-extension entry points, and render descriptor
  pointer fields.
- Updated the Vala conventions page to state that custom-geometry callbacks run
  on native callback threads and callback code dispatches to the map owner
  thread before calling thread-affine map APIs.
- Consolidated duplicate Round 6 review-log content, recorded that several
  earlier Round 1 deferrals were later resolved, and refreshed the Vala SPEC
  implementation map for current modules and generation tools.

### Rejected or deferred findings

- Full replacement of the remaining direct weak string fields with owned
  descriptor wrappers or setters is deferred because it is a broader public Vala
  API-shape change outside the focused `StringView`/query/tile/selector fix.
- Destroying custom-geometry callback state on `set_style_url()` success is
  deferred because URL style replacement completes asynchronously after the new
  style loads; a future fix needs load-state/event coordination or a
  maintainer-approved policy.

### User-input-needed findings

- Decide whether direct weak string fields such as `RuntimeOptions.asset_path`,
  `RuntimeOptions.cache_path`, `ResourceResponse.error_message`, and
  `ResourceResponse.etag` should remain low-level fields or move behind owned
  validated setters.

### Validation

- `mise run fix`
- `python bindings/vala/tools/check_generated_surfaces.py bindings/vala/build/vapi/maplibre-native.vapi bindings/vala/build/gir/MaplibreNative-0.1.gir`
- `mise run //bindings/vala:generate`
- `mise run //bindings/vala:ci`
- `mise run test`

## Round 12

Review artifacts:

- `review-loop/round12-api-spec.md`
- `review-loop/round12-runtime-lifecycle.md`
- `review-loop/round12-build-generation-tests.md`
- `review-loop/round12-maintainability-docs.md`

### Applied findings

- Initialized hidden `FeatureStateSelector.size` inside every semantic selector
  setter so Vala-created selectors satisfy the C ABI before feature-state
  operations.
- Initialized hidden offline region definition `size` fields in the adapter
  before starting offline region creation, including the selected nested tile or
  geometry definition.
- Removed undocumented exported helper symbols for the hidden boxed `Feature`
  surface and the unused `StringList.is_empty()` helper, keeping exported
  symbols aligned with the scanner header and metadata inventory.
- Kept the custom-geometry callback cleanup tests from the earlier Round 12
  pass, proving destroy-notify state is released after successful matching
  source removal and successful inline style JSON replacement.
- Added direct in-flight custom-geometry teardown coverage that holds an active
  callback guard during state destruction, verifies destroy-notify is delayed,
  then releases the guard and verifies teardown completes exactly once.

### Rejected or deferred findings

- `set_style_url()` custom-geometry teardown remains deferred because URL style
  replacement completes asynchronously after the new style loads and needs
  load-state/event coordination or a maintainer-approved policy.
- Remaining descriptor-owned string retention for low-level value records is
  recorded as user-input-needed rather than guessed in this review loop.

### User-input-needed findings

- Decide whether public result/list/snapshot handles such as
  `FeatureQueryResultHandle`, `JsonSnapshotHandle`, `OfflineRegionListHandle`,
  and `StyleIdListHandle` are accepted low-level Vala exceptions or should be
  replaced with copied GLib-owned values/arrays to match the stronger convention
  wording.
- Decide whether direct weak string fields such as `RuntimeOptions.asset_path`,
  `RuntimeOptions.cache_path`, `ResourceResponse.error_message`, and
  `ResourceResponse.etag` should remain low-level fields or move behind owned
  validated setters.
- Decide whether the current global sidecar storage for reusable plain-value
  descriptor strings is an acceptable interim low-level retention policy or
  whether those descriptors should become owned boxed/object wrappers with
  destructor-backed cleanup.

### Validation

- `mise run fix`
- `mise run //bindings/vala:ci` (50-test run before the in-flight coverage, then
  51-test run after adding the in-flight custom-geometry teardown test)

## Round 13

Review artifacts:

- `review-loop/round13-api-spec.md`
- `review-loop/round13-runtime-lifecycle.md`
- `review-loop/round13-build-generation-tests.md`
- `review-loop/round13-maintainability-docs.md`

### Applied findings

- Added direct Rust regression coverage for hidden feature-state selector size
  initialization through the Vala-visible semantic setters.
- Added direct Rust regression coverage for offline region definition size
  initialization, including selected nested tile-pyramid and geometry variant
  sizes before C ABI submission.
- Added direct Rust regression coverage for custom-geometry in-flight callback
  teardown waiting, proving destroy-notify runs only after the active callback
  guard exits.

### Rejected or deferred findings

- The typelib round-trip drops anonymous offline definition union fields; this
  does not affect the generated Vala VAPI path and remains deferred unless
  non-Vala GI consumers become in scope.
- `set_style_url()` custom-geometry teardown remains deferred because URL style
  replacement completes asynchronously after the new style loads and needs
  load-state/event coordination or a maintainer-approved policy.

### User-input-needed findings

- Public result/list/snapshot handles, direct weak string fields, and global
  sidecar descriptor string storage remain the same recorded API-policy
  decisions from Round 12.

### Validation

- `mise run fix`
- `mise run //bindings/vala:ci`
- `mise run test`

## Round 14

Review artifacts:

- `review-loop/round14-api-runtime.md`
- `review-loop/round14-validation-docs.md`

### Applied findings

- Strengthened the feature-state selector regression test so each semantic
  setter is proven to initialize hidden ABI size when called first on zeroed
  storage.
- Made the custom-geometry in-flight teardown test assert that teardown entered
  the closing/wait path before the active callback guard is released.

### Rejected or deferred findings

- `set_style_url()` custom-geometry teardown, non-Vala GI union ergonomics, and
  the recorded descriptor/result-handle API policy choices remain deferred or
  user-input-needed as recorded in Rounds 12 and 13.

### User-input-needed findings

- No new user-input-needed findings beyond the recorded API-policy decisions.

### Validation

- `mise run fix`
- `mise run //bindings/vala:ci`

## Round 15

Review artifacts:

- `review-loop/round15-sanity.md`

### Applied findings

- None. The sanity review found the Round 14 regression-test tightening and
  review-log update correct.

### Rejected or deferred findings

- Existing deferred/user-input-needed API-policy items remain recorded in prior
  rounds and were not expanded.

### User-input-needed findings

- None new.

### Validation

- `mise run //bindings/vala:test -- feature_state_selector_setters_initialize_hidden_size`
- `mise run //bindings/vala:test -- custom_geometry_destroy_waits_for_in_flight_callback`
- `git diff --check 6bb8bd6..HEAD`

## Round 16

Review artifacts:

- `review-loop/api-decisions-initial-audit.md`
- `handoff/vala-api-decisions-context.md`

### Applied findings

- Hid native result/list/snapshot handle classes and methods from the generated
  Vala, sanitized GIR, and typelib-derived GIR surfaces, including feature query
  results, feature extension results, JSON snapshots, offline region snapshots
  and lists, and style ID lists.
- Hid public weak string fields from generated surfaces, including resource
  request/response string fields, runtime option paths, and offline definition
  style URLs.
- Hid the raw `OfflineRegionDefinition.data` union from generated GIR and VAPI
  review artifacts so future GI-facing surfaces do not expose the anonymous C
  union shape.
- Extended `check_generated_surfaces.py` to fail on public weak strings,
  result/list/snapshot handles, raw offline definition union data, and the newly
  hidden string fields.
- Changed `set_style_url()` to reject URL style replacement while custom
  geometry callbacks are registered, preventing an async URL-style replacement
  path from violating callback lifetime policy until a fuller load-state
  coordination API exists.
- Hid public setter methods that depended on global sidecar descriptor string
  storage for feature-state selectors, query option string lists, and style tile
  source attribution.
- Updated the Vala compile fixture to exercise only the currently public
  convention-compliant surfaces.

### Rejected or deferred findings

- Full owned replacement APIs for the hidden result/list/snapshot and descriptor
  surfaces remain future implementation work; the pre-merge public surface no
  longer exposes the convention-breaking handles or lifetime hazards.

### User-input-needed findings

- None new.

### Validation

- `mise run //bindings/vala:generate`
- `mise run //bindings/vala:compile-tests`
- `python bindings/vala/tools/check_generated_surfaces.py bindings/vala/build/vapi/maplibre-native.vapi bindings/vala/build/gir/MaplibreNative-0.1.gir`
- `python bindings/vala/tools/check_generated_surfaces.py bindings/vala/build/vapi/maplibre-native.vapi bindings/vala/build/gir/MaplibreNative-0.1.typelib.gir`
- `mise run //bindings/vala:ci`
- `mise run fix`
- `mise run test`

## Round 17

Review artifacts:

- `review-loop/round17-api-surface.md`
- `review-loop/round17-lifecycle.md`
- `review-loop/round17-validation-docs.md`

### Applied findings

- Hid runtime-event raw diagnostic payload fields from generated Vala, GIR, and
  typelib-derived GIR surfaces while preserving typed event fields and existing
  `RuntimeEvent` raw getter methods.
- Removed scanner-header and metadata entries for sidecar-backed descriptor
  setters that had been hidden from Vala, including feature-state selector
  string setters, query option string-list setters, and style tile source
  attribution.
- Removed the now-unused global sidecar storage implementations for those hidden
  setters from the Rust adapter.
- Extended `check_generated_surfaces.py` to reject runtime-event raw payload
  fields and the removed sidecar-backed descriptor setters in future generated
  VAPI/GIR artifacts.

### Rejected or deferred findings

- The map-close custom-geometry in-flight test suggestion remains optional for
  now because the lower-level in-flight state teardown invariant and the public
  map-close callback release path are both covered; no reviewer classified it as
  a blocker.

### User-input-needed findings

- None new.

### Validation

- `mise run //bindings/vala:ci`
- `mise run fix`
- `mise run test`
- `python bindings/vala/tools/check_generated_surfaces.py bindings/vala/build/vapi/maplibre-native.vapi bindings/vala/build/gir/MaplibreNative-0.1.gir`
- `python bindings/vala/tools/check_generated_surfaces.py bindings/vala/build/vapi/maplibre-native.vapi bindings/vala/build/gir/MaplibreNative-0.1.typelib.gir`

## Round 18

Review artifacts:

- `review-loop/round18-api-surface.md`
- `review-loop/round18-lifecycle.md`
- `review-loop/round18-validation-docs.md`

### Applied findings

- Removed public raw runtime-event getter functions from the scanner header,
  Rust adapter exports, and metadata inventory.
- Extended `check_generated_surfaces.py` to reject those raw runtime-event
  getter methods in generated VAPI/GIR surfaces.

### Rejected or deferred findings

- None.

### User-input-needed findings

- None new.

### Validation

- `mise run fix`
- `mise run //bindings/vala:ci`
- `mise run test`
- `python bindings/vala/tools/check_generated_surfaces.py bindings/vala/build/vapi/maplibre-native.vapi bindings/vala/build/gir/MaplibreNative-0.1.gir`
- `python bindings/vala/tools/check_generated_surfaces.py bindings/vala/build/vapi/maplibre-native.vapi bindings/vala/build/gir/MaplibreNative-0.1.typelib.gir`

## Round 19

Review artifacts:

- `review-loop/round19-api-surface.md`
- `review-loop/round19-lifecycle-validation.md`

### Applied findings

- None. Reviewers found no remaining actionable API-surface,
  lifecycle/ownership, generated-surface, validation, or review-log blockers.

### Rejected or deferred findings

- None.

### User-input-needed findings

- None new.

### Validation

- Reviewers rechecked generated-surface gates for both sanitized GIR and
  typelib-derived GIR.
- Reviewers verified the branch was clean and pushed at `5062c2a`; the follow-up
  `8c60ee0` commit records this final review outcome only.

## Round 20

Review artifacts:

- Independent completion auditor for goal `mpith25e-nv475v`.

### Applied findings

- Updated `bindings/vala/SPEC.md` so the implementation map no longer describes
  native style ID list handles, JSON snapshot handles, feature query result
  handles, or feature extension result handles as public generated Vala
  surfaces.
- Hid `OfflineGeometryRegionDefinition.geometry` from generated VAPI, GIR, and
  typelib-derived GIR because the transparent struct field was a public borrowed
  geometry lifetime workaround. The field remains available only to the adapter
  internals until an owned offline-region-definition API shape exists.
- Extended `check_generated_surfaces.py` so future generated artifacts reject a
  public borrowed geometry field in VAPI and `OfflineGeometryRegionDefinition`
  raw geometry fields in GIR.
- Updated the Vala binding conventions page to say runtime-event raw ABI fields
  and getters are hidden rather than public diagnostic getters.

### Rejected or deferred findings

- The owned offline geometry-region constructor/API remains future work. The
  current pre-merge policy is satisfied by hiding the borrowed field and keeping
  the native definition detail out of the public generated surface.

### User-input-needed findings

- None new.

### Validation

- `mise run //bindings/vala:generate`
- `mise run fix`
- `mise run //bindings/vala:ci`
- `mise run test`
- `python bindings/vala/tools/check_generated_surfaces.py bindings/vala/build/vapi/maplibre-native.vapi bindings/vala/build/gir/MaplibreNative-0.1.gir`
- `python bindings/vala/tools/check_generated_surfaces.py bindings/vala/build/vapi/maplibre-native.vapi bindings/vala/build/gir/MaplibreNative-0.1.typelib.gir`

## Round 21

Review artifacts:

- `review-loop/round21-api-doc-surface.md`
- `review-loop/round21-lifecycle-validation.md`
- `review-loop/round21-completion-audit.md`

### Applied findings

- None. Reviewers found no remaining actionable API-decision, generated-surface,
  documentation/spec, lifecycle/ownership, validation, or review-log blockers.

### Rejected or deferred findings

- None.

### User-input-needed findings

- None new.

### Validation

- Independent reviewers verified the generated VAPI, sanitized GIR, and
  typelib-derived GIR no longer expose the blocked result/list/snapshot handles,
  weak strings, raw runtime-event surfaces, sidecar-backed setters, raw offline
  union data, or `OfflineGeometryRegionDefinition.geometry`.
- Independent reviewers verified custom-geometry callback policy coverage for
  source removal, inline style replacement, URL style replacement, map close,
  and in-flight callback teardown.
- Independent reviewers verified the branch was clean and pushed at `c52c3fa`
  with `HEAD...@{upstream}` at `0 0`.

## Round 22

Review artifacts:

- Synchronous scout/planner/reviewer subagent run for goal `mpiwuf4w-ayx14h`.
- `review-loop/round22-api-surface.md`
- `review-loop/round22-lifecycle.md`
- `review-loop/round22-validation-docs.md`

### Applied findings

- Added owned public Vala/GObject replacements for hidden result/list/snapshot
  capabilities: `StringList`, copied `JsonValue?` snapshot returns,
  `OfflineRegionInfo`, `OfflineRegionInfoList`, `QueriedFeature`,
  `QueriedFeatureList`, and `FeatureExtensionResult`.
- Added owned `Feature` and opaque `OfflineRegionDefinition` boxed values with
  tile-pyramid and geometry constructors so public Vala code can create offline
  geometry region definitions without borrowed geometry fields.
- Kept native result/list/snapshot handles and raw offline region definition
  records hidden from generated VAPI, sanitized GIR, and typelib-derived GIR.
- Updated metadata, scanner annotations, generated-surface checks, Vala compile
  coverage, Rust adapter tests, API decisions, SPEC, and Vala binding
  conventions for the owned API policy.
- Fixed Round 22 validation-review findings by annotating
  `OfflineRegionInfo.get_definition()` and `QueriedFeature.get_feature()` as
  transfer-full nullable returns, covering
  `FeatureExtensionResult.get_feature_collection()` in the Vala fixture, and
  documenting the new public `Feature` boxed value.

### Rejected or deferred findings

- Non-Vala GI consumer support remains out of scope for this branch; the GIR and
  typelib shapes are review artifacts that preserve future-compatible ownership
  and hiding policy.

### User-input-needed findings

- None new.

### Validation

- `mise run //bindings/vala:generate`
- `mise run fix`
- `mise run //bindings/vala:ci`
- `mise run test`
- `python bindings/vala/tools/check_generated_surfaces.py bindings/vala/build/vapi/maplibre-native.vapi bindings/vala/build/gir/MaplibreNative-0.1.gir`
- `python bindings/vala/tools/check_generated_surfaces.py bindings/vala/build/vapi/maplibre-native.vapi bindings/vala/build/gir/MaplibreNative-0.1.typelib.gir`
- `git diff --check`

## Round 23

Review artifacts:

- `review-loop/round23-api-surface.md`
- `review-loop/round23-lifecycle.md`
- `review-loop/round23-validation-docs.md`

### Applied findings

- None. Independent reviewers found no remaining actionable API-surface,
  ownership/lifetime/error-propagation, generated-surface, validation, test,
  documentation, branch hygiene, or push-state blockers.

### Rejected or deferred findings

- None.

### User-input-needed findings

- None new.

### Validation

- Reviewers verified the generated VAPI, sanitized GIR, and typelib-derived GIR
  expose owned result/list/snapshot and offline-region-definition replacements
  while hiding raw native handles and borrowed offline geometry records.
- Reviewers verified Vala compile/runtime fixture coverage and Rust adapter
  tests cover the owned APIs where practical.
- Reviewers verified generated-surface checker runs pass for sanitized GIR and
  typelib-derived GIR.
- Reviewers verified branch `vala` was clean and pushed at `a2d77a9` with
  `HEAD...@{upstream}` at `0 0`.

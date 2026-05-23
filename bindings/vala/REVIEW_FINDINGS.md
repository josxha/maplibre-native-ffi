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

### Applied findings

- Added boxed JSON, geometry, and GeoJSON value wrappers with Vala-visible
  constructors, native materialization at call boundaries, and copied JSON
  snapshot returns.
- Hid public Vala descriptor `size` fields, field masks, backend raw pointer
  fields, and struct byte-buffer pointer fields through vapigen metadata; added
  semantic setters where needed for render descriptors, resource responses, and
  copied resource request bytes.
- Added `NativePointer` setter APIs for Metal and Vulkan render descriptor
  backend handles while keeping raw `void*` out of generated VAPI descriptor
  construction.
- Reworked runtime events as opaque copied boxed values with typed
  event/source/payload enums, raw diagnostic getters, copied message storage,
  payload accessors, source `NativePointer`, and copied unknown payload bytes.
- Strengthened resource-provider request handling with an explicit
  `retain_for_async()` API and a decision-finalization state machine so inline
  completion forces `HANDLE`, pass-through disarms the temporary wrapper, and
  retained handles complete/release exactly once.
- Added a captured-delegate custom geometry source registration API that stores
  delegate closures in the owning map wrapper and releases destroy-notify state
  on map close.
- Updated the Vala compile fixture to construct JSON/geometry/GeoJSON through
  wrappers, use typed runtime event accessors, exercise copied JSON snapshot
  ownership, use `ResourceRequestHandle.retain_for_async()`, and keep render
  descriptor backend handles behind `NativePointer` setters.
- Regenerated GIR/VAPI from the scanner header and metadata.
  `mise run //bindings/vala:ci` passes locally on macOS arm64 Metal.

### Remaining validation notes

- Local validation covered the macOS arm64 Metal profile. Linux x64/aarch64 Vala
  CI is configured through `.github/config/variants.toml` for
  `//bindings/vala:ci`; final Linux proof still depends on running the
  configured GitHub Actions or a Linux host.

## Round 6

Review artifacts:

- Active-session parity implementation notes for value/buffer and event/callback
  clusters.

### Applied findings

- Added Vala-visible JSON, geometry, and GeoJSON boxed constructors that
  materialize native value trees at the call boundary.
- Hid descriptor `size` fields, field masks, and backend raw pointer fields from
  generated VAPI; render descriptor setup now uses semantic defaults plus
  `NativePointer` setters.
- Replaced raw runtime-event fields with typed event/source/payload enums, raw
  diagnostic getters, copied message access, and typed payload accessors.
- Added `GLib.Bytes` access for resource request prior data, resource response
  byte setters, async request-handle retention, and one-shot provider decision
  finalization.
- Added captured custom-geometry source delegate registration with
  closure/destroy-notify metadata and map-scoped callback state retention.
- Updated the Vala compile fixture to use typed event getters, boxed GeoJSON
  construction, custom-geometry captured delegates, and `NativePointer` render
  descriptor setters.
- Confirmed local `mise run //bindings/vala:ci` passes on macOS arm64 Metal
  after regeneration, Vala compile/runtime fixture execution, Rust tests, and
  clippy.

### Rejected or deferred findings

- Linux validation still requires Linux runners or CI artifacts; this workspace
  only confirms macOS arm64 Metal. Matrix evidence continues to show Vala
  scheduled for Linux variants.

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

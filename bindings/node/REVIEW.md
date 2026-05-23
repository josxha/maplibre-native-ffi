# Node binding review log

This file records review-loop findings for the `node` branch and how each
finding was handled.

## Round 1

Review evidence:

- Ran three independent review agents against the `node` branch diff versus
  `origin/main`:
  - FFI correctness and safety review.
  - JavaScript/TypeScript API surface review.
  - SPEC and test conformance review.
- Applied accepted findings in this round and validated with `mise run fix` plus
  `mise run //bindings/node:ci`.

Applied findings:

1. Runtime/map callback `user_data` lifetime on GC/drop leak paths.
   - Action: when native runtime/map handles are intentionally leaked by `Drop`,
     callback state backing raw `user_data` is also intentionally retained
     instead of freed.
2. Render-session frame-scope exclusion.
   - Action: `NativeRenderSessionHandle` now tracks active owned-texture frames
     and rejects reentrant session operations while a frame is acquired.
3. Render descriptor pointer API.
   - Action: public render descriptors now use `NativePointer` fields; the
     JavaScript wrapper unwraps those values to native address fields for the
     N-API add-on.
4. Private-constructor runtime surface mismatch.
   - Action: constructors declared private in TypeScript now require a
     module-private token at runtime.
5. Texture-frame callback async foot-gun.
   - Action: TypeScript declarations reject Promise-returning texture-frame
     callbacks, with checked `@ts-expect-error` coverage.
6. `NativeBuffer` shared-memory copy behavior.
   - Action: typed-array inputs now copy through a fresh `ArrayBuffer`; tests
     cover `SharedArrayBuffer` views.
7. Package subpath export coverage.
   - Action: tests now exercise every package subpath through the package
     `exports` map.
8. Stale SPEC current-file block.
   - Action: `SPEC.md` now lists the current concept modules and Rust modules.

Recorded limitations / not applied:

- A deterministic end-to-end Node test for real native resource-provider request
  completion was not added. The Node test suite has no stable network-request
  trigger that avoids broader integration policy. The implementation still
  enforces one-shot completion and exact-once release through the native request
  registry in `src/runtime.rs`; existing tests cover callback registration and
  JavaScript handle guards, and `SPEC.md` now records the Node-suite limitation.

Findings requiring user input: none.

## Round 2

Review evidence:

- Ran three independent review agents after the Round 1 fixes:
  - FFI/lifetime review.
  - JavaScript/TypeScript API surface review.
  - SPEC/REVIEW/test adequacy review.
- The FFI/lifetime reviewer reported no remaining actionable findings.
- Applied accepted findings in this round and validated with `mise run fix` plus
  `mise run //bindings/node:ci`.

Applied findings:

1. `NativeBuffer` constructor aliases caller-owned `ArrayBuffer`.
   - Action: `new NativeBuffer(existingArrayBuffer)` now copies with `slice(0)`,
     and tests verify source mutation does not affect the `NativeBuffer`.
2. Subpath export tests did not cover `types` conditions or most curated module
   shapes.
   - Action: tests assert representative value exports for each value-bearing
     subpath, and `test/subpath-types.test.cts` imports every concept subpath
     through the package export map for type resolution.
3. `SPEC.md` current-file block omitted the review log.
   - Action: `SPEC.md` now includes `REVIEW.md` and the subpath type fixture.

Recorded limitations / not applied: none.

Findings requiring user input: none.

## Round 3

Review evidence:

- Ran three independent review agents after the Round 2 fixes:
  - FFI/lifetime review.
  - JavaScript/TypeScript API surface review.
  - SPEC/REVIEW/test adequacy review.

Applied findings:

1. Retired custom-geometry callback state could be freed while native worker
   callbacks still hold `user_data`.
   - Action: removed/replaced custom-geometry callback state is retained in a
     retired list until map close; leaked native maps leak active and retired
     callback state together.
2. Owned-texture frame pointers could escape the active frame scope as reusable
   `NativePointer` values.
   - Action: frame-derived `NativePointer` instances now share the frame
     validity predicate and throw after frame scope close.
3. ESM named imports type-checked but failed at runtime for CommonJS exports.
   - Action: root and value-bearing subpath CommonJS modules now use analyzable
     named `module.exports.*` / `exports.*` assignments, with
     `test/esm-smoke.mjs` covering ESM named imports.
4. Published declarations depended on `Symbol.dispose` without loading the
   disposable lib.
   - Action: `index.d.cts` now references `esnext.disposable`.
5. Round 2 validation was not recorded in this review log.
   - Action: this log now records validation evidence for Round 2.
6. The subpath type fixture omitted the camera subpath.
   - Action: `test/subpath-types.test.cts` now imports and uses `CameraOptions`
     from `@maplibre/native-ffi-node/camera`.
7. The SPEC Worker checklist overstated native wrong-thread coverage.
   - Action: the checklist now says the Node tests cover worker-local runtime
     creation and detached public-handle guards, while native wrong-thread
     status remains covered at the C ABI layer.

Recorded limitations / not applied: none.

Findings requiring user input: none.

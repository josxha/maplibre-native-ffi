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

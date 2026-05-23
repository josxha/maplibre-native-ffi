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
- Applied accepted findings in this round and validated with Rust formatting for
  the Node crate, `mise run fix`, and `mise run //bindings/node:ci`.

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

## Round 4

Review evidence:

- Ran three independent review agents after the Round 3 fixes:
  - FFI/lifetime/render/resource safety review.
  - JavaScript/TypeScript API, ESM/CJS, declaration, and test review.
  - SPEC/REVIEW record-conformance review.
- The JavaScript/TypeScript reviewer reported no remaining actionable findings.
- Applied accepted findings in this round and validated with Rust formatting for
  the Node crate, `mise run fix`, and `mise run //bindings/node:ci`.

Applied findings:

1. Resource request completion removed the handle from the native registry
   before response validation and native completion succeeded.
   - Action: completion now validates the response and performs native
     completion before removing the request handle from the registry, so
     binding-owned validation errors leave the request retryable/closable.
2. Resource-transform replacement URL storage grew without bound.
   - Action: replacement URLs are now stored per callback thread and overwritten
     on that thread's next transform result, preserving pointer validity for the
     current C callback without retaining every past URL.
3. Custom-geometry callback state was retained after style replacement removed
   the native source.
   - Action: successful inline `setStyleJson` clears custom-geometry callback
     state because replacement has completed before return; successful
     `setStyleUrl` retires active state conservatively until map close because
     style loading is asynchronous.
4. Round 3 validation evidence was missing from the review log.
   - Action: Round 3 now records the formatter/fix/CI validation commands.

Recorded limitations / not applied: none.

Findings requiring user input:

- A reviewer found that resource transform/provider callbacks currently block
  native worker/network threads while waiting for JavaScript return values. The
  repository's Node-binding design notes prefer native-owned routing/rewrite
  rules and asynchronous JavaScript handoff instead. Changing this now would
  alter the public callback API in `bindings/node/SPEC.md` and the implemented
  low-level Node surface, so it needs an explicit scope/API decision before this
  review loop can either redesign the callbacks or record the synchronous
  callback API as an accepted limitation.

## Resource callback design correction

Review evidence:

- Revisited `bindings/node/SPEC.md` against
  `docs/src/content/docs/development/bindings-node.md`, especially the callback
  and event-loop handoff rules.
- Updated the SPEC to require native-owned resource transform rules and provider
  routes instead of synchronous JavaScript-return-value decisions on native
  worker/network threads.
- Implemented the corrected API and validated with Rust formatting for the Node
  crate, `cargo check -p maplibre-native-node`, `mise run fix`, and
  `mise run //bindings/node:ci`.

Applied findings:

1. Resource transform callbacks blocked native threads waiting for JavaScript
   return values.
   - Action: replaced the public callback API with
     `RuntimeHandle.setResourceTransformRules()`. Native transform callbacks now
     apply stored exact-URL and URL-prefix rewrite rules synchronously without
     crossing into JavaScript.
2. Resource provider callbacks blocked native threads waiting for JavaScript
   routing decisions.
   - Action: replaced the return-value decision API with
     `RuntimeHandle.setResourceProviderRoutes(routes, callback)`. Native
     callbacks pass through non-matching requests immediately and enqueue
     matching requests to JavaScript with a one-shot `ResourceRequestHandle`.
3. Type declarations and tests still described the drifted callback API.
   - Action: TypeScript declarations, the resource subpath declarations, and
     Node tests now cover resource routes, transform rules, void provider
     callbacks, and binding-owned validation for invalid rule shapes.

Recorded limitations / not applied: none.

Findings requiring user input: none.

## Post-redesign review round 1

Review evidence:

- Ran three independent review agents after the resource callback design
  correction:
  - FFI/threading/handle-lifetime review.
  - JavaScript/TypeScript API and test review.
  - SPEC/REVIEW conformance review.
- Applied accepted findings in this round and validated with Rust formatting for
  the Node crate, `cargo check -p maplibre-native-node`, `mise run fix`, and
  `mise run //bindings/node:ci`.

Applied findings:

1. Provider callback throws or rejected thenables could close/release a handled
   request without completing it with an error response.
   - Action: provider callback handoff now completes the request with an error
     response on synchronous throws or rejected thenables, falling back to close
     only when completion is already invalid.
2. `ResourceRequestHandle` cleanup depended only on explicit user calls.
   - Action: the wrapper registers resource request handles with a best-effort
     `FinalizationRegistry`, and the native runtime tracks pending provider
     request IDs so runtime/provider teardown releases outstanding handles.
3. Replaced/cleared resource provider and transform state could drop while
   native callbacks still held raw `user_data` pointers.
   - Action: replaced providers/transforms are retained in retired state until
     runtime close/destruction, and leaked runtimes intentionally forget active
     and retired callback state.
4. `ResourceTransformRule` declarations allowed invalid rule shapes.
   - Action: declarations now encode exactly one replacement form and require
     `urlPrefix` for prefix replacement, with `@ts-expect-error` coverage for
     rejected shapes.
5. Provider-route tests did not execute the JavaScript handoff wrapper.
   - Action: Node tests now invoke the wrapper through a fake native provider
     hook to verify `handleId` stripping, `ResourceRequestHandle` wrapping,
     ignored return values, synchronous throw handling, and rejected thenable
     handling.

Recorded limitations / not applied: none.

Findings requiring user input: none.

## Post-redesign review round 2

Review evidence:

- Ran three independent review agents after the resource request handoff
  hardening:
  - FFI/threading/pending-request lifetime review.
  - JavaScript/TypeScript API, declaration, export, and test review.
  - SPEC/REVIEW record-conformance review.
- The JavaScript/TypeScript reviewer reported no remaining actionable findings.
- Applied accepted findings in this round and validated with Rust formatting for
  the Node crate, `cargo check -p maplibre-native-node`, `mise run fix`, and
  `mise run //bindings/node:ci`.

Applied findings:

1. Non-matching provider requests copied the full borrowed C request before
   native route matching.
   - Action: the provider trampoline now inspects only borrowed `kind` and `url`
     for route matching, returns pass-through before copying non-matching
     requests, and copies the full request only after a route matches.
2. Rejected thenable handling used `.catch()` directly and missed valid
   thenables without a `catch` method.
   - Action: provider callback results now use `Promise.resolve(result).catch`,
     preserving thrown-then-getter handling through the outer try/catch.
3. Provider error-completion tests proved fallback close behavior rather than
   the submitted error response.
   - Action: wrapper tests now patch the native completion/close functions and
     assert the error response for synchronous throw, rejected Promise, and
     rejected custom thenable cases.
4. The review log did not record the post-hardening review loop.
   - Action: this round records the review evidence, applied findings, and
     validation commands.

Recorded limitations / not applied: none.

Findings requiring user input: none.

## Post-redesign review round 3

Review evidence:

- Ran three independent review agents after the route handoff refinement:
  - FFI/threading/minimal-route-matching review.
  - JavaScript/TypeScript API, declaration, export, and test review.
  - SPEC/REVIEW record-conformance review.
- The JavaScript/TypeScript and SPEC/REVIEW reviewers reported no remaining
  actionable findings.
- Applied accepted findings in this round and validated with Rust formatting for
  the Node crate, `cargo check -p maplibre-native-node`, `mise run fix`, and
  `mise run //bindings/node:ci`.

Applied findings:

1. The provider handoff wrapper used a one-argument callback shape, while
   napi-rs invokes `ThreadsafeFunction<ResourceProviderRequest>` callbacks as
   `(error, request)` under the default callee-handled error convention.
   - Action: the wrapper now accepts `(error, request)`, throws the error before
     request wrapping, and unwraps the request from the second argument.
   - Test action: provider wrapper tests now invoke the fake native callback as
     `(null, request)` to match the real ThreadsafeFunction call shape.

Recorded limitations / not applied: none.

Findings requiring user input: none.

## Post-redesign review round 4

Review evidence:

- Ran three independent review agents after the provider callback handoff shape
  fix:
  - FFI/threading/ThreadsafeFunction convention review.
  - JavaScript/TypeScript API, declaration, export, and test review.
  - SPEC/REVIEW record-conformance review.
- The FFI and SPEC/REVIEW reviewers reported no remaining actionable findings.
- Applied accepted findings in this round and validated with Rust formatting for
  the Node crate, `mise run fix`, and `mise run //bindings/node:ci`.

Applied findings:

1. ESM smoke coverage did not exercise the resource or runtime subpaths central
   to the redesigned resource API.
   - Action: `test/esm-smoke.mjs` now imports `ResourceRequestHandle` from the
     resource subpath and `RuntimeHandle` from the runtime subpath.
2. Typecheck coverage did not exercise the corrected provider callback API.
   - Action: `test/subpath-types.test.cts` now imports
     `ResourceProviderCallback`, defines a callback that receives
     `request.handle`, and type-checks a
     `RuntimeHandle.setResourceProviderRoutes()` call.

Recorded limitations / not applied: none.

Findings requiring user input: none.

# Python binding decisions

This note records product and maintenance decisions that came out of the Python
binding review loops. It keeps the deferred items explicit so future review work
can distinguish accepted trade-offs from forgotten tasks.

## Public root exports

Use minimal root exports. The package root (`maplibre_native`) should expose the
core handles, options, backend flags, runtime event values, global functions,
and errors that make the package approachable. Concept-specific symbols should
live in their modules (`maplibre_native.offline`, `maplibre_native.render`,
`maplibre_native.resource`, and so on) rather than being duplicated broadly at
the root.

This keeps one sensible import location for most symbols and avoids turning
`__init__.py` into a second public index for the entire low-level API.

## Enum and wrapper deduplication

Keep this branch polished, but defer deeper deduplication below the Python
public binding layer.

Within the Python binding stack, remove repetition when it improves clarity and
keeps the code mechanically regular. Across Rust shared-core helpers, the C ABI,
and other language bindings, prefer a follow-up design rather than broad
refactoring in this Python PR.

## Runtime annotation cycles

Remove Python module import cycles rather than keeping runtime-only `Any`
fallbacks indefinitely. Static `TYPE_CHECKING` imports are acceptable as a
short-term bridge, but the desired shape is that public annotations resolve
sensibly at runtime without module cycles.

Ruff does not currently provide a dedicated import-cycle rule suitable for this
package. The Python tests instead include a focused guard that rejects
`TYPE_CHECKING`/`Any` runtime annotation fallbacks in public modules, alongside
representative `typing.get_type_hints()` assertions for formerly cyclic public
APIs.

## Offline operation lifecycle

Prefer correct lifecycle behavior now when it is not far out of the way. Java
FFM currently routes offline result taking and discard through `RuntimeHandle`,
and an operation whose runtime is already closed becomes closed when discard
sees the closed runtime. Python should provide comparable Python-side lifecycle
safety.

The immediate Python fix already landed for closed or double-taken operation
handles: `OfflineOperationHandle.take_*()` methods validate closed state before
calling native code. The remaining lifecycle polish is coordination between
`RuntimeHandle.close()` and still-live `OfflineOperationHandle` wrappers. The
preferred behavior is that a successful runtime close marks outstanding offline
operation wrappers closed, so later `take_*()` or `close()` calls fail/no-op
from Python-owned state rather than reaching native with stale runtime state.

Keep failed native operations retryable: if a native take or discard fails, the
offline operation wrapper should remain live unless the failure proves the
runtime is already closed and the wrapper can no longer be used.

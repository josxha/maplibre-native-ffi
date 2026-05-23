# Vala API decisions before merge

Audience: maintainers and binding contributors. Documentation category:
explanation / decision record.

This document records API-policy decisions for the Vala binding review loop. The
project currently promises a Vala API, but the implementation uses GObject
Introspection machinery. These decisions keep the Vala API aligned with the
shared binding conventions and avoid shapes that would block future PyGObject,
GJS, lgi, Ruby, Perl, or other GI consumers.

## Governing decision

All deviations from the shared binding conventions and Vala binding conventions
are pre-merge blockers for this PR. The Vala binding should follow the same
model as the other bindings unless the project explicitly changes the shared
conventions first.

The shared model is:

- long-lived native lifecycle objects may be public `*Handle` types;
- snapshot, result, list, event, descriptor, geometry, JSON, and copied data
  types become language-owned values;
- raw C ABI bookkeeping stays internal;
- public APIs validate language-owned state and materialize temporary native
  storage at the call boundary.

## Public handle policy

Public `*Handle` types are reserved for native objects with identity and
deterministic native lifecycle. Examples include `RuntimeHandle`, `MapHandle`,
`MapProjectionHandle`, `RenderSessionHandle`, texture frame handles, and the
documented request-lifetime exception `ResourceRequestHandle`.

Native snapshot, result, and list handles must remain internal implementation
details. Public Vala APIs should copy their contents into GLib/Vala-owned
values, arrays, boxed values, or containers, then release the native handle on
every exit path.

Pre-merge consequence: public APIs should not expose handles such as
`FeatureQueryResultHandle`, `JsonSnapshotHandle`, `OfflineRegionListHandle`, or
`StyleIdListHandle` as ordinary result/list/snapshot data surfaces.

Implemented policy: style ID lists use boxed `StringList`; JSON snapshots return
copied `JsonValue?`; offline region snapshots/lists use boxed
`OfflineRegionInfo` and `OfflineRegionInfoList`; feature query results use boxed
`QueriedFeatureList`/`QueriedFeature`; feature extension results use boxed
`FeatureExtensionResult`. Native handles for those concepts stay internal and
hidden from generated VAPI, sanitized GIR, and typelib-derived GIR.

## String ownership policy

Public Vala APIs should expose normal owned UTF-8 strings, properties, setters,
or copied values. The adapter should validate strings before crossing into C,
including embedded NUL checks for null-terminated C inputs.

Public structs or descriptors should not expose weak or borrowed string fields
whose lifetime callers must preserve manually. The adapter should own the
Vala-side string state or materialize temporary native string views for one C
call.

Pre-merge consequence: direct weak string fields such as
`RuntimeOptions.asset_path`, `RuntimeOptions.cache_path`,
`ResourceResponse.error_message`, and `ResourceResponse.etag` must move behind
owned, validated Vala/GLib API shapes if they are currently public.

## Descriptor storage policy

Descriptors should own language fields or materialize native structs at the call
boundary. Global sidecar storage for descriptor strings is not an acceptable
public-lifetime strategy.

An internal cache or side table is acceptable only when it cannot leak a public
lifetime contract and when tests prove cleanup and replacement behavior. Prefer
boxed or object descriptor wrappers with destructor-backed cleanup when reusable
descriptor storage must outlive one call.

Pre-merge consequence: descriptor string retention should not rely on global
sidecar storage to make public transparent structs safe.

## Custom geometry callback policy

Custom geometry source callbacks are map/style scoped. They live until the
source is removed, the style is replaced, or the map is closed, and until any
in-flight callback returns.

Inline style replacement, source removal, map close, and URL style replacement
must each have an explicit callback-state policy. URL style replacement is
asynchronous, so its teardown policy needs load-state or event coordination
rather than an implicit leak or guess.

Pre-merge consequence: `set_style_url()` custom-geometry teardown is a blocker
unless the API is changed so it cannot violate the callback lifetime convention.

## GIR and typelib future-compatibility policy

The project promises only a Vala API today. Even so, the GObject/GIR surface
should avoid API shapes that would preclude future GI language support.

Future GI consumers may include PyGObject, GJS, lgi, Ruby GObject Introspection,
Perl GObject Introspection, or generated bindings that read GIR metadata. These
languages consume `.gir` or `.typelib` directly rather than Vala's `.vapi` file.

Design the generated GObject/GIR surface as a future public shape:

- expose constructors, methods, properties, and boxed/object values;
- hide anonymous unions and raw C-ish fields;
- hide ABI `size` fields, field masks, descriptor bookkeeping, raw `void*`, raw
  string-view arrays, and native snapshot/result/list handles;
- use `NativePointer` only for borrowed backend-native handles already present
  in the C API contract;
- use `GLib.Bytes`, arrays, boxed values, or copied strings for copied data.

Pre-merge consequence: even if PyGObject, GJS, lgi, and similar consumers remain
unsupported for now, the Vala adapter should not ship a public GIR/typelib shape
that exposes raw unions or other convention-breaking ABI details when the Vala
API can hide them behind GLib-friendly constructors and accessors.

Implemented policy: `OfflineRegionDefinition` is an opaque boxed value with
`tile_pyramid` and `geometry` constructors plus owned accessors. Raw
`OfflineTilePyramidRegionDefinition` and `OfflineGeometryRegionDefinition`
records stay hidden from VAPI, sanitized GIR, and typelib-derived GIR.

## Documentation promise

Documentation should say that this PR supports the Vala binding. It should not
imply polished support for every GI language until those consumers have tests
and support policy.

At the same time, docs and tests should treat the generated VAPI, GIR, and
typelib-derived GIR as review artifacts. Regression checks should prevent raw
ABI surfaces from reappearing in any of them.

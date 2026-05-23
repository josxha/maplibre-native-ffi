"""JSON value trees that preserve numeric shape and duplicate object keys."""

from __future__ import annotations

from dataclasses import dataclass
import math
from typing import Any, TypeAlias


@dataclass(frozen=True, slots=True)
class JsonUInt:
    """Unsigned JSON integer value."""

    value: int

    def __post_init__(self) -> None:
        if self.value < 0:
            msg = "JsonUInt value must be non-negative"
            raise ValueError(msg)


@dataclass(frozen=True, slots=True)
class JsonInt:
    """Signed JSON integer value."""

    value: int


@dataclass(frozen=True, slots=True)
class JsonDouble:
    """Finite JSON double value."""

    value: float

    def __post_init__(self) -> None:
        if not math.isfinite(self.value):
            msg = "JsonDouble value must be finite"
            raise ValueError(msg)


@dataclass(frozen=True, slots=True)
class JsonMember:
    """Ordered JSON object member. Duplicate keys are preserved."""

    key: str
    value: JsonValue


@dataclass(frozen=True, slots=True)
class JsonArray:
    """Ordered JSON array value."""

    values: tuple[JsonValue, ...]


@dataclass(frozen=True, slots=True)
class JsonObject:
    """Ordered JSON object value that preserves duplicate keys."""

    members: tuple[JsonMember, ...]

    @classmethod
    def from_pairs(cls, pairs: list[tuple[str, JsonValue]]) -> "JsonObject":
        """Build an object from ordered key/value pairs."""
        return cls(tuple(JsonMember(key, value) for key, value in pairs))


JsonScalar: TypeAlias = None | bool | str | JsonUInt | JsonInt | JsonDouble
JsonValue: TypeAlias = JsonScalar | JsonArray | JsonObject


def json_uint(value: int) -> JsonUInt:
    """Create an unsigned JSON integer."""
    return JsonUInt(value)


def json_int(value: int) -> JsonInt:
    """Create a signed JSON integer."""
    return JsonInt(value)


def json_double(value: float) -> JsonDouble:
    """Create a finite JSON double."""
    return JsonDouble(value)


def json_array(values: list[JsonValue] | tuple[JsonValue, ...]) -> JsonArray:
    """Create an ordered JSON array."""
    return JsonArray(tuple(values))


def json_object(members: list[JsonMember] | tuple[JsonMember, ...]) -> JsonObject:
    """Create an ordered JSON object."""
    return JsonObject(tuple(members))


def from_python(value: Any) -> JsonValue:
    """Convert ordinary Python values to explicit JSON values.

    Python `dict` preserves insertion order, but cannot represent duplicate
    keys. Use `JsonObject.from_pairs()` when duplicate object members matter.
    Python `int` converts to `JsonInt`; use `json_uint()` to preserve unsigned
    integer shape.
    """
    if value is None or isinstance(value, bool | str):
        return value
    if isinstance(value, int):
        return JsonInt(value)
    if isinstance(value, float):
        return json_double(value)
    if isinstance(value, list | tuple):
        return JsonArray(tuple(from_python(item) for item in value))
    if isinstance(value, dict):
        return JsonObject(
            tuple(
                JsonMember(str(key), from_python(item)) for key, item in value.items()
            )
        )
    msg = f"unsupported JSON value: {type(value).__name__}"
    raise TypeError(msg)


def to_python(value: JsonValue) -> Any:
    """Convert a JSON value to Python containers.

    `JsonObject` converts to an ordered list of `(key, value)` pairs so duplicate
    keys survive the conversion.
    """
    if isinstance(value, JsonUInt | JsonInt | JsonDouble):
        return value.value
    if isinstance(value, JsonArray):
        return [to_python(item) for item in value.values]
    if isinstance(value, JsonObject):
        return [(member.key, to_python(member.value)) for member in value.members]
    return value


def _to_native_wire(value: JsonValue) -> Any:
    """Convert an explicit JSON tree to private native-bridge wire values."""
    if value is None or isinstance(value, bool | str):
        return value
    if isinstance(value, JsonUInt):
        return {"type": "uint", "value": value.value}
    if isinstance(value, JsonInt):
        return {"type": "int", "value": value.value}
    if isinstance(value, JsonDouble):
        return {"type": "double", "value": value.value}
    if isinstance(value, JsonArray):
        return {
            "type": "array",
            "values": [_to_native_wire(item) for item in value.values],
        }
    if isinstance(value, JsonObject):
        return {
            "type": "object",
            "members": [
                (member.key, _to_native_wire(member.value)) for member in value.members
            ],
        }
    msg = f"unsupported JSON value: {type(value).__name__}"
    raise TypeError(msg)


def _from_native_wire(raw: Any) -> JsonValue:
    """Convert private native-bridge wire values to an explicit JSON tree."""
    if raw is None or isinstance(raw, bool | str):
        return raw
    kind = raw["type"]
    if kind == "bool":
        return raw["value"]
    if kind == "uint":
        return JsonUInt(raw["value"])
    if kind == "int":
        return JsonInt(raw["value"])
    if kind == "double":
        return JsonDouble(raw["value"])
    if kind == "array":
        return JsonArray(tuple(_from_native_wire(item) for item in raw["values"]))
    if kind == "object":
        return JsonObject(
            tuple(
                JsonMember(key, _from_native_wire(value))
                for key, value in raw["members"]
            )
        )
    msg = f"unsupported native JSON wire kind: {kind}"
    raise TypeError(msg)


__all__ = [
    "JsonArray",
    "JsonDouble",
    "JsonInt",
    "JsonMember",
    "JsonObject",
    "JsonScalar",
    "JsonUInt",
    "JsonValue",
    "from_python",
    "json_array",
    "json_double",
    "json_int",
    "json_object",
    "json_uint",
    "to_python",
]

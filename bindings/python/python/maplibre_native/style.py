"""Style source, layer, image, light, and property APIs."""

from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum
from typing import Any


class StyleSourceType(IntEnum):
    """Style source type values returned by MapLibre Native."""

    UNKNOWN = 0
    VECTOR = 1
    RASTER = 2
    RASTER_DEM = 3
    GEOJSON = 4
    IMAGE = 5
    VIDEO = 6
    ANNOTATIONS = 7
    CUSTOM_VECTOR = 8

    @classmethod
    def _missing_(cls, value: object) -> "StyleSourceType | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown

    @property
    def native_code(self) -> int:
        """Return the C enum value for this source type."""
        return int(self)


@dataclass(frozen=True, slots=True)
class StyleSourceInfo:
    """Copied fixed metadata for one style source."""

    source_type: StyleSourceType
    is_volatile: bool
    attribution: str | None = None

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "StyleSourceInfo":
        """Build source metadata from private native bridge values."""
        return cls(
            source_type=StyleSourceType(raw["source_type"]),
            is_volatile=raw["is_volatile"],
            attribution=raw["attribution"],
        )


class CustomGeometrySourceEventType(IntEnum):
    """Custom geometry source callback event kind."""

    FETCH_TILE = 0
    CANCEL_TILE = 1


@dataclass(frozen=True, slots=True)
class CanonicalTileId:
    """Canonical tile identity used by custom geometry source callbacks."""

    z: int
    x: int
    y: int


@dataclass(frozen=True, slots=True)
class CustomGeometrySourceEvent:
    """Queued custom geometry source callback event."""

    event_type: CustomGeometrySourceEventType
    tile_id: CanonicalTileId

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "CustomGeometrySourceEvent":
        """Build an event from private native values."""
        return cls(
            event_type=CustomGeometrySourceEventType(raw["kind"]),
            tile_id=CanonicalTileId(z=raw["z"], x=raw["x"], y=raw["y"]),
        )


@dataclass(frozen=True, slots=True)
class CustomGeometrySourceOptions:
    """Options used when adding a custom geometry source."""

    min_zoom: float | None = None
    max_zoom: float | None = None
    tolerance: float | None = None
    tile_size: int | None = None
    buffer: int | None = None
    clip: bool | None = None
    wrap: bool | None = None
    has_cancel_tile: bool = False
    max_queued_events: int = 1024


class CustomGeometrySourceHandle:
    """Owner-thread handle for queued custom geometry source callback events."""

    def __init__(self, native: Any) -> None:
        self._native = native

    @property
    def closed(self) -> bool:
        """Return whether the native callback state has been released."""
        return bool(self._native.closed)

    @property
    def dropped_event_count(self) -> int:
        """Return how many callback events were dropped because the queue was full."""
        return self._native.dropped_event_count

    def poll_event(self) -> CustomGeometrySourceEvent | None:
        """Return one queued fetch/cancel event copied into Python values."""
        event = self._native.poll_event()
        if event is None:
            return None
        return CustomGeometrySourceEvent.from_native(event)


__all__ = [
    "CanonicalTileId",
    "CustomGeometrySourceEvent",
    "CustomGeometrySourceEventType",
    "CustomGeometrySourceHandle",
    "CustomGeometrySourceOptions",
    "StyleSourceInfo",
    "StyleSourceType",
]

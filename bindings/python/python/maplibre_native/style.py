"""Style source, layer, image, light, and property APIs."""

from __future__ import annotations

from ._lifecycle import warn_unclosed as _warn_unclosed
from dataclasses import dataclass
from enum import IntEnum
from typing import Any

from .geo import LatLngBounds
from .render import PremultipliedRgba8Image, TextureImageInfo


class TileScheme(IntEnum):
    """Tile URL coordinate scheme values."""

    XYZ = 0
    TMS = 1


class VectorTileEncoding(IntEnum):
    """Vector tile encoding values."""

    MVT = 0
    MLT = 1


class RasterDemEncoding(IntEnum):
    """DEM raster encoding values."""

    MAPBOX = 0
    TERRARIUM = 1


@dataclass(frozen=True, slots=True)
class TileSourceOptions:
    """Options for vector, raster, and raster DEM tile sources."""

    min_zoom: float | None = None
    max_zoom: float | None = None
    attribution: str | None = None
    scheme: TileScheme | None = None
    bounds: LatLngBounds | None = None
    tile_size: int | None = None
    vector_encoding: VectorTileEncoding | None = None
    raster_dem_encoding: RasterDemEncoding | None = None


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


@dataclass(frozen=True, slots=True)
class StyleImageOptions:
    """Options for adding or replacing a runtime style image."""

    pixel_ratio: float | None = None
    sdf: bool | None = None


@dataclass(frozen=True, slots=True)
class StyleImageInfo:
    """Fixed metadata for one runtime style image."""

    width: int
    height: int
    stride: int
    byte_length: int
    pixel_ratio: float
    sdf: bool

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "StyleImageInfo":
        """Build style image metadata from private native bridge values."""
        return cls(
            width=raw["width"],
            height=raw["height"],
            stride=raw["stride"],
            byte_length=raw["byte_length"],
            pixel_ratio=raw["pixel_ratio"],
            sdf=raw["sdf"],
        )


@dataclass(frozen=True, slots=True)
class StyleImage:
    """Copied runtime style image pixels with style-specific metadata."""

    image: PremultipliedRgba8Image
    pixel_ratio: float
    sdf: bool

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "StyleImage":
        """Build a copied style image from private native bridge values."""
        info = StyleImageInfo.from_native(raw["info"])
        return cls(
            image=PremultipliedRgba8Image(
                TextureImageInfo(
                    width=info.width,
                    height=info.height,
                    stride=info.stride,
                    byte_length=info.byte_length,
                ),
                raw["data"],
            ),
            pixel_ratio=info.pixel_ratio,
            sdf=info.sdf,
        )


class LocationIndicatorImageKind(IntEnum):
    """Location indicator image-name properties."""

    TOP = 0
    BEARING = 1
    SHADOW = 2

    @property
    def native_code(self) -> int:
        """Return the C enum value for this image property slot."""
        return int(self)


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

    def close(self) -> None:
        """Release queued callback state for this source handle."""
        self._native.close()

    def __del__(self, _warn_unclosed=_warn_unclosed) -> None:
        try:
            _warn_unclosed("CustomGeometrySourceHandle", getattr(self, "closed", True))
        except BaseException:
            return

    def poll_event(self) -> CustomGeometrySourceEvent | None:
        """Return one queued fetch/cancel event copied into Python values."""
        event = self._native.poll_event()
        if event is None:
            return None
        return CustomGeometrySourceEvent.from_native(event)

    def __enter__(self) -> "CustomGeometrySourceHandle":
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: object | None,
    ) -> None:
        self.close()


__all__ = [
    "CanonicalTileId",
    "CustomGeometrySourceEvent",
    "CustomGeometrySourceEventType",
    "CustomGeometrySourceHandle",
    "CustomGeometrySourceOptions",
    "LocationIndicatorImageKind",
    "RasterDemEncoding",
    "StyleImage",
    "StyleImageInfo",
    "StyleImageOptions",
    "StyleSourceInfo",
    "StyleSourceType",
    "TileScheme",
    "TileSourceOptions",
    "VectorTileEncoding",
]

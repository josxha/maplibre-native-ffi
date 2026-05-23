"""Rendered, source, and extension query descriptors and results."""

from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum
from typing import Any

from .camera import ScreenPoint
from .geo import Feature
from .json import JsonValue


class RenderedQueryGeometryType(IntEnum):
    """Rendered feature query geometry variants."""

    POINT = 1
    BOX = 2
    LINE_STRING = 3


@dataclass(frozen=True, slots=True)
class ScreenBox:
    """Screen-space box in logical map pixels."""

    min: ScreenPoint
    max: ScreenPoint


@dataclass(frozen=True, slots=True)
class RenderedQueryGeometry:
    """Screen-space geometry used for rendered feature queries."""

    type: RenderedQueryGeometryType
    point: ScreenPoint | None = None
    box: ScreenBox | None = None
    line_string: tuple[ScreenPoint, ...] | None = None

    @classmethod
    def point_geometry(cls, point: ScreenPoint) -> "RenderedQueryGeometry":
        """Create a point query geometry."""
        return cls(RenderedQueryGeometryType.POINT, point=point)

    @classmethod
    def box_geometry(cls, box_: ScreenBox) -> "RenderedQueryGeometry":
        """Create a box query geometry."""
        return cls(RenderedQueryGeometryType.BOX, box=box_)

    @classmethod
    def line_string_geometry(
        cls,
        points: list[ScreenPoint] | tuple[ScreenPoint, ...],
    ) -> "RenderedQueryGeometry":
        """Create a line-string query geometry."""
        return cls(RenderedQueryGeometryType.LINE_STRING, line_string=tuple(points))

    def __post_init__(self) -> None:
        active = sum(
            value is not None for value in (self.point, self.box, self.line_string)
        )
        if active != 1:
            msg = "rendered query geometry must contain exactly one geometry value"
            raise ValueError(msg)
        if self.type is RenderedQueryGeometryType.POINT and self.point is None:
            msg = "point query geometry requires point"
            raise ValueError(msg)
        if self.type is RenderedQueryGeometryType.BOX and self.box is None:
            msg = "box query geometry requires box"
            raise ValueError(msg)
        if (
            self.type is RenderedQueryGeometryType.LINE_STRING
            and self.line_string is None
        ):
            msg = "line-string query geometry requires points"
            raise ValueError(msg)


@dataclass(frozen=True, slots=True)
class RenderedFeatureQueryOptions:
    """Options for rendered feature queries."""

    layer_ids: tuple[str, ...] | None = None
    filter: JsonValue | None = None


@dataclass(frozen=True, slots=True)
class SourceFeatureQueryOptions:
    """Options for source feature queries."""

    source_layer_ids: tuple[str, ...] | None = None
    filter: JsonValue | None = None


@dataclass(frozen=True, slots=True)
class FeatureStateSelector:
    """Source, feature, and state-key selector for render-session feature state."""

    source_id: str
    source_layer_id: str | None = None
    feature_id: str | None = None
    state_key: str | None = None

    def __post_init__(self) -> None:
        if self.state_key is not None and self.feature_id is None:
            msg = "state_key requires feature_id"
            raise ValueError(msg)


@dataclass(frozen=True, slots=True)
class QueriedFeature:
    """One copied query result feature."""

    feature: Feature
    source_id: str | None = None
    source_layer_id: str | None = None
    state: JsonValue | None = None

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "QueriedFeature":
        """Build a queried feature from private native values."""
        return cls(
            feature=raw["feature"],
            source_id=raw.get("source_id"),
            source_layer_id=raw.get("source_layer_id"),
            state=raw.get("state"),
        )


class FeatureExtensionResultType(IntEnum):
    """Feature extension query result variants."""

    VALUE = 1
    FEATURE_COLLECTION = 2
    UNKNOWN = 0


@dataclass(frozen=True, slots=True)
class FeatureExtensionResult:
    """Copied feature-extension query result."""

    type: FeatureExtensionResultType
    value: JsonValue | None = None
    feature_collection: tuple[Feature, ...] | None = None
    raw_type: int | None = None

    @classmethod
    def value_result(cls, value: JsonValue) -> "FeatureExtensionResult":
        """Create a JSON value result."""
        return cls(FeatureExtensionResultType.VALUE, value=value)

    @classmethod
    def feature_collection_result(
        cls,
        features: list[Feature] | tuple[Feature, ...],
    ) -> "FeatureExtensionResult":
        """Create a feature collection result."""
        return cls(
            FeatureExtensionResultType.FEATURE_COLLECTION,
            feature_collection=tuple(features),
        )

    @classmethod
    def unknown_result(cls, raw_type: int) -> "FeatureExtensionResult":
        """Create a result that preserves an unknown native result type."""
        return cls(FeatureExtensionResultType.UNKNOWN, raw_type=raw_type)


__all__ = [
    "FeatureExtensionResult",
    "FeatureExtensionResultType",
    "FeatureStateSelector",
    "QueriedFeature",
    "RenderedFeatureQueryOptions",
    "RenderedQueryGeometry",
    "RenderedQueryGeometryType",
    "ScreenBox",
    "SourceFeatureQueryOptions",
]

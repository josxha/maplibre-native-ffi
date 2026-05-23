"""Offline database operation values and event payloads."""

from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum
from types import TracebackType
from typing import TYPE_CHECKING

from .geo import GeoJson, LatLngBounds

if TYPE_CHECKING:
    from .runtime import RuntimeHandle


class AmbientCacheOperation(IntEnum):
    """Ambient cache maintenance operation kinds."""

    RESET_DATABASE = 1
    PACK_DATABASE = 2
    INVALIDATE = 3
    CLEAR = 4

    @property
    def native_code(self) -> int:
        """Return the C enum value for this operation."""
        return int(self)


class OfflineRegionDefinitionType(IntEnum):
    """Offline region definition descriptor variants."""

    TILE_PYRAMID = 1
    GEOMETRY = 2


class OfflineRegionDownloadState(IntEnum):
    """Offline region download state values."""

    INACTIVE = 0
    ACTIVE = 1

    @classmethod
    def _missing_(cls, value: object) -> "OfflineRegionDownloadState | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown

    @property
    def native_code(self) -> int:
        """Return the C enum value for this download state."""
        return int(self)


class OfflineOperationKind(IntEnum):
    """Offline database operation kinds reported by completion events."""

    AMBIENT_CACHE = 1
    REGION_CREATE = 2
    REGION_GET = 3
    REGIONS_LIST = 4
    REGIONS_MERGE_DATABASE = 5
    REGION_UPDATE_METADATA = 6
    REGION_GET_STATUS = 7
    REGION_SET_OBSERVED = 8
    REGION_SET_DOWNLOAD_STATE = 9
    REGION_INVALIDATE = 10
    REGION_DELETE = 11

    @classmethod
    def _missing_(cls, value: object) -> "OfflineOperationKind | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown

    @property
    def native_code(self) -> int:
        """Return the C enum value for this operation kind."""
        return int(self)


class OfflineOperationResultKind(IntEnum):
    """Offline database operation result kinds reported by completion events."""

    NONE = 0
    REGION = 1
    OPTIONAL_REGION = 2
    REGION_LIST = 3
    REGION_STATUS = 4

    @classmethod
    def _missing_(cls, value: object) -> "OfflineOperationResultKind | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown

    @property
    def native_code(self) -> int:
        """Return the C enum value for this result kind."""
        return int(self)


class OfflineOperationHandle:
    """Runtime-owned offline database operation token."""

    def __init__(self, runtime: "RuntimeHandle", operation_id: int) -> None:
        self._runtime = runtime
        self._operation_id = operation_id
        self._closed = False

    @property
    def operation_id(self) -> int:
        """Return the native offline operation ID."""
        return self._operation_id

    @property
    def closed(self) -> bool:
        """Return whether this operation token has been discarded."""
        return self._closed

    def close(self) -> None:
        """Discard runtime-owned state for this operation."""
        if self._closed:
            return
        self._runtime._native.offline_operation_discard(self._operation_id)  # noqa: SLF001
        self._closed = True

    def __enter__(self) -> "OfflineOperationHandle":
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()


@dataclass(frozen=True, slots=True)
class OfflineRegionStatus:
    """Offline region status snapshot."""

    download_state: OfflineRegionDownloadState
    completed_resource_count: int
    completed_resource_size: int
    completed_tile_count: int
    required_tile_count: int
    completed_tile_size: int
    required_resource_count: int
    required_resource_count_is_precise: bool
    complete: bool


@dataclass(frozen=True, slots=True)
class OfflineTilePyramidRegionDefinition:
    """Tile-pyramid offline region definition."""

    style_url: str
    bounds: LatLngBounds
    min_zoom: float
    max_zoom: float
    pixel_ratio: float
    include_ideographs: bool = True

    @property
    def definition_type(self) -> OfflineRegionDefinitionType:
        """Return this definition variant."""
        return OfflineRegionDefinitionType.TILE_PYRAMID


@dataclass(frozen=True, slots=True)
class OfflineGeometryRegionDefinition:
    """Geometry offline region definition."""

    style_url: str
    geometry: GeoJson
    min_zoom: float
    max_zoom: float
    pixel_ratio: float
    include_ideographs: bool = True

    @property
    def definition_type(self) -> OfflineRegionDefinitionType:
        """Return this definition variant."""
        return OfflineRegionDefinitionType.GEOMETRY


OfflineRegionDefinition = (
    OfflineTilePyramidRegionDefinition | OfflineGeometryRegionDefinition
)


@dataclass(frozen=True, slots=True)
class OfflineRegionInfo:
    """Copied offline region metadata."""

    id: int
    definition: OfflineRegionDefinition
    metadata: bytes


@dataclass(frozen=True, slots=True)
class OfflineOperationCompleted:
    """Offline operation completion event payload."""

    operation_id: int
    operation_kind: OfflineOperationKind
    result_kind: OfflineOperationResultKind
    result_status: int
    found: bool

    @classmethod
    def from_runtime_payload(
        cls, payload: dict[str, object]
    ) -> "OfflineOperationCompleted":
        """Build a completion payload from RuntimeEvent.payload."""
        return cls(
            operation_id=_payload_int(payload, "operation_id"),
            operation_kind=OfflineOperationKind(
                _payload_int(payload, "operation_kind")
            ),
            result_kind=OfflineOperationResultKind(
                _payload_int(payload, "result_kind")
            ),
            result_status=_payload_int(payload, "result_status"),
            found=_payload_bool(payload, "found"),
        )


def _payload_int(payload: dict[str, object], key: str) -> int:
    value = payload[key]
    if not isinstance(value, int):
        msg = f"offline payload {key} must be an int"
        raise TypeError(msg)
    return value


def _payload_bool(payload: dict[str, object], key: str) -> bool:
    value = payload[key]
    if not isinstance(value, bool):
        msg = f"offline payload {key} must be a bool"
        raise TypeError(msg)
    return value


__all__ = [
    "AmbientCacheOperation",
    "OfflineGeometryRegionDefinition",
    "OfflineOperationCompleted",
    "OfflineOperationHandle",
    "OfflineOperationKind",
    "OfflineOperationResultKind",
    "OfflineRegionDefinition",
    "OfflineRegionDefinitionType",
    "OfflineRegionDownloadState",
    "OfflineRegionInfo",
    "OfflineRegionStatus",
    "OfflineTilePyramidRegionDefinition",
]

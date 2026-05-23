"""Offline database operation values and event payloads."""

from __future__ import annotations

from ._lifecycle import warn_unclosed as _warn_unclosed
from dataclasses import dataclass
from enum import IntEnum
from types import TracebackType
from typing import TYPE_CHECKING, Any

from .geo import Geometry, LatLngBounds
from .resource import ResourceErrorReason

if TYPE_CHECKING:
    from .runtime import RuntimeHandle

else:
    RuntimeHandle = Any


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

    def native_code_for_set(self) -> int:
        """Return the C enum value for setter calls, rejecting unknown values."""
        if self.name.startswith("UNKNOWN_"):
            from .errors import InvalidArgumentError

            raise InvalidArgumentError(
                None,
                f"unknown offline region download state cannot be set: {int(self)}",
            )
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
        runtime._register_offline_operation(self)  # noqa: SLF001

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
        self._mark_closed()

    def _mark_closed(self) -> None:
        if self._closed:
            return
        self._closed = True
        self._runtime._unregister_offline_operation(self)  # noqa: SLF001

    def _mark_runtime_closed(self) -> None:
        self._mark_closed()

    def _ensure_open(self) -> None:
        if self._closed:
            from .errors import InvalidStateError

            raise InvalidStateError(None, "offline operation handle is already closed")

    def __del__(self, _warn_unclosed=_warn_unclosed) -> None:
        try:
            _warn_unclosed("OfflineOperationHandle", getattr(self, "closed", True))
        except BaseException:
            return

    def take_region(self) -> "OfflineRegionInfo":
        """Take a completed region snapshot result."""
        self._ensure_open()
        raw = self._runtime._native.offline_region_create_take_result(
            self._operation_id
        )  # noqa: SLF001
        self._mark_closed()
        return OfflineRegionInfo.from_native(raw)

    def take_optional_region(self) -> "OfflineRegionInfo | None":
        """Take a completed optional region snapshot result."""
        self._ensure_open()
        raw = self._runtime._native.offline_region_get_take_result(self._operation_id)  # noqa: SLF001
        self._mark_closed()
        return OfflineRegionInfo.from_native(raw) if raw is not None else None

    def take_region_list(
        self,
        *,
        merge_result: bool = False,
    ) -> tuple["OfflineRegionInfo", ...]:
        """Take a completed region-list result."""
        self._ensure_open()
        if merge_result:
            raw = self._runtime._native.offline_regions_merge_database_take_result(
                self._operation_id
            )  # noqa: SLF001
        else:
            raw = self._runtime._native.offline_regions_list_take_result(
                self._operation_id
            )  # noqa: SLF001
        self._mark_closed()
        return tuple(OfflineRegionInfo.from_native(region) for region in raw)

    def take_updated_region(self) -> "OfflineRegionInfo":
        """Take a completed updated region snapshot result."""
        self._ensure_open()
        raw = self._runtime._native.offline_region_update_metadata_take_result(
            self._operation_id
        )  # noqa: SLF001
        self._mark_closed()
        return OfflineRegionInfo.from_native(raw)

    def take_status(self) -> "OfflineRegionStatus":
        """Take a completed offline region status result."""
        self._ensure_open()
        raw = self._runtime._native.offline_region_get_status_take_result(
            self._operation_id
        )  # noqa: SLF001
        self._mark_closed()
        return OfflineRegionStatus.from_native(raw)

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

    @classmethod
    def from_native(cls, raw: dict[str, object]) -> "OfflineRegionStatus":
        """Build a status snapshot from private native bridge values."""
        return cls(
            download_state=OfflineRegionDownloadState(raw["download_state"]),
            completed_resource_count=raw["completed_resource_count"],
            completed_resource_size=raw["completed_resource_size"],
            completed_tile_count=raw["completed_tile_count"],
            required_tile_count=raw["required_tile_count"],
            completed_tile_size=raw["completed_tile_size"],
            required_resource_count=raw["required_resource_count"],
            required_resource_count_is_precise=raw[
                "required_resource_count_is_precise"
            ],
            complete=raw["complete"],
        )


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
    geometry: Geometry
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

    @classmethod
    def from_native(cls, raw: dict[str, object]) -> "OfflineRegionInfo":
        """Build region metadata from private native bridge values."""
        return cls(
            id=raw["id"],
            definition=_definition_from_native_wire(raw["definition"]),
            metadata=raw["metadata"],
        )


@dataclass(frozen=True, slots=True)
class OfflineRegionStatusChanged:
    """Offline region status-change event payload."""

    region_id: int
    status: OfflineRegionStatus

    @classmethod
    def from_runtime_payload(
        cls, payload: dict[str, object]
    ) -> "OfflineRegionStatusChanged":
        """Build a status-change payload from RuntimeEvent.payload."""
        return cls(
            region_id=_payload_int(payload, "region_id"),
            status=OfflineRegionStatus.from_native(payload["status"]),
        )


@dataclass(frozen=True, slots=True)
class OfflineRegionResponseError:
    """Offline region response-error event payload."""

    region_id: int
    reason: ResourceErrorReason

    @classmethod
    def from_runtime_payload(
        cls, payload: dict[str, object]
    ) -> "OfflineRegionResponseError":
        """Build a response-error payload from RuntimeEvent.payload."""
        from .resource import ResourceErrorReason

        return cls(
            region_id=_payload_int(payload, "region_id"),
            reason=ResourceErrorReason(_payload_int(payload, "reason")),
        )


@dataclass(frozen=True, slots=True)
class OfflineRegionTileCountLimitExceeded:
    """Offline region tile-count-limit event payload."""

    region_id: int
    limit: int

    @classmethod
    def from_runtime_payload(
        cls, payload: dict[str, object]
    ) -> "OfflineRegionTileCountLimitExceeded":
        """Build a tile-count-limit payload from RuntimeEvent.payload."""
        return cls(
            region_id=_payload_int(payload, "region_id"),
            limit=_payload_int(payload, "limit"),
        )


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


def _definition_from_native_wire(raw: dict[str, object]) -> OfflineRegionDefinition:
    kind = raw["type"]
    if kind == "tile_pyramid":
        bounds = raw["bounds"]
        return OfflineTilePyramidRegionDefinition(
            style_url=raw["style_url"],
            bounds=LatLngBounds(
                southwest=_lat_lng_from_native_wire(bounds["southwest"]),
                northeast=_lat_lng_from_native_wire(bounds["northeast"]),
            ),
            min_zoom=raw["min_zoom"],
            max_zoom=raw["max_zoom"],
            pixel_ratio=raw["pixel_ratio"],
            include_ideographs=raw["include_ideographs"],
        )
    if kind == "geometry":
        from .geo import _geometry_from_native_wire

        return OfflineGeometryRegionDefinition(
            style_url=raw["style_url"],
            geometry=_geometry_from_native_wire(raw["geometry"]),
            min_zoom=raw["min_zoom"],
            max_zoom=raw["max_zoom"],
            pixel_ratio=raw["pixel_ratio"],
            include_ideographs=raw["include_ideographs"],
        )
    msg = f"unsupported native offline region definition kind: {kind}"
    raise TypeError(msg)


def _lat_lng_from_native_wire(raw: dict[str, object]):
    from .geo import LatLng

    return LatLng(latitude=raw["latitude"], longitude=raw["longitude"])


def _definition_to_native_wire(
    definition: OfflineRegionDefinition,
) -> dict[str, object]:
    """Convert an offline region definition to private native-bridge values."""
    if isinstance(definition, OfflineTilePyramidRegionDefinition):
        return {
            "type": "tile_pyramid",
            "style_url": definition.style_url,
            "bounds": (
                (
                    definition.bounds.southwest.latitude,
                    definition.bounds.southwest.longitude,
                ),
                (
                    definition.bounds.northeast.latitude,
                    definition.bounds.northeast.longitude,
                ),
            ),
            "min_zoom": definition.min_zoom,
            "max_zoom": definition.max_zoom,
            "pixel_ratio": definition.pixel_ratio,
            "include_ideographs": definition.include_ideographs,
        }
    if isinstance(definition, OfflineGeometryRegionDefinition):
        from .geo import _geometry_to_native_wire

        return {
            "type": "geometry",
            "style_url": definition.style_url,
            "geometry": _geometry_to_native_wire(definition.geometry),
            "min_zoom": definition.min_zoom,
            "max_zoom": definition.max_zoom,
            "pixel_ratio": definition.pixel_ratio,
            "include_ideographs": definition.include_ideographs,
        }
    msg = f"unsupported offline region definition: {type(definition).__name__}"
    raise TypeError(msg)


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
    "OfflineRegionResponseError",
    "OfflineRegionStatus",
    "OfflineRegionStatusChanged",
    "OfflineRegionTileCountLimitExceeded",
    "OfflineTilePyramidRegionDefinition",
]

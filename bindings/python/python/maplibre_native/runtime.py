"""Runtime values and handles for the Python binding."""

from __future__ import annotations

from ._lifecycle import warn_unclosed as _warn_unclosed
from dataclasses import dataclass
from enum import IntEnum
from types import TracebackType
from typing import TYPE_CHECKING, Any

from . import _native

if TYPE_CHECKING:
    from .map import MapHandle, MapOptions
    from .offline import (
        AmbientCacheOperation,
        OfflineOperationHandle,
        OfflineRegionDefinition,
        OfflineRegionDownloadState,
    )
    from .resource import ResourceProviderCallback, ResourceTransformCallback

else:
    MapHandle = Any
    MapOptions = Any
    AmbientCacheOperation = Any
    OfflineOperationHandle = Any
    OfflineRegionDefinition = Any
    OfflineRegionDownloadState = Any
    ResourceProviderCallback = Any
    ResourceTransformCallback = Any


class NetworkStatus(IntEnum):
    """Process-global network reachability state."""

    ONLINE = 1
    OFFLINE = 2

    @classmethod
    def _missing_(cls, value: object) -> "NetworkStatus | None":
        if not isinstance(value, int) or value < 0:
            return None

        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown

    @property
    def native_code(self) -> int:
        """Return the C enum value for this network status."""
        return int(self)

    @property
    def is_unknown(self) -> bool:
        """Return whether this value preserves an unknown native status."""
        return self.name.startswith("UNKNOWN_")


class RuntimeEventType(IntEnum):
    """Runtime event type values reported by the C API."""

    MAP_CAMERA_WILL_CHANGE = 1
    MAP_CAMERA_IS_CHANGING = 2
    MAP_CAMERA_DID_CHANGE = 3
    MAP_STYLE_LOADED = 4
    MAP_LOADING_STARTED = 5
    MAP_LOADING_FINISHED = 6
    MAP_LOADING_FAILED = 7
    MAP_IDLE = 8
    MAP_RENDER_UPDATE_AVAILABLE = 9
    MAP_RENDER_ERROR = 10
    MAP_STILL_IMAGE_FINISHED = 11
    MAP_STILL_IMAGE_FAILED = 12
    MAP_RENDER_FRAME_STARTED = 13
    MAP_RENDER_FRAME_FINISHED = 14
    MAP_RENDER_MAP_STARTED = 15
    MAP_RENDER_MAP_FINISHED = 16
    MAP_STYLE_IMAGE_MISSING = 17
    MAP_TILE_ACTION = 18
    OFFLINE_REGION_STATUS_CHANGED = 19
    OFFLINE_REGION_RESPONSE_ERROR = 20
    OFFLINE_REGION_TILE_COUNT_LIMIT_EXCEEDED = 21
    OFFLINE_OPERATION_COMPLETED = 22

    @classmethod
    def _missing_(cls, value: object) -> "RuntimeEventType | None":
        if not isinstance(value, int) or value < 0:
            return None

        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown

    @property
    def native_code(self) -> int:
        """Return the C enum value for this runtime event type."""
        return int(self)

    @property
    def is_unknown(self) -> bool:
        """Return whether this value preserves an unknown native event type."""
        return self.name.startswith("UNKNOWN_")


class RuntimeEventSourceType(IntEnum):
    """Runtime event source kind values reported by the C API."""

    RUNTIME = 0
    MAP = 1

    @classmethod
    def _missing_(cls, value: object) -> "RuntimeEventSourceType | None":
        if not isinstance(value, int) or value < 0:
            return None

        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown


class RenderMode(IntEnum):
    """Render modes reported by runtime render events."""

    PARTIAL = 0
    FULL = 1

    @classmethod
    def _missing_(cls, value: object) -> "RenderMode | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown

    @property
    def native_code(self) -> int:
        """Return the C enum value for this render mode."""
        return int(self)


class TileOperation(IntEnum):
    """Tile operations reported by runtime tile events."""

    REQUESTED_FROM_CACHE = 0
    REQUESTED_FROM_NETWORK = 1
    LOAD_FROM_NETWORK = 2
    LOAD_FROM_CACHE = 3
    START_PARSE = 4
    END_PARSE = 5
    ERROR = 6
    CANCELLED = 7
    NULL = 8

    @classmethod
    def _missing_(cls, value: object) -> "TileOperation | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown

    @property
    def native_code(self) -> int:
        """Return the C enum value for this tile operation."""
        return int(self)


@dataclass(frozen=True, slots=True)
class RenderingStats:
    """Rendering statistics copied from a render-frame event."""

    encoding_time: float
    rendering_time: float
    frame_count: int
    draw_call_count: int
    total_draw_call_count: int

    @classmethod
    def from_native(cls, raw: dict[str, object]) -> "RenderingStats":
        """Build rendering statistics from private native bridge values."""
        return cls(
            encoding_time=raw["encoding_time"],
            rendering_time=raw["rendering_time"],
            frame_count=raw["frame_count"],
            draw_call_count=raw["draw_call_count"],
            total_draw_call_count=raw["total_draw_call_count"],
        )


@dataclass(frozen=True, slots=True)
class RenderFramePayload:
    """Runtime render-frame event payload."""

    mode: RenderMode
    needs_repaint: bool
    placement_changed: bool
    stats: RenderingStats

    @classmethod
    def from_runtime_payload(cls, payload: dict[str, object]) -> "RenderFramePayload":
        """Build a render-frame payload from RuntimeEvent.payload."""
        return cls(
            mode=RenderMode(payload["mode"]),
            needs_repaint=payload["needs_repaint"],
            placement_changed=payload["placement_changed"],
            stats=RenderingStats.from_native(payload["stats"]),
        )


@dataclass(frozen=True, slots=True)
class RenderMapPayload:
    """Runtime render-map event payload."""

    mode: RenderMode

    @classmethod
    def from_runtime_payload(cls, payload: dict[str, object]) -> "RenderMapPayload":
        """Build a render-map payload from RuntimeEvent.payload."""
        return cls(mode=RenderMode(payload["mode"]))


@dataclass(frozen=True, slots=True)
class StyleImageMissingPayload:
    """Runtime style-image-missing event payload."""

    image_id: str

    @classmethod
    def from_runtime_payload(
        cls, payload: dict[str, object]
    ) -> "StyleImageMissingPayload":
        """Build a style-image-missing payload from RuntimeEvent.payload."""
        return cls(image_id=payload["image_id"])


@dataclass(frozen=True, slots=True)
class TileId:
    """Overscaled tile identity copied from a tile-action event."""

    overscaled_z: int
    wrap: int
    canonical_z: int
    canonical_x: int
    canonical_y: int

    @classmethod
    def from_native(cls, raw: dict[str, object]) -> "TileId":
        """Build a tile identity from private native bridge values."""
        return cls(
            overscaled_z=raw["overscaled_z"],
            wrap=raw["wrap"],
            canonical_z=raw["canonical_z"],
            canonical_x=raw["canonical_x"],
            canonical_y=raw["canonical_y"],
        )


@dataclass(frozen=True, slots=True)
class TileActionPayload:
    """Runtime tile-action event payload."""

    operation: TileOperation
    tile_id: TileId
    source_id: str

    @classmethod
    def from_runtime_payload(cls, payload: dict[str, object]) -> "TileActionPayload":
        """Build a tile-action payload from RuntimeEvent.payload."""
        return cls(
            operation=TileOperation(payload["operation"]),
            tile_id=TileId.from_native(payload["tile_id"]),
            source_id=payload["source_id"],
        )


@dataclass(frozen=True, slots=True)
class RuntimeEventSource:
    """Copied runtime event source metadata."""

    source_type: RuntimeEventSourceType
    source_address: int


@dataclass(frozen=True, slots=True)
class RuntimeEvent:
    """Runtime event copied into Python-owned values."""

    event_type: RuntimeEventType
    source: RuntimeEventSource
    code: int
    message: str | None
    payload: dict[str, Any]

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "RuntimeEvent":
        """Build a copied Python event from a private native event dictionary."""
        return cls(
            event_type=RuntimeEventType(raw["event_type"]),
            source=RuntimeEventSource(
                source_type=RuntimeEventSourceType(raw["source_type"]),
                source_address=raw["source_address"],
            ),
            code=raw["code"],
            message=raw["message"],
            payload=dict(raw["payload"]),
        )


@dataclass(slots=True)
class RuntimeOptions:
    """Options used when creating a runtime."""

    asset_path: str | None = None
    cache_path: str | None = None
    maximum_cache_size: int | None = None


class RuntimeHandle:
    """Owner-thread runtime handle."""

    def __init__(self, options: RuntimeOptions | None = None) -> None:
        options = options or RuntimeOptions()
        self._native = _native.create_runtime(
            options.asset_path,
            options.cache_path,
            options.maximum_cache_size,
        )

    @property
    def closed(self) -> bool:
        """Return whether this handle has been closed."""
        return bool(self._native.closed)

    def close(self) -> None:
        """Release this runtime handle exactly once."""
        self._native.close()

    def __del__(self, _warn_unclosed=_warn_unclosed) -> None:
        try:
            _warn_unclosed("RuntimeHandle", getattr(self, "closed", True))
        except BaseException:
            return

    def run_once(self) -> None:
        """Run one pending owner-thread task for this runtime."""
        self._native.run_once()

    def run_ambient_cache_operation(
        self, operation: AmbientCacheOperation
    ) -> OfflineOperationHandle:
        """Start an ambient cache maintenance operation."""
        from .offline import AmbientCacheOperation, OfflineOperationHandle

        operation_id = self._native.run_ambient_cache_operation_start(
            AmbientCacheOperation(operation).native_code
        )
        return OfflineOperationHandle(self, operation_id)

    def create_offline_region(
        self, definition: OfflineRegionDefinition, metadata: bytes = b""
    ) -> OfflineOperationHandle:
        """Start creating an offline region."""
        from .offline import OfflineOperationHandle, _definition_to_native_wire

        operation_id = self._native.offline_region_create_start(
            _definition_to_native_wire(definition),
            metadata,
        )
        return OfflineOperationHandle(self, operation_id)

    def get_offline_region(self, region_id: int) -> OfflineOperationHandle:
        """Start getting an offline region snapshot by ID."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_region_get_start(region_id),
        )

    def list_offline_regions(self) -> OfflineOperationHandle:
        """Start listing offline region snapshots."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(self, self._native.offline_regions_list_start())

    def merge_offline_regions_database(
        self, side_database_path: str
    ) -> OfflineOperationHandle:
        """Start merging offline regions from another database path."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_regions_merge_database_start(side_database_path),
        )

    def update_offline_region_metadata(
        self,
        region_id: int,
        metadata: bytes,
    ) -> OfflineOperationHandle:
        """Start updating opaque binary metadata for an offline region."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_region_update_metadata_start(region_id, metadata),
        )

    def get_offline_region_status(self, region_id: int) -> OfflineOperationHandle:
        """Start getting completed/download status for an offline region."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_region_get_status_start(region_id),
        )

    def set_offline_region_observed(
        self, region_id: int, observed: bool
    ) -> OfflineOperationHandle:
        """Start enabling or disabling runtime events for an offline region."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_region_set_observed_start(region_id, observed),
        )

    def set_offline_region_download_state(
        self,
        region_id: int,
        state: OfflineRegionDownloadState,
    ) -> OfflineOperationHandle:
        """Start setting an offline region's native download state."""
        from .offline import OfflineOperationHandle, OfflineRegionDownloadState

        return OfflineOperationHandle(
            self,
            self._native.offline_region_set_download_state_start(
                region_id,
                OfflineRegionDownloadState(state).native_code_for_set(),
            ),
        )

    def invalidate_offline_region(self, region_id: int) -> OfflineOperationHandle:
        """Start invalidating cached resources for an offline region."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_region_invalidate_start(region_id),
        )

    def delete_offline_region(self, region_id: int) -> OfflineOperationHandle:
        """Start deleting an offline region."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_region_delete_start(region_id),
        )

    def set_resource_transform(
        self,
        callback: ResourceTransformCallback,
        *,
        max_pending_callbacks: int = 64,
    ) -> None:
        """Install or replace the runtime-scoped network URL transform."""
        from .resource import adapt_resource_transform_callback

        self._native.set_resource_transform(
            adapt_resource_transform_callback(callback),
            max_pending_callbacks,
        )

    def clear_resource_transform(self) -> None:
        """Clear the runtime-scoped network URL transform."""
        self._native.clear_resource_transform()

    def set_resource_provider(
        self,
        callback: ResourceProviderCallback,
        *,
        max_pending_callbacks: int = 64,
    ) -> None:
        """Install or replace the runtime-scoped network resource provider."""
        from .resource import adapt_resource_provider_callback

        self._native.set_resource_provider(
            adapt_resource_provider_callback(callback),
            max_pending_callbacks,
        )

    def poll_event(self) -> RuntimeEvent | None:
        """Poll and copy one queued runtime event."""
        event = self._native.poll_event()
        if event is None:
            return None
        return RuntimeEvent.from_native(event)

    def create_map(self, options: MapOptions | None = None) -> MapHandle:
        """Create a map owned by this runtime."""
        from .map import MapHandle

        return MapHandle(self, options)

    def __enter__(self) -> "RuntimeHandle":
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()


__all__ = [
    "NetworkStatus",
    "RenderFramePayload",
    "RenderMapPayload",
    "RenderMode",
    "RenderingStats",
    "RuntimeEvent",
    "RuntimeEventSource",
    "RuntimeEventSourceType",
    "RuntimeEventType",
    "RuntimeHandle",
    "RuntimeOptions",
    "StyleImageMissingPayload",
    "TileActionPayload",
    "TileId",
    "TileOperation",
]

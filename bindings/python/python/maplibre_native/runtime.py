"""Runtime values and handles for the Python binding."""

from dataclasses import dataclass
from enum import IntEnum
from types import TracebackType
from typing import Any, cast

from . import _native


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

    def run_once(self) -> None:
        """Run one pending owner-thread task for this runtime."""
        self._native.run_once()

    def run_ambient_cache_operation(self, operation: object) -> object:
        """Start an ambient cache maintenance operation."""
        from .offline import AmbientCacheOperation, OfflineOperationHandle

        operation_id = self._native.run_ambient_cache_operation_start(
            AmbientCacheOperation(operation).native_code
        )
        return OfflineOperationHandle(self, operation_id)

    def create_offline_region(
        self, definition: object, metadata: bytes = b""
    ) -> object:
        """Start creating an offline region."""
        from .offline import OfflineOperationHandle, _definition_to_native_wire

        operation_id = self._native.offline_region_create_start(
            _definition_to_native_wire(definition),
            metadata,
        )
        return OfflineOperationHandle(self, operation_id)

    def get_offline_region(self, region_id: int) -> object:
        """Start getting an offline region snapshot by ID."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_region_get_start(region_id),
        )

    def list_offline_regions(self) -> object:
        """Start listing offline region snapshots."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(self, self._native.offline_regions_list_start())

    def merge_offline_regions_database(self, side_database_path: str) -> object:
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
    ) -> object:
        """Start updating opaque binary metadata for an offline region."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_region_update_metadata_start(region_id, metadata),
        )

    def get_offline_region_status(self, region_id: int) -> object:
        """Start getting completed/download status for an offline region."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_region_get_status_start(region_id),
        )

    def set_offline_region_observed(self, region_id: int, observed: bool) -> object:
        """Start enabling or disabling runtime events for an offline region."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_region_set_observed_start(region_id, observed),
        )

    def set_offline_region_download_state(
        self,
        region_id: int,
        state: object,
    ) -> object:
        """Start setting an offline region's native download state."""
        from .offline import OfflineOperationHandle, OfflineRegionDownloadState

        return OfflineOperationHandle(
            self,
            self._native.offline_region_set_download_state_start(
                region_id,
                OfflineRegionDownloadState(state).native_code,
            ),
        )

    def invalidate_offline_region(self, region_id: int) -> object:
        """Start invalidating cached resources for an offline region."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_region_invalidate_start(region_id),
        )

    def delete_offline_region(self, region_id: int) -> object:
        """Start deleting an offline region."""
        from .offline import OfflineOperationHandle

        return OfflineOperationHandle(
            self,
            self._native.offline_region_delete_start(region_id),
        )

    def set_resource_transform(
        self,
        callback: object,
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
        callback: object,
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

    def create_map(self, options: object | None = None) -> object:
        """Create a map owned by this runtime."""
        from .map import MapHandle, MapOptions

        return MapHandle(self, cast("MapOptions | None", options))

    def __enter__(self) -> "RuntimeHandle":
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()

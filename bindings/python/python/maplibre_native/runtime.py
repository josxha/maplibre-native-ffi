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

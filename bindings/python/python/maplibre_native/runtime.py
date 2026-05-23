"""Runtime values and handles for the Python binding."""

from dataclasses import dataclass
from enum import IntEnum
from types import TracebackType

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

    def create_map(self, options: object | None = None) -> object:
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

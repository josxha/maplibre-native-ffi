"""Map handles, options, and map lifecycle entry points."""

from dataclasses import dataclass
from enum import IntEnum
from types import TracebackType

from . import _native
from .runtime import RuntimeHandle


class MapMode(IntEnum):
    """Map rendering mode used when creating a map."""

    CONTINUOUS = 0
    STATIC = 1
    TILE = 2

    @property
    def native_code(self) -> int:
        """Return the C enum value for this map mode."""
        return int(self)


@dataclass(slots=True)
class MapOptions:
    """Options used when creating a map."""

    width: int = 64
    height: int = 64
    scale_factor: float = 1.0
    mode: MapMode = MapMode.CONTINUOUS


class MapHandle:
    """Owner-thread map handle."""

    def __init__(
        self,
        runtime: RuntimeHandle,
        options: MapOptions | None = None,
    ) -> None:
        options = options or MapOptions()
        map_mode = (
            options.mode if isinstance(options.mode, MapMode) else MapMode(options.mode)
        )
        self._runtime = runtime
        self._native = _native.create_map(
            runtime._native,
            options.width,
            options.height,
            options.scale_factor,
            map_mode.native_code,
        )

    @property
    def closed(self) -> bool:
        """Return whether this handle has been closed."""
        return bool(self._native.closed)

    def close(self) -> None:
        """Release this map handle exactly once."""
        self._native.close()

    def request_repaint(self) -> None:
        """Request a repaint for a continuous map."""
        self._native.request_repaint()

    def set_style_json(self, json: str) -> None:
        """Load inline style JSON through MapLibre Native style APIs."""
        self._native.set_style_json(json)

    def __enter__(self) -> "MapHandle":
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()

"""Render target values and backend interop helpers."""

from dataclasses import dataclass
from enum import IntFlag


class RenderBackend(IntFlag):
    """Render backend support bits reported by the native library."""

    NONE = 0
    METAL = 1 << 0
    VULKAN = 1 << 1


@dataclass(frozen=True, slots=True)
class NativePointer:
    """Borrowed opaque backend-native address value."""

    address: int

    def __post_init__(self) -> None:
        if self.address < 0:
            msg = "native pointer address must be non-negative"
            raise ValueError(msg)

    @classmethod
    def null(cls) -> "NativePointer":
        """Return a null native pointer value."""
        return cls(0)

    @property
    def is_null(self) -> bool:
        """Return whether this pointer stores the null address."""
        return self.address == 0

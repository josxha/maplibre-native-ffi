"""Runtime values and handles for the Python binding."""

from enum import IntEnum


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

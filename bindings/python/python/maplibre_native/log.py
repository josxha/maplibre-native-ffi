"""Process-global logging callback APIs."""

from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum, IntFlag
from typing import Any

from . import _native


class LogSeverity(IntEnum):
    """Log severity values emitted by MapLibre Native."""

    INFO = 1
    WARNING = 2
    ERROR = 3

    @classmethod
    def _missing_(cls, value: object) -> "LogSeverity | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown

    @property
    def native_code(self) -> int:
        """Return the C enum value for this severity."""
        return int(self)


class LogSeverityMask(IntFlag):
    """Mask of severities MapLibre Native may dispatch asynchronously."""

    INFO = 1 << LogSeverity.INFO
    WARNING = 1 << LogSeverity.WARNING
    ERROR = 1 << LogSeverity.ERROR
    DEFAULT = INFO | WARNING
    ALL = INFO | WARNING | ERROR


class LogEvent(IntEnum):
    """Log event categories emitted by MapLibre Native."""

    GENERAL = 0
    SETUP = 1
    SHADER = 2
    PARSE_STYLE = 3
    PARSE_TILE = 4
    RENDER = 5
    STYLE = 6
    DATABASE = 7
    HTTP_REQUEST = 8
    SPRITE = 9
    IMAGE = 10
    OPENGL = 11
    JNI = 12
    ANDROID = 13
    CRASH = 14
    GLYPH = 15
    TIMING = 16

    @classmethod
    def _missing_(cls, value: object) -> "LogEvent | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown

    @property
    def native_code(self) -> int:
        """Return the C enum value for this event."""
        return int(self)


@dataclass(frozen=True, slots=True)
class LogRecord:
    """Copied MapLibre Native log record."""

    severity: LogSeverity
    event: LogEvent
    code: int
    message: str

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "LogRecord":
        """Build a copied log record from private native values."""
        return cls(
            severity=LogSeverity(raw["severity"]),
            event=LogEvent(raw["event"]),
            code=raw["code"],
            message=raw["message"],
        )


class LogReceiver:
    """Bounded queue receiving copied process-global native log records."""

    def __init__(self, native: Any) -> None:
        self._native = native

    @property
    def dropped_record_count(self) -> int:
        """Return records dropped because the queue was full."""
        return int(self._native.dropped_record_count)

    def poll_record(self) -> LogRecord | None:
        """Return one copied log record from the receiver queue."""
        record = self._native.poll_record()
        if record is None:
            return None
        return LogRecord.from_native(record)


def set_log_callback(
    *,
    max_queued_records: int = 1024,
    consume: bool = False,
) -> LogReceiver:
    """Install a process-global bounded log-record receiver."""
    return LogReceiver(_native.set_log_callback(max_queued_records, consume))


def clear_log_callback() -> None:
    """Clear the process-global native log callback."""
    _native.clear_log_callback()


def set_async_log_severity_mask(mask: LogSeverityMask) -> None:
    """Configure severities that may dispatch asynchronously."""
    _native.set_async_log_severity_mask(int(mask))


__all__ = [
    "LogEvent",
    "LogReceiver",
    "LogRecord",
    "LogSeverity",
    "LogSeverityMask",
    "clear_log_callback",
    "set_async_log_severity_mask",
    "set_log_callback",
]

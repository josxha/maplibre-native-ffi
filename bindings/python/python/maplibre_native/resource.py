"""Resource request, response, transform, and provider APIs."""

from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum
from typing import Any, Callable


class ResourceKind(IntEnum):
    """Network resource kind passed to resource callbacks."""

    UNKNOWN = 0
    STYLE = 1
    SOURCE = 2
    TILE = 3
    GLYPHS = 4
    SPRITE_IMAGE = 5
    SPRITE_JSON = 6
    IMAGE = 7

    @classmethod
    def _missing_(cls, value: object) -> "ResourceKind | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown

    @property
    def native_code(self) -> int:
        """Return the C enum value for this resource kind."""
        return int(self)


class ResourceLoadingMethod(IntEnum):
    """Resource loading method requested by MapLibre Native."""

    ALL = 0
    CACHE_ONLY = 1
    NETWORK_ONLY = 2

    @classmethod
    def _missing_(cls, value: object) -> "ResourceLoadingMethod | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown


class ResourcePriority(IntEnum):
    """Resource request priority."""

    REGULAR = 0
    LOW = 1

    @classmethod
    def _missing_(cls, value: object) -> "ResourcePriority | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown


class ResourceUsage(IntEnum):
    """Resource request usage."""

    ONLINE = 0
    OFFLINE = 1

    @classmethod
    def _missing_(cls, value: object) -> "ResourceUsage | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown


class ResourceStoragePolicy(IntEnum):
    """Resource cache storage policy."""

    PERMANENT = 0
    VOLATILE = 1

    @classmethod
    def _missing_(cls, value: object) -> "ResourceStoragePolicy | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown


class ResourceResponseStatus(IntEnum):
    """Status for a resource provider response."""

    OK = 0
    ERROR = 1
    NO_CONTENT = 2
    NOT_MODIFIED = 3

    @property
    def native_code(self) -> int:
        """Return the C enum value for this response status."""
        return int(self)


class ResourceErrorReason(IntEnum):
    """Resource error reason used in provider responses and events."""

    NONE = 0
    NOT_FOUND = 1
    SERVER = 2
    CONNECTION = 3
    RATE_LIMIT = 4
    OTHER = 5

    @classmethod
    def _missing_(cls, value: object) -> "ResourceErrorReason | None":
        if not isinstance(value, int) or value < 0:
            return None
        unknown = int.__new__(cls, value)
        unknown._name_ = f"UNKNOWN_{value}"
        unknown._value_ = value
        return unknown

    @property
    def native_code(self) -> int:
        """Return the C enum value for this error reason."""
        return int(self)


class ResourceProviderDecision(IntEnum):
    """Decision returned by a resource provider callback."""

    PASS_THROUGH = 0
    HANDLE = 1

    @property
    def native_code(self) -> int:
        """Return the C enum value for this provider decision."""
        return int(self)


@dataclass(frozen=True, slots=True)
class ByteRange:
    """Byte range requested for a network resource."""

    start: int
    end: int

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "ByteRange":
        """Build a byte range from private native values."""
        return cls(start=raw["start"], end=raw["end"])


@dataclass(frozen=True, slots=True)
class ResourceRequest:
    """Copied request passed to a resource provider callback."""

    url: str
    kind: ResourceKind
    loading_method: ResourceLoadingMethod
    priority: ResourcePriority
    usage: ResourceUsage
    storage_policy: ResourceStoragePolicy
    range: ByteRange | None
    prior_modified_unix_ms: int | None
    prior_expires_unix_ms: int | None
    prior_etag: str | None
    prior_data: bytes

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "ResourceRequest":
        """Build a copied request from private native values."""
        raw_range = raw["range"]
        return cls(
            url=raw["url"],
            kind=ResourceKind(raw["kind"]),
            loading_method=ResourceLoadingMethod(raw["loading_method"]),
            priority=ResourcePriority(raw["priority"]),
            usage=ResourceUsage(raw["usage"]),
            storage_policy=ResourceStoragePolicy(raw["storage_policy"]),
            range=ByteRange.from_native(raw_range) if raw_range is not None else None,
            prior_modified_unix_ms=raw["prior_modified_unix_ms"],
            prior_expires_unix_ms=raw["prior_expires_unix_ms"],
            prior_etag=raw["prior_etag"],
            prior_data=raw["prior_data"],
        )


@dataclass(frozen=True, slots=True)
class ResourceTransformRequest:
    """Copied request passed to a resource transform callback."""

    kind: ResourceKind
    url: str

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "ResourceTransformRequest":
        """Build a copied transform request from private native values."""
        return cls(kind=ResourceKind(raw["kind"]), url=raw["url"])


@dataclass(frozen=True, slots=True)
class ResourceResponse:
    """Response used to complete a handled resource request."""

    status: ResourceResponseStatus = ResourceResponseStatus.OK
    error_reason: ResourceErrorReason = ResourceErrorReason.NONE
    bytes: bytes = b""
    error_message: str | None = None
    must_revalidate: bool = False
    modified_unix_ms: int | None = None
    expires_unix_ms: int | None = None
    etag: str | None = None
    retry_after_unix_ms: int | None = None

    @classmethod
    def ok(cls, bytes_: bytes = b"") -> "ResourceResponse":
        """Return a successful response with optional bytes."""
        return cls(status=ResourceResponseStatus.OK, bytes=bytes_)

    @classmethod
    def no_content(cls) -> "ResourceResponse":
        """Return a successful no-content response."""
        return cls(status=ResourceResponseStatus.NO_CONTENT)

    @classmethod
    def not_modified(cls) -> "ResourceResponse":
        """Return a successful not-modified response."""
        return cls(status=ResourceResponseStatus.NOT_MODIFIED)

    @classmethod
    def error(cls, reason: ResourceErrorReason, message: str) -> "ResourceResponse":
        """Return an error response."""
        return cls(
            status=ResourceResponseStatus.ERROR,
            error_reason=reason,
            error_message=message,
        )

    def to_native(self) -> dict[str, Any]:
        """Return private native bridge values for this response."""
        return {
            "status": self.status.native_code,
            "error_reason": self.error_reason.native_code,
            "bytes": self.bytes,
            "error_message": self.error_message,
            "must_revalidate": self.must_revalidate,
            "modified_unix_ms": self.modified_unix_ms,
            "expires_unix_ms": self.expires_unix_ms,
            "etag": self.etag,
            "retry_after_unix_ms": self.retry_after_unix_ms,
        }


class ResourceRequestHandle:
    """One-shot handle for a resource provider request selected for handling."""

    def __init__(self, native: Any) -> None:
        self._native = native

    def complete(self, response: ResourceResponse) -> None:
        """Complete this request with a copied response."""
        self._native.complete(response.to_native())

    def is_cancelled(self) -> bool:
        """Return whether native code has cancelled the request."""
        return bool(self._native.is_cancelled())

    def close(self) -> None:
        """Release this request handle without completing it."""
        self._native.close()


ResourceTransformCallback = Callable[[ResourceTransformRequest], str | None]
ResourceProviderCallback = Callable[
    [ResourceRequest, ResourceRequestHandle], ResourceProviderDecision
]


def adapt_resource_transform_callback(
    callback: ResourceTransformCallback,
) -> Callable[[dict[str, Any]], str | None]:
    """Adapt a public resource transform callback for the native bridge."""

    def adapted(raw_request: dict[str, Any]) -> str | None:
        return callback(ResourceTransformRequest.from_native(raw_request))

    return adapted


def adapt_resource_provider_callback(
    callback: ResourceProviderCallback,
) -> Callable[[dict[str, Any], Any], int]:
    """Adapt a public resource provider callback for the native bridge."""

    def adapted(raw_request: dict[str, Any], native_handle: Any) -> int:
        decision = callback(
            ResourceRequest.from_native(raw_request),
            ResourceRequestHandle(native_handle),
        )
        return ResourceProviderDecision(decision).native_code

    return adapted


__all__ = [
    "ByteRange",
    "ResourceErrorReason",
    "ResourceKind",
    "ResourceLoadingMethod",
    "ResourcePriority",
    "ResourceProviderCallback",
    "ResourceProviderDecision",
    "ResourceRequest",
    "ResourceRequestHandle",
    "ResourceResponse",
    "ResourceResponseStatus",
    "ResourceStoragePolicy",
    "ResourceTransformCallback",
    "ResourceTransformRequest",
    "ResourceUsage",
]

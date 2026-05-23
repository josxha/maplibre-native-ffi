"""Render target values and backend interop helpers."""

from __future__ import annotations

from ._lifecycle import warn_unclosed as _warn_unclosed
from dataclasses import dataclass
from enum import IntFlag
from types import TracebackType
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from .geo import Feature
    from .json import JsonLike, JsonObjectLike, JsonValue
    from .map import MapHandle
    from .query import (
        FeatureExtensionResult,
        FeatureStateSelector,
        QueriedFeature,
        RenderedFeatureQueryOptions,
        RenderedQueryGeometry,
        SourceFeatureQueryOptions,
    )


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


@dataclass(frozen=True, slots=True)
class RenderTargetExtent:
    """Logical render target size in UI pixels."""

    width: int = 256
    height: int = 256
    scale_factor: float = 1.0


@dataclass(frozen=True, slots=True)
class MetalContextDescriptor:
    """Borrowed Metal context values shared by Metal render targets."""

    device: NativePointer = NativePointer(0)


@dataclass(frozen=True, slots=True)
class VulkanContextDescriptor:
    """Borrowed Vulkan context values shared by Vulkan render targets."""

    instance: NativePointer = NativePointer(0)
    physical_device: NativePointer = NativePointer(0)
    device: NativePointer = NativePointer(0)
    graphics_queue: NativePointer = NativePointer(0)
    graphics_queue_family_index: int = 0


@dataclass(frozen=True, slots=True)
class MetalSurfaceDescriptor:
    """Metal native surface attachment descriptor."""

    extent: RenderTargetExtent = RenderTargetExtent()
    context: MetalContextDescriptor = MetalContextDescriptor()
    layer: NativePointer = NativePointer(0)


@dataclass(frozen=True, slots=True)
class VulkanSurfaceDescriptor:
    """Vulkan native surface attachment descriptor."""

    extent: RenderTargetExtent = RenderTargetExtent()
    context: VulkanContextDescriptor = VulkanContextDescriptor()
    surface: NativePointer = NativePointer(0)


@dataclass(frozen=True, slots=True)
class MetalOwnedTextureDescriptor:
    """Metal session-owned texture attachment descriptor."""

    extent: RenderTargetExtent = RenderTargetExtent()
    context: MetalContextDescriptor = MetalContextDescriptor()


@dataclass(frozen=True, slots=True)
class MetalBorrowedTextureDescriptor:
    """Metal caller-owned texture attachment descriptor."""

    extent: RenderTargetExtent = RenderTargetExtent()
    texture: NativePointer = NativePointer(0)


@dataclass(frozen=True, slots=True)
class VulkanOwnedTextureDescriptor:
    """Vulkan session-owned texture attachment descriptor."""

    extent: RenderTargetExtent = RenderTargetExtent()
    context: VulkanContextDescriptor = VulkanContextDescriptor()


@dataclass(frozen=True, slots=True)
class VulkanBorrowedTextureDescriptor:
    """Vulkan caller-owned texture attachment descriptor."""

    extent: RenderTargetExtent = RenderTargetExtent()
    context: VulkanContextDescriptor = VulkanContextDescriptor()
    image: NativePointer = NativePointer(0)
    image_view: NativePointer = NativePointer(0)
    format: int = 0
    initial_layout: int = 0
    final_layout: int = 0


@dataclass(frozen=True, slots=True)
class TextureImageInfo:
    """CPU image readback metadata for a texture session frame."""

    width: int
    height: int
    stride: int
    byte_length: int

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "TextureImageInfo":
        """Build metadata from private native values."""
        return cls(
            width=raw["width"],
            height=raw["height"],
            stride=raw["stride"],
            byte_length=raw["byte_length"],
        )


@dataclass(frozen=True, slots=True)
class PremultipliedRgba8Image:
    """Copied premultiplied RGBA8 image bytes and metadata."""

    info: TextureImageInfo
    data: bytes

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "PremultipliedRgba8Image":
        """Build an image from private native values."""
        return cls(info=TextureImageInfo.from_native(raw["info"]), data=raw["data"])


@dataclass(frozen=True, slots=True)
class MetalOwnedTextureFrame:
    """Copied metadata for an acquired Metal session-owned texture frame."""

    generation: int
    width: int
    height: int
    scale_factor: float
    frame_id: int
    pixel_format: int

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "MetalOwnedTextureFrame":
        """Build frame metadata from private native values."""
        return cls(
            generation=raw["generation"],
            width=raw["width"],
            height=raw["height"],
            scale_factor=raw["scale_factor"],
            frame_id=raw["frame_id"],
            pixel_format=raw["pixel_format"],
        )


@dataclass(frozen=True, slots=True)
class VulkanOwnedTextureFrame:
    """Copied metadata for an acquired Vulkan session-owned texture frame."""

    generation: int
    width: int
    height: int
    scale_factor: float
    frame_id: int
    format: int
    layout: int

    @classmethod
    def from_native(cls, raw: dict[str, Any]) -> "VulkanOwnedTextureFrame":
        """Build frame metadata from private native values."""
        return cls(
            generation=raw["generation"],
            width=raw["width"],
            height=raw["height"],
            scale_factor=raw["scale_factor"],
            frame_id=raw["frame_id"],
            format=raw["format"],
            layout=raw["layout"],
        )


class DetachedRenderSessionHandle:
    """Close-only render session handle after backend resources detach."""

    def __init__(self, native: Any) -> None:
        self._native = native

    @property
    def closed(self) -> bool:
        """Return whether this detached handle has been closed."""
        return bool(self._native.closed)

    def close(self) -> None:
        """Release this detached render session exactly once."""
        self._native.close()

    def __del__(self, _warn_unclosed=_warn_unclosed) -> None:
        try:
            _warn_unclosed("DetachedRenderSessionHandle", getattr(self, "closed", True))
        except BaseException:
            return

    def __enter__(self) -> "DetachedRenderSessionHandle":
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()


class RenderSessionHandle:
    """Owner-thread render session handle bound to a retained map wrapper."""

    def __init__(self, native: Any, map_handle: MapHandle) -> None:
        self._native = native
        self._map = map_handle

    @property
    def closed(self) -> bool:
        """Return whether this render session has been closed."""
        return bool(self._native.closed)

    @property
    def detached(self) -> bool:
        """Return whether backend resources have been detached."""
        return bool(self._native.detached)

    def close(self) -> None:
        """Release this render session exactly once."""
        self._native.close()

    def __del__(self, _warn_unclosed=_warn_unclosed) -> None:
        try:
            _warn_unclosed("RenderSessionHandle", getattr(self, "closed", True))
        except BaseException:
            return

    def resize(self, width: int, height: int, scale_factor: float) -> None:
        """Resize this attached render session."""
        self._native.resize(width, height, scale_factor)

    def render_update(self) -> None:
        """Process the latest map render update for this target."""
        self._native.render_update()

    def detach(self) -> DetachedRenderSessionHandle:
        """Detach backend resources and return a close-only handle."""
        return DetachedRenderSessionHandle(self._native.detach())

    def reduce_memory_use(self) -> None:
        """Ask the session renderer to release cached resources where possible."""
        self._native.reduce_memory_use()

    def clear_data(self) -> None:
        """Clear renderer data for the session."""
        self._native.clear_data()

    def dump_debug_logs(self) -> None:
        """Dump renderer debug logs through MapLibre Native logging."""
        self._native.dump_debug_logs()

    def texture_image_info(self) -> TextureImageInfo:
        """Return readback metadata for the latest texture frame."""
        return TextureImageInfo.from_native(self._native.texture_image_info())

    def read_premultiplied_rgba8_into(self, buffer: object) -> TextureImageInfo:
        """Read the latest texture frame into caller-owned writable storage."""
        return TextureImageInfo.from_native(
            self._native.read_premultiplied_rgba8_into(buffer)
        )

    def read_premultiplied_rgba8(self) -> PremultipliedRgba8Image:
        """Read the latest texture frame into copied bytes."""
        return PremultipliedRgba8Image.from_native(
            self._native.read_premultiplied_rgba8()
        )

    def acquire_metal_owned_texture_frame(self) -> "MetalOwnedTextureFrameHandle":
        """Acquire a borrowed Metal frame from a session-owned texture target."""
        return MetalOwnedTextureFrameHandle(
            self._native.acquire_metal_owned_texture_frame()
        )

    def acquire_vulkan_owned_texture_frame(self) -> "VulkanOwnedTextureFrameHandle":
        """Acquire a borrowed Vulkan frame from a session-owned texture target."""
        return VulkanOwnedTextureFrameHandle(
            self._native.acquire_vulkan_owned_texture_frame()
        )

    def query_rendered_features(
        self,
        geometry: RenderedQueryGeometry,
        options: RenderedFeatureQueryOptions | None = None,
    ) -> tuple[QueriedFeature, ...]:
        """Query rendered features from the latest render session state."""
        from .query import (
            QueriedFeature,
            _geometry_to_native_wire,
            _rendered_options_to_native_wire,
        )

        layer_ids, filter_ = _rendered_options_to_native_wire(options)
        raw = self._native.query_rendered_features(
            _geometry_to_native_wire(geometry),
            layer_ids,
            filter_,
        )
        return tuple(QueriedFeature.from_native(feature) for feature in raw)

    def query_source_features(
        self,
        source_id: str,
        options: SourceFeatureQueryOptions | None = None,
    ) -> tuple[QueriedFeature, ...]:
        """Query source features from the latest render session state."""
        from .query import (
            QueriedFeature,
            _source_options_to_native_wire,
        )

        source_layer_ids, filter_ = _source_options_to_native_wire(options)
        raw = self._native.query_source_features(source_id, source_layer_ids, filter_)
        return tuple(QueriedFeature.from_native(feature) for feature in raw)

    def query_feature_extensions(
        self,
        source_id: str,
        feature: Feature,
        extension: str,
        extension_field: str,
        arguments: JsonObjectLike | None = None,
    ) -> FeatureExtensionResult:
        """Query a feature extension from the latest render session state."""
        from .geo import _feature_to_native_wire
        from .json import _to_native_wire as _json_to_native_wire
        from .query import FeatureExtensionResult

        raw = self._native.query_feature_extensions(
            source_id,
            _feature_to_native_wire(feature),
            extension,
            extension_field,
            _json_to_native_wire(arguments) if arguments is not None else None,
        )
        return FeatureExtensionResult.from_native(raw)

    def set_feature_state(
        self,
        selector: FeatureStateSelector,
        state: JsonLike,
    ) -> None:
        """Set per-feature state on a render source for this render session."""
        from .json import _to_native_wire as _json_to_native_wire

        self._native.set_feature_state(
            selector.source_id,
            selector.source_layer_id,
            selector.feature_id,
            selector.state_key,
            _json_to_native_wire(state),
        )

    def get_feature_state(self, selector: FeatureStateSelector) -> JsonValue:
        """Return copied per-feature state from a render source."""
        from .json import _from_native_wire as _json_from_native_wire

        return _json_from_native_wire(
            self._native.get_feature_state(
                selector.source_id,
                selector.source_layer_id,
                selector.feature_id,
                selector.state_key,
            )
        )

    def remove_feature_state(self, selector: FeatureStateSelector) -> None:
        """Remove per-feature state from a render source."""
        self._native.remove_feature_state(
            selector.source_id,
            selector.source_layer_id,
            selector.feature_id,
            selector.state_key,
        )

    def __enter__(self) -> "RenderSessionHandle":
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()


class MetalOwnedTextureFrameHandle:
    """Scoped handle for an acquired Metal session-owned texture frame."""

    def __init__(self, native: Any) -> None:
        self._native = native

    @property
    def closed(self) -> bool:
        """Return whether this frame has been released."""
        return bool(self._native.closed)

    @property
    def frame(self) -> MetalOwnedTextureFrame:
        """Return copied frame metadata."""
        return MetalOwnedTextureFrame.from_native(self._native.frame())

    @property
    def texture(self) -> NativePointer:
        """Return the borrowed Metal texture pointer while the frame is open."""
        return NativePointer(self._native.texture_address())

    @property
    def device(self) -> NativePointer:
        """Return the borrowed Metal device pointer while the frame is open."""
        return NativePointer(self._native.device_address())

    def close(self) -> None:
        """Release this acquired frame exactly once."""
        self._native.close()

    def __del__(self, _warn_unclosed=_warn_unclosed) -> None:
        try:
            _warn_unclosed(
                "MetalOwnedTextureFrameHandle", getattr(self, "closed", True)
            )
        except BaseException:
            return

    def __enter__(self) -> "MetalOwnedTextureFrameHandle":
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()


class VulkanOwnedTextureFrameHandle:
    """Scoped handle for an acquired Vulkan session-owned texture frame."""

    def __init__(self, native: Any) -> None:
        self._native = native

    @property
    def closed(self) -> bool:
        """Return whether this frame has been released."""
        return bool(self._native.closed)

    @property
    def frame(self) -> VulkanOwnedTextureFrame:
        """Return copied frame metadata."""
        return VulkanOwnedTextureFrame.from_native(self._native.frame())

    @property
    def image(self) -> NativePointer:
        """Return the borrowed Vulkan image pointer while the frame is open."""
        return NativePointer(self._native.image_address())

    @property
    def image_view(self) -> NativePointer:
        """Return the borrowed Vulkan image-view pointer while the frame is open."""
        return NativePointer(self._native.image_view_address())

    @property
    def device(self) -> NativePointer:
        """Return the borrowed Vulkan device pointer while the frame is open."""
        return NativePointer(self._native.device_address())

    def close(self) -> None:
        """Release this acquired frame exactly once."""
        self._native.close()

    def __del__(self, _warn_unclosed=_warn_unclosed) -> None:
        try:
            _warn_unclosed(
                "VulkanOwnedTextureFrameHandle", getattr(self, "closed", True)
            )
        except BaseException:
            return

    def __enter__(self) -> "VulkanOwnedTextureFrameHandle":
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()


__all__ = [
    "DetachedRenderSessionHandle",
    "MetalBorrowedTextureDescriptor",
    "MetalContextDescriptor",
    "MetalOwnedTextureDescriptor",
    "MetalOwnedTextureFrame",
    "MetalOwnedTextureFrameHandle",
    "MetalSurfaceDescriptor",
    "NativePointer",
    "PremultipliedRgba8Image",
    "RenderBackend",
    "RenderSessionHandle",
    "RenderTargetExtent",
    "TextureImageInfo",
    "VulkanBorrowedTextureDescriptor",
    "VulkanContextDescriptor",
    "VulkanOwnedTextureDescriptor",
    "VulkanOwnedTextureFrame",
    "VulkanOwnedTextureFrameHandle",
    "VulkanSurfaceDescriptor",
]

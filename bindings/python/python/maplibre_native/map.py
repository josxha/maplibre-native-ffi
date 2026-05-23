"""Map handles, options, and map lifecycle entry points."""

from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum
from types import TracebackType
from typing import TYPE_CHECKING

from . import _native
from .runtime import RuntimeHandle

if TYPE_CHECKING:
    from .camera import CameraOptions
    from .render import (
        MetalBorrowedTextureDescriptor,
        MetalOwnedTextureDescriptor,
        MetalSurfaceDescriptor,
        RenderSessionHandle,
        VulkanBorrowedTextureDescriptor,
        VulkanOwnedTextureDescriptor,
        VulkanSurfaceDescriptor,
    )
    from .style import CustomGeometrySourceHandle, CustomGeometrySourceOptions


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

    def dump_debug_logs(self) -> None:
        """Dump map debug logs through MapLibre Native logging."""
        self._native.dump_debug_logs()

    def set_style_json(self, json: str) -> None:
        """Load inline style JSON through MapLibre Native style APIs."""
        self._native.set_style_json(json)

    def get_camera(self) -> CameraOptions:
        """Return the current camera snapshot."""
        from .camera import CameraOptions

        return CameraOptions.from_native(self._native.get_camera())

    def jump_to(self, camera: CameraOptions) -> None:
        """Apply a camera jump command."""
        center = (
            (camera.center.latitude, camera.center.longitude)
            if camera.center is not None
            else None
        )
        padding = (
            (
                camera.padding.top,
                camera.padding.left,
                camera.padding.bottom,
                camera.padding.right,
            )
            if camera.padding is not None
            else None
        )
        anchor = (
            (camera.anchor.x, camera.anchor.y) if camera.anchor is not None else None
        )
        self._native.jump_to(
            center,
            camera.zoom,
            camera.bearing,
            camera.pitch,
            padding,
            anchor,
        )

    def move_by(self, delta_x: float, delta_y: float) -> None:
        """Apply a screen-space pan command."""
        self._native.move_by(delta_x, delta_y)

    def cancel_transitions(self) -> None:
        """Cancel active camera transitions."""
        self._native.cancel_transitions()

    def add_custom_geometry_source(
        self,
        source_id: str,
        options: CustomGeometrySourceOptions | None = None,
    ) -> CustomGeometrySourceHandle:
        """Add a custom geometry source and return its queued-event handle."""
        from .style import CustomGeometrySourceHandle, CustomGeometrySourceOptions

        options = options or CustomGeometrySourceOptions()
        native = self._native.add_custom_geometry_source(
            source_id,
            options.max_queued_events,
            options.min_zoom,
            options.max_zoom,
            options.tolerance,
            options.tile_size,
            options.buffer,
            options.clip,
            options.wrap,
            options.has_cancel_tile,
        )
        return CustomGeometrySourceHandle(native)

    def attach_metal_surface(
        self, descriptor: MetalSurfaceDescriptor
    ) -> RenderSessionHandle:
        """Attach a Metal native surface render target to this map."""
        from . import _native
        from .render import RenderSessionHandle

        native = _native.attach_metal_surface(
            self._native,
            descriptor.extent.width,
            descriptor.extent.height,
            descriptor.extent.scale_factor,
            descriptor.context.device.address,
            descriptor.layer.address,
        )
        return RenderSessionHandle(native, self)

    def attach_vulkan_surface(
        self, descriptor: VulkanSurfaceDescriptor
    ) -> RenderSessionHandle:
        """Attach a Vulkan native surface render target to this map."""
        from . import _native
        from .render import RenderSessionHandle

        native = _native.attach_vulkan_surface(
            self._native,
            descriptor.extent.width,
            descriptor.extent.height,
            descriptor.extent.scale_factor,
            descriptor.context.instance.address,
            descriptor.context.physical_device.address,
            descriptor.context.device.address,
            descriptor.context.graphics_queue.address,
            descriptor.context.graphics_queue_family_index,
            descriptor.surface.address,
        )
        return RenderSessionHandle(native, self)

    def attach_metal_owned_texture(
        self, descriptor: MetalOwnedTextureDescriptor
    ) -> RenderSessionHandle:
        """Attach a Metal session-owned texture render target to this map."""
        from . import _native
        from .render import RenderSessionHandle

        native = _native.attach_metal_owned_texture(
            self._native,
            descriptor.extent.width,
            descriptor.extent.height,
            descriptor.extent.scale_factor,
            descriptor.context.device.address,
        )
        return RenderSessionHandle(native, self)

    def attach_metal_borrowed_texture(
        self, descriptor: MetalBorrowedTextureDescriptor
    ) -> RenderSessionHandle:
        """Attach a Metal caller-owned texture render target to this map."""
        from . import _native
        from .render import RenderSessionHandle

        native = _native.attach_metal_borrowed_texture(
            self._native,
            descriptor.extent.width,
            descriptor.extent.height,
            descriptor.extent.scale_factor,
            descriptor.texture.address,
        )
        return RenderSessionHandle(native, self)

    def attach_vulkan_owned_texture(
        self, descriptor: VulkanOwnedTextureDescriptor
    ) -> RenderSessionHandle:
        """Attach a Vulkan session-owned texture render target to this map."""
        from . import _native
        from .render import RenderSessionHandle

        native = _native.attach_vulkan_owned_texture(
            self._native,
            descriptor.extent.width,
            descriptor.extent.height,
            descriptor.extent.scale_factor,
            descriptor.context.instance.address,
            descriptor.context.physical_device.address,
            descriptor.context.device.address,
            descriptor.context.graphics_queue.address,
            descriptor.context.graphics_queue_family_index,
        )
        return RenderSessionHandle(native, self)

    def attach_vulkan_borrowed_texture(
        self, descriptor: VulkanBorrowedTextureDescriptor
    ) -> RenderSessionHandle:
        """Attach a Vulkan caller-owned texture render target to this map."""
        from . import _native
        from .render import RenderSessionHandle

        native = _native.attach_vulkan_borrowed_texture(
            self._native,
            descriptor.extent.width,
            descriptor.extent.height,
            descriptor.extent.scale_factor,
            descriptor.context.instance.address,
            descriptor.context.physical_device.address,
            descriptor.context.device.address,
            descriptor.context.graphics_queue.address,
            descriptor.context.graphics_queue_family_index,
            descriptor.image.address,
            descriptor.image_view.address,
            descriptor.format,
            descriptor.initial_layout,
            descriptor.final_layout,
        )
        return RenderSessionHandle(native, self)

    def __enter__(self) -> "MapHandle":
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()

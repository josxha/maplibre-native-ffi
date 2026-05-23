"""Map handles, options, and map lifecycle entry points."""

from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum, IntFlag
from types import TracebackType
from typing import TYPE_CHECKING

from . import _native
from .runtime import RuntimeHandle

if TYPE_CHECKING:
    from .camera import AnimationOptions, CameraOptions, EdgeInsets, ScreenPoint
    from .geo import LatLng
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


class MapDebugOptions(IntFlag):
    """Map debug overlay mask bits."""

    NONE = 0
    TILE_BORDERS = 1 << 1
    PARSE_STATUS = 1 << 2
    TIMESTAMPS = 1 << 3
    COLLISION = 1 << 4
    OVERDRAW = 1 << 5
    STENCIL_CLIP = 1 << 6
    DEPTH_BUFFER = 1 << 7


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


@dataclass(frozen=True, slots=True)
class ProjectedMeters:
    """Spherical Mercator projected-meter coordinate."""

    northing: float
    easting: float


def _camera_parts(
    camera: CameraOptions,
) -> tuple[
    tuple[float, float] | None,
    float | None,
    float | None,
    float | None,
    tuple[float, float, float, float] | None,
    tuple[float, float] | None,
]:
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
    anchor = (camera.anchor.x, camera.anchor.y) if camera.anchor is not None else None
    return center, camera.zoom, camera.bearing, camera.pitch, padding, anchor


def _animation_parts(
    animation: AnimationOptions | None,
) -> (
    tuple[
        float | None,
        float | None,
        float | None,
        tuple[float, float, float, float] | None,
    ]
    | None
):
    if animation is None:
        return None
    easing = (
        (
            animation.easing.p1x,
            animation.easing.p1y,
            animation.easing.p2x,
            animation.easing.p2y,
        )
        if animation.easing is not None
        else None
    )
    return animation.duration_ms, animation.velocity, animation.min_zoom, easing


def projected_meters_for_lat_lng(coordinate: LatLng) -> ProjectedMeters:
    """Convert a geographic coordinate to spherical Mercator projected meters."""
    raw = _native.projected_meters_for_lat_lng(
        coordinate.latitude,
        coordinate.longitude,
    )
    return ProjectedMeters(northing=raw["northing"], easting=raw["easting"])


def lat_lng_for_projected_meters(meters: ProjectedMeters) -> LatLng:
    """Convert spherical Mercator projected meters to a geographic coordinate."""
    from .geo import LatLng

    raw = _native.lat_lng_for_projected_meters(meters.northing, meters.easting)
    return LatLng(latitude=raw["latitude"], longitude=raw["longitude"])


class MapProjectionHandle:
    """Standalone projection helper snapshotted from a map transform."""

    def __init__(self, native: object) -> None:
        self._native = native

    @property
    def closed(self) -> bool:
        """Return whether this projection helper has been closed."""
        return bool(self._native.closed)

    def close(self) -> None:
        """Release this projection helper exactly once."""
        self._native.close()

    def get_camera(self) -> CameraOptions:
        """Return the helper's current camera snapshot."""
        from .camera import CameraOptions

        return CameraOptions.from_native(self._native.get_camera())

    def set_camera(self, camera: CameraOptions) -> None:
        """Apply camera fields to this projection helper."""
        self._native.set_camera(*_camera_parts(camera))

    def set_visible_coordinates(
        self,
        coordinates: list[LatLng] | tuple[LatLng, ...],
        padding: EdgeInsets,
    ) -> None:
        """Update this helper's camera so coordinates are visible."""
        self._native.set_visible_coordinates(
            [(coordinate.latitude, coordinate.longitude) for coordinate in coordinates],
            (padding.top, padding.left, padding.bottom, padding.right),
        )

    def pixel_for_lat_lng(self, coordinate: LatLng) -> ScreenPoint:
        """Convert a geographic coordinate to a screen-space point."""
        from .camera import ScreenPoint

        raw = self._native.pixel_for_lat_lng(
            coordinate.latitude,
            coordinate.longitude,
        )
        return ScreenPoint(x=raw["x"], y=raw["y"])

    def lat_lng_for_pixel(self, point: ScreenPoint) -> LatLng:
        """Convert a screen-space point to a geographic coordinate."""
        from .geo import LatLng

        raw = self._native.lat_lng_for_pixel(point.x, point.y)
        return LatLng(latitude=raw["latitude"], longitude=raw["longitude"])

    def __enter__(self) -> "MapProjectionHandle":
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.close()


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

    def set_debug_options(self, options: MapDebugOptions) -> None:
        """Apply MapLibre debug overlay mask bits."""
        self._native.set_debug_options(int(options))

    def get_debug_options(self) -> MapDebugOptions:
        """Return the current MapLibre debug overlay mask bits."""
        return MapDebugOptions(self._native.get_debug_options())

    def set_rendering_stats_view_enabled(self, enabled: bool) -> None:
        """Enable or disable MapLibre's rendering stats overlay view."""
        self._native.set_rendering_stats_view_enabled(enabled)

    def get_rendering_stats_view_enabled(self) -> bool:
        """Return whether MapLibre's rendering stats overlay view is enabled."""
        return self._native.get_rendering_stats_view_enabled()

    def is_fully_loaded(self) -> bool:
        """Return whether MapLibre currently considers the map fully loaded."""
        return self._native.is_fully_loaded()

    def dump_debug_logs(self) -> None:
        """Dump map debug logs through MapLibre Native logging."""
        self._native.dump_debug_logs()

    def set_style_json(self, json: str) -> None:
        """Load inline style JSON through MapLibre Native style APIs."""
        self._native.set_style_json(json)

    def create_projection(self) -> MapProjectionHandle:
        """Create a standalone projection helper from the current map transform."""
        return MapProjectionHandle(self._native.create_projection())

    def get_camera(self) -> CameraOptions:
        """Return the current camera snapshot."""
        from .camera import CameraOptions

        return CameraOptions.from_native(self._native.get_camera())

    def jump_to(self, camera: CameraOptions) -> None:
        """Apply a camera jump command."""
        self._native.jump_to(*_camera_parts(camera))

    def ease_to(
        self,
        camera: CameraOptions,
        animation: AnimationOptions | None = None,
    ) -> None:
        """Apply a camera ease transition command."""
        self._native.ease_to(*_camera_parts(camera), _animation_parts(animation))

    def fly_to(
        self,
        camera: CameraOptions,
        animation: AnimationOptions | None = None,
    ) -> None:
        """Apply a camera fly transition command."""
        self._native.fly_to(*_camera_parts(camera), _animation_parts(animation))

    def move_by(self, delta_x: float, delta_y: float) -> None:
        """Apply a screen-space pan command."""
        self._native.move_by(delta_x, delta_y)

    def move_by_animated(
        self,
        delta_x: float,
        delta_y: float,
        animation: AnimationOptions | None = None,
    ) -> None:
        """Apply an animated screen-space pan command."""
        self._native.move_by_animated(delta_x, delta_y, _animation_parts(animation))

    def scale_by(self, scale: float, anchor: ScreenPoint | None = None) -> None:
        """Apply a screen-space zoom command."""
        raw_anchor = (anchor.x, anchor.y) if anchor is not None else None
        self._native.scale_by(scale, raw_anchor)

    def scale_by_animated(
        self,
        scale: float,
        anchor: ScreenPoint | None = None,
        animation: AnimationOptions | None = None,
    ) -> None:
        """Apply an animated screen-space zoom command."""
        raw_anchor = (anchor.x, anchor.y) if anchor is not None else None
        self._native.scale_by_animated(scale, raw_anchor, _animation_parts(animation))

    def rotate_by(self, first: ScreenPoint, second: ScreenPoint) -> None:
        """Apply a screen-space rotate command."""
        self._native.rotate_by((first.x, first.y), (second.x, second.y))

    def rotate_by_animated(
        self,
        first: ScreenPoint,
        second: ScreenPoint,
        animation: AnimationOptions | None = None,
    ) -> None:
        """Apply an animated screen-space rotate command."""
        self._native.rotate_by_animated(
            (first.x, first.y),
            (second.x, second.y),
            _animation_parts(animation),
        )

    def pitch_by(self, pitch: float) -> None:
        """Apply a pitch delta command."""
        self._native.pitch_by(pitch)

    def pitch_by_animated(
        self,
        pitch: float,
        animation: AnimationOptions | None = None,
    ) -> None:
        """Apply an animated pitch delta command."""
        self._native.pitch_by_animated(pitch, _animation_parts(animation))

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

"""Map handles, options, and map lifecycle entry points."""

from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum, IntFlag
from types import TracebackType
from typing import TYPE_CHECKING

from . import _native
from .camera import EdgeInsets
from .runtime import RuntimeHandle

if TYPE_CHECKING:
    from .camera import (
        AnimationOptions,
        CameraOptions,
        EdgeInsets,
        FreeCameraOptions,
        ProjectionMode,
        ScreenPoint,
    )
    from .geo import GeoJson, LatLng, LatLngBounds
    from .json import JsonValue
    from .render import (
        MetalBorrowedTextureDescriptor,
        MetalOwnedTextureDescriptor,
        MetalSurfaceDescriptor,
        PremultipliedRgba8Image,
        RenderSessionHandle,
        VulkanBorrowedTextureDescriptor,
        VulkanOwnedTextureDescriptor,
        VulkanSurfaceDescriptor,
    )
    from .style import (
        CanonicalTileId,
        CustomGeometrySourceHandle,
        CustomGeometrySourceOptions,
        StyleImage,
        StyleImageInfo,
        LocationIndicatorImageKind,
        StyleImageOptions,
        StyleSourceInfo,
        StyleSourceType,
        TileSourceOptions,
    )


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


class NorthOrientation(IntEnum):
    """Map north orientation values."""

    UP = 0
    RIGHT = 1
    DOWN = 2
    LEFT = 3


class ConstrainMode(IntEnum):
    """Map camera constraint modes."""

    NONE = 0
    HEIGHT_ONLY = 1
    WIDTH_AND_HEIGHT = 2
    SCREEN = 3


class ViewportMode(IntEnum):
    """Viewport orientation modes."""

    DEFAULT = 0
    FLIPPED_Y = 1


class TileLodMode(IntEnum):
    """Tile LOD algorithm values."""

    DEFAULT = 0
    DISTANCE = 1


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
class MapViewportOptions:
    """Live map viewport and render-transform controls."""

    north_orientation: NorthOrientation | None = None
    constrain_mode: ConstrainMode | None = None
    viewport_mode: ViewportMode | None = None
    frustum_offset: EdgeInsets | None = None

    @classmethod
    def from_native(cls, raw: dict[str, object]) -> "MapViewportOptions":
        """Build viewport options from private native bridge values."""
        frustum_offset = raw["frustum_offset"]
        return cls(
            north_orientation=NorthOrientation(raw["north_orientation"])
            if raw["north_orientation"] is not None
            else None,
            constrain_mode=ConstrainMode(raw["constrain_mode"])
            if raw["constrain_mode"] is not None
            else None,
            viewport_mode=ViewportMode(raw["viewport_mode"])
            if raw["viewport_mode"] is not None
            else None,
            frustum_offset=EdgeInsets(**frustum_offset)
            if isinstance(frustum_offset, dict)
            else None,
        )


@dataclass(frozen=True, slots=True)
class MapTileOptions:
    """Tile prefetch and LOD tuning controls."""

    prefetch_zoom_delta: int | None = None
    lod_min_radius: float | None = None
    lod_scale: float | None = None
    lod_pitch_threshold: float | None = None
    lod_zoom_shift: float | None = None
    lod_mode: TileLodMode | None = None

    @classmethod
    def from_native(cls, raw: dict[str, object]) -> "MapTileOptions":
        """Build tile options from private native bridge values."""
        return cls(
            prefetch_zoom_delta=raw["prefetch_zoom_delta"],
            lod_min_radius=raw["lod_min_radius"],
            lod_scale=raw["lod_scale"],
            lod_pitch_threshold=raw["lod_pitch_threshold"],
            lod_zoom_shift=raw["lod_zoom_shift"],
            lod_mode=TileLodMode(raw["lod_mode"])
            if raw["lod_mode"] is not None
            else None,
        )


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


def _coordinate_parts(
    coordinates: list[LatLng] | tuple[LatLng, ...],
) -> list[tuple[float, float]]:
    return [(coordinate.latitude, coordinate.longitude) for coordinate in coordinates]


def _image_parts(image: PremultipliedRgba8Image) -> tuple[int, int, int, bytes]:
    return image.info.width, image.info.height, image.info.stride, image.data


def _tile_source_parts(
    options: TileSourceOptions | None,
) -> tuple[
    float | None,
    float | None,
    str | None,
    int | None,
    tuple[tuple[float, float], tuple[float, float]] | None,
    int | None,
    int | None,
    int | None,
]:
    if options is None:
        return None, None, None, None, None, None, None, None
    bounds = (
        (
            (options.bounds.southwest.latitude, options.bounds.southwest.longitude),
            (options.bounds.northeast.latitude, options.bounds.northeast.longitude),
        )
        if options.bounds is not None
        else None
    )
    return (
        options.min_zoom,
        options.max_zoom,
        options.attribution,
        int(options.scheme) if options.scheme is not None else None,
        bounds,
        options.tile_size,
        int(options.vector_encoding) if options.vector_encoding is not None else None,
        int(options.raster_dem_encoding)
        if options.raster_dem_encoding is not None
        else None,
    )


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

    def request_still_image(self) -> None:
        """Request one still image for a static or tile map."""
        self._native.request_still_image()

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

    def get_viewport_options(self) -> MapViewportOptions:
        """Return live map viewport and render-transform controls."""
        return MapViewportOptions.from_native(self._native.get_viewport_options())

    def set_viewport_options(self, options: MapViewportOptions) -> None:
        """Apply selected live map viewport and render-transform controls."""
        frustum_offset = (
            (
                options.frustum_offset.top,
                options.frustum_offset.left,
                options.frustum_offset.bottom,
                options.frustum_offset.right,
            )
            if options.frustum_offset is not None
            else None
        )
        self._native.set_viewport_options(
            int(options.north_orientation)
            if options.north_orientation is not None
            else None,
            int(options.constrain_mode) if options.constrain_mode is not None else None,
            int(options.viewport_mode) if options.viewport_mode is not None else None,
            frustum_offset,
        )

    def get_tile_options(self) -> MapTileOptions:
        """Return tile prefetch and LOD tuning controls."""
        return MapTileOptions.from_native(self._native.get_tile_options())

    def set_tile_options(self, options: MapTileOptions) -> None:
        """Apply selected tile prefetch and LOD tuning controls."""
        self._native.set_tile_options(
            options.prefetch_zoom_delta,
            options.lod_min_radius,
            options.lod_scale,
            options.lod_pitch_threshold,
            options.lod_zoom_shift,
            int(options.lod_mode) if options.lod_mode is not None else None,
        )

    def set_style_url(self, url: str) -> None:
        """Load a style URL through MapLibre Native style APIs."""
        self._native.set_style_url(url)

    def set_style_json(self, json: str) -> None:
        """Load inline style JSON through MapLibre Native style APIs."""
        self._native.set_style_json(json)

    def add_style_source_json(self, source_id: str, source_json: JsonValue) -> None:
        """Add one style source from a style-spec source JSON object."""
        from .json import _to_native_wire as _json_to_native_wire

        self._native.add_style_source_json(source_id, _json_to_native_wire(source_json))

    def add_geojson_source_url(self, source_id: str, url: str) -> None:
        """Add a GeoJSON source that loads data from a URL."""
        self._native.add_geojson_source_url(source_id, url)

    def add_geojson_source_data(self, source_id: str, data: GeoJson) -> None:
        """Add a GeoJSON source with inline data."""
        from .geo import _to_native_wire as _geojson_to_native_wire

        self._native.add_geojson_source_data(source_id, _geojson_to_native_wire(data))

    def set_geojson_source_url(self, source_id: str, url: str) -> None:
        """Update one GeoJSON source to load data from a URL."""
        self._native.set_geojson_source_url(source_id, url)

    def set_geojson_source_data(self, source_id: str, data: GeoJson) -> None:
        """Update one GeoJSON source with inline data."""
        from .geo import _to_native_wire as _geojson_to_native_wire

        self._native.set_geojson_source_data(source_id, _geojson_to_native_wire(data))

    def add_vector_source_url(
        self,
        source_id: str,
        url: str,
        options: TileSourceOptions | None = None,
    ) -> None:
        """Add a vector source with a TileJSON URL."""
        self._native.add_vector_source_url(source_id, url, *_tile_source_parts(options))

    def add_raster_source_url(
        self,
        source_id: str,
        url: str,
        options: TileSourceOptions | None = None,
    ) -> None:
        """Add a raster source with a TileJSON URL."""
        self._native.add_raster_source_url(source_id, url, *_tile_source_parts(options))

    def add_raster_dem_source_url(
        self,
        source_id: str,
        url: str,
        options: TileSourceOptions | None = None,
    ) -> None:
        """Add a raster DEM source with a TileJSON URL."""
        self._native.add_raster_dem_source_url(
            source_id, url, *_tile_source_parts(options)
        )

    def add_vector_source_tiles(
        self,
        source_id: str,
        tiles: list[str] | tuple[str, ...],
        options: TileSourceOptions | None = None,
    ) -> None:
        """Add a vector source with inline tile URLs."""
        self._native.add_vector_source_tiles(
            source_id,
            list(tiles),
            *_tile_source_parts(options),
        )

    def add_raster_source_tiles(
        self,
        source_id: str,
        tiles: list[str] | tuple[str, ...],
        options: TileSourceOptions | None = None,
    ) -> None:
        """Add a raster source with inline tile URLs."""
        self._native.add_raster_source_tiles(
            source_id,
            list(tiles),
            *_tile_source_parts(options),
        )

    def add_raster_dem_source_tiles(
        self,
        source_id: str,
        tiles: list[str] | tuple[str, ...],
        options: TileSourceOptions | None = None,
    ) -> None:
        """Add a raster DEM source with inline tile URLs."""
        self._native.add_raster_dem_source_tiles(
            source_id,
            list(tiles),
            *_tile_source_parts(options),
        )

    def remove_style_source(self, source_id: str) -> bool:
        """Remove a style source by ID and report whether it existed."""
        return self._native.remove_style_source(source_id)

    def style_source_exists(self, source_id: str) -> bool:
        """Return whether a style source ID exists."""
        return self._native.style_source_exists(source_id)

    def get_style_source_type(self, source_id: str) -> StyleSourceType | None:
        """Return a style source type, or None when the source is missing."""
        from .style import StyleSourceType

        raw = self._native.get_style_source_type(source_id)
        return StyleSourceType(raw) if raw is not None else None

    def get_style_source_info(self, source_id: str) -> StyleSourceInfo | None:
        """Return copied fixed metadata for one style source."""
        from .style import StyleSourceInfo

        raw = self._native.get_style_source_info(source_id)
        return StyleSourceInfo.from_native(raw) if raw is not None else None

    def list_style_source_ids(self) -> tuple[str, ...]:
        """Return style source IDs in style order."""
        return tuple(self._native.list_style_source_ids())

    def add_hillshade_layer(
        self,
        layer_id: str,
        source_id: str,
        before_layer_id: str | None = None,
    ) -> None:
        """Add a hillshade layer for a raster DEM source."""
        self._native.add_hillshade_layer(layer_id, source_id, before_layer_id)

    def add_color_relief_layer(
        self,
        layer_id: str,
        source_id: str,
        before_layer_id: str | None = None,
    ) -> None:
        """Add a color-relief layer for a raster DEM source."""
        self._native.add_color_relief_layer(layer_id, source_id, before_layer_id)

    def add_location_indicator_layer(
        self,
        layer_id: str,
        before_layer_id: str | None = None,
    ) -> None:
        """Add a source-free location indicator layer."""
        self._native.add_location_indicator_layer(layer_id, before_layer_id)

    def set_location_indicator_location(
        self,
        layer_id: str,
        coordinate: LatLng,
        altitude: float,
    ) -> None:
        """Set a location indicator layer location."""
        self._native.set_location_indicator_location(
            layer_id,
            coordinate.latitude,
            coordinate.longitude,
            altitude,
        )

    def set_location_indicator_bearing(self, layer_id: str, bearing: float) -> None:
        """Set a location indicator layer bearing in degrees."""
        self._native.set_location_indicator_bearing(layer_id, bearing)

    def set_location_indicator_accuracy_radius(
        self,
        layer_id: str,
        radius: float,
    ) -> None:
        """Set a location indicator layer accuracy radius in logical pixels."""
        self._native.set_location_indicator_accuracy_radius(layer_id, radius)

    def set_location_indicator_image_name(
        self,
        layer_id: str,
        image_kind: LocationIndicatorImageKind,
        image_id: str,
    ) -> None:
        """Set one location indicator image-name property."""
        self._native.set_location_indicator_image_name(
            layer_id,
            image_kind.native_code,
            image_id,
        )

    def remove_style_layer(self, layer_id: str) -> bool:
        """Remove a style layer by ID and report whether it existed."""
        return self._native.remove_style_layer(layer_id)

    def style_layer_exists(self, layer_id: str) -> bool:
        """Return whether a style layer ID exists."""
        return self._native.style_layer_exists(layer_id)

    def get_style_layer_type(self, layer_id: str) -> str | None:
        """Return a style layer type string, or None when the layer is missing."""
        return self._native.get_style_layer_type(layer_id)

    def list_style_layer_ids(self) -> tuple[str, ...]:
        """Return style layer IDs in style order."""
        return tuple(self._native.list_style_layer_ids())

    def move_style_layer(
        self,
        layer_id: str,
        before_layer_id: str | None = None,
    ) -> None:
        """Move one style layer before another layer or to the top."""
        self._native.move_style_layer(layer_id, before_layer_id)

    def add_style_layer_json(
        self,
        layer_json: object,
        before_layer_id: str | None = None,
    ) -> None:
        """Add one style layer from a full style-spec layer JSON object."""
        from .json import _to_native_wire

        self._native.add_style_layer_json(_to_native_wire(layer_json), before_layer_id)

    def get_style_layer_json(self, layer_id: str) -> object | None:
        """Return one style layer as a full style-spec layer JSON object."""
        from .json import _from_native_wire

        raw = self._native.get_style_layer_json(layer_id)
        return _from_native_wire(raw) if raw is not None else None

    def set_style_light_json(self, light_json: object) -> None:
        """Set the style light from a style-spec light JSON object."""
        from .json import _to_native_wire

        self._native.set_style_light_json(_to_native_wire(light_json))

    def set_style_light_property(self, property_name: str, value: object) -> None:
        """Set one style light property by style-spec property name."""
        from .json import _to_native_wire

        self._native.set_style_light_property(property_name, _to_native_wire(value))

    def get_style_light_property(self, property_name: str) -> object | None:
        """Return one style light property as a style-spec JSON value."""
        from .json import _from_native_wire

        raw = self._native.get_style_light_property(property_name)
        return _from_native_wire(raw) if raw is not None else None

    def set_layer_property(
        self,
        layer_id: str,
        property_name: str,
        value: object,
    ) -> None:
        """Set one layer property by style-spec property name."""
        from .json import _to_native_wire

        self._native.set_layer_property(layer_id, property_name, _to_native_wire(value))

    def get_layer_property(self, layer_id: str, property_name: str) -> object | None:
        """Return one layer property as a style-spec JSON value."""
        from .json import _from_native_wire

        raw = self._native.get_layer_property(layer_id, property_name)
        return _from_native_wire(raw) if raw is not None else None

    def set_layer_filter(self, layer_id: str, filter: object | None) -> None:
        """Set or clear one layer filter."""
        from .json import _to_native_wire

        self._native.set_layer_filter(
            layer_id,
            _to_native_wire(filter) if filter is not None else None,
        )

    def get_layer_filter(self, layer_id: str) -> object | None:
        """Return one layer filter as a style-spec JSON value."""
        from .json import _from_native_wire

        raw = self._native.get_layer_filter(layer_id)
        return _from_native_wire(raw) if raw is not None else None

    def set_style_image(
        self,
        image_id: str,
        image: object,
        options: StyleImageOptions | None = None,
    ) -> None:
        """Add or replace one runtime style image."""
        from .render import PremultipliedRgba8Image
        from .style import StyleImageOptions

        options = options or StyleImageOptions()
        if not isinstance(image, PremultipliedRgba8Image):
            msg = "image must be a PremultipliedRgba8Image"
            raise TypeError(msg)
        self._native.set_style_image(
            image_id,
            image.info.width,
            image.info.height,
            image.info.stride,
            image.data,
            options.pixel_ratio,
            options.sdf,
        )

    def remove_style_image(self, image_id: str) -> bool:
        """Remove a runtime style image by ID and report whether it existed."""
        return self._native.remove_style_image(image_id)

    def style_image_exists(self, image_id: str) -> bool:
        """Return whether a runtime style image ID exists."""
        return self._native.style_image_exists(image_id)

    def get_style_image_info(self, image_id: str) -> StyleImageInfo | None:
        """Return fixed metadata for one runtime style image."""
        from .style import StyleImageInfo

        raw = self._native.get_style_image_info(image_id)
        return StyleImageInfo.from_native(raw) if raw is not None else None

    def copy_style_image_premultiplied_rgba8(self, image_id: str) -> StyleImage | None:
        """Copy one runtime style image as premultiplied RGBA8 pixels."""
        from .style import StyleImage

        raw = self._native.copy_style_image_premultiplied_rgba8(image_id)
        return StyleImage.from_native(raw) if raw is not None else None

    def add_image_source_url(
        self,
        source_id: str,
        coordinates: list[LatLng] | tuple[LatLng, ...],
        url: str,
    ) -> None:
        """Add an image source that loads its image from a URL."""
        self._native.add_image_source_url(
            source_id, _coordinate_parts(coordinates), url
        )

    def add_image_source_image(
        self,
        source_id: str,
        coordinates: list[LatLng] | tuple[LatLng, ...],
        image: PremultipliedRgba8Image,
    ) -> None:
        """Add an image source with inline image pixels."""
        self._native.add_image_source_image(
            source_id,
            _coordinate_parts(coordinates),
            *_image_parts(image),
        )

    def set_image_source_url(self, source_id: str, url: str) -> None:
        """Update an image source to load its image from a URL."""
        self._native.set_image_source_url(source_id, url)

    def set_image_source_image(
        self,
        source_id: str,
        image: PremultipliedRgba8Image,
    ) -> None:
        """Update an image source with inline image pixels."""
        self._native.set_image_source_image(source_id, *_image_parts(image))

    def set_image_source_coordinates(
        self,
        source_id: str,
        coordinates: list[LatLng] | tuple[LatLng, ...],
    ) -> None:
        """Update image source coordinates."""
        self._native.set_image_source_coordinates(
            source_id, _coordinate_parts(coordinates)
        )

    def get_image_source_coordinates(self, source_id: str) -> tuple[LatLng, ...] | None:
        """Return copied image source coordinates, or None when missing."""
        from .geo import LatLng

        raw = self._native.get_image_source_coordinates(source_id)
        if raw is None:
            return None
        return tuple(LatLng(**coordinate) for coordinate in raw)

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

    def get_free_camera_options(self) -> FreeCameraOptions:
        """Return the current free camera position and orientation."""
        from .camera import FreeCameraOptions

        return FreeCameraOptions.from_native(self._native.get_free_camera_options())

    def set_free_camera_options(self, options: FreeCameraOptions) -> None:
        """Apply selected free camera position and orientation fields."""
        position = (
            (options.position.x, options.position.y, options.position.z)
            if options.position is not None
            else None
        )
        orientation = (
            (
                options.orientation.x,
                options.orientation.y,
                options.orientation.z,
                options.orientation.w,
            )
            if options.orientation is not None
            else None
        )
        self._native.set_free_camera_options(position, orientation)

    def get_projection_mode(self) -> ProjectionMode:
        """Return the current axonometric rendering options."""
        from .camera import ProjectionMode

        return ProjectionMode.from_native(self._native.get_projection_mode())

    def set_projection_mode(self, mode: ProjectionMode) -> None:
        """Apply axonometric rendering option fields to the map."""
        self._native.set_projection_mode(mode.axonometric, mode.x_skew, mode.y_skew)

    def pixel_for_lat_lng(self, coordinate: LatLng) -> ScreenPoint:
        """Convert a geographic world coordinate to a screen point for this map."""
        from .camera import ScreenPoint

        raw = self._native.pixel_for_lat_lng(
            coordinate.latitude,
            coordinate.longitude,
        )
        return ScreenPoint(x=raw["x"], y=raw["y"])

    def lat_lng_for_pixel(self, point: ScreenPoint) -> LatLng:
        """Convert a screen point to a geographic world coordinate for this map."""
        from .geo import LatLng

        raw = self._native.lat_lng_for_pixel(point.x, point.y)
        return LatLng(latitude=raw["latitude"], longitude=raw["longitude"])

    def pixels_for_lat_lngs(
        self,
        coordinates: list[LatLng] | tuple[LatLng, ...],
    ) -> tuple[ScreenPoint, ...]:
        """Convert geographic world coordinates to screen points for this map."""
        from .camera import ScreenPoint

        raw = self._native.pixels_for_lat_lngs(_coordinate_parts(coordinates))
        return tuple(ScreenPoint(x=point["x"], y=point["y"]) for point in raw)

    def lat_lngs_for_pixels(
        self,
        points: list[ScreenPoint] | tuple[ScreenPoint, ...],
    ) -> tuple[LatLng, ...]:
        """Convert screen points to geographic world coordinates for this map."""
        from .geo import LatLng

        raw = self._native.lat_lngs_for_pixels([(point.x, point.y) for point in points])
        return tuple(LatLng(**coordinate) for coordinate in raw)

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

    def set_custom_geometry_source_tile_data(
        self,
        source_id: str,
        tile_id: CanonicalTileId,
        data: GeoJson,
    ) -> None:
        """Set custom geometry source data for one canonical tile."""
        from .geo import _to_native_wire as _geojson_to_native_wire

        self._native.set_custom_geometry_source_tile_data(
            source_id,
            tile_id.z,
            tile_id.x,
            tile_id.y,
            _geojson_to_native_wire(data),
        )

    def invalidate_custom_geometry_source_tile(
        self,
        source_id: str,
        tile_id: CanonicalTileId,
    ) -> None:
        """Invalidate custom geometry source data for one canonical tile."""
        self._native.invalidate_custom_geometry_source_tile(
            source_id,
            tile_id.z,
            tile_id.x,
            tile_id.y,
        )

    def invalidate_custom_geometry_source_region(
        self,
        source_id: str,
        bounds: LatLngBounds,
    ) -> None:
        """Invalidate custom geometry source data inside one geographic region."""
        self._native.invalidate_custom_geometry_source_region(
            source_id,
            (bounds.southwest.latitude, bounds.southwest.longitude),
            (bounds.northeast.latitude, bounds.northeast.longitude),
        )

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

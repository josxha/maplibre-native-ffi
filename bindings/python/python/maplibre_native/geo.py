"""Geographic coordinates, geometry, and GeoJSON value trees."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TypeAlias

from .json import JsonMember
from .json import _to_native_wire as _json_to_native_wire


@dataclass(frozen=True, slots=True)
class LatLng:
    """Geographic coordinate in degrees."""

    latitude: float
    longitude: float


@dataclass(frozen=True, slots=True)
class LatLngBounds:
    """Geographic bounds in degrees."""

    southwest: LatLng
    northeast: LatLng


@dataclass(frozen=True, slots=True)
class EmptyGeometry:
    """Empty geometry value."""


@dataclass(frozen=True, slots=True)
class Point:
    """Point geometry."""

    coordinate: LatLng


@dataclass(frozen=True, slots=True)
class LineString:
    """Line string geometry."""

    coordinates: tuple[LatLng, ...]


@dataclass(frozen=True, slots=True)
class Polygon:
    """Polygon geometry represented as rings."""

    rings: tuple[tuple[LatLng, ...], ...]


@dataclass(frozen=True, slots=True)
class MultiPoint:
    """Multi-point geometry."""

    coordinates: tuple[LatLng, ...]


@dataclass(frozen=True, slots=True)
class MultiLineString:
    """Multi-line-string geometry."""

    lines: tuple[tuple[LatLng, ...], ...]


@dataclass(frozen=True, slots=True)
class MultiPolygon:
    """Multi-polygon geometry."""

    polygons: tuple[tuple[tuple[LatLng, ...], ...], ...]


@dataclass(frozen=True, slots=True)
class GeometryCollection:
    """Geometry collection value."""

    geometries: tuple[Geometry, ...]


Geometry: TypeAlias = (
    EmptyGeometry
    | Point
    | LineString
    | Polygon
    | MultiPoint
    | MultiLineString
    | MultiPolygon
    | GeometryCollection
)


@dataclass(frozen=True, slots=True)
class FeatureIdentifierUInt:
    """Unsigned GeoJSON feature identifier."""

    value: int

    def __post_init__(self) -> None:
        if self.value < 0:
            msg = "FeatureIdentifierUInt value must be non-negative"
            raise ValueError(msg)


@dataclass(frozen=True, slots=True)
class FeatureIdentifierInt:
    """Signed GeoJSON feature identifier."""

    value: int


@dataclass(frozen=True, slots=True)
class FeatureIdentifierDouble:
    """Double GeoJSON feature identifier."""

    value: float


@dataclass(frozen=True, slots=True)
class FeatureIdentifierString:
    """String GeoJSON feature identifier."""

    value: str


FeatureIdentifier: TypeAlias = (
    None
    | FeatureIdentifierUInt
    | FeatureIdentifierInt
    | FeatureIdentifierDouble
    | FeatureIdentifierString
)


@dataclass(frozen=True, slots=True)
class Feature:
    """GeoJSON feature descriptor."""

    geometry: Geometry
    properties: tuple[JsonMember, ...] = ()
    identifier: FeatureIdentifier = None


@dataclass(frozen=True, slots=True)
class GeometryGeoJson:
    """GeoJSON descriptor containing one geometry."""

    geometry: Geometry


@dataclass(frozen=True, slots=True)
class FeatureGeoJson:
    """GeoJSON descriptor containing one feature."""

    feature: Feature


@dataclass(frozen=True, slots=True)
class FeatureCollection:
    """GeoJSON descriptor containing an ordered feature collection."""

    features: tuple[Feature, ...]


GeoJson: TypeAlias = GeometryGeoJson | FeatureGeoJson | FeatureCollection


def empty_geometry() -> EmptyGeometry:
    """Create an empty geometry."""
    return EmptyGeometry()


def point(latitude: float, longitude: float) -> Point:
    """Create a point geometry from latitude and longitude."""
    return Point(LatLng(latitude, longitude))


def line_string(coordinates: list[LatLng] | tuple[LatLng, ...]) -> LineString:
    """Create a line string geometry."""
    return LineString(tuple(coordinates))


def polygon(rings: list[list[LatLng]] | tuple[tuple[LatLng, ...], ...]) -> Polygon:
    """Create a polygon geometry."""
    return Polygon(tuple(tuple(ring) for ring in rings))


def geometry_collection(
    geometries: list[Geometry] | tuple[Geometry, ...],
) -> GeometryCollection:
    """Create a geometry collection."""
    return GeometryCollection(tuple(geometries))


def _coordinate_to_native_wire(coordinate: LatLng) -> tuple[float, float]:
    return (coordinate.latitude, coordinate.longitude)


def _coordinates_to_native_wire(
    coordinates: tuple[LatLng, ...],
) -> list[tuple[float, float]]:
    return [_coordinate_to_native_wire(coordinate) for coordinate in coordinates]


def _geometry_to_native_wire(geometry: Geometry) -> dict[str, object]:
    if isinstance(geometry, EmptyGeometry):
        return {"type": "empty"}
    if isinstance(geometry, Point):
        return {
            "type": "point",
            "coordinate": _coordinate_to_native_wire(geometry.coordinate),
        }
    if isinstance(geometry, LineString):
        return {
            "type": "line_string",
            "coordinates": _coordinates_to_native_wire(geometry.coordinates),
        }
    if isinstance(geometry, Polygon):
        return {
            "type": "polygon",
            "rings": [_coordinates_to_native_wire(ring) for ring in geometry.rings],
        }
    if isinstance(geometry, MultiPoint):
        return {
            "type": "multi_point",
            "coordinates": _coordinates_to_native_wire(geometry.coordinates),
        }
    if isinstance(geometry, MultiLineString):
        return {
            "type": "multi_line_string",
            "lines": [_coordinates_to_native_wire(line) for line in geometry.lines],
        }
    if isinstance(geometry, MultiPolygon):
        return {
            "type": "multi_polygon",
            "polygons": [
                [_coordinates_to_native_wire(ring) for ring in polygon]
                for polygon in geometry.polygons
            ],
        }
    if isinstance(geometry, GeometryCollection):
        return {
            "type": "geometry_collection",
            "geometries": [
                _geometry_to_native_wire(child) for child in geometry.geometries
            ],
        }
    msg = f"unsupported geometry value: {type(geometry).__name__}"
    raise TypeError(msg)


def _identifier_to_native_wire(identifier: FeatureIdentifier) -> dict[str, object]:
    if identifier is None:
        return {"type": "null"}
    if isinstance(identifier, FeatureIdentifierUInt):
        return {"type": "uint", "value": identifier.value}
    if isinstance(identifier, FeatureIdentifierInt):
        return {"type": "int", "value": identifier.value}
    if isinstance(identifier, FeatureIdentifierDouble):
        return {"type": "double", "value": identifier.value}
    if isinstance(identifier, FeatureIdentifierString):
        return {"type": "string", "value": identifier.value}
    msg = f"unsupported feature identifier: {type(identifier).__name__}"
    raise TypeError(msg)


def _feature_to_native_wire(feature: Feature) -> dict[str, object]:
    return {
        "geometry": _geometry_to_native_wire(feature.geometry),
        "properties": [
            (member.key, _json_to_native_wire(member.value))
            for member in feature.properties
        ],
        "identifier": _identifier_to_native_wire(feature.identifier),
    }


def _to_native_wire(value: GeoJson) -> dict[str, object]:
    """Convert an explicit GeoJSON tree to private native-bridge wire values."""
    if isinstance(value, GeometryGeoJson):
        return {
            "type": "geometry",
            "geometry": _geometry_to_native_wire(value.geometry),
        }
    if isinstance(value, FeatureGeoJson):
        return {"type": "feature", "feature": _feature_to_native_wire(value.feature)}
    if isinstance(value, FeatureCollection):
        return {
            "type": "feature_collection",
            "features": [
                _feature_to_native_wire(feature) for feature in value.features
            ],
        }
    msg = f"unsupported GeoJSON value: {type(value).__name__}"
    raise TypeError(msg)


__all__ = [
    "EmptyGeometry",
    "Feature",
    "FeatureCollection",
    "FeatureGeoJson",
    "FeatureIdentifier",
    "FeatureIdentifierDouble",
    "FeatureIdentifierInt",
    "FeatureIdentifierString",
    "FeatureIdentifierUInt",
    "GeoJson",
    "Geometry",
    "GeometryCollection",
    "GeometryGeoJson",
    "LatLng",
    "LatLngBounds",
    "LineString",
    "MultiLineString",
    "MultiPoint",
    "MultiPolygon",
    "Point",
    "Polygon",
    "empty_geometry",
    "geometry_collection",
    "line_string",
    "point",
    "polygon",
]

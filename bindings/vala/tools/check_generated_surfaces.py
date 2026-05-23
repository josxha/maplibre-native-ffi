#!/usr/bin/env python3
"""Fail when generated Vala review artifacts expose raw C ABI surfaces."""

from __future__ import annotations

from pathlib import Path
import re
import sys
import xml.etree.ElementTree as ET

CORE_NS = "http://www.gtk.org/introspection/core/1.0"

VAPI_FORBIDDEN_PATTERNS = {
    r"\bvoid\s*\*": "raw void pointer",
    r"\bScreenLineString\b": "raw screen-line storage record",
    r"\bStringView\b": "raw string pointer/length record",
    r"\bCustomGeometrySourceTileCallback\b": "raw callback record",
    r"\bquery_feature_extensions\b": "raw feature-extension query entry point",
    r"\bFeatureCollection\b": "raw feature collection record",
    r"\bJsonMember\b": "raw JSON member record",
    r"\b[A-Za-z]+OptionFields\b": "raw field-mask enum",
    r"\b(?:FeatureExtensionResultHandle|FeatureQueryResultHandle|JsonSnapshotHandle|OfflineRegionListHandle|OfflineRegionSnapshotHandle|StyleIdListHandle)(?:Class)?\b": "native result/list/snapshot handle",
    r"\bpublic\s+weak\s+string\b": "public weak string field",
    r"\bpublic\s+weak\s+MaplibreNative\.Geometry\b": "public borrowed geometry field",
    r"\bset_(?:source_id|source_layer_id|feature_id|state_key|layer_ids|source_layer_ids|attribution)\b": "sidecar-backed descriptor setter",
    r"\bget_raw_(?:type|source_type|payload_type)\b": "raw runtime event getter",
}

VAPI_DESCRIPTOR_RECORDS = {
    "AnimationOptions",
    "BoundOptions",
    "CameraFitOptions",
    "CameraOptions",
    "CustomGeometrySourceOptions",
    "FeatureStateSelector",
    "FreeCameraOptions",
    "MapTileOptions",
    "MapViewportOptions",
    "ProjectionMode",
    "RenderedFeatureQueryOptions",
    "RuntimeOptions",
    "SourceFeatureQueryOptions",
    "StyleImageOptions",
    "StyleTileSourceOptions",
}

GIR_TOP_LEVEL_FORBIDDEN = {
    "AnimationOptionFields",
    "BoundOptionFields",
    "CameraFitOptionFields",
    "CameraOptionFields",
    "CustomGeometrySourceOptionFields",
    "CustomGeometrySourceTileCallback",
    "Feature",
    "FeatureCollection",
    "FeatureExtensionResultHandle",
    "FeatureExtensionResultHandleClass",
    "FeatureQueryResultHandle",
    "FeatureQueryResultHandleClass",
    "FeatureStateSelectorFields",
    "FreeCameraOptionFields",
    "JsonMember",
    "JsonSnapshotHandle",
    "JsonSnapshotHandleClass",
    "MapTileOptionFields",
    "MapViewportOptionFields",
    "OfflineRegionListHandle",
    "OfflineRegionListHandleClass",
    "OfflineRegionSnapshotHandle",
    "OfflineRegionSnapshotHandleClass",
    "ProjectionModeFields",
    "QueriedFeatureFields",
    "RenderedFeatureQueryOptionFields",
    "RuntimeOptionFlags",
    "ScreenLineString",
    "StringView",
    "SourceFeatureQueryOptionFields",
    "StringView",
    "StyleIdListHandle",
    "StyleIdListHandleClass",
    "StyleImageOptionFields",
    "StyleTileSourceOptionFields",
}

GIR_FORBIDDEN_METHODS = {
    "FeatureStateSelector": {
        "set_source_id",
        "set_source_layer_id",
        "set_feature_id",
        "set_state_key",
    },
    "RenderedFeatureQueryOptions": {"set_layer_ids"},
    "SourceFeatureQueryOptions": {"set_source_layer_ids"},
    "StyleTileSourceOptions": {"set_attribution"},
    "RuntimeEvent": {"get_raw_type", "get_raw_source_type", "get_raw_payload_type"},
}

GIR_FORBIDDEN_FIELDS = {
    "AnimationOptions": {"size", "fields"},
    "BoundOptions": {"size", "fields"},
    "CameraFitOptions": {"size", "fields"},
    "CameraOptions": {"size", "fields"},
    "CustomGeometrySourceOptions": {
        "size",
        "fields",
        "fetch_tile",
        "cancel_tile",
        "user_data",
    },
    "FeatureStateSelector": {"size", "fields"},
    "FreeCameraOptions": {"size", "fields"},
    "MapOptions": {"size"},
    "MapTileOptions": {"size", "fields"},
    "MapViewportOptions": {"size", "fields"},
    "MetalBorrowedTextureDescriptor": {"size", "texture"},
    "MetalContextDescriptor": {"size", "device"},
    "MetalOwnedTextureDescriptor": {"size"},
    "MetalSurfaceDescriptor": {"size", "layer"},
    "OfflineGeometryRegionDefinition": {"size", "style_url", "geometry"},
    "OfflineRegionDefinition": {"size", "data"},
    "OfflineRegionInfo": {"size", "metadata", "metadata_size"},
    "OfflineTilePyramidRegionDefinition": {"size", "style_url"},
    "PremultipliedRgba8Image": {"size", "pixels", "byte_length"},
    "ProjectionMode": {"size", "fields"},
    "RenderedFeatureQueryOptions": {
        "size",
        "fields",
        "layer_ids",
        "layer_id_count",
        "filter",
    },
    "RenderedQueryGeometry": {"size", "data"},
    "ResourceRequest": {"size", "url", "prior_etag", "prior_data", "prior_data_size"},
    "ResourceResponse": {"size", "bytes", "byte_count", "error_message", "etag"},
    "RuntimeEventOfflineOperationCompleted": {"raw_operation_kind", "raw_result_kind"},
    "RuntimeEventOfflineRegionResponseError": {"raw_reason"},
    "RuntimeEventRenderFrame": {"raw_mode"},
    "RuntimeEventRenderMap": {"raw_mode"},
    "RuntimeEventTileAction": {"raw_operation"},
    "RuntimeOptions": {"size", "flags", "asset_path", "cache_path"},
    "SourceFeatureQueryOptions": {
        "size",
        "fields",
        "source_layer_ids",
        "source_layer_id_count",
        "filter",
    },
    "StyleImageInfo": {"size", "byte_length"},
    "StyleImageOptions": {"size", "fields"},
    "StyleSourceInfo": {"size", "id_size", "attribution_size"},
    "StyleTileSourceOptions": {"size", "fields", "attribution_size"},
    "TextureImageInfo": {"size", "byte_length"},
    "VulkanBorrowedTextureDescriptor": {"size", "image", "image_view"},
    "VulkanContextDescriptor": {
        "size",
        "instance",
        "physical_device",
        "device",
        "graphics_queue",
        "graphics_queue_family_index",
    },
    "VulkanOwnedTextureDescriptor": {"size"},
    "VulkanSurfaceDescriptor": {"size", "surface"},
}


def tag(name: str) -> str:
    return f"{{{CORE_NS}}}{name}"


def local_name(element: ET.Element) -> str:
    return element.tag.rsplit("}", 1)[-1]


def record_blocks(vapi: str) -> dict[str, str]:
    blocks: dict[str, str] = {}
    pattern = re.compile(r"\tpublic struct (\w+) \{\n(.*?)\n\t\}", re.S)
    for match in pattern.finditer(vapi):
        blocks[match.group(1)] = match.group(2)
    return blocks


def check_vapi(path: Path) -> list[str]:
    text = path.read_text()
    errors: list[str] = []
    for pattern, description in VAPI_FORBIDDEN_PATTERNS.items():
        if re.search(pattern, text):
            errors.append(f"{path}: exposes {description} ({pattern})")
    blocks = record_blocks(text)
    for record in sorted(VAPI_DESCRIPTOR_RECORDS):
        block = blocks.get(record)
        if block is None:
            continue
        for field in ("size", "fields"):
            if re.search(rf"\bpublic\s+\w+\s+{field}\s*;", block):
                errors.append(f"{path}: {record}.{field} is public")
    return errors


def check_gir(path: Path) -> list[str]:
    tree = ET.parse(path)
    root = tree.getroot()
    namespace = root.find(tag("namespace"))
    if namespace is None:
        return [f"{path}: missing GIR namespace"]

    errors: list[str] = []
    for child in namespace:
        child_name = child.attrib.get("name", "")
        if child_name in GIR_TOP_LEVEL_FORBIDDEN:
            errors.append(f"{path}: exposes top-level raw type {child_name}")
        forbidden_fields = GIR_FORBIDDEN_FIELDS.get(child_name, set())
        forbidden_methods = GIR_FORBIDDEN_METHODS.get(child_name, set())
        for descendant in child.iter():
            descendant_name = descendant.attrib.get("name", "")
            match local_name(descendant):
                case "field":
                    if descendant_name in forbidden_fields:
                        errors.append(f"{path}: exposes {child_name}.{descendant_name}")
                case "method" | "function":
                    if descendant_name in forbidden_methods:
                        errors.append(f"{path}: exposes {child_name}.{descendant_name}")
    return errors


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: check_generated_surfaces.py VAPI GIR")
    errors = check_vapi(Path(sys.argv[1])) + check_gir(Path(sys.argv[2]))
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()

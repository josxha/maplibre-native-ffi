#!/usr/bin/env python3
"""Remove C-ABI bookkeeping surfaces from generated Vala review GIR files."""

from __future__ import annotations

from pathlib import Path
import sys
import xml.etree.ElementTree as ET

CORE_NS = "http://www.gtk.org/introspection/core/1.0"
C_NS = "http://www.gtk.org/introspection/c/1.0"
DOC_NS = "http://www.gtk.org/introspection/doc/1.0"
GLIB_NS = "http://www.gtk.org/introspection/glib/1.0"

ET.register_namespace("", CORE_NS)
ET.register_namespace("c", C_NS)
ET.register_namespace("doc", DOC_NS)
ET.register_namespace("glib", GLIB_NS)

TOP_LEVEL_SKIP = {
    "AnimationOptionFields",
    "BoundOptionFields",
    "CameraFitOptionFields",
    "CameraOptionFields",
    "CustomGeometrySourceOptionFields",
    "CustomGeometrySourceTileCallback",
    "Feature",
    "FeatureCollection",
    "FeatureStateSelectorFields",
    "FreeCameraOptionFields",
    "JsonMember",
    "MapTileOptionFields",
    "MapViewportOptionFields",
    "ProjectionModeFields",
    "QueriedFeatureFields",
    "RenderedFeatureQueryOptionFields",
    "RuntimeOptionFlags",
    "SourceFeatureQueryOptionFields",
    "StyleImageOptionFields",
    "StyleTileSourceOptionFields",
}

FIELD_SKIP = {
    "attribution_size",
    "byte_count",
    "byte_length",
    "bytes",
    "data",
    "feature",
    "feature_count",
    "fields",
    "filter",
    "id_size",
    "layer_id_count",
    "layer_ids",
    "metadata",
    "metadata_size",
    "pixels",
    "prior_data",
    "prior_data_size",
    "property_count",
    "size",
    "source_layer_id_count",
    "source_layer_ids",
    "state",
}

METHOD_SKIP = {
    "RenderSessionHandle": {"query_feature_extensions"},
}


def tag(name: str) -> str:
    return f"{{{CORE_NS}}}{name}"


def local_name(element: ET.Element) -> str:
    return element.tag.rsplit("}", 1)[-1]


def sanitize(path: Path) -> None:
    tree = ET.parse(path)
    root = tree.getroot()
    namespace = root.find(tag("namespace"))
    if namespace is None:
        raise SystemExit(f"{path}: missing GIR namespace")

    for child in list(namespace):
        if child.attrib.get("name") in TOP_LEVEL_SKIP:
            namespace.remove(child)
            continue
        child_name = child.attrib.get("name")
        for nested in list(child):
            nested_tag = local_name(nested)
            nested_name = nested.attrib.get("name")
            if nested_tag == "field" and nested_name in FIELD_SKIP:
                child.remove(nested)
            elif nested_tag in {
                "method",
                "function",
            } and nested_name in METHOD_SKIP.get(child_name, set()):
                child.remove(nested)

    tree.write(path, encoding="utf-8", xml_declaration=True)


def main() -> None:
    if len(sys.argv) < 2:
        raise SystemExit("usage: sanitize_gir.py GIR [GIR ...]")
    for arg in sys.argv[1:]:
        sanitize(Path(arg))


if __name__ == "__main__":
    main()

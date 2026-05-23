#!/usr/bin/env python3
"""Remove C-ABI bookkeeping surfaces from generated Vala review GIR files."""

from __future__ import annotations

from pathlib import Path
import re
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

ROOT = Path(__file__).resolve().parents[1]
VAPIGEN_METADATA = ROOT / "metadata" / "MaplibreNative-0.1.vapigen.metadata"

METHOD_SKIP = {
    "RenderSessionHandle": {"query_feature_extensions"},
}

TOP_LEVEL_RE = re.compile(r"^(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s+skip$")
FIELD_RE = re.compile(
    r"^(?P<owner>[A-Za-z_][A-Za-z0-9_]*)\.(?P<field>[A-Za-z_][A-Za-z0-9_]*)(?:\.\*)?\s+skip$"
)


def metadata_skips() -> tuple[set[str], dict[str, set[str]]]:
    top_level: set[str] = set()
    fields: dict[str, set[str]] = {}
    for raw_line in VAPIGEN_METADATA.read_text().splitlines():
        line = raw_line.split("#", 1)[0].strip()
        if not line or line.startswith("MaplibreNative "):
            continue
        if match := TOP_LEVEL_RE.match(line):
            top_level.add(match.group("name"))
            continue
        if match := FIELD_RE.match(line):
            fields.setdefault(match.group("owner"), set()).add(match.group("field"))
    return top_level, fields


def tag(name: str) -> str:
    return f"{{{CORE_NS}}}{name}"


def local_name(element: ET.Element) -> str:
    return element.tag.rsplit("}", 1)[-1]


def sanitize_child(
    parent: ET.Element,
    owner_name: str | None,
    field_skip: dict[str, set[str]],
) -> None:
    owner_fields = field_skip.get(owner_name or "", set())
    for child in list(parent):
        child_tag = local_name(child)
        child_name = child.attrib.get("name")
        if child_tag in {"field", "union"} and child_name in owner_fields:
            parent.remove(child)
            continue
        if child_tag in {"method", "function"} and child_name in METHOD_SKIP.get(
            owner_name,
            set(),
        ):
            parent.remove(child)
            continue
        sanitize_child(child, owner_name, field_skip)


def sanitize(
    path: Path, top_level_skip: set[str], field_skip: dict[str, set[str]]
) -> None:
    tree = ET.parse(path)
    root = tree.getroot()
    namespace = root.find(tag("namespace"))
    if namespace is None:
        raise SystemExit(f"{path}: missing GIR namespace")

    for child in list(namespace):
        child_name = child.attrib.get("name")
        if child_name in top_level_skip:
            namespace.remove(child)
            continue
        sanitize_child(child, child_name, field_skip)

    tree.write(path, encoding="utf-8", xml_declaration=True)


def main() -> None:
    if len(sys.argv) < 2:
        raise SystemExit("usage: sanitize_gir.py GIR [GIR ...]")
    top_level_skip, field_skip = metadata_skips()
    for arg in sys.argv[1:]:
        sanitize(Path(arg), top_level_skip, field_skip)


if __name__ == "__main__":
    main()

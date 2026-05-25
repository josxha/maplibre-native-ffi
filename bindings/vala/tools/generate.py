#!/usr/bin/env python3
"""Generate scanner-facing Vala binding artifacts from binding metadata."""

from __future__ import annotations

from collections import defaultdict
from pathlib import Path
import json
import re
import tomllib

ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = ROOT.parents[1]
METADATA = ROOT / "metadata" / "api.toml"
HEADER_METADATA = ROOT / "metadata" / "header.jsonl"
C_HEADERS = sorted((REPO_ROOT / "include" / "maplibre_native_c").glob("*.h"))


def metadata_path(value: str) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    return (ROOT / path).resolve()


def load_metadata() -> dict:
    with METADATA.open("rb") as metadata_file:
        return tomllib.load(metadata_file)


def load_header_records() -> list[dict]:
    records: list[dict] = []
    for line in HEADER_METADATA.read_text().splitlines():
        stripped = line.strip()
        if stripped:
            records.append(json.loads(stripped))
    return records


def group_records(records: list[dict]) -> dict[str, list[dict]]:
    grouped: dict[str, list[dict]] = defaultdict(list)
    for record in records:
        grouped[record["kind"]].append(record)
    return grouped


def vala_to_mln_type(name: str) -> str:
    suffix = name.removeprefix("MlnVala")
    return "mln_" + re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", suffix).lower()


def c_header_text() -> str:
    return "\n".join(path.read_text() for path in C_HEADERS)


def c_struct_bodies(headers: str) -> dict[str, str]:
    pattern = re.compile(
        r"typedef\s+struct\s+(?P<c_name>mln_\w+)\s*\{(?P<body>.*?)\}\s*(?P=c_name)\s*;",
        re.S,
    )
    return {
        match.group("c_name"): match.group("body")
        for match in pattern.finditer(headers)
    }


def c_simple_typedefs(headers: str) -> dict[str, str]:
    pattern = re.compile(
        r"typedef\s+(?P<target>[A-Za-z_][\w\s\*]*?)\s+(?P<name>mln_\w+)\s*;"
    )
    return {
        match.group("name"): " ".join(match.group("target").split())
        for match in pattern.finditer(headers)
        if not match.group("target").strip().startswith(("struct", "enum"))
    }


def strip_c_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    text = re.sub(r"//.*", "", text)
    return text


def render_enum(record: dict) -> str:
    lines = ["typedef enum {"]
    for value in record["values"]:
        assignment = f" = {value['value']}" if "value" in value else ""
        lines.append(f"  {value['name']}{assignment},")
    lines.append(f"}} {record['name']};")
    return "\n".join(lines)


def render_struct_alias(
    record: dict,
    bodies: dict[str, str],
    type_names: dict[str, str],
    simple_typedefs: dict[str, str],
) -> str:
    c_type = record["c_type"]
    if "body" in record:
        body = record["body"]
    else:
        if c_type not in bodies:
            raise SystemExit(f"missing C struct definition for {c_type}")
        body = strip_c_comments(bodies[c_type])
        for c_name, vala_name in sorted(
            type_names.items(), key=lambda item: len(item[0]), reverse=True
        ):
            body = re.sub(rf"\b{re.escape(c_name)}\b", vala_name, body)
        for c_name, target in sorted(
            simple_typedefs.items(), key=lambda item: len(item[0]), reverse=True
        ):
            body = re.sub(rf"\b{re.escape(c_name)}\b", target, body)
    body = re.sub(r"\n\s*\n+", "\n", body).strip("\n")
    return f"struct {record['name']} {{\n{body}\n}};"


def render_boxed(record: dict) -> str:
    lines: list[str] = []
    macro = record.get("macro")
    if macro:
        lines.append(
            f"#define MLN_VALA_TYPE_{macro} (mln_vala_{record['lower']}_get_type())"
        )
    if record.get("forward"):
        lines.append(f"typedef struct _{record['name']} {record['name']};")
    lines.append(f"GType mln_vala_{record['lower']}_get_type(void);")
    return "\n".join(lines)


def render_object(record: dict) -> str:
    return (
        f"#define MLN_VALA_TYPE_{record['macro']} "
        f"(mln_vala_{record['lower']}_get_type())\n"
        f"G_DECLARE_FINAL_TYPE(\n"
        f"  {record['name']}, mln_vala_{record['lower']}, MLN_VALA, "
        f"{record['macro']}, GObject\n"
        f")"
    )


def render_doc(
    name: str,
    parameters: list[dict],
    returns: str,
    throws: bool,
    return_annotations: str = "",
) -> str:
    annotated = [parameter for parameter in parameters if parameter.get("annotations")]
    if not annotated and not throws and not returns and not return_annotations:
        return ""
    lines = ["/**", f" * {name}:"]
    for parameter in annotated:
        param_name = parameter["declaration"].split()[-1].lstrip("*")
        lines.append(f" * @{param_name}: {parameter['annotations']}: parameter.")
    if return_annotations:
        lines.extend([" *", f" * Returns: {return_annotations}: result."])
    elif returns:
        lines.extend([" *", f" * Returns: (transfer {returns}): result."])
    if throws:
        lines.extend([" *", " * Throws: MlnValaError"])
    lines.append(" */")
    return "\n".join(lines)


def render_callable(record: dict, typedef: bool) -> str:
    doc = render_doc(
        record["name"],
        record["parameters"],
        record.get("return_transfer", "") if not typedef else "",
        bool(record.get("throws")) if not typedef else False,
        record.get("return_annotations", ""),
    )
    params = ",\n  ".join(
        parameter["declaration"] for parameter in record["parameters"]
    )
    if typedef:
        declaration = f"typedef {record['return']} (*{record['name']})(\n  {params}\n);"
    else:
        declaration = f"{record['return']} {record['name']}(\n  {params}\n);"
    return f"{doc}\n{declaration}" if doc else declaration


def render_header(metadata: dict, records: list[dict]) -> str:
    namespace = metadata["namespace"]
    grouped = group_records(records)
    type_names: dict[str, str] = {}
    for record in grouped["alias"]:
        type_names[record["c_type"]] = record["name"]
    for record in grouped["enum"]:
        # These names intentionally include Vala-specific plurals for flags.
        snake = re.sub(
            r"([a-z0-9])([A-Z])", r"\1_\2", record["name"].removeprefix("MlnVala")
        ).lower()
        type_names[f"mln_{snake}"] = record["name"]
    for kind in ("opaque", "boxed", "callback"):
        for record in grouped[kind]:
            type_names[vala_to_mln_type(record["name"])] = record["name"]
    headers = c_header_text()
    bodies = c_struct_bodies(headers)
    simple_typedefs = c_simple_typedefs(headers)

    sections: list[str] = [
        "#pragma once",
        "",
        "#include <glib-object.h>",
        "#include <stdbool.h>",
        "#include <stddef.h>",
        "#include <stdint.h>",
        "",
        "G_BEGIN_DECLS",
        "",
        "#define MLN_VALA_ERROR (mln_vala_error_quark())",
        "",
    ]
    sections.extend(render_enum(record) for record in grouped["enum"])
    sections.append("")
    sections.extend(
        f"typedef struct {record['name']} {record['name']};"
        for record in grouped["alias"]
    )
    sections.extend(
        f"typedef struct _{record['name']} {record['name']};"
        for record in grouped["opaque"]
    )
    sections.append("")
    sections.extend(
        render_callable(record, typedef=True) for record in grouped["callback"]
    )
    sections.append("")
    sections.extend(
        render_struct_alias(record, bodies, type_names, simple_typedefs)
        for record in grouped["alias"]
    )
    sections.append("")
    sections.extend(render_boxed(record) for record in grouped["boxed"])
    sections.append("")
    sections.extend(render_object(record) for record in grouped["object"])
    sections.append("")
    sections.extend(
        render_callable(record, typedef=False)
        for record in grouped["function"]
        if not record["name"].endswith("_get_type")
    )
    sections.extend(["", "G_END_DECLS", ""])

    header = "\n\n".join(section for section in sections if section != "")
    return (
        "/*\n"
        " * This scanner-facing header is generated by bindings/vala/tools/generate.py.\n"
        " * Inputs: bindings/vala/metadata/api.toml, bindings/vala/metadata/header.jsonl,\n"
        " * and public C structs from include/maplibre_native_c/.\n"
        " * Edit metadata or Rust adapter sources; do not edit build/include output.\n"
        f" * GIR namespace: {namespace['gir_namespace']}-{namespace['gir_version']}; "
        f"Vala namespace: {namespace['vala_namespace']}.\n"
        f" * Metadata inventory: {len(grouped['function'])} functions, "
        f"{sum(len(grouped[k]) for k in ('enum', 'alias', 'opaque', 'boxed', 'object', 'callback'))} public C types.\n"
        " */\n\n" + header
    )


def main() -> None:
    metadata = load_metadata()
    records = load_header_records()
    output_path = metadata_path(metadata["scanner_header"]["output"])
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(render_header(metadata, records))


if __name__ == "__main__":
    main()

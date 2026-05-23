#!/usr/bin/env python3
"""Generate scanner-facing Vala binding artifacts for the current proof slice."""

from __future__ import annotations

from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]
CRATE_HEADER = (
    ROOT
    / "crates"
    / "maplibre-native-vala"
    / "include"
    / "maplibre-native-vala"
    / "maplibre-native-vala.h"
)
GENERATED_INCLUDE = ROOT / "build" / "include" / "maplibre-native-vala.h"


def main() -> None:
    GENERATED_INCLUDE.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(CRATE_HEADER, GENERATED_INCLUDE)


if __name__ == "__main__":
    main()

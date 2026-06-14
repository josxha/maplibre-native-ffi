#!/usr/bin/env bash
# Install headless software graphics drivers for CI only (Windows Mesa, macOS MoltenVK).
set -euo pipefail

repo_root="${GITHUB_WORKSPACE:-${MLN_FFI_REPO_ROOT:?}}"
target_triple="${MLN_FFI_TARGET_TRIPLE:?}"
render_backend="${MLN_FFI_RENDER_BACKEND:-}"

graphics_root="$repo_root/build/native-graphics/$target_triple"
mkdir -p "$graphics_root"

case "$target_triple" in
  windows-x64)
    mesa_version="26.0.3"
    mesa_build="h667bac9_0"
    mesa_prefix="$graphics_root/mesa"
    mesa_marker="$mesa_prefix/Library/share/vulkan/icd.d/lvp_icd.x86_64.json"

    if [[ ! -f "$mesa_marker" ]]; then
      tmp_dir="$(mktemp -d)"
      curl -fsSL \
        "https://conda.anaconda.org/conda-forge/win-64/mesalib-${mesa_version}-${mesa_build}.conda" \
        -o "$tmp_dir/mesa.conda"
      mkdir -p "$mesa_prefix"
      python3 - "$tmp_dir/mesa.conda" "$mesa_prefix" <<'PY'
import sys
import zipfile
from pathlib import Path

archive = Path(sys.argv[1])
prefix = Path(sys.argv[2])
with zipfile.ZipFile(archive) as zf:
    for member in zf.namelist():
        if member.startswith("Library/"):
            target = prefix / member
            target.parent.mkdir(parents=True, exist_ok=True)
            with zf.open(member) as src, target.open("wb") as dst:
                dst.write(src.read())
PY
      rm -rf "$tmp_dir"
    fi

    if [[ -n "${GITHUB_ENV:-}" ]]; then
      mesa_lib="$mesa_prefix/Library/bin"
      vk_icd="$mesa_prefix/Library/share/vulkan/icd.d/lvp_icd.x86_64.json"
      {
        echo "MLN_FFI_GRAPHICS_ROOT=$graphics_root"
        echo "MLN_FFI_GRAPHICS_LIBRARY_DIR=$mesa_lib"
        echo "VK_ADD_DRIVER_FILES=$vk_icd"
      } >>"$GITHUB_ENV"
    fi
    ;;
  macos-arm64)
    if [[ "$render_backend" != "vulkan" ]]; then
      exit 0
    fi

    molten_version="1.4.1"
    molten_build="h407b865_0"
    molten_prefix="$graphics_root/moltenvk"
    molten_marker="$molten_prefix/share/vulkan/icd.d/MoltenVK_icd.json"

    if [[ ! -f "$molten_marker" ]]; then
      tmp_dir="$(mktemp -d)"
      curl -fsSL \
        "https://conda.anaconda.org/conda-forge/osx-arm64/moltenvk-${molten_version}-${molten_build}.conda" \
        -o "$tmp_dir/moltenvk.conda"
      mkdir -p "$molten_prefix"
      python3 - "$tmp_dir/moltenvk.conda" "$molten_prefix" <<'PY'
import sys
import zipfile
from pathlib import Path

archive = Path(sys.argv[1])
prefix = Path(sys.argv[2])
with zipfile.ZipFile(archive) as zf:
    for member in zf.namelist():
        if not member.endswith("/"):
            target = prefix / member
            target.parent.mkdir(parents=True, exist_ok=True)
            with zf.open(member) as src, target.open("wb") as dst:
                dst.write(src.read())
PY
      rm -rf "$tmp_dir"
    fi

    if [[ -n "${GITHUB_ENV:-}" ]]; then
      molten_lib="$molten_prefix/lib"
      vk_icd="$molten_prefix/share/vulkan/icd.d/MoltenVK_icd.json"
      {
        echo "MLN_FFI_GRAPHICS_ROOT=$graphics_root"
        echo "MLN_FFI_GRAPHICS_LIBRARY_DIR=$molten_lib"
        echo "VK_ICD_FILENAMES=$vk_icd"
        echo "JDK_JAVA_OPTIONS=-Djava.library.path=$molten_lib -Dorg.lwjgl.vulkan.libname=libvulkan.1.dylib"
      } >>"$GITHUB_ENV"
    fi
    ;;
  *)
    exit 0
    ;;
esac

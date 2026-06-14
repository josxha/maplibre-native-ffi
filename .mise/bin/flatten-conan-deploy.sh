#!/usr/bin/env bash
# Conan full_deploy lays out one directory per package (curl, zlib, png, …).
# CMake and Zig tests expect a single prefix with merged include/, lib/, and
# bin/ trees — the same layout the old pixi/conda CONDA_PREFIX provided for
# MLN_FFI_DEPENDENCY_* paths and runtime search paths.
set -euo pipefail

deploy_root="${1:?usage: flatten-conan-deploy.sh <deploy-root> <output-root>}"
output_root="${2:?usage: flatten-conan-deploy.sh <deploy-root> <output-root>}"
host_root="$deploy_root/full_deploy/host"

if [[ ! -d "$host_root" ]]; then
  echo "missing Conan deploy host root: $host_root" >&2
  exit 1
fi

mkdir -p "$output_root/include" "$output_root/lib" "$output_root/bin"

while IFS= read -r -d '' package_root; do
  if [[ -d "$package_root/include" ]]; then
    cp -a "$package_root/include/." "$output_root/include/"
  fi
  if [[ -d "$package_root/lib" ]]; then
    cp -a "$package_root/lib/." "$output_root/lib/"
  fi
  if [[ -d "$package_root/bin" ]]; then
    cp -a "$package_root/bin/." "$output_root/bin/"
  fi
done < <(find "$host_root" -mindepth 1 -maxdepth 1 -type d -print0)

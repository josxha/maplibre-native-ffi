#!/usr/bin/env bash
#MISE description="Fetch Android Prefab native deps for standalone CMake builds"
set -euo pipefail

: "${MLN_FFI_REPO_ROOT:?MLN_FFI_REPO_ROOT is required}"
: "${MLN_FFI_ANDROID_ABI:?MLN_FFI_ANDROID_ABI is required}"
: "${MLN_FFI_ANDROID_PLATFORM:?MLN_FFI_ANDROID_PLATFORM is required}"
: "${MLN_FFI_ANDROID_NDK_VERSION:?MLN_FFI_ANDROID_NDK_VERSION is required}"

source "$MLN_FFI_REPO_ROOT/.mise/bin/android-sdk-env.sh"

versions_file="$MLN_FFI_REPO_ROOT/platform/android-deps/dep-versions.toml"
read_versions() {
  python3 - "$versions_file" <<'PY'
import sys
import tomllib

with open(sys.argv[1], "rb") as file:
    data = tomllib.load(file)

print(data["curl"]["version"])
print(data["curl"]["boringssl_version"])
print(data["prefab-cli"]["version"])
print(data["prefab-cli"]["url"])
PY
}

mapfile -t versions < <(read_versions)
CURL_VERSION="${versions[0]}"
BORINGSSL_VERSION="${versions[1]}"
PREFAB_CLI_VERSION="${versions[2]}"
PREFAB_CLI_URL="${versions[3]}"

ANDROID_API_LEVEL="${MLN_FFI_ANDROID_PLATFORM#android-}"
NDK_MAJOR_VERSION="${MLN_FFI_ANDROID_NDK_VERSION%%.*}"
STL="c++_static"

prefab_root="$MLN_FFI_REPO_ROOT/build/android-prefab"
prefab_out="$prefab_root/$MLN_FFI_ANDROID_ABI"
cache_dir="$prefab_root/cache"
curl_aar="$cache_dir/curl-${CURL_VERSION}.aar"
curl_extract="$cache_dir/curl-${CURL_VERSION}"
boringssl_aar="$cache_dir/boringssl-${BORINGSSL_VERSION}.aar"
boringssl_extract="$cache_dir/boringssl-${BORINGSSL_VERSION}"
prefab_jar="$cache_dir/prefab-cli-${PREFAB_CLI_VERSION}.jar"
stamp_file="$prefab_out/.prefab-stamp"

if [[ -f "$stamp_file" ]]; then
  IFS= read -r cached_stamp <"$stamp_file"
  current_stamp="${CURL_VERSION}:${BORINGSSL_VERSION}:${PREFAB_CLI_VERSION}:${MLN_FFI_ANDROID_ABI}:${ANDROID_API_LEVEL}:${NDK_MAJOR_VERSION}:${STL}"
  if [[ "$cached_stamp" == "$current_stamp" ]]; then
    exit 0
  fi
fi

mkdir -p "$cache_dir" "$prefab_out"

download() {
  env -u LD_LIBRARY_PATH /usr/bin/curl -fsSL "$@"
}

if [[ ! -f "$curl_aar" ]]; then
  maven_url="https://repo1.maven.org/maven2/io/github/vvb2060/ndk/curl/${CURL_VERSION}/curl-${CURL_VERSION}.aar"
  download "$maven_url" -o "$curl_aar"
fi

if [[ ! -f "$boringssl_aar" ]]; then
  maven_url="https://repo1.maven.org/maven2/io/github/vvb2060/ndk/boringssl/${BORINGSSL_VERSION}/boringssl-${BORINGSSL_VERSION}.aar"
  download "$maven_url" -o "$boringssl_aar"
fi

rm -rf "$curl_extract" "$boringssl_extract"
mkdir -p "$curl_extract" "$boringssl_extract"
unzip -qo "$curl_aar" "prefab/*" -d "$curl_extract"
unzip -qo "$boringssl_aar" "prefab/*" -d "$boringssl_extract"

if [[ ! -f "$prefab_jar" ]]; then
  download "$PREFAB_CLI_URL" -o "$prefab_jar"
fi

rm -rf "$prefab_out"
mkdir -p "$prefab_out"

java -jar "$prefab_jar" \
  --build-system cmake \
  --platform android \
  --abi "$MLN_FFI_ANDROID_ABI" \
  --os-version "$ANDROID_API_LEVEL" \
  --stl "$STL" \
  --ndk-version "$NDK_MAJOR_VERSION" \
  --output "$prefab_out" \
  "$boringssl_extract/prefab" \
  "$curl_extract/prefab"

printf '%s\n' "${CURL_VERSION}:${BORINGSSL_VERSION}:${PREFAB_CLI_VERSION}:${MLN_FFI_ANDROID_ABI}:${ANDROID_API_LEVEL}:${NDK_MAJOR_VERSION}:${STL}" >"$stamp_file"

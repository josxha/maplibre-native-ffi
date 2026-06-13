#!/usr/bin/env bash
set -euo pipefail

: "${MLN_FFI_ANDROID_NDK_VERSION:?MLN_FFI_ANDROID_NDK_VERSION is required}"

if [[ -z "${ANDROID_SDK_ROOT:-}" ]]; then
  ANDROID_SDK_ROOT="$(mise where "vfox:mise-plugins/vfox-android-sdk")"
  export ANDROID_SDK_ROOT
fi

export ANDROID_NDK_ROOT="${ANDROID_SDK_ROOT}/ndk/${MLN_FFI_ANDROID_NDK_VERSION}"
export ANDROID_NDK="${ANDROID_NDK_ROOT}"

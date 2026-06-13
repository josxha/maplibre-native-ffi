#!/usr/bin/env bash
set -euo pipefail

: "${MLN_FFI_ANDROID_NDK_VERSION:?MLN_FFI_ANDROID_NDK_VERSION is required}"

if [[ -z "${ANDROID_SDK_ROOT:-}" && -z "${ANDROID_HOME:-}" ]]; then
  cat >&2 <<'EOF'
Android builds require ANDROID_SDK_ROOT or ANDROID_HOME, similar to Xcode on macOS.

Install the Android SDK command-line tools or Android Studio, then export one of:
  export ANDROID_SDK_ROOT=/path/to/Android/Sdk
  export ANDROID_HOME=/path/to/Android/Sdk

Optional: run `mise run //:ensure-android-sdk` to install the pinned NDK with sdkmanager.
EOF
  exit 1
fi

export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-${ANDROID_HOME}}"
export ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT}}"
export ANDROID_NDK_ROOT="${ANDROID_SDK_ROOT}/ndk/${MLN_FFI_ANDROID_NDK_VERSION}"
export ANDROID_NDK="${ANDROID_NDK_ROOT}"

if [[ ! -d "${ANDROID_NDK_ROOT}/build/cmake" ]]; then
  cat >&2 <<EOF
Android NDK ${MLN_FFI_ANDROID_NDK_VERSION} was not found at:
  ${ANDROID_NDK_ROOT}

Install it with sdkmanager:
  sdkmanager "ndk;${MLN_FFI_ANDROID_NDK_VERSION}"
EOF
  exit 1
fi

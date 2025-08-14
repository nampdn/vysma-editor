#!/usr/bin/env bash

set -euo pipefail

RELEASE_MODE=${1:-}
CRATE_NAME=${CRATE:-bevy-in-app}
TARGET_TRIPLE=aarch64-linux-android
LIB_FOLDER="debug"

if [[ "${RELEASE_MODE:-}" == "--release" ]]; then
  LIB_FOLDER="release"
fi

# Build .so for Android (requires cargo-subcommand for NDK or cross)
# If you use `cargo so`, keep it; otherwise replace with your preferred toolchain.
if command -v cargo-so >/dev/null 2>&1; then
  if [[ "${RELEASE_MODE:-}" == "--release" ]]; then
    cargo so b -p "${CRATE_NAME}" --lib --target "${TARGET_TRIPLE}" ${RELEASE_MODE}
  else
    RUST_BACKTRACE=full RUST_LOG=wgpu_hal=debug cargo so b -p "${CRATE_NAME}" --lib --target "${TARGET_TRIPLE}"
  fi
else
  cargo build -p "${CRATE_NAME}" --target "${TARGET_TRIPLE}" ${RELEASE_MODE}
fi

# Copy .so files to jniLibs folder
ARM64="android/app/libs/arm64-v8a"
mkdir -p "${ARM64}"

CRATE_LIB_BASENAME="lib$(echo "${CRATE_NAME}" | tr '-' '_')"
OUT_SO="target/${TARGET_TRIPLE}/${LIB_FOLDER}/${CRATE_LIB_BASENAME}.so"
cp "${OUT_SO}" "${ARM64}/${CRATE_LIB_BASENAME}.so"

echo "Copied ${OUT_SO} -> ${ARM64}/${CRATE_LIB_BASENAME}.so"
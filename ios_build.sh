#!/usr/bin/env bash

set -euo pipefail

TARGET=${1:-aarch64-apple-ios}
RELEASE_MODE=${2:-}
CRATE_NAME=${CRATE:-bevy-in-app}

# Convenience: allow calling with only --release
if [[ "${TARGET}" == "--release" ]]; then
  TARGET="aarch64-apple-ios"
  RELEASE_MODE="--release"
fi

# Build the specific crate for iOS target
cargo build -p "${CRATE_NAME}" --target "${TARGET}" ${RELEASE_MODE}

# Copy static lib into iOS/libs/{debug,release}
LIB_FOLDER="debug"
if [[ "${RELEASE_MODE:-}" == "--release" ]]; then
  LIB_FOLDER="release"
fi

CRATE_LIB_BASENAME="lib$(echo "${CRATE_NAME}" | tr '-' '_')"
OUT_LIB="target/${TARGET}/${LIB_FOLDER}/${CRATE_LIB_BASENAME}.a"
DEST_DIR="iOS/libs/${LIB_FOLDER}"
mkdir -p "${DEST_DIR}"
cp "${OUT_LIB}" "${DEST_DIR}/${CRATE_LIB_BASENAME}.a"

echo "Copied ${OUT_LIB} -> ${DEST_DIR}/${CRATE_LIB_BASENAME}.a"

#!/usr/bin/env bash
#
# Build a portable AppImage from the release binary using linuxdeploy.
# Usage: packaging/build-appimage.sh [output-dir]   (default: dist)
#
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUT_DIR="${1:-dist}"
BIN="target/release/rust-viewer-pro"

if [[ ! -x "$BIN" ]]; then
    echo "error: $BIN not found — run 'cargo build --release' first" >&2
    exit 1
fi

# Work on the repo's filesystem (target/), which is executable — some CI and
# sandbox setups mount /tmp as noexec, which breaks running the linuxdeploy
# AppImage and its FUSE-less extraction.
WORK="target/appimage-build"
rm -rf "$WORK"
mkdir -p "$WORK"
trap 'rm -rf "$WORK"' EXIT
export TMPDIR="$PWD/$WORK"
APPDIR="$WORK/AppDir"
mkdir -p "$APPDIR"

# Fetch linuxdeploy (pinned to the continuous channel; widely used in CI).
LD="$WORK/linuxdeploy-x86_64.AppImage"
curl -fsSL -o "$LD" \
    https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
chmod +x "$LD"

export ARCH=x86_64
# Run the tool without FUSE (CI runners lack it).
export APPIMAGE_EXTRACT_AND_RUN=1
export OUTPUT="rust-viewer-pro-x86_64.AppImage"

"$LD" --appdir "$APPDIR" \
    --executable "$BIN" \
    --desktop-file assets/rust-viewer-pro.desktop \
    --icon-file assets/icon-256.png \
    --icon-filename rust-viewer-pro \
    --output appimage

mkdir -p "$OUT_DIR"
mv -f "$OUTPUT" "$OUT_DIR/"
echo "AppImage -> $OUT_DIR/$OUTPUT"

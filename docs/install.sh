#!/usr/bin/env bash
set -euo pipefail

PROJECT="k0fiscan"
REPO="timson/{$PROJECT}"
BIN="k0fi"
INSTALL_DIR="/usr/local/bin"
[ -w "$INSTALL_DIR" ] || INSTALL_DIR="$HOME/.local/bin"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
    Linux-x86_64)    TARGET="x86_64-unknown-linux-gnu"    ;;
    Linux-aarch64)   TARGET="aarch64-unknown-linux-gnu"   ;;
    Darwin-x86_64)   TARGET="x86_64-apple-darwin"         ;;
    Darwin-arm64)    TARGET="aarch64-apple-darwin"        ;;
    *) echo "❌ Unsupported platform: $OS $ARCH"; exit 1 ;;
esac

echo https://api.github.com/repos/${REPO}/releases/latest

LATEST=$(curl -s https://api.github.com/repos/timson/k0fiscan/releases/latest \
           | grep '"tag_name":' \
           | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
[ -n "$LATEST" ] || { echo "❌ Could not fetch latest release tag"; exit 1; }

ASSET="${PROJECT}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"

echo "☕ Downloading ${BIN} ${LATEST} for ${TARGET} …"
curl -# -L "$URL" -o "/tmp/${ASSET}"

mkdir -p "$INSTALL_DIR"
tar -xzf "/tmp/${ASSET}" -C /tmp
chmod +x "/tmp/${BIN}"
mv "/tmp/${BIN}" "${INSTALL_DIR}/${BIN}"

echo "✅ Installed to ${INSTALL_DIR}/${BIN}"
echo "   Run  '${BIN} --help'  to get started."

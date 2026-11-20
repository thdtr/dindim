#!/bin/sh
# Installs dindim for the current user on Linux or macOS. No root needed.
#
#   curl -fsSL https://raw.githubusercontent.com/thdtr/dindim/main/scripts/install.sh | sh
#
# Options (pass after `sh -s --` when piping):
#   --version v0.1.0   install a specific release instead of the newest one
#   --uninstall        remove the app; your data is left untouched
set -eu

REPO="thdtr/dindim"
VERSION="${DINDIM_VERSION:-}"
UNINSTALL=0

while [ $# -gt 0 ]; do
    case "$1" in
        --version) VERSION="$2"; shift 2 ;;
        --uninstall) UNINSTALL=1; shift ;;
        *) echo "Unknown option: $1" >&2; exit 2 ;;
    esac
done

say() { printf '%s\n' "$*"; }
fail() { printf 'dindim install: %s\n' "$*" >&2; exit 1; }

OS="$(uname -s)"
ARCH="$(uname -m)"
BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
APPS_DIR="$DATA_HOME/applications"
ICON_DIR="$DATA_HOME/icons/hicolor"
MAC_APP="$HOME/Applications/dindim.app"

if [ "$UNINSTALL" -eq 1 ]; then
    case "$OS" in
        Linux)
            rm -f "$BIN_DIR/dindim" "$APPS_DIR/dindim.desktop" \
                "$ICON_DIR/scalable/apps/dindim.svg" "$ICON_DIR/256x256/apps/dindim.png"
            ;;
        Darwin) rm -rf "$MAC_APP" ;;
    esac
    say "dindim was removed. Your data was not touched."
    exit 0
fi

need() { command -v "$1" >/dev/null 2>&1 || fail "this script needs '$1'"; }
need curl
need tar

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64 | amd64) TARGET="linux-x86_64" ;;
            aarch64 | arm64) TARGET="linux-aarch64" ;;
            *) fail "no build for $ARCH yet" ;;
        esac
        ;;
    Darwin) TARGET="macos-universal" ;;
    *) fail "unsupported system: $OS (on Windows, use install.ps1)" ;;
esac

if [ -z "$VERSION" ]; then
    VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases?per_page=1" \
        | grep -m1 '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"
    [ -n "$VERSION" ] || fail "could not find the latest release"
fi

ASSET="dindim-$VERSION-$TARGET.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

say "Downloading dindim $VERSION for $TARGET..."
curl -fL --progress-bar -o "$TMP/$ASSET" "$URL" || fail "download failed: $URL"
curl -fsSL -o "$TMP/$ASSET.sha256" "$URL.sha256" || fail "checksum download failed"

expected="$(cut -d ' ' -f 1 < "$TMP/$ASSET.sha256")"
if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$TMP/$ASSET" | cut -d ' ' -f 1)"
else
    actual="$(shasum -a 256 "$TMP/$ASSET" | cut -d ' ' -f 1)"
fi
[ "$expected" = "$actual" ] || fail "checksum mismatch, the download may be corrupted"

tar -xzf "$TMP/$ASSET" -C "$TMP"
SRC="$TMP/dindim-$VERSION-$TARGET"

case "$OS" in
    Linux)
        mkdir -p "$BIN_DIR" "$APPS_DIR" "$ICON_DIR/scalable/apps" "$ICON_DIR/256x256/apps"
        install -m 755 "$SRC/dindim" "$BIN_DIR/dindim"
        install -m 644 "$SRC/dindim.svg" "$ICON_DIR/scalable/apps/dindim.svg"
        install -m 644 "$SRC/dindim.png" "$ICON_DIR/256x256/apps/dindim.png"
        sed "s|^Exec=.*|Exec=$BIN_DIR/dindim|" "$SRC/dindim.desktop" > "$APPS_DIR/dindim.desktop"
        command -v update-desktop-database >/dev/null 2>&1 \
            && update-desktop-database -q "$APPS_DIR" || true
        command -v gtk-update-icon-cache >/dev/null 2>&1 \
            && gtk-update-icon-cache -q -t "$ICON_DIR" || true
        say "dindim $VERSION is installed. Find it in your app menu or run: $BIN_DIR/dindim"
        ;;
    Darwin)
        mkdir -p "$HOME/Applications"
        rm -rf "$MAC_APP"
        cp -R "$SRC/dindim.app" "$MAC_APP"
        xattr -dr com.apple.quarantine "$MAC_APP" 2>/dev/null || true
        say "dindim $VERSION is installed in $MAC_APP. Open it from Launchpad or Spotlight."
        ;;
esac

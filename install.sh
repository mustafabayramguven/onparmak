#!/usr/bin/env bash
# onparmak'ı paket yöneticisi olmadan kurar: son sürümü ~/.local/bin'e indirir.
# Arch'ta tercih edilen yol packaging/PKGBUILD (makepkg -si).
set -euo pipefail

REPO="${ONPARMAK_REPO:-mustafabayramguven/onparmak}"
BIN_DIR="${ONPARMAK_BIN_DIR:-$HOME/.local/bin}"
NAME="onparmak-x86_64-linux"

if [[ "$(uname -s)/$(uname -m)" != "Linux/x86_64" ]]; then
  echo "Yalnızca x86_64 Linux destekleniyor." >&2
  exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

url="https://github.com/$REPO/releases/latest/download/$NAME.tar.gz"
echo "İndiriliyor: $url"
curl -fsSL "$url" -o "$tmp/$NAME.tar.gz"
tar xzf "$tmp/$NAME.tar.gz" -C "$tmp"

mkdir -p "$BIN_DIR"
install -m 755 "$tmp/$NAME/onparmak" "$BIN_DIR/onparmak"
echo "Kuruldu: $BIN_DIR/onparmak"

apps="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
mkdir -p "$apps"
sed "s|^Exec=onparmak|Exec=$BIN_DIR/onparmak|" "$tmp/$NAME/onparmak.desktop" > "$apps/onparmak.desktop"
echo "Başlatıcıya eklendi: $apps/onparmak.desktop"
